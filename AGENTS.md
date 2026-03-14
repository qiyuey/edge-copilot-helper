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

This is a cross-platform Rust utility that monitors Microsoft Edge and modifies its configuration files when Edge exits to bypass Copilot region restrictions. It applies fixes to two files:

**Seed files (User Data directory):**
1. Deletes `VariationsSeedV2`, `VariationsSafeSeedV2`, `VariationsRuntimeSeedV2` — these binary files embed the country code and cause Edge to regenerate Copilot disable flags

**Local State:**
2. Sets `variations_country` to `TARGET_COUNTRY` (currently `"SG"`)
3. Sets `variations_safe_seed_session_consistency_country` to `TARGET_COUNTRY`
4. Removes `disablecopilotmodeenp` and other Copilot disable flags from `variations_config_ids`
5. Clears seed metadata fields (`variations_seed_date`, `variations_seed_etag`, etc.) to prevent stale seed loading

**Each profile's Preferences:**
6. Sets `chat_ip_eligibility_status` to `true`
7. Enables the Copilot sidebar app (UUID `cd4688a9-...`) if it was hidden (value 0 → 1)

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
2. Deletes variations seed files from User Data directory (binary files that embed the CN country code)
3. Patches `Local State`: `variations_country`, `variations_safe_seed_session_consistency_country`, removes Copilot disable flags from `variations_config_ids`, and clears seed metadata
4. Patches each profile's `Preferences`: `chat_ip_eligibility_status` and Copilot sidebar visibility
5. Writes back only if modifications were actually made

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
