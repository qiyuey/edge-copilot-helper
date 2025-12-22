mod common;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(target_os = "macos"))]
mod polling;

use anyhow::Result;

fn main() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        macos::run_event_loop()
    }

    #[cfg(not(target_os = "macos"))]
    {
        polling::run_polling_loop()
    }
}

