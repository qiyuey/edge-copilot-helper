# Edge Copilot Helper

跨平台工具，用于在 Microsoft Edge 退出时自动修正配置文件，绕过 Copilot 地区限制。

## 功能特性

- **跨平台支持**: macOS (ARM64)、Windows (x64)、Linux (x64)
- **macOS 原生监听**: 使用 NSWorkspace API 监听应用退出事件，零 CPU 占用
- **Windows/Linux 轮询**: 使用 sysinfo 低频轮询监控进程状态
- **系统服务**: 支持安装为系统服务，开机自启

## 安装

### 从 Release 下载

前往 [Releases](https://github.com/qiyuey/edge-copilot-helper/releases) 页面下载对应平台的二进制文件。

### 从源码编译

```bash
# 需要 Rust 工具链
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# 编译
cargo build --release

# 二进制文件位于
./target/release/edge-copilot-helper
```

## 使用方法

### 直接运行

```bash
# 前台运行（默认）
./edge-copilot-helper run
./edge-copilot-helper        # run 是默认命令
```

### 安装为系统服务

```bash
# 安装服务（macOS: LaunchAgent, Windows: SCM, Linux: systemd）
./edge-copilot-helper install

# 卸载服务
./edge-copilot-helper uninstall
```

### 查看日志

```bash
# macOS
tail -f ~/Library/Logs/top.qiyuey.edge-copilot-helper/service.log

# Linux
journalctl --user -u edge-copilot-helper -f

# Windows
# 查看 %LOCALAPPDATA%\EdgeCopilotHelper\logs\
```

## 工作原理

当 Microsoft Edge 退出时，程序会：

1. 检测 Edge 进程退出事件
2. 读取 Edge 配置文件（Preferences）
3. 将所有值为 "CN" 的字符串替换为 "SG"
4. 保存修改后的配置

这使得 Edge Copilot 功能可以在受地区限制的区域正常使用。

## 项目结构

```
src/
├── main.rs          # 入口点，CLI 命令处理
├── common.rs        # 通用 JSON 处理逻辑
├── macos.rs         # macOS 事件监听实现
├── polling.rs       # Windows/Linux 轮询实现
├── constants.rs     # 平台相关常量和路径
└── service/         # 服务安装/卸载逻辑
    ├── macos.rs     # LaunchAgent
    ├── windows.rs   # Windows SCM
    └── linux.rs     # systemd
```

## License

MIT
