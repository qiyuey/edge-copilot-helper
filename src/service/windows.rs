use anyhow::{Context, Result};
use std::fs;
use std::process::Command;

use crate::constants::paths;
use sysinfo::{Pid, System};

const REG_KEY_NAME: &str = "EdgeCopilotHelper";
const REG_PATH: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";

pub fn install() -> Result<()> {
    log::info!("Installing Edge Copilot Helper...");

    let current_exe = std::env::current_exe().context("Failed to get current executable path")?;
    let install_dir = paths::install_dir();
    let log_dir = paths::log_dir();
    let binary_path = paths::binary_path();

    // 0. Stop existing running instance to avoid copy failures
    log::info!("Stopping running instances (if any)...");
    stop_running_instances();

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

    // Remove old binary if present (best-effort overwrite)
    if binary_path.exists() {
        log::info!("Removing existing binary...");
        let _ = fs::remove_file(&binary_path);
    }

    // 2. Copy binary
    log::info!("Installing binary...");
    fs::copy(&current_exe, &binary_path)
        .with_context(|| format!("Failed to copy binary to {}", binary_path.display()))?;

    // 3. Add to startup registry (HKCU\Run)
    log::info!("Adding to startup registry...");
    let bin_path = binary_path.to_str().unwrap_or("");
    // Format: "C:\path\to\exe" daemon
    let reg_value = format!("\"{}\" daemon", bin_path);

    let status = Command::new("reg")
        .args([
            "add",
            REG_PATH,
            "/v",
            REG_KEY_NAME,
            "/t",
            "REG_SZ",
            "/d",
            &reg_value,
            "/f",
        ])
        .status()
        .context("Failed to execute reg add")?;

    if !status.success() {
        anyhow::bail!("Failed to add registry entry for startup");
    }

    log::info!("");
    log::info!("Service installed successfully!");
    log::info!("  Binary: {}", binary_path.display());
    log::info!("  Startup: Registry (HKCU\\Run)");
    log::info!("");
    log::info!("The application will start automatically when you log in.");
    log::info!("To remove from startup, run: edge-copilot-helper uninstall");

    Ok(())
}

pub fn uninstall() -> Result<()> {
    log::info!("Uninstalling Edge Copilot Helper...");

    let install_dir = paths::install_dir();

    // Stop running instance
    log::info!("Stopping running instances (if any)...");
    stop_running_instances();

    // 1. Remove from startup registry
    log::info!("Removing from startup registry...");
    let status = Command::new("reg")
        .args(["delete", REG_PATH, "/v", REG_KEY_NAME, "/f"])
        .status();

    match status {
        Ok(s) if s.success() => {
            log::info!("Removed from startup registry.");
        }
        _ => {
            log::warn!("Warning: Failed to remove registry entry. It may not exist.");
        }
    }

    // 2. Remove install directory (includes binary and logs)
    if install_dir.exists() {
        log::info!("Removing files: {}", install_dir.display());
        fs::remove_dir_all(&install_dir)
            .with_context(|| format!("Failed to remove {}", install_dir.display()))?;
    }

    log::info!("");
    log::info!("Uninstallation complete.");

    Ok(())
}

fn stop_running_instances() {
    let mut sys = System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);
    let current_pid = Pid::from_u32(std::process::id());

    for (pid, process) in sys.processes() {
        let name = process.name().to_string_lossy();
        if name.eq_ignore_ascii_case("edge-copilot-helper.exe") && *pid != current_pid {
            log::info!("Stopping existing instance (pid {}): {}", pid, name);
            // Try graceful kill first
            let _ = process.kill();
        }
    }
}
