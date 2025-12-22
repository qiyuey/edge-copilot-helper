#![cfg_attr(not(target_os = "macos"), allow(dead_code))]

#[cfg(target_os = "macos")]
mod inner {
    use anyhow::Result;
    use block2::RcBlock;
    use objc2::rc::Retained;
    use objc2_app_kit::{
        NSRunningApplication, NSWorkspace, NSWorkspaceApplicationKey,
        NSWorkspaceDidTerminateApplicationNotification,
    };
    use objc2_foundation::{NSNotification, NSRunLoop};
    use std::ptr::NonNull;

    use crate::common::apply_fix;

    pub fn run_event_loop() -> Result<()> {
        println!("🍎 macOS Mode: Starting Event Loop...");
        println!("   Monitoring for: Microsoft Edge");

        unsafe {
            let workspace = NSWorkspace::sharedWorkspace();
            let center = workspace.notificationCenter();

            let handler = RcBlock::new(|note: NonNull<NSNotification>| {
                let note = note.as_ref();

                if let Some(user_info) = note.userInfo() {
                    let app_obj = user_info.objectForKey(NSWorkspaceApplicationKey);

                    if let Some(obj) = app_obj {
                        // Safety: NSWorkspaceApplicationKey guarantees the value is NSRunningApplication
                        let app: Retained<NSRunningApplication> =
                            Retained::cast_unchecked(obj);

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
}

#[cfg(target_os = "macos")]
pub use inner::run_event_loop;

#[cfg(not(target_os = "macos"))]
pub fn run_event_loop() -> anyhow::Result<()> {
    Ok(())
}
