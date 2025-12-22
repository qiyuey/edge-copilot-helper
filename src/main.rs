mod common;
mod constants;
mod service;

#[cfg(target_os = "macos")]
mod macos;
#[cfg(not(target_os = "macos"))]
mod polling;

use anyhow::Result;
use clap::{Parser, Subcommand};

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
    /// Run the service (default)
    Run,
    /// Install as system service
    Install,
    /// Uninstall the system service
    Uninstall,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command.unwrap_or(Command::Run) {
        Command::Run => run_service(),
        Command::Install => service::install(),
        Command::Uninstall => service::uninstall(),
    }
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
