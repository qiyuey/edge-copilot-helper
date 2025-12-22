use std::{fs, path::PathBuf};
use serde_json::Value;
use anyhow::{Context, Result};

pub fn apply_fix() -> Result<()> {
    let prefs_paths = get_prefs_paths()?;

    let mut found_existing = false;
    let mut any_modified = false;

    for prefs_path in prefs_paths {
        if !prefs_path.exists() {
            continue;
        }

        found_existing = true;

        // 1. Read
        let content = fs::read_to_string(&prefs_path)
            .with_context(|| format!("Failed to read preferences at {:?}", prefs_path))?;
        let mut json: Value = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse JSON at {:?}", prefs_path))?;

        // 2. Modify: replace any string value exactly "CN" with "SG"
        let modified = replace_cn_values(&mut json);

        // 3. Write (only if modified)
        if modified {
            let new_content = serde_json::to_string_pretty(&json)?;
            fs::write(&prefs_path, new_content)
                .with_context(|| format!("Failed to write preferences at {:?}", prefs_path))?;
            println!("✅ Edge Copilot region fix applied at {:?}", prefs_path);
            any_modified = true;
        }
    }

    if !found_existing {
        println!("⚠️ Preferences file not found in known locations.");
    } else if !any_modified {
        println!("ℹ️ No CN values found to update.");
    }

    Ok(())
}

fn replace_cn_values(value: &mut Value) -> bool {
    match value {
        Value::String(s) if s == "CN" => {
            *s = "SG".to_string();
            true
        }
        Value::Array(arr) => {
            let mut changed = false;
            for v in arr.iter_mut() {
                changed |= replace_cn_values(v);
            }
            changed
        }
        Value::Object(map) => {
            let mut changed = false;
            for v in map.values_mut() {
                changed |= replace_cn_values(v);
            }
            changed
        }
        _ => false,
    }
}

fn get_prefs_paths() -> Result<Vec<PathBuf>> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let mut paths = Vec::new();

    #[cfg(target_os = "macos")]
    {
        let mac_channels = [
            "Library/Application Support/Microsoft Edge/Default/Preferences",
            "Library/Application Support/Microsoft Edge Beta/Default/Preferences",
            "Library/Application Support/Microsoft Edge Dev/Default/Preferences",
            "Library/Application Support/Microsoft Edge Canary/Default/Preferences",
        ];
        for channel in mac_channels {
            paths.push(home.join(channel));
        }
    }

    #[cfg(target_os = "linux")]
    {
        let linux_channels = [
            ".config/microsoft-edge/Default/Preferences",
            ".config/microsoft-edge-beta/Default/Preferences",
            ".config/microsoft-edge-dev/Default/Preferences",
            ".config/microsoft-edge-canary/Default/Preferences",
        ];
        for channel in linux_channels {
            paths.push(home.join(channel));
        }
    }

    #[cfg(target_os = "windows")]
    {
        let windows_channels = [
            "AppData/Local/Microsoft/Edge/User Data/Default/Preferences",
            "AppData/Local/Microsoft/Edge Beta/User Data/Default/Preferences",
            "AppData/Local/Microsoft/Edge Dev/User Data/Default/Preferences",
            "AppData/Local/Microsoft/Edge SxS/User Data/Default/Preferences",
        ];
        for channel in windows_channels {
            paths.push(home.join(channel));
        }
    }

    Ok(paths)
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_replace_cn_simple_string() {
        let mut value = json!("CN");
        assert!(replace_cn_values(&mut value));
        assert_eq!(value, json!("SG"));
    }

    #[test]
    fn test_replace_cn_no_change() {
        let mut value = json!("US");
        assert!(!replace_cn_values(&mut value));
        assert_eq!(value, json!("US"));
    }

    #[test]
    fn test_replace_cn_in_object() {
        let mut value = json!({
            "region": "CN",
            "name": "test",
            "nested": {
                "country": "CN",
                "city": "Beijing"
            }
        });
        assert!(replace_cn_values(&mut value));
        assert_eq!(value, json!({
            "region": "SG",
            "name": "test",
            "nested": {
                "country": "SG",
                "city": "Beijing"
            }
        }));
    }

    #[test]
    fn test_replace_cn_in_array() {
        let mut value = json!(["CN", "US", "CN", "JP"]);
        assert!(replace_cn_values(&mut value));
        assert_eq!(value, json!(["SG", "US", "SG", "JP"]));
    }

    #[test]
    fn test_replace_cn_mixed_structure() {
        let mut value = json!({
            "regions": ["CN", "US"],
            "default": "CN",
            "config": {
                "locale": "CN",
                "enabled": true,
                "count": 42
            }
        });
        assert!(replace_cn_values(&mut value));
        assert_eq!(value, json!({
            "regions": ["SG", "US"],
            "default": "SG",
            "config": {
                "locale": "SG",
                "enabled": true,
                "count": 42
            }
        }));
    }

    #[test]
    fn test_replace_cn_no_cn_values() {
        let mut value = json!({
            "region": "US",
            "list": ["JP", "KR"],
            "nested": {"country": "UK"}
        });
        assert!(!replace_cn_values(&mut value));
    }

    #[test]
    fn test_replace_cn_partial_match_ignored() {
        let mut value = json!({
            "code": "CNN",
            "name": "CN_test",
            "prefix": "preCN"
        });
        assert!(!replace_cn_values(&mut value));
        // Values should remain unchanged
        assert_eq!(value, json!({
            "code": "CNN",
            "name": "CN_test",
            "prefix": "preCN"
        }));
    }
}
