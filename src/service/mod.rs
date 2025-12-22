#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "linux")]
mod linux;

use anyhow::Result;

pub fn install() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        macos::install()
    }

    #[cfg(target_os = "windows")]
    {
        windows::install()
    }

    #[cfg(target_os = "linux")]
    {
        linux::install()
    }
}

pub fn uninstall() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        macos::uninstall()
    }

    #[cfg(target_os = "windows")]
    {
        windows::uninstall()
    }

    #[cfg(target_os = "linux")]
    {
        linux::uninstall()
    }
}
