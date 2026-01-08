mod common;
mod constants;
mod service;

#[cfg(target_os = "windows")]
mod logger;

#[cfg(target_os = "windows")]
use std::ffi::OsStr;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(target_os = "macos"))]
mod polling;

use anyhow::Result;
use clap::{Parser, Subcommand};
use std::fs::OpenOptions;

#[derive(Parser)]
#[command(name = "edge-copilot-helper")]
#[command(about = "Cross-platform utility to bypass Microsoft Edge Copilot region restrictions")]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Subcommand)]
enum Command {
    /// Show help information
    Help,
    /// Show version information
    Version,
    /// Run the service in foreground (with console output)
    Run,
    /// Run the service in background (daemon mode, file logging only)
    Daemon,
    /// Install as system service
    Install,
    /// Uninstall the system service
    Uninstall,
}

fn main() -> Result<()> {
    // 使用 try_parse 捕获 clap 的 help/version 自动处理，先确保控制台已附着
    let cli = match Cli::try_parse() {
        Ok(cli) => cli,
        Err(err) => {
            #[cfg(target_os = "windows")]
            {
                ensure_console();
            }
            err.exit();
        }
    };

    // 默认执行 help
    let command = cli.command.unwrap_or(Command::Help);

    match command {
        Command::Help => show_help(),
        Command::Version => show_version(),
        Command::Run => {
            // run 命令：只输出到控制台
            #[cfg(target_os = "windows")]
            {
                ensure_console();
                logger::init_console_logger().unwrap_or_default();
            }
            #[cfg(not(target_os = "windows"))]
            {
                init_console_logger();
            }

            let _lock = acquire_single_instance_lock()?;
            run_service()
        }
        Command::Daemon => {
            // daemon 命令：只输出到日志文件（无控制台窗口）
            #[cfg(target_os = "windows")]
            {
                detach_console();
                logger::init_file_logger().unwrap_or_default();
            }
            #[cfg(not(target_os = "windows"))]
            {
                init_file_logger();
            }

            let _lock = acquire_single_instance_lock()?;
            run_service()
        }
        Command::Install => {
            // install 命令：只输出到控制台
            #[cfg(target_os = "windows")]
            {
                ensure_console();
                logger::init_console_logger().unwrap_or_default();
            }
            #[cfg(not(target_os = "windows"))]
            {
                init_console_logger();
            }

            service::install()
        }
        Command::Uninstall => {
            // uninstall 命令：只输出到控制台
            #[cfg(target_os = "windows")]
            {
                ensure_console();
                logger::init_console_logger().unwrap_or_default();
            }
            #[cfg(not(target_os = "windows"))]
            {
                init_console_logger();
            }

            service::uninstall()
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn init_file_logger() {
    use crate::constants::{paths, LOG_RETENTION_DAYS};
    use simplelog::{Config, LevelFilter, WriteLogger};
    use std::fs::OpenOptions;

    let log_dir = paths::log_dir();
    let config = Config::default();

    // 只写入文件
    if let Ok(_) = std::fs::create_dir_all(&log_dir) {
        // 清理旧日志文件
        cleanup_old_logs(&log_dir, LOG_RETENTION_DAYS);

        let log_file = log_dir.join(format!(
            "edge-copilot-helper-{}.log",
            chrono::Local::now().format("%Y%m%d")
        ));

        if let Ok(file) = OpenOptions::new().create(true).append(true).open(&log_file) {
            let _ = WriteLogger::init(LevelFilter::Info, config, file);
        }
    }
}

/// 清理超过保留天数的旧日志文件
fn cleanup_old_logs(log_dir: &std::path::Path, retention_days: u32) {
    use std::fs;
    use std::time::{Duration, SystemTime};

    let cutoff = SystemTime::now() - Duration::from_secs(retention_days as u64 * 24 * 60 * 60);

    if let Ok(entries) = fs::read_dir(log_dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            // 只处理 .log 文件
            if path.extension().map_or(false, |ext| ext == "log") {
                if let Ok(metadata) = entry.metadata() {
                    // 使用文件修改时间判断是否过期
                    if let Ok(modified) = metadata.modified() {
                        if modified < cutoff {
                            let _ = fs::remove_file(&path);
                        }
                    }
                }
            }
        }
    }
}

#[cfg(not(target_os = "windows"))]
fn init_console_logger() {
    use simplelog::{ColorChoice, Config, LevelFilter, TermLogger, TerminalMode};

    let config = Config::default();

    // 只输出到控制台
    let _ = TermLogger::init(
        LevelFilter::Info,
        config,
        TerminalMode::Mixed,
        ColorChoice::Auto,
    );
}

fn show_help() -> Result<()> {
    // 如果是 help 命令或默认行为，需要控制台来显示 help
    #[cfg(target_os = "windows")]
    {
        ensure_console();
    }

    use clap::CommandFactory;
    use std::io::Write;
    let mut cmd = Cli::command();
    // 将 help 输出到 buffer
    let mut buffer = Vec::new();
    cmd.write_help(&mut buffer)?;
    let help_text = String::from_utf8_lossy(&buffer);
    // 输出 help 信息
    println!("{}", help_text);
    // 刷新输出流
    let _ = std::io::stdout().flush();
    let _ = std::io::stderr().flush();
    Ok(())
}

fn show_version() -> Result<()> {
    #[cfg(target_os = "windows")]
    {
        ensure_console();
    }

    use clap::CommandFactory;
    let cmd = Cli::command();
    println!("{}", cmd.render_version());
    Ok(())
}

#[cfg(target_os = "windows")]
fn ensure_console() {
    use std::os::windows::ffi::OsStrExt;
    use winapi::um::consoleapi::AllocConsole;
    use winapi::um::fileapi::{CreateFileW, OPEN_EXISTING};
    use winapi::um::handleapi::INVALID_HANDLE_VALUE;
    use winapi::um::processenv::SetStdHandle;
    use winapi::um::winbase::{STD_ERROR_HANDLE, STD_INPUT_HANDLE, STD_OUTPUT_HANDLE};
    use winapi::um::wincon::{ATTACH_PARENT_PROCESS, AttachConsole};
    use winapi::um::winnt::{FILE_SHARE_READ, FILE_SHARE_WRITE, GENERIC_READ, GENERIC_WRITE};

    unsafe {
        // 尝试附加到父进程控制台，失败则新建
        if AttachConsole(ATTACH_PARENT_PROCESS) == 0 {
            AllocConsole();
        }

        let wide = |s: &str| {
            OsStr::new(s)
                .encode_wide()
                .chain(std::iter::once(0))
                .collect::<Vec<u16>>()
        };

        let conout = CreateFileW(
            wide("CONOUT$").as_ptr(),
            GENERIC_WRITE | GENERIC_READ,
            FILE_SHARE_WRITE | FILE_SHARE_READ,
            std::ptr::null_mut(),
            OPEN_EXISTING,
            0,
            std::ptr::null_mut(),
        );
        if conout != INVALID_HANDLE_VALUE {
            SetStdHandle(STD_OUTPUT_HANDLE, conout);
            SetStdHandle(STD_ERROR_HANDLE, conout);
        }

        let conin = CreateFileW(
            wide("CONIN$").as_ptr(),
            GENERIC_READ | GENERIC_WRITE,
            FILE_SHARE_READ | FILE_SHARE_WRITE,
            std::ptr::null_mut(),
            OPEN_EXISTING,
            0,
            std::ptr::null_mut(),
        );
        if conin != INVALID_HANDLE_VALUE {
            SetStdHandle(STD_INPUT_HANDLE, conin);
        }
    }
}

#[cfg(target_os = "windows")]
fn detach_console() {
    use winapi::um::wincon::FreeConsole;
    unsafe {
        FreeConsole();
    }
}

fn acquire_single_instance_lock() -> Result<std::fs::File> {
    use crate::constants::paths;
    use fs2::FileExt;

    let install_dir = paths::install_dir();
    std::fs::create_dir_all(&install_dir)?;

    let lock_path = install_dir.join("edge-copilot-helper.lock");
    let file = OpenOptions::new()
        .create(true)
        .read(true)
        .write(true)
        .open(&lock_path)?;

    file.try_lock_exclusive()
        .map_err(|_| anyhow::anyhow!("Another instance is already running"))?;

    Ok(file)
}

fn run_service() -> Result<()> {
    #[cfg(target_os = "macos")]
    {
        macos::run_event_loop()
    }

    #[cfg(not(target_os = "macos"))]
    {
        polling::run_polling_loop()
    }
}
