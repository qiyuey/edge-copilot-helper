# Edge Copilot Helper

[![996.icu](https://img.shields.io/badge/link-996.icu-red.svg)](https://996.icu)
[![Anti-996 License](https://img.shields.io/badge/License-Anti%20996-blue.svg)](https://github.com/996icu/996.ICU/blob/master/LICENSE)

一个跨平台工具，用于自动修正 Microsoft Edge 配置文件，绕过 Copilot 的地区限制。

## ✨ 特性

- 🌍 **跨平台支持**：macOS (ARM64)、Windows (x64)、Linux (x64)
- 🚀 **高效监控**：
  - macOS：使用 NSWorkspace API 原生监听应用退出事件，零 CPU 占用
  - Windows/Linux：使用 sysinfo 进行低频轮询监控进程状态
- 🔧 **自动修复**：Edge 退出时自动修改配置文件
- 📦 **多版本支持**：自动检测并修复所有 Edge 版本（Stable、Beta、Dev、Canary）
- 🔄 **多配置文件支持**：自动处理所有用户配置文件（Default、Profile 1、Profile 2 等）
- 🛠️ **系统服务**：支持安装为系统服务，实现开机自启
- 📝 **详细日志**：记录所有操作，便于排查问题

## 📋 工作原理

当 Microsoft Edge 退出时，程序会：

1. **检测退出事件**：通过系统 API 或轮询检测 Edge 进程退出
2. **读取配置文件**：
   - `Local State`：修改 `variations_country` 为 `"US"`
   - `Preferences`：设置 `browser.chat_ip_eligibility_status` 为 `true`
3. **保存修改**：将修改后的配置写回文件

这些修改使得 Edge Copilot 功能可以在受地区限制的区域正常使用。

## 📥 安装

### 方式一：从 Release 下载（推荐）

前往 [Releases](https://github.com/qiyuey/edge-copilot-helper/releases) 页面下载对应平台的预编译二进制文件。

### 方式二：从源码编译

**前置要求**：需要安装 Rust 工具链

```bash
# 安装 Rust（如果尚未安装）
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 克隆仓库
git clone https://github.com/qiyuey/edge-copilot-helper.git
cd edge-copilot-helper

# 编译 Release 版本
cargo build --release

# 二进制文件位于
./target/release/edge-copilot-helper
```

## 🚀 使用方法

### 命令概览

- `help`：显示帮助信息（默认行为）
- `version`：显示版本信息（等同于 `--version`）
- `run`：前台运行，输出到控制台
- `daemon`：后台运行，不弹出窗口，输出到日志文件
- `install`：安装程序并配置开机自启（后台运行）
- `uninstall`：卸载程序并移除自启动配置

### 直接运行

```bash
# 查看帮助（默认命令）
./edge-copilot-helper
# 或显式执行
./edge-copilot-helper help

# 显示版本信息
./edge-copilot-helper version

# 前台运行（控制台输出，命令行会保持占用）
./edge-copilot-helper run

# 后台运行（日志输出，不弹出窗口）
./edge-copilot-helper daemon
```

运行模式说明：
- `run`：控制台保持前台，持续监听 Edge 状态并自动修复。
- `daemon`：后台运行，输出到日志文件，不弹出控制台窗口。

### 安装为系统服务（推荐）

安装为系统服务后，程序会在后台自动运行，开机自启。

```bash
# 安装服务
./edge-copilot-helper install

# 卸载服务
./edge-copilot-helper uninstall
```

**各平台服务类型**：
- **macOS**：LaunchAgent（用户级服务）
- **Windows**：注册表自启动（HKCU\Run，用户级）
- **Linux**：systemd user service（用户级服务）

### 查看日志

```bash
# macOS
tail -f ~/Library/Logs/top.qiyuey.edge-copilot-helper/service.log

# Linux
journalctl --user -u edge-copilot-helper -f

# Windows
# 日志位于：%LOCALAPPDATA%\EdgeCopilotHelper\logs\
```

## ⚠️ 重要提示

### Windows 用户

为了确保修复生效，请遵循以下步骤：

1. **关闭 Edge 后台运行**：
   - 打开 Edge 设置 → 系统 → 关闭 "Microsoft Edge 关闭后继续运行后台应用"
   - 或者手动关闭所有 Edge 窗口

2. **手动终止进程**（如果修复未生效）：
   ```powershell
   # 使用任务管理器结束所有 msedge.exe 进程
   # 或使用命令行：
   taskkill /IM msedge.exe /F /T
   ```

**原因**：如果 Edge 后台进程仍在运行，配置文件可能被锁定，导致修复无法应用。

### macOS 用户

- 首次运行时，系统可能会提示需要辅助功能权限，请按照提示在系统设置中授予权限
- 如果使用系统服务，确保 LaunchAgent 已正确加载

### Linux 用户

- 如果使用 systemd 服务，确保用户级 systemd 已启用：
  ```bash
  systemctl --user enable --now edge-copilot-helper
  ```

## 📁 项目结构

```
src/
├── main.rs          # 入口点，CLI 命令处理
├── common.rs        # 通用 JSON 处理逻辑（修复配置文件）
├── constants.rs     # 平台相关常量和路径定义
├── macos.rs         # macOS 事件监听实现（NSWorkspace API）
├── polling.rs       # Windows/Linux 轮询实现
└── service/         # 服务安装/卸载逻辑
    ├── mod.rs       # 服务模块入口
    ├── macos.rs     # LaunchAgent 安装/卸载
    ├── windows.rs   # Windows Service 安装/卸载
    └── linux.rs     # systemd 服务安装/卸载
```

## 🔍 技术细节

### 修改的配置文件

1. **Local State**（位于 User Data 目录）
   - 修改 `variations_country` 字段为 `"US"`

2. **Preferences**（位于各 Profile 目录）
   - 设置 `browser.chat_ip_eligibility_status` 为 `true`

### 支持的 Edge 版本

- Microsoft Edge (Stable)
- Microsoft Edge Beta
- Microsoft Edge Dev
- Microsoft Edge Canary

### 支持的配置文件

- Default Profile
- Profile 1, Profile 2, ...（所有自定义配置文件）

## 🐛 故障排除

### 修复未生效

1. 确认 Edge 已完全退出（包括后台进程）
2. 检查日志文件，查看是否有错误信息
3. 手动运行程序，查看控制台输出
4. 确认配置文件路径正确且可写

### 服务未启动

1. **macOS**：检查 LaunchAgent 是否加载
   ```bash
   launchctl list | grep edge-copilot-helper
   ```

2. **Linux**：检查 systemd 服务状态
   ```bash
   systemctl --user status edge-copilot-helper
   ```

3. **Windows**：检查注册表自启动项
   ```powershell
   reg query "HKCU\Software\Microsoft\Windows\CurrentVersion\Run" /v EdgeCopilotHelper
   ```

## 📄 许可证

本项目采用 [Anti-996 License](https://github.com/996icu/996.ICU/blob/master/LICENSE)（反996许可证）。

该许可证旨在防止违反劳动法的公司使用本软件，并强制这些公司权衡其工作方式。

- [英文版许可证](LICENSE)
- [中文版许可证](LICENSE_CN)
- [了解更多关于 996.ICU](https://996.icu)

## 🤝 贡献

欢迎提交 Issue 和 Pull Request！

## 📮 反馈

如有问题或建议，请前往 [GitHub Issues](https://github.com/qiyuey/edge-copilot-helper/issues) 反馈。
