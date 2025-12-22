use anyhow::{Context, Result};
use std::fs;
use std::process::Command;

use crate::constants::{APP_LABEL, paths};

pub fn install() -> Result<()> {
    println!("Installing Edge Copilot Helper...");

    let current_exe = std::env::current_exe().context("Failed to get current executable path")?;
    let install_dir = paths::install_dir();
    let log_dir = paths::log_dir();
    let plist_path = paths::plist_path();
    let binary_path = paths::binary_path();

    // 1. Create directories
    println!("Creating directories...");
    fs::create_dir_all(&install_dir).with_context(|| {
        format!(
            "Failed to create install directory: {}",
            install_dir.display()
        )
    })?;
    fs::create_dir_all(&log_dir)
        .with_context(|| format!("Failed to create log directory: {}", log_dir.display()))?;

    // Ensure LaunchAgents directory exists
    if let Some(parent) = plist_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Failed to create LaunchAgents directory: {}",
                parent.display()
            )
        })?;
    }

    // 2. Copy binary
    println!("Installing binary...");
    fs::copy(&current_exe, &binary_path)
        .with_context(|| format!("Failed to copy binary to {}", binary_path.display()))?;

    // Make executable (should already be, but ensure it)
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&binary_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&binary_path, perms)?;
    }

    // 3. Unload existing service if present
    println!("Checking for existing service...");
    let uid = get_uid();
    let _ = Command::new("launchctl")
        .args([
            "bootout",
            &format!("gui/{uid}"),
            plist_path.to_str().unwrap_or(""),
        ])
        .output();

    // 4. Generate and write plist
    println!("Creating Launch Agent plist...");
    let plist_content = generate_plist(&binary_path, &log_dir);
    fs::write(&plist_path, plist_content)
        .with_context(|| format!("Failed to write plist to {}", plist_path.display()))?;

    // 5. Load service
    println!("Loading service...");
    let status = Command::new("launchctl")
        .args(["load", "-w", plist_path.to_str().unwrap_or("")])
        .status()
        .context("Failed to execute launchctl load")?;

    if !status.success() {
        anyhow::bail!("Failed to load Launch Agent");
    }

    println!();
    println!("Service installed and loaded successfully!");
    println!("  Binary: {}", binary_path.display());
    println!("  Logs:   {}", log_dir.display());
    println!();
    println!("Monitor with: tail -f {}/service.log", log_dir.display());

    Ok(())
}

pub fn uninstall() -> Result<()> {
    println!("Uninstalling Edge Copilot Helper...");

    let install_dir = paths::install_dir();
    let log_dir = paths::log_dir();
    let plist_path = paths::plist_path();

    // 1. Unload service
    println!("Stopping service...");
    let uid = get_uid();
    let _ = Command::new("launchctl")
        .args([
            "bootout",
            &format!("gui/{uid}"),
            plist_path.to_str().unwrap_or(""),
        ])
        .output();

    // 2. Remove plist
    if plist_path.exists() {
        println!("Removing plist: {}", plist_path.display());
        fs::remove_file(&plist_path)
            .with_context(|| format!("Failed to remove {}", plist_path.display()))?;
    }

    // 3. Remove install directory
    if install_dir.exists() {
        println!("Removing files: {}", install_dir.display());
        fs::remove_dir_all(&install_dir)
            .with_context(|| format!("Failed to remove {}", install_dir.display()))?;
    }

    // 4. Remove logs
    if log_dir.exists() {
        println!("Removing logs: {}", log_dir.display());
        fs::remove_dir_all(&log_dir)
            .with_context(|| format!("Failed to remove {}", log_dir.display()))?;
    }

    println!();
    println!("Uninstallation complete.");

    Ok(())
}

fn get_uid() -> String {
    Command::new("id")
        .arg("-u")
        .output()
        .map(|o| String::from_utf8_lossy(&o.stdout).trim().to_string())
        .unwrap_or_else(|_| "501".to_string())
}

fn generate_plist(binary_path: &std::path::Path, log_dir: &std::path::Path) -> String {
    let binary_str = binary_path.to_str().unwrap_or("");
    let stdout_log = log_dir.join("service.log");
    let stderr_log = log_dir.join("service.err");

    format!(
        r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>Label</key>
    <string>{label}</string>
    <key>ProgramArguments</key>
    <array>
        <string>{binary}</string>
        <string>run</string>
    </array>
    <key>RunAtLoad</key>
    <true/>
    <key>KeepAlive</key>
    <true/>
    <key>StandardOutPath</key>
    <string>{stdout}</string>
    <key>StandardErrorPath</key>
    <string>{stderr}</string>
</dict>
</plist>
"#,
        label = APP_LABEL,
        binary = binary_str,
        stdout = stdout_log.to_str().unwrap_or(""),
        stderr = stderr_log.to_str().unwrap_or("")
    )
}
