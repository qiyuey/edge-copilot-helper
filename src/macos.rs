use anyhow::{Context, Result};
use block2::RcBlock;
use objc2::rc::Retained;
use objc2_app_kit::{
    NSRunningApplication, NSWorkspace, NSWorkspaceApplicationKey,
    NSWorkspaceDidTerminateApplicationNotification,
};
use objc2_foundation::{NSNotification, NSRunLoop};
use std::fs;
use std::io;
use std::path::Path;
use std::process::Command;
use std::ptr::NonNull;

use crate::common::apply_fix;
use crate::constants::edge::BUNDLE_ID_PREFIX;

/// 运行 macOS 事件循环
///
/// 使用 NSWorkspace 通知中心监听应用程序终止事件。
/// 当检测到 Edge 退出时，自动应用配置修复。
/// 此方法使用原生事件机制，零 CPU 占用。
pub fn run_event_loop() -> Result<()> {
    log::info!("🍎 macOS Mode: Starting Event Loop...");
    log::info!("   Monitoring for: Microsoft Edge");

    unsafe {
        let workspace = NSWorkspace::sharedWorkspace();
        let center = workspace.notificationCenter();

        let handler = RcBlock::new(|note: NonNull<NSNotification>| {
            let note = note.as_ref();

            if let Some(user_info) = note.userInfo() {
                let app_obj = user_info.objectForKey(NSWorkspaceApplicationKey);

                if let Some(obj) = app_obj {
                    // Safety: NSWorkspaceApplicationKey guarantees the value is NSRunningApplication
                    let app: Retained<NSRunningApplication> = Retained::cast_unchecked(obj);

                    if let Some(bundle_id) = app.bundleIdentifier() {
                        let bid = bundle_id.to_string();
                        if bid.contains(BUNDLE_ID_PREFIX) {
                            log::info!("🛑 Edge termination detected.");
                            if let Err(e) = apply_fix() {
                                log::error!("❌ Failed to apply fix: {}", e);
                            }
                        }
                    }
                }
            }
        });

        center.addObserverForName_object_queue_usingBlock(
            Some(NSWorkspaceDidTerminateApplicationNotification),
            None,
            None,
            &handler,
        );

        NSRunLoop::currentRunLoop().run();
    }
    Ok(())
}

/// 主动触发 macOS 对 Edge 应用数据访问的权限检查，并打开授权入口。
///
/// macOS 27 会把读取其他 App 的数据归到 `SystemPolicyAppDataDetailed`。
/// 程序无法自行授予该权限；这里通过访问 Edge 配置文件让 TCC 建立记录，
/// 然后打开 Privacy & Security，并在 Finder 中选中当前二进制，方便用户添加。
pub fn request_app_data_permission() -> Result<()> {
    let binary_path = std::env::current_exe().context("Failed to get current executable path")?;

    log::info!("Checking permission to access Microsoft Edge application data...");
    match preflight_edge_app_data_access() {
        Ok(AccessCheck::Granted) => {
            log::info!("Foreground permission check passed.");
            log::info!(
                "If the LaunchAgent is still denied, grant Full Disk Access to this helper."
            );
        }
        Ok(AccessCheck::NoEdgeData) => {
            log::warn!("Microsoft Edge data was not found in the standard location.");
            log::warn!("If Edge is installed, open it once before running this check again.");
        }
        Err(err) if is_permission_error(&err) => {
            log::warn!("macOS blocked access to Microsoft Edge application data.");
            log::warn!("Reason: {err}");
        }
        Err(err) => return Err(err),
    }

    open_privacy_settings();
    reveal_binary(&binary_path);
    print_manual_permission_steps(&binary_path);

    Ok(())
}

enum AccessCheck {
    Granted,
    NoEdgeData,
}

fn preflight_edge_app_data_access() -> Result<AccessCheck> {
    let home = dirs::home_dir().context("Could not determine home directory")?;
    let edge_dir = home.join("Library/Application Support/Microsoft Edge");
    let local_state = edge_dir.join("Local State");
    let default_preferences = edge_dir.join("Default/Preferences");

    if !edge_dir.exists() {
        return Ok(AccessCheck::NoEdgeData);
    }

    fs::read_dir(&edge_dir).with_context(|| {
        format!(
            "Failed to read Edge data directory at {}",
            edge_dir.display()
        )
    })?;

    read_if_exists(&local_state, "Local State")?;
    read_if_exists(&default_preferences, "Default profile Preferences")?;

    Ok(AccessCheck::Granted)
}

fn read_if_exists(path: &Path, label: &str) -> Result<()> {
    if path.exists() {
        fs::read_to_string(path)
            .with_context(|| format!("Failed to read {label} at {}", path.display()))?;
    }
    Ok(())
}

fn is_permission_error(err: &anyhow::Error) -> bool {
    err.chain().any(|cause| {
        cause.downcast_ref::<io::Error>().is_some_and(|io_err| {
            io_err.kind() == io::ErrorKind::PermissionDenied || io_err.raw_os_error() == Some(1)
        })
    })
}

fn open_privacy_settings() {
    let targets = [
        "x-apple.systempreferences:com.apple.preference.security?Privacy_AllFiles",
        "x-apple.systempreferences:com.apple.preference.security",
    ];

    for target in targets {
        if Command::new("open")
            .arg(target)
            .status()
            .is_ok_and(|status| status.success())
        {
            return;
        }
    }

    log::warn!("Could not open Privacy & Security settings automatically.");
}

fn reveal_binary(binary_path: &Path) {
    if let Err(err) = Command::new("open").arg("-R").arg(binary_path).status() {
        log::warn!("Could not reveal binary in Finder: {err}");
    }
}

fn print_manual_permission_steps(binary_path: &Path) {
    log::info!("");
    log::info!("Grant permission manually:");
    log::info!("  1. System Settings → Privacy & Security → Full Disk Access");
    log::info!("  2. Click +");
    log::info!("  3. Select this binary:");
    log::info!("     {}", binary_path.display());
    log::info!("  4. Restart the service or run install again");
    log::info!("");
}
