use anyhow::{Context, Result};
use std::fs;
use std::process::Command;

use crate::constants::paths;

const REG_KEY_NAME: &str = "EdgeCopilotHelper";
const REG_PATH: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";

pub fn install() -> Result<()> {
    println!("Installing Edge Copilot Helper...");

    let current_exe = std::env::current_exe().context("Failed to get current executable path")?;
    let install_dir = paths::install_dir();
    let log_dir = paths::log_dir();
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

    // 2. Copy binary
    println!("Installing binary...");
    fs::copy(&current_exe, &binary_path)
        .with_context(|| format!("Failed to copy binary to {}", binary_path.display()))?;

    // 3. Add to startup registry (HKCU\Run)
    println!("Adding to startup registry...");
    let bin_path = binary_path.to_str().unwrap_or("");
    // Format: "C:\path\to\exe" run
    let reg_value = format!("\"{}\" run", bin_path);

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

    println!();
    println!("Service installed successfully!");
    println!("  Binary: {}", binary_path.display());
    println!("  Startup: Registry (HKCU\\Run)");
    println!();
    println!("The application will start automatically when you log in.");
    println!("To remove from startup, run: edge-copilot-helper uninstall");

    Ok(())
}

pub fn uninstall() -> Result<()> {
    println!("Uninstalling Edge Copilot Helper...");

    let install_dir = paths::install_dir();

    // 1. Remove from startup registry
    println!("Removing from startup registry...");
    let status = Command::new("reg")
        .args(["delete", REG_PATH, "/v", REG_KEY_NAME, "/f"])
        .status();

    match status {
        Ok(s) if s.success() => {
            println!("Removed from startup registry.");
        }
        _ => {
            println!("Warning: Failed to remove registry entry. It may not exist.");
        }
    }

    // 2. Remove install directory (includes binary and logs)
    if install_dir.exists() {
        println!("Removing files: {}", install_dir.display());
        fs::remove_dir_all(&install_dir)
            .with_context(|| format!("Failed to remove {}", install_dir.display()))?;
    }

    println!();
    println!("Uninstallation complete.");

    Ok(())
}
