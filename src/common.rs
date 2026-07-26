use anyhow::{Context, Result};
use serde_json::Value;
use std::{
    fs,
    path::{Path, PathBuf},
};

const TARGET_COUNTRY: &str = "SG";
const COPILOT_SIDEBAR_UUID: &str = "cd4688a9-e888-48ea-ad81-76193d56b1be";

/// Edge 用来存储 variations 实验种子的二进制文件。
/// 这些文件内嵌了国家代码，即使修改 Local State 中的 `variations_country`，
/// Edge 仍会从种子文件中读取原始国家并重新生成禁用标志。
/// 删除后 Edge 会通过网络重新获取（此时应通过代理，以获取目标国家的种子）。
const SEED_FILES: &[&str] = &[
    "VariationsSeedV2",
    "VariationsSafeSeedV2",
    "VariationsRuntimeSeedV2",
];

/// 与种子关联的 Local State 元数据字段，删除种子文件后需要一并清除，
/// 避免 Edge 尝试加载已不存在的种子。
const SEED_METADATA_FIELDS: &[&str] = &[
    "variations_safe_seed_date",
    "variations_safe_seed_fetch_time",
    "variations_safe_seed_locale",
    "variations_safe_seed_milestone",
    "variations_safe_seed_signature",
    "variations_seed_date",
    "variations_seed_etag",
    "variations_seed_signature",
    "variations_seed_serial_number",
    "variations_seed_milestone",
];

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
    path: &Path,
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
/// 2. 修改 `Local State`：`variations_country`、`variations_safe_seed_session_consistency_country`，
///    并移除 `variations_config_ids` 中的 Copilot 禁用标志
/// 3. 修改各 Profile 的 `Preferences`：设置 `chat_ip_eligibility_status` 为 true，
///    启用 Copilot 侧边栏按钮
pub fn apply_fix() -> Result<()> {
    let (local_state_paths, prefs_paths) = get_all_paths()?;

    let mut found_existing = false;
    let mut any_modified = false;

    for local_state_path in local_state_paths {
        found_existing = true;
        if process_json_file(&local_state_path, "Local State", |json| {
            let mut modified = false;
            modified |= patch_variations_country(json);
            modified |= patch_safe_seed_country(json);
            modified |= remove_copilot_disable_flags(json);
            modified |= clear_seed_metadata(json);
            modified
        })? {
            if let Some(user_data_dir) = local_state_path.parent() {
                delete_seed_files(user_data_dir);
            }
            any_modified = true;
        }
    }

    for prefs_path in prefs_paths {
        found_existing = true;
        if process_json_file(&prefs_path, "Preferences", |json| {
            let mut modified = false;
            modified |= set_chat_ip_eligibility_status(json);
            modified |= enable_copilot_sidebar(json);
            modified
        })? {
            any_modified = true;
        }
    }

    if !found_existing {
        log::warn!("⚠️ Edge configuration files not found in known locations.");
    } else if !any_modified {
        log::info!("ℹ️ No changes needed: all Copilot settings are already correct.");
    }

    Ok(())
}

