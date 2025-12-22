#![cfg(not(target_os = "macos"))]

use anyhow::Result;
use std::{thread, time::Duration};
use sysinfo::System;

use crate::common::apply_fix;

#[cfg(target_os = "windows")]
const PROCESS_NAMES: &[&str] = &["msedge.exe", "msedge"];

#[cfg(target_os = "linux")]
const PROCESS_NAMES: &[&str] = &[
    "msedge",
    "microsoft-edge",
    "microsoft-edge-stable",
    "microsoft-edge-beta",
    "microsoft-edge-dev",
];

#[cfg(all(not(target_os = "windows"), not(target_os = "linux")))]
const PROCESS_NAMES: &[&str] = &["msedge"];

pub fn run_polling_loop() -> Result<()> {
    println!("🐧/🪟 Polling Mode: Starting Loop...");
    println!("   Monitoring process: {}", PROCESS_NAMES.join(", "));

    let mut sys = System::new();
    let mut was_running = false;

    loop {
        sys.refresh_processes(sysinfo::ProcessesToUpdate::All, true);

        // Check if Edge is running (multiple processes, any one means Edge is open)
        let is_running = PROCESS_NAMES.iter().any(|name| {
            sys.processes()
                .values()
                .any(|p| p.name().to_string_lossy() == *name)
        });

        if was_running && !is_running {
            println!("🛑 Edge exited. Applying fix...");
            if let Err(e) = apply_fix() {
                eprintln!("❌ Failed to apply fix: {}", e);
            }
        }

        was_running = is_running;
        thread::sleep(Duration::from_secs(2));
    }
}
