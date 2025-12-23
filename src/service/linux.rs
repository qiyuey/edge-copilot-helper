use anyhow::{Context, Result};
use std::fs;
use std::process::Command;

use crate::constants::{APP_LABEL, paths};

pub fn install() -> Result<()> {
    log::info!("Installing Edge Copilot Helper...");

    let current_exe = std::env::current_exe().context("Failed to get current executable path")?;
    let install_dir = paths::install_dir();
    let log_dir = paths::log_dir();
    let unit_path = paths::unit_path();
    let binary_path = paths::binary_path();

    // 1. Create directories
    log::info!("Creating directories...");
    fs::create_dir_all(&install_dir).with_context(|| {
        format!(
            "Failed to create install directory: {}",
            install_dir.display()
        )
    })?;
    fs::create_dir_all(&log_dir)
        .with_context(|| format!("Failed to create log directory: {}", log_dir.display()))?;

    // Ensure systemd user directory exists
    if let Some(parent) = unit_path.parent() {
        fs::create_dir_all(parent).with_context(|| {
            format!(
                "Failed to create systemd user directory: {}",
                parent.display()
            )
        })?;
    }

    // 2. Copy binary
    log::info!("Installing binary...");
    fs::copy(&current_exe, &binary_path)
        .with_context(|| format!("Failed to copy binary to {}", binary_path.display()))?;

    // Make executable
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = fs::metadata(&binary_path)?.permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&binary_path, perms)?;
    }

    // 3. Stop existing service if present
    log::info!("Checking for existing service...");
    let _ = Command::new("systemctl")
        .args(["--user", "stop", APP_LABEL])
        .output();
    let _ = Command::new("systemctl")
        .args(["--user", "disable", APP_LABEL])
        .output();

    // 4. Generate and write unit file
    log::info!("Creating systemd unit file...");
    let unit_content = generate_unit_file(&binary_path);
    fs::write(&unit_path, unit_content)
        .with_context(|| format!("Failed to write unit file to {}", unit_path.display()))?;

    // 5. Reload systemd
    log::info!("Reloading systemd...");
    let status = Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .status()
        .context("Failed to execute systemctl daemon-reload")?;

    if !status.success() {
        anyhow::bail!("Failed to reload systemd daemon");
    }

    // 6. Enable and start service
    log::info!("Enabling service...");
    let status = Command::new("systemctl")
        .args(["--user", "enable", APP_LABEL])
        .status()
        .context("Failed to execute systemctl enable")?;

    if !status.success() {
        anyhow::bail!("Failed to enable service");
    }

    log::info!("Starting service...");
    let status = Command::new("systemctl")
        .args(["--user", "start", APP_LABEL])
        .status()
        .context("Failed to execute systemctl start")?;

    if !status.success() {
        anyhow::bail!("Failed to start service");
    }

    log::info!("");
    log::info!("Service installed and started successfully!");
    log::info!("  Binary: {}", binary_path.display());
    log::info!("  Unit:   {}", unit_path.display());
    log::info!("");
    log::info!("Manage with:");
    log::info!("  systemctl --user status {APP_LABEL}");
    log::info!("  systemctl --user stop {APP_LABEL}");
    log::info!("  systemctl --user start {APP_LABEL}");
    log::info!("  journalctl --user -u {APP_LABEL} -f");

    Ok(())
}

pub fn uninstall() -> Result<()> {
    log::info!("Uninstalling Edge Copilot Helper...");

    let install_dir = paths::install_dir();
    let unit_path = paths::unit_path();

    // 1. Stop and disable service
    log::info!("Stopping service...");
    let _ = Command::new("systemctl")
        .args(["--user", "stop", APP_LABEL])
        .output();

    log::info!("Disabling service...");
    let _ = Command::new("systemctl")
        .args(["--user", "disable", APP_LABEL])
        .output();

    // 2. Remove unit file
    if unit_path.exists() {
        log::info!("Removing unit file: {}", unit_path.display());
        fs::remove_file(&unit_path)
            .with_context(|| format!("Failed to remove {}", unit_path.display()))?;
    }

    // 3. Reload systemd
    let _ = Command::new("systemctl")
        .args(["--user", "daemon-reload"])
        .output();

    // 4. Remove install directory (includes binary and logs)
    if install_dir.exists() {
        log::info!("Removing files: {}", install_dir.display());
        fs::remove_dir_all(&install_dir)
            .with_context(|| format!("Failed to remove {}", install_dir.display()))?;
    }

    log::info!("");
    log::info!("Uninstallation complete.");

    Ok(())
}

fn generate_unit_file(binary_path: &std::path::Path) -> String {
    let binary_str = binary_path.to_str().unwrap_or("");

    format!(
        r#"[Unit]
Description=Edge Copilot Helper - Bypass Microsoft Edge Copilot region restrictions
After=default.target

[Service]
Type=simple
ExecStart={binary} daemon
Restart=always
RestartSec=5

[Install]
WantedBy=default.target
"#,
        binary = binary_str
    )
}
