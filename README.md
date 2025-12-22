# Edge Copilot Helper (Rust Version)

这是一个跨平台的辅助工具，用于在 Microsoft Edge 退出时自动修正其配置文件，确保 Edge Copilot 功能（如地区限制）可用。

本项目已重构为 **Rust** 实现，支持 macOS（原生事件监听）、Windows 和 Linux（轮询监控）。

## 核心特性

*   **跨平台核心**: 统一的 Rust 逻辑处理 JSON 修改。
*   **macOS 原生体验**: 使用 `objc2` 调用系统 API 监听应用退出，零 CPU 占用。
*   **Windows/Linux 支持**: 使用 `sysinfo` 低频轮询监控进程状态。
*   **单一二进制**: 编译后仅生成一个可执行文件，无需依赖 Python、jq 或 Shell 环境。

## 目录结构

*   `src/`: Rust 源代码
    *   `main.rs`: 入口点
    *   `macos.rs`: macOS 事件监听实现
    *   `polling.rs`: Windows/Linux 轮询实现
    *   `common.rs`: 通用 JSON 处理逻辑
*   `install.sh`: macOS 一键编译安装脚本
*   `uninstall.sh`: macOS 一键卸载脚本
*   `legacy/`: 旧版 Swift/Shell 实现归档

## macOS 安装

1.  确保已安装 Rust 工具链:
    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```
2.  运行安装脚本:
    ```bash
    ./install.sh
    ```

脚本将自动执行以下操作：
*   使用 `cargo build --release` 编译项目。
*   将二进制文件安装到 `~/Library/Application Support/top.qiyuey.edge-copilot-helper/`。
*   配置并启动 Launch Agent (`top.qiyuey.edge-copilot-helper`)。

## 查看状态

```bash
tail -f ~/Library/Logs/top.qiyuey.edge-copilot-helper/service.log
```

## 卸载

```bash
./uninstall.sh
```

## Windows / Linux 使用

目前提供的脚本仅针对 macOS。在 Windows 或 Linux 上使用：

1.  编译项目: `cargo build --release`
2.  运行生成的二进制文件: `./target/release/edge-copilot-helper`
3.  建议配合 Systemd (Linux) 或 任务计划程序 (Windows) 设置开机自启。
