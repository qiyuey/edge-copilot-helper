use anyhow::{Context, Result};
use serde_json::Value;
use std::{fs, path::PathBuf};

/// 处理单个 JSON 配置文件
///
/// # 参数
/// - `path`: 文件路径
/// - `file_type`: 文件类型描述（用于日志）
/// - `modify_fn`: 修改函数，返回 true 表示进行了修改
///
/// # 返回
/// - `Ok(true)`: 文件已修改并保存
/// - `Ok(false)`: 文件未修改（不存在或无需修改）
fn process_json_file(
    path: &PathBuf,
    file_type: &str,
    modify_fn: impl FnOnce(&mut Value) -> bool,
) -> Result<bool> {
    if !path.exists() {
        return Ok(false);
    }

    let content = fs::read_to_string(path)
        .with_context(|| format!("Failed to read {} at {}", file_type, path.display()))?;

    let mut json: Value = serde_json::from_str(&content)
        .with_context(|| format!("Failed to parse JSON at {}", path.display()))?;

    let modified = modify_fn(&mut json);

    if modified {
        let new_content = serde_json::to_string_pretty(&json)?;
        fs::write(path, new_content)
            .with_context(|| format!("Failed to write {} at {}", file_type, path.display()))?;
        log::info!(
            "✅ Edge Copilot region fix applied to {} at {}",
            file_type,
            path.display()
        );
    }

    Ok(modified)
}

/// 应用 Edge Copilot 区域修复
///
/// 此函数是核心入口点，在 Edge 退出时调用。它执行以下操作：
/// 1. 定位所有 Edge 配置文件（支持多个 Edge 版本：Stable、Beta、Dev、Canary）
/// 2. 修改 `Local State` 文件中的 `variations_country` 为 "US"
/// 3. 修改各 Profile 的 `Preferences` 文件，设置 `chat_ip_eligibility_status` 为 true
///
/// # 错误
/// 返回 `Err` 如果无法读取或写入配置文件
pub fn apply_fix() -> Result<()> {
    let (local_state_paths, prefs_paths) = get_all_paths()?;

    let mut found_existing = false;
    let mut any_modified = false;

    // 处理 Local State 文件
    for local_state_path in local_state_paths {
        found_existing = true;
        if process_json_file(&local_state_path, "Local State", |json| {
            patch_variations_country(json)
        })? {
            any_modified = true;
        }
    }

    // 处理 Preferences 文件（所有 Profile）
    for prefs_path in prefs_paths {
        found_existing = true;
        if process_json_file(&prefs_path, "Preferences", |json| {
            set_chat_ip_eligibility_status(json)
        })? {
            any_modified = true;
        }
    }

    if !found_existing {
        log::warn!("⚠️ Edge configuration files not found in known locations.");
    } else if !any_modified {
        log::info!(
            "ℹ️ No changes needed: variations_country already US and chat_ip_eligibility_status already set."
        );
    }

    Ok(())
}

/// 修改 Local State 中的 variations_country 字段为 "US"
fn patch_variations_country(json: &mut Value) -> bool {
    if let Some(obj) = json.as_object_mut() {
        if let Some(variations_country) = obj.get("variations_country")
            && variations_country.as_str() == Some("US")
        {
            return false;
        }
        obj.insert(
            "variations_country".to_string(),
            Value::String("US".to_string()),
        );
        return true;
    }
    false
}

/// 设置 chat_ip_eligibility_status 为 true
/// 只处理根级别的 browser 对象，不递归遍历
fn set_chat_ip_eligibility_status(json: &mut Value) -> bool {
    if let Some(obj) = json.as_object_mut() {
        // 检查是否有 browser 字段
        if let Some(browser) = obj.get_mut("browser") {
            if let Some(browser_obj) = browser.as_object_mut() {
                // 检查 chat_ip_eligibility_status 字段
                if let Some(status) = browser_obj.get("chat_ip_eligibility_status") {
                    // 如果已经是 true，不需要修改
                    if status.as_bool() != Some(true) {
                        browser_obj
                            .insert("chat_ip_eligibility_status".to_string(), Value::Bool(true));
                        return true;
                    }
                    return false;
                } else {
                    // 字段不存在，添加它
                    browser_obj.insert("chat_ip_eligibility_status".to_string(), Value::Bool(true));
                    return true;
                }
            }
        } else {
            // browser 字段不存在，创建它
            let mut browser_obj = serde_json::Map::new();
            browser_obj.insert("chat_ip_eligibility_status".to_string(), Value::Bool(true));
            obj.insert("browser".to_string(), Value::Object(browser_obj));
            return true;
        }
    }
    false
}

