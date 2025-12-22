#[cfg(target_os = "macos")]
use anyhow::Result;
#[cfg(target_os = "macos")]
use std::ptr::NonNull;
#[cfg(target_os = "macos")]
use objc2_foundation::{NSNotification, NSRunLoop};
#[cfg(target_os = "macos")]
use objc2_app_kit::{NSWorkspace, NSRunningApplication, NSWorkspaceDidTerminateApplicationNotification, NSWorkspaceApplicationKey};
#[cfg(target_os = "macos")]
use block2::RcBlock;
#[cfg(target_os = "macos")]
use crate::common::apply_fix;

#[cfg(target_os = "macos")]
pub fn run_event_loop() -> Result<()> {
    println!("🍎 macOS Mode: Starting Event Loop...");
    println!("   Monitoring for: Microsoft Edge");

    unsafe {
        let workspace = NSWorkspace::sharedWorkspace();
        let center = workspace.notificationCenter();

        // Define callback block
        let handler = RcBlock::new(|note: NonNull<NSNotification>| {
            // Safety: Convert NonNull to reference
            let note = note.as_ref();
            
            if let Some(user_info) = note.userInfo() {
                // Get the application object using the specific key constant
                let app_obj = user_info.objectForKey(NSWorkspaceApplicationKey);
                
                if let Some(obj) = app_obj {
                    // Cast to NSRunningApplication
                    let app: &NSRunningApplication = std::mem::transmute(obj);
                    
                    if let Some(bundle_id) = app.bundleIdentifier() {
                        let bid = bundle_id.to_string();
                        if bid.contains("com.microsoft.edgemac") {
                             println!("🛑 Edge termination detected.");
                             if let Err(e) = apply_fix() {
                                 eprintln!("❌ Failed to apply fix: {}", e);
                             }
                        }
                    }
                }
            }
        });

        // Register observer
        center.addObserverForName_object_queue_usingBlock(
            Some(NSWorkspaceDidTerminateApplicationNotification), 
            None, // object
            None, // queue (nil means default/current)
            &handler
        );

        // Start RunLoop
        NSRunLoop::currentRunLoop().run();
    }
    Ok(())
}

// Dummy implementation for non-macOS
#[cfg(not(target_os = "macos"))]
pub fn run_event_loop() -> anyhow::Result<()> {
    Ok(())
}
