use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

pub const APP_LABEL: &str = "top.qiyuey.edge-copilot-helper";
pub const BINARY_NAME: &str = "edge-copilot-helper";

/// 日志文件保留天数
pub const LOG_RETENTION_DAYS: u32 = 7;

/// Edge 浏览器相关标识符
pub mod edge {
    /// macOS 上 Edge 的 Bundle ID 前缀
    #[cfg(target_os = "macos")]
    pub const BUNDLE_ID_PREFIX: &str = "com.microsoft.edgemac";

    /// Windows 上 Edge 进程名
    #[cfg(target_os = "windows")]
    pub const PROCESS_NAMES: &[&str] = &["msedge.exe"];

    /// Linux 上 Edge 进程名列表
    #[cfg(target_os = "linux")]
    pub const PROCESS_NAMES: &[&str] = &[
        "msedge",
        "microsoft-edge",
        "microsoft-edge-stable",
        "microsoft-edge-beta",
        "microsoft-edge-dev",
    ];

    /// 其他平台的默认进程名
    #[cfg(all(
        not(target_os = "windows"),
        not(target_os = "linux"),
        not(target_os = "macos")
    ))]
    pub const PROCESS_NAMES: &[&str] = &["msedge"];
}

/// 清理超过保留天数的旧日志文件
///
/// # 参数
/// - `log_dir`: 日志目录路径
/// - `retention_days`: 保留天数，超过此天数的日志文件将被删除
pub fn cleanup_old_logs(log_dir: &Path, retention_days: u32) {
    let cutoff = SystemTime::now() - Duration::from_secs(u64::from(retention_days) * 24 * 60 * 60);

    if let Ok(entries) = std::fs::read_dir(log_dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            // 只处理 .log 文件
            if path.extension().is_some_and(|ext| ext == "log")
                && let Ok(metadata) = entry.metadata()
                && let Ok(modified) = metadata.modified()
                && modified < cutoff
            {
                let _ = std::fs::remove_file(&path);
            }
        }
    }
}

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
    use super::{APP_LABEL, BINARY_NAME, PathBuf};

    pub fn install_dir() -> PathBuf {
        dirs::data_local_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(APP_LABEL)
    }

    pub fn log_dir() -> PathBuf {
        install_dir().join("logs")
    }

    pub fn binary_path() -> PathBuf {
        install_dir().join(format!("{BINARY_NAME}.exe"))
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