/// 获取所有需要修改的文件路径
/// 返回 (Local State 路径列表, Preferences 路径列表)
fn get_all_paths() -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let home = dirs::home_dir().context("Could not determine home directory")?;

    #[cfg(target_os = "macos")]
    let user_data_paths: &[&str] = &[
        "Library/Application Support/Microsoft Edge",
        "Library/Application Support/Microsoft Edge Beta",
        "Library/Application Support/Microsoft Edge Dev",
        "Library/Application Support/Microsoft Edge Canary",
    ];

    #[cfg(target_os = "linux")]
    let user_data_paths: &[&str] = &[
        ".config/microsoft-edge",
        ".config/microsoft-edge-beta",
        ".config/microsoft-edge-dev",
        ".config/microsoft-edge-canary",
    ];

    #[cfg(target_os = "windows")]
    let user_data_paths: &[&str] = &[
        "AppData/Local/Microsoft/Edge/User Data",
        "AppData/Local/Microsoft/Edge Beta/User Data",
        "AppData/Local/Microsoft/Edge Dev/User Data",
        "AppData/Local/Microsoft/Edge SxS/User Data",
    ];

    collect_edge_paths(&home, user_data_paths)
}

/// 从指定的用户数据目录收集 Edge 配置文件路径
fn collect_edge_paths(
    home: &std::path::Path,
    user_data_paths: &[&str],
) -> Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let mut local_state_paths = Vec::new();
    let mut prefs_paths = Vec::new();

    for user_data_path in user_data_paths {
        let user_data = home.join(user_data_path);
        if !user_data.exists() {
            continue;
        }

        // Local State 文件
        let local_state = user_data.join("Local State");
        if local_state.exists() {
            local_state_paths.push(local_state);
        }

        // 遍历所有 Profile 目录
        if let Ok(entries) = fs::read_dir(&user_data) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    let dir_name = path.file_name().and_then(|n| n.to_str());
                    if dir_name == Some("Default")
                        || dir_name.is_some_and(|n| n.starts_with("Profile "))
                    {
                        let prefs = path.join("Preferences");
                        if prefs.exists() {
                            prefs_paths.push(prefs);
                        }
                    }
                }
            }
        }
    }

    Ok((local_state_paths, prefs_paths))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_patch_variations_country_from_cn() {
        let mut value = json!({
            "variations_country": "CN",
            "other_field": "test"
        });
        assert!(patch_variations_country(&mut value));
        assert_eq!(value["variations_country"], json!("US"));
        assert_eq!(value["other_field"], json!("test"));
    }

    #[test]
    fn test_patch_variations_country_from_other() {
        let mut value = json!({
            "variations_country": "SG",
            "other_field": "test"
        });
        assert!(patch_variations_country(&mut value));
        assert_eq!(value["variations_country"], json!("US"));
    }

    #[test]
    fn test_patch_variations_country_already_us() {
        let mut value = json!({
            "variations_country": "US",
            "other_field": "test"
        });
        assert!(!patch_variations_country(&mut value));
        assert_eq!(value["variations_country"], json!("US"));
    }

    #[test]
    fn test_patch_variations_country_missing_field() {
        let mut value = json!({
            "other_field": "test"
        });
        assert!(patch_variations_country(&mut value));
        assert_eq!(value["variations_country"], json!("US"));
        assert_eq!(value["other_field"], json!("test"));
    }

    #[test]
    fn test_patch_variations_country_not_object() {
        let mut value = json!("not an object");
        assert!(!patch_variations_country(&mut value));
        assert_eq!(value, json!("not an object"));
    }

    #[test]
    fn test_set_chat_ip_eligibility_status_missing() {
        let mut value = json!({
            "other_field": "test"
        });
        assert!(set_chat_ip_eligibility_status(&mut value));
        assert_eq!(value["browser"]["chat_ip_eligibility_status"], json!(true));
    }

    #[test]
    fn test_set_chat_ip_eligibility_status_false() {
        let mut value = json!({
            "browser": {
                "chat_ip_eligibility_status": false
            }
        });
        assert!(set_chat_ip_eligibility_status(&mut value));
        assert_eq!(value["browser"]["chat_ip_eligibility_status"], json!(true));
    }

    #[test]
    fn test_set_chat_ip_eligibility_status_already_true() {
        let mut value = json!({
            "browser": {
                "chat_ip_eligibility_status": true
            }
        });
        assert!(!set_chat_ip_eligibility_status(&mut value));
        assert_eq!(value["browser"]["chat_ip_eligibility_status"], json!(true));
    }

    #[test]
    fn test_set_chat_ip_eligibility_status_not_object() {
        let mut value = json!("not an object");
        assert!(!set_chat_ip_eligibility_status(&mut value));
    }
}
