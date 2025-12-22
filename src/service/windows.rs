use anyhow::{Context, Result};
use std::fs;
use std::process::Command;

use crate::constants::{paths, APP_LABEL, BINARY_NAME};

const SERVICE_NAME: &str = "EdgeCopilotHelper";
const DISPLAY_NAME: &str = "Edge Copilot Helper";

pub fn install() -> Result<()> {
    println!("Installing Edge Copilot Helper...");

    let current_exe = std::env::current_exe().context("Failed to get current executable path")?;
    let install_dir = paths::install_dir();
    let log_dir = paths::log_dir();
    let binary_path = paths::binary_path();

    // 1. Create directories
    println!("Creating directories...");
    fs::create_dir_all(&install_dir)
        .with_context(|| format!("Failed to create install directory: {:?}", install_dir))?;
    fs::create_dir_all(&log_dir)
        .with_context(|| format!("Failed to create log directory: {:?}", log_dir))?;

    // 2. Copy binary
    println!("Installing binary...");
    fs::copy(&current_exe, &binary_path)
        .with_context(|| format!("Failed to copy binary to {:?}", binary_path))?;

    // 3. Remove existing service if present
    println!("Checking for existing service...");
    let _ = Command::new("sc").args(["stop", SERVICE_NAME]).output();
    let _ = Command::new("sc").args(["delete", SERVICE_NAME]).output();

    // 4. Create service
    println!("Creating Windows service...");
    let bin_path = format!("\"{}\" run", binary_path.to_str().unwrap_or(""));
    let status = Command::new("sc")
        .args([
            "create",
            SERVICE_NAME,
            &format!("binPath={}", bin_path),
            "start=auto",
            &format!("DisplayName={}", DISPLAY_NAME),
        ])
        .status()
        .context("Failed to execute sc create")?;

    if !status.success() {
        anyhow::bail!(
            "Failed to create Windows service. Make sure you're running as Administrator."
        );
    }

    // 5. Configure failure recovery (restart on failure)
    println!("Configuring failure recovery...");
    let _ = Command::new("sc")
        .args([
            "failure",
            SERVICE_NAME,
            "reset=86400",
            "actions=restart/5000/restart/10000/restart/30000",
        ])
        .status();

    // 6. Start service
    println!("Starting service...");
    let status = Command::new("sc")
        .args(["start", SERVICE_NAME])
        .status()
        .context("Failed to execute sc start")?;

    if !status.success() {
        println!(
            "Warning: Service created but failed to start. You may need to start it manually."
        );
    }

    println!();
    println!("Service installed successfully!");
    println!("  Binary: {:?}", binary_path);
    println!("  Service Name: {}", SERVICE_NAME);
    println!();
    println!("Manage with:");
    println!("  sc query {}", SERVICE_NAME);
    println!("  sc stop {}", SERVICE_NAME);
    println!("  sc start {}", SERVICE_NAME);

    Ok(())
}

pub fn uninstall() -> Result<()> {
    println!("Uninstalling Edge Copilot Helper...");

    let install_dir = paths::install_dir();

    // 1. Stop and delete service
    println!("Stopping service...");
    let _ = Command::new("sc").args(["stop", SERVICE_NAME]).output();

    println!("Removing service...");
    let status = Command::new("sc")
        .args(["delete", SERVICE_NAME])
        .status()
        .context("Failed to execute sc delete")?;

    if !status.success() {
        println!("Warning: Failed to delete service. It may not exist or you need Administrator privileges.");
    }

    // 2. Remove install directory (includes binary and logs)
    if install_dir.exists() {
        println!("Removing files: {:?}", install_dir);
        fs::remove_dir_all(&install_dir)
            .with_context(|| format!("Failed to remove {:?}", install_dir))?;
    }

    println!();
    println!("Uninstallation complete.");

    Ok(())
}
