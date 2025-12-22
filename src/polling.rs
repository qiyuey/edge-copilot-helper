use anyhow::Result;
use std::{thread, time::Duration};
use sysinfo::{System, SystemExt, ProcessExt};
use crate::common::apply_fix;

const PROCESS_NAME: &str = "msedge";

pub fn run_polling_loop() -> Result<()> {
    println!("🐧/🪟 Polling Mode: Starting Loop...");
    println!("   Monitoring process: {}", PROCESS_NAME);

    let mut sys = System::new_all();
    let mut was_running = false;

    loop {
        // Refresh processes
        sys.refresh_processes();

        // Check if Edge is running
        // Note: Edge has multiple processes, as long as one is running we consider it open
        let is_running = sys.processes_by_name(PROCESS_NAME).next().is_some();

        if was_running && !is_running {
            println!("🛑 Edge exited. Applying fix...");
            if let Err(e) = apply_fix() {
                eprintln!("❌ Failed to apply fix: {}", e);
            }
        }

        was_running = is_running;
        
        // Low frequency polling
        thread::sleep(Duration::from_secs(2));
    }
}