fn patch_variations_country(json: &mut Value) -> bool {
    set_string_field(json, "variations_country", TARGET_COUNTRY)
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

fn patch_safe_seed_country(json: &mut Value) -> bool {
    set_string_field(
        json,
        "variations_safe_seed_session_consistency_country",
        TARGET_COUNTRY,
    )
}

fn set_string_field(json: &mut Value, key: &str, value: &str) -> bool {
    let Some(obj) = json.as_object_mut() else {
        return false;
    };
    if obj.get(key).and_then(Value::as_str) == Some(value) {
        return false;
    }
    obj.insert(key.to_owned(), Value::String(value.to_owned()));
    true
}

/// 从 `variations_config_ids` 中移除包含 "disablecopilot" 的服务端下发标志，
/// 这些标志会阻止 Copilot 功能在受限区域显示。
fn remove_copilot_disable_flags(json: &mut Value) -> bool {
    if let Some(obj) = json.as_object_mut()
        && let Some(config_ids) = obj.get("variations_config_ids").and_then(|v| v.as_str())
    {
        let original_count = config_ids.split(',').count();
        let filtered: Vec<&str> = config_ids
            .split(',')
            .filter(|s| !s.to_lowercase().contains("disablecopilot"))
            .collect();
        if filtered.len() < original_count {
            log::info!(
                "🔧 Removed {} Copilot disable flag(s) from variations_config_ids",
                original_count - filtered.len()
            );
            obj.insert(
                "variations_config_ids".to_string(),
                Value::String(filtered.join(",")),
            );
            return true;
        }
    }
    false
}

/// 删除 User Data 目录中的 variations 种子文件。
/// 这些文件内嵌了国家代码，会导致 Edge 重新生成 Copilot 禁用标志。
fn delete_seed_files(user_data_dir: &Path) -> bool {
    let mut deleted_any = false;
    for name in SEED_FILES {
        let path = user_data_dir.join(name);
        if path.exists() {
            match fs::remove_file(&path) {
                Ok(()) => {
                    log::info!("🗑️ Deleted seed file: {}", name);
                    deleted_any = true;
                }
                Err(e) => log::warn!("⚠️ Failed to delete {}: {}", path.display(), e),
            }
        }
    }
    deleted_any
}

/// 清除 Local State 中与种子关联的元数据字段，
/// 防止 Edge 尝试加载已删除的种子文件。
fn clear_seed_metadata(json: &mut Value) -> bool {
    let Some(obj) = json.as_object_mut() else {
        return false;
    };

    let mut modified = false;
    for field in SEED_METADATA_FIELDS {
        if obj.remove(*field).is_some() {
            modified = true;
        }
    }
    modified
}

/// 将 Copilot 侧边栏应用从隐藏状态（值为 0）恢复为可见（值为 1）。
fn enable_copilot_sidebar(json: &mut Value) -> bool {
    let browser_obj = json
        .as_object_mut()
        .and_then(|obj| obj.get_mut("browser"))
        .and_then(|b| b.as_object_mut());

    let Some(browser_obj) = browser_obj else {
        return false;
    };

    let sidebar_obj = browser_obj
        .get_mut("show_hub_app_in_sidebar_buttons")
        .and_then(|s| s.as_object_mut());

    if let Some(sidebar_obj) = sidebar_obj
        && sidebar_obj
            .get(COPILOT_SIDEBAR_UUID)
            .and_then(|v| v.as_i64())
            == Some(0)
    {
        sidebar_obj.insert(COPILOT_SIDEBAR_UUID.to_string(), Value::Number(1.into()));
        return true;
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
    home: &Path,
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
        assert_eq!(value["variations_country"], json!(TARGET_COUNTRY));
        assert_eq!(value["other_field"], json!("test"));
    }

    #[test]
    fn test_patch_variations_country_from_other() {
        let mut value = json!({
            "variations_country": "US",
            "other_field": "test"
        });
        assert!(patch_variations_country(&mut value));
        assert_eq!(value["variations_country"], json!(TARGET_COUNTRY));
    }

    #[test]
    fn test_patch_variations_country_already_target() {
        let mut value = json!({
            "variations_country": TARGET_COUNTRY,
            "other_field": "test"
        });
        assert!(!patch_variations_country(&mut value));
        assert_eq!(value["variations_country"], json!(TARGET_COUNTRY));
    }

    #[test]
    fn test_patch_variations_country_missing_field() {
        let mut value = json!({
            "other_field": "test"
        });
        assert!(patch_variations_country(&mut value));
        assert_eq!(value["variations_country"], json!(TARGET_COUNTRY));
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

    // --- patch_safe_seed_country ---

    #[test]
    fn test_patch_safe_seed_country_from_cn() {
        let mut value = json!({
            "variations_safe_seed_session_consistency_country": "CN"
        });
        assert!(patch_safe_seed_country(&mut value));
        assert_eq!(
            value["variations_safe_seed_session_consistency_country"],
            json!(TARGET_COUNTRY)
        );
    }

    #[test]
    fn test_patch_safe_seed_country_already_target() {
        let mut value = json!({
            "variations_safe_seed_session_consistency_country": TARGET_COUNTRY
        });
        assert!(!patch_safe_seed_country(&mut value));
    }

    #[test]
    fn test_patch_safe_seed_country_missing() {
        let mut value = json!({ "other": "test" });
        assert!(patch_safe_seed_country(&mut value));
        assert_eq!(
            value["variations_safe_seed_session_consistency_country"],
            json!(TARGET_COUNTRY)
        );
    }

    // --- remove_copilot_disable_flags ---

    #[test]
    fn test_remove_copilot_disable_flags_present() {
        let mut value = json!({
            "variations_config_ids": "flag1:123,disablecopilotmodeenp:916054,flag2:456"
        });
        assert!(remove_copilot_disable_flags(&mut value));
        assert_eq!(value["variations_config_ids"], json!("flag1:123,flag2:456"));
    }

    #[test]
    fn test_remove_copilot_disable_flags_multiple() {
        let mut value = json!({
            "variations_config_ids": "disablecopilotA:1,keep:2,disablecopilotB:3"
        });
        assert!(remove_copilot_disable_flags(&mut value));
        assert_eq!(value["variations_config_ids"], json!("keep:2"));
    }

    #[test]
    fn test_remove_copilot_disable_flags_not_present() {
        let mut value = json!({
            "variations_config_ids": "flag1:123,flag2:456"
        });
        assert!(!remove_copilot_disable_flags(&mut value));
    }

    #[test]
    fn test_remove_copilot_disable_flags_missing_field() {
        let mut value = json!({ "other": "test" });
        assert!(!remove_copilot_disable_flags(&mut value));
    }

    // --- clear_seed_metadata ---

    #[test]
    fn test_clear_seed_metadata_present() {
        let mut value = json!({
            "variations_safe_seed_date": "13417107639000000",
            "variations_safe_seed_fetch_time": "13417983884617129",
            "variations_safe_seed_locale": "zh-CN",
            "variations_safe_seed_milestone": 146,
            "variations_seed_date": "13417983882000000",
            "variations_seed_etag": "some-etag",
            "variations_country": "SG",
            "other_field": "keep"
        });
        assert!(clear_seed_metadata(&mut value));
        assert!(value.get("variations_safe_seed_date").is_none());
        assert!(value.get("variations_safe_seed_fetch_time").is_none());
        assert!(value.get("variations_safe_seed_locale").is_none());
        assert!(value.get("variations_seed_date").is_none());
        assert!(value.get("variations_seed_etag").is_none());
        assert_eq!(value["variations_country"], json!("SG"));
        assert_eq!(value["other_field"], json!("keep"));
    }

    #[test]
    fn test_clear_seed_metadata_none_present() {
        let mut value = json!({
            "variations_country": "SG",
            "other_field": "keep"
        });
        assert!(!clear_seed_metadata(&mut value));
    }

    #[test]
    fn test_clear_seed_metadata_not_object() {
        let mut value = json!("not an object");
        assert!(!clear_seed_metadata(&mut value));
    }

    // --- enable_copilot_sidebar ---

    #[test]
    fn test_enable_copilot_sidebar_from_hidden() {
        let mut value = json!({
            "browser": {
                "show_hub_app_in_sidebar_buttons": {
                    COPILOT_SIDEBAR_UUID: 0,
                    "other-uuid": 3
                }
            }
        });
        assert!(enable_copilot_sidebar(&mut value));
        assert_eq!(
            value["browser"]["show_hub_app_in_sidebar_buttons"][COPILOT_SIDEBAR_UUID],
            json!(1)
        );
        assert_eq!(
            value["browser"]["show_hub_app_in_sidebar_buttons"]["other-uuid"],
            json!(3)
        );
    }

    #[test]
    fn test_enable_copilot_sidebar_already_visible() {
        let mut value = json!({
            "browser": {
                "show_hub_app_in_sidebar_buttons": {
                    COPILOT_SIDEBAR_UUID: 1
                }
            }
        });
        assert!(!enable_copilot_sidebar(&mut value));
    }

    #[test]
    fn test_enable_copilot_sidebar_value_3() {
        let mut value = json!({
            "browser": {
                "show_hub_app_in_sidebar_buttons": {
                    COPILOT_SIDEBAR_UUID: 3
                }
            }
        });
        assert!(!enable_copilot_sidebar(&mut value));
    }

    #[test]
    fn test_enable_copilot_sidebar_no_browser() {
        let mut value = json!({ "other": "test" });
        assert!(!enable_copilot_sidebar(&mut value));
    }

    #[test]
    fn test_enable_copilot_sidebar_no_sidebar_buttons() {
        let mut value = json!({
            "browser": { "other": "test" }
        });
        assert!(!enable_copilot_sidebar(&mut value));
    }
}
