# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build Commands

```bash
# Build release binary
cargo build --release

# Build debug binary
cargo build

# Run directly (debug mode)
cargo run

# Check compilation without building
cargo check

# Run clippy lints
cargo clippy

# Format code
cargo fmt

# Run tests
cargo test
```

## Service Management

```bash
# Install as system service (macOS: LaunchAgent, Windows: SCM, Linux: systemd)
./edge-copilot-helper install

# Uninstall service and remove all files
./edge-copilot-helper uninstall

# Run directly (foreground)
./edge-copilot-helper run
./edge-copilot-helper        # 'run' is default

# View service logs (macOS)
tail -f ~/Library/Logs/top.qiyuey.edge-copilot-helper/service.log
```

## Architecture

This is a cross-platform Rust utility that monitors Microsoft Edge and modifies its preferences file when Edge exits (replacing "CN" region values with "SG" to bypass Copilot region restrictions).

### Platform-Specific Monitoring

- **macOS** (`macos.rs`): Uses native NSWorkspace notification center via `objc2` bindings to listen for `NSWorkspaceDidTerminateApplicationNotification`. Zero CPU usage while waiting.
- **Windows/Linux** (`polling.rs`): Uses `sysinfo` crate for 2-second polling to detect Edge process termination.

### Conditional Compilation

The project uses `#[cfg(target_os = "...")]` extensively:
- `main.rs` dispatches to either `macos::run_event_loop()` or `polling::run_polling_loop()`
- Platform-specific dependencies are declared conditionally in `Cargo.toml`
- `common.rs` has platform-specific preference file paths

### Core Logic (`common.rs`)

`apply_fix()` is the shared entry point called when Edge exits:
1. Locates Edge preferences files (handles multiple Edge channels: Stable, Beta, Dev, Canary)
2. Recursively traverses JSON to find all string values equal to "CN"
3. Replaces them with "SG" and writes back only if modified

### Platform Constants (`constants.rs`)

Defines platform-specific paths via conditional compilation (`#[cfg(target_os = "...")]`):
- `paths::install_dir()` - Where binary is installed
- `paths::log_dir()` - Where logs are stored
- `paths::binary_path()` - Full path to installed binary
- Platform-specific: `plist_path()` (macOS), `unit_path()` (Linux)

### Service Module (`service/`)

Each platform has its own service installer:
- `macos.rs` - LaunchAgent plist generation and launchctl commands
- `windows.rs` - Windows Service Control Manager (sc.exe)
- `linux.rs` - systemd user service unit file
