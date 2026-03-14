# AGENTS.md

This file provides guidance to AI coding agents when working with code in this repository.

## Build Commands

```bash
cargo build --release    # Build release binary
cargo build              # Build debug binary
cargo run                # Run directly (debug mode)
cargo check              # Check compilation without building
cargo clippy             # Run clippy lints
cargo fmt                # Format code
cargo test               # Run tests
```

## Service Management

```bash
# Install as system service (macOS: LaunchAgent, Windows: Registry Run, Linux: systemd)
./edge-copilot-helper install

# Uninstall service and remove all files
./edge-copilot-helper uninstall

# Run in foreground (console output)
./edge-copilot-helper run

# Run in background (daemon mode, file logging only)
./edge-copilot-helper daemon

# Default command (no args) shows help
./edge-copilot-helper

# View service logs (macOS)
tail -f ~/Library/Logs/top.qiyuey.edge-copilot-helper/service.log
```

## Architecture

This is a cross-platform Rust utility that monitors Microsoft Edge and modifies its configuration files when Edge exits to bypass Copilot region restrictions. It applies two fixes:
1. Sets `variations_country` to `"SG"` in `Local State`
2. Sets `chat_ip_eligibility_status` to `true` in each profile's `Preferences`

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
1. Locates Edge configuration files (handles multiple Edge channels: Stable, Beta, Dev, Canary)
2. Patches `variations_country` to `"SG"` in `Local State`
3. Sets `chat_ip_eligibility_status` to `true` in each profile's `Preferences`
4. Writes back only if modifications were actually made

### Platform Constants (`constants.rs`)

Defines platform-specific paths via conditional compilation (`#[cfg(target_os = "...")]`):
- `paths::install_dir()` — Where binary is installed
- `paths::log_dir()` — Where logs are stored
- `paths::binary_path()` — Full path to installed binary
- Platform-specific: `plist_path()` (macOS), `unit_path()` (Linux)

Also contains `cleanup_old_logs()` which removes log files older than `LOG_RETENTION_DAYS` (7 days).

### Service Module (`service/`)

Each platform has its own service installer:
- `macos.rs` — LaunchAgent plist generation and `launchctl` commands
- `windows.rs` — Registry `HKCU\Run` entry for auto-start on login
- `linux.rs` — systemd user service unit file
