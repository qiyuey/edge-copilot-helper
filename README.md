# Edge Copilot Helper

A cross-platform utility that automatically bypasses Microsoft Edge Copilot region restrictions. It monitors the Edge browser and patches its configuration files when Edge exits, ensuring Copilot features remain accessible regardless of your geographic region.

## How It Works

Edge stores region information in its local configuration files. In certain regions (e.g., China), Copilot features are restricted. This tool watches for Edge process termination and applies two fixes:

1. **`variations_country`** in `Local State` is set to `"US"`
2. **`chat_ip_eligibility_status`** in each profile's `Preferences` is set to `true`

Changes are only written when necessary — if values are already correct, the files are left untouched.

## Supported Platforms

| Platform | Monitoring Strategy | Service Type |
|----------|-------------------|--------------|
| **macOS** | NSWorkspace notifications (zero CPU idle) | LaunchAgent |
| **Windows** | 2-second polling via `sysinfo` | Registry startup (`HKCU\Run`) |
| **Linux** | 2-second polling via `sysinfo` | systemd user service |

All Edge channels are supported: **Stable**, **Beta**, **Dev**, and **Canary**.

## Installation

### From Source

```bash
cargo build --release
```

The binary will be at `target/release/edge-copilot-helper` (or `.exe` on Windows).

### Install as Service

```bash
./edge-copilot-helper install
```

This will:
- Copy the binary to a platform-specific install directory
- Register it to start automatically on login
- Start the service immediately (macOS/Linux)

| Platform | Install Directory | Auto-start Mechanism |
|----------|------------------|---------------------|
| macOS | `~/Library/Application Support/top.qiyuey.edge-copilot-helper/` | LaunchAgent plist |
| Windows | `%LOCALAPPDATA%\top.qiyuey.edge-copilot-helper\` | `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` |
| Linux | `~/.local/share/top.qiyuey.edge-copilot-helper/` | systemd user service |

## Usage

```
edge-copilot-helper [COMMAND]
```

| Command | Description |
|---------|-------------|
| `help` | Show help information (default) |
| `version` | Show version |
| `run` | Run in foreground with console output |
| `daemon` | Run in background with file logging only |
| `install` | Install as a system service |
| `uninstall` | Uninstall the service and remove all files |

### Run in Foreground

```bash
edge-copilot-helper run
```

Useful for testing — logs are printed directly to the console.

### Run as Daemon

```bash
edge-copilot-helper daemon
```

Runs silently in the background with output directed to log files. This is the mode used by the installed service.

## Logs

Log files are stored in the platform-specific log directory and rotated daily. Files older than 7 days are automatically cleaned up.

| Platform | Log Directory |
|----------|--------------|
| macOS | `~/Library/Logs/top.qiyuey.edge-copilot-helper/` |
| Windows | `%LOCALAPPDATA%\top.qiyuey.edge-copilot-helper\logs\` |
| Linux | `~/.local/share/top.qiyuey.edge-copilot-helper/logs/` |

**Viewing logs:**

```bash
# macOS
tail -f ~/Library/Logs/top.qiyuey.edge-copilot-helper/service.log

# Linux
journalctl --user -u top.qiyuey.edge-copilot-helper -f

# Windows (PowerShell)
Get-Content "$env:LOCALAPPDATA\top.qiyuey.edge-copilot-helper\logs\edge-copilot-helper-*.log" -Tail 50
```

## Uninstall

```bash
./edge-copilot-helper uninstall
```

This will stop the running service, remove the startup registration, and delete all installed files including logs.

## Building

```bash
cargo build --release    # Release binary
cargo check              # Check compilation
cargo test               # Run tests
cargo clippy             # Lint
cargo fmt                # Format code
```

## License

[Anti-996](https://github.com/996icu/996.ICU/blob/master/LICENSE)
