---
name: build-and-install
description: Build the Rust project in release mode and install it as a system service. Use when the user asks to compile, build, install, deploy, or update the service.
---

# Build and Install Service

编译 release 二进制文件并安装为系统服务。遇到错误时**仅报告错误信息，不要尝试修复**。

## Workflow

### Step 1: Build

Run in the project root:

```bash
cargo build --release
```

- If the build fails, **stop immediately** and report the full error output to the user.
- Do **NOT** attempt to fix compilation errors.

### Step 2: Install

After a successful build, run the install command:

- **Windows**: `./target/release/edge-copilot-helper.exe install`
- **macOS / Linux**: `./target/release/edge-copilot-helper install`

- If install fails, **stop immediately** and report the full error output to the user.
- Do **NOT** attempt to fix install errors.

### Step 3: macOS Permission Check

On macOS, the install command automatically runs the installed helper with:

```bash
request-permissions
```

This opens Privacy & Security and reveals the installed binary so the user can grant Full Disk Access for Edge application data. If the user reports that permission was granted, restart the LaunchAgent or rerun install before verifying with an Edge quit event.

## Rules

1. **No auto-fix**: If any step produces an error, present the complete error output and stop. Never modify source code or configuration to resolve errors.
2. **Report clearly**: When reporting errors, include the full command output so the user can diagnose the issue.
3. **Success message**: If both steps succeed, confirm that the build and installation completed successfully.
