#![cfg(not(target_os = "macos"))]

use anyhow::Result;
use std::{thread, time::Duration};
use sysinfo::System;

use crate::common::apply_fix;

#[cfg(target_os = "windows")]
const PROCESS_NAMES: &[&str] = &["msedge.exe"];

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
    let process_list = PROCESS_NAMES.iter().copied().collect::<Vec<_>>().join(", ");
    println!("   Monitoring process: {process_list}");

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
            println!("🛑 Edge exited. Applying fix...");
            if let Err(e) = apply_fix() {
                eprintln!("❌ Failed to apply fix: {e}");
            }
        }

        was_running = is_running;
        thread::sleep(Duration::from_secs(2));
    }
}
