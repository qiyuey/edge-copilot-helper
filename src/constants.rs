use std::path::PathBuf;

pub const APP_LABEL: &str = "top.qiyuey.edge-copilot-helper";
pub const BINARY_NAME: &str = "edge-copilot-helper";

#[cfg(target_os = "macos")]
pub mod paths {
    use super::*;

    pub fn install_dir() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("~"))
            .join(APP_LABEL)
    }

    pub fn log_dir() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("~"))
            .join("Library/Logs")
            .join(APP_LABEL)
    }

    pub fn plist_path() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("~"))
            .join("Library/LaunchAgents")
            .join(format!("{}.plist", APP_LABEL))
    }

    pub fn binary_path() -> PathBuf {
        install_dir().join(BINARY_NAME)
    }
}

#[cfg(target_os = "windows")]
pub mod paths {
    use super::*;

    pub fn install_dir() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(APP_LABEL)
    }

    pub fn log_dir() -> PathBuf {
        install_dir().join("logs")
    }

    pub fn binary_path() -> PathBuf {
        install_dir().join(format!("{}.exe", BINARY_NAME))
    }
}

#[cfg(target_os = "linux")]
pub mod paths {
    use super::*;

    pub fn install_dir() -> PathBuf {
        dirs::data_dir()
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("~"))
                    .join(".local/share")
            })
            .join(APP_LABEL)
    }

    pub fn log_dir() -> PathBuf {
        install_dir().join("logs")
    }

    pub fn unit_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| {
                dirs::home_dir()
                    .unwrap_or_else(|| PathBuf::from("~"))
                    .join(".config")
            })
            .join("systemd/user")
            .join(format!("{}.service", APP_LABEL))
    }

    pub fn binary_path() -> PathBuf {
        install_dir().join(BINARY_NAME)
    }
}
