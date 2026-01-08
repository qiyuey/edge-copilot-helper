#![cfg(target_os = "windows")]

use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode, WriteLogger};
use std::fs::OpenOptions;

use crate::constants::{LOG_RETENTION_DAYS, cleanup_old_logs, paths};

/// 初始化文件日志记录器（仅输出到日志文件）
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

/// 初始化控制台日志记录器（仅输出到终端）
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
