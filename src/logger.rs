#![cfg(target_os = "windows")]

use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger};
use std::fs::OpenOptions;
use std::path::Path;
use std::time::{Duration, SystemTime};

use crate::constants::{LOG_RETENTION_DAYS, paths};

pub fn init_file_logger() -> Result<(), Box<dyn std::error::Error>> {
    let log_dir = paths::log_dir();
    std::fs::create_dir_all(&log_dir)?;

    // 清理旧日志文件
    cleanup_old_logs(&log_dir, LOG_RETENTION_DAYS);

    let log_file = log_dir.join(format!(
        "edge-copilot-helper-{}.log",
        chrono::Local::now().format("%Y%m%d")
    ));

    let file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_file)?;

    let config = Config::default();

    // 只写入文件
    WriteLogger::init(LevelFilter::Info, config, file)?;

    Ok(())
}

/// 清理超过保留天数的旧日志文件
fn cleanup_old_logs(log_dir: &Path, retention_days: u32) {
    let cutoff = SystemTime::now() - Duration::from_secs(retention_days as u64 * 24 * 60 * 60);

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

pub fn init_console_logger() -> Result<(), Box<dyn std::error::Error>> {
    let config = Config::default();

    // 只输出到控制台
    TermLogger::init(
        LevelFilter::Info,
        config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    )?;

    Ok(())
}
