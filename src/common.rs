use std::{fs, path::PathBuf};
use serde_json::Value;
use anyhow::{Result, Context};

pub fn apply_fix() -> Result<()> {
    let prefs_path = get_prefs_path().context("Could not determine preferences path")?;
    
    if !prefs_path.exists() {
        // Only log if we expect it to be there but it's not. 
        // For a watcher, sometimes the file might not be created yet if Edge was never run.
        println!("⚠️ Preferences file not found at {:?}", prefs_path);
        return Ok(());
    }

    // 1. Read
    let content = fs::read_to_string(&prefs_path)?;
    let mut json: Value = serde_json::from_str(&content).context("Failed to parse Preferences JSON")?;

    // 2. Modify
    // We want to set browser.custom_services.region_search = "SG"
    let mut modified = false;

    if let Some(browser) = json.get_mut("browser") {
        if let Some(services) = browser.get_mut("custom_services") {
            // Check if it's already SG to avoid unnecessary writes
            if services["region_search"] != "SG" {
                services["region_search"] = Value::String("SG".to_string());
                modified = true;
            }
        } else {
            // "custom_services" missing, create it
            // This is a bit tricky with untyped Value, but we can try to insert if it's an object
            if let Some(obj) = browser.as_object_mut() {
                let mut services = serde_json::Map::new();
                services.insert("region_search".to_string(), Value::String("SG".to_string()));
                obj.insert("custom_services".to_string(), Value::Object(services));
                modified = true;
            }
        }
    } else {
         // "browser" missing, this is unlikely for a valid Prefs file, but handle gracefully
         println!("⚠️ 'browser' node missing in Preferences.");
    }

    // 3. Write (only if modified)
    if modified {
        // We use pretty print? Original file is usually compact, but jq often prettifies it or vice versa.
        // Edge handles both. Pretty is safer for debugging.
        // But for minimal diff, maybe not. However, serde_json::to_string is compact. 
        // Let's use pretty to be consistent with previous jq behavior if jq was used.
        // Actually, previous jq command didn't specify compact, so it probably pretty printed.
        // Let's stick to standard write.
        let new_content = serde_json::to_string(&json)?;
        fs::write(&prefs_path, new_content)?;
        println!("✅ Edge Copilot region fix applied (set to SG).");
    } else {
        // println!("ℹ️ Region already set to SG.");
    }

    Ok(())
}

fn get_prefs_path() -> Option<PathBuf> {
    let mut path = dirs::home_dir()?;

    #[cfg(target_os = "macos")]
    path.push("Library/Application Support/Microsoft Edge/Default/Preferences");

    #[cfg(target_os = "linux")]
    path.push(".config/microsoft-edge/Default/Preferences");

    #[cfg(target_os = "windows")]
    path.push("AppData/Local/Microsoft/Edge/User Data/Default/Preferences");

    Some(path)
}

