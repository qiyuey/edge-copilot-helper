#![cfg(not(target_os = "macos"))]

use anyhow::Result;
use std::{thread, time::Duration};
use sysinfo::System;

use crate::common::apply_fix;
use crate::constants::edge::PROCESS_NAMES;

/// 运行轮询监控循环
///
/// 在 Windows 和 Linux 平台上使用，每 2 秒检查一次 Edge 进程状态。
/// 当检测到 Edge 退出时，自动应用配置修复。
pub fn run_polling_loop() -> Result<()> {
    log::info!("🐧/🪟 Polling Mode: Starting Loop...");
    let process_list = PROCESS_NAMES.join(", ");
    log::info!("   Monitoring process: {process_list}");

    let mut sys = System::new();
    let mut was_running = false;

    loop {
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        // Check if any Edge process exists
        let is_running = sys.processes().values().any(|process| {
            let pname = process.name().to_string_lossy();
            PROCESS_NAMES.iter().any(|&n| n == pname)
        });

        if was_running && !is_running {
            log::info!("🛑 Edge exited. Applying fix...");
            if let Err(e) = apply_fix() {
                log::error!("❌ Failed to apply fix: {e}");
            }
        }

        was_running = is_running;
        thread::sleep(Duration::from_secs(2));
    }
}
