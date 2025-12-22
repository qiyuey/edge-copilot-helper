# Edge Copilot Helper

这是一个 macOS 辅助工具，用于在 Microsoft Edge 退出时自动修正其配置文件，确保 Edge Copilot 功能（如地区限制）可用。

## 功能

*   **自动监控**：后台静默运行，监听 Microsoft Edge 的退出事件。
*   **即时修复**：一旦检测到 Edge 退出，立即执行修复脚本。
*   **地区修正**：将 `Preferences` 文件中的 `browser.custom_services.region_search` 强制设置为 `SG` (新加坡)，以绕过地区限制。

## 依赖

*   **macOS**
*   **Xcode Command Line Tools** (用于编译 Swift 监控程序)
    *   安装命令: `xcode-select --install`
*   **jq** (用于解析和修改 JSON 配置文件)
    *   安装命令: `brew install jq`

## 安装

1.  克隆或下载本项目。
2.  在终端中进入项目目录。
3.  运行安装脚本：

```bash
./install.sh
```

脚本将自动执行以下操作：
*   编译 Swift 监控程序。
*   安装程序和脚本到 `~/Library/Application Support/top.qiyuey.edge-copilot-helper/`。
*   配置并启动 Launch Agent (`top.qiyuey.edge-copilot-helper`) 实现开机自启。

## 目录结构

*   **源码**:
    *   `EdgeExitWatcher.swift`: 监控程序源码。
    *   `fix-edge-copilot.sh`: 修复脚本源码。
    *   `install.sh`: 一键安装脚本。
    *   `uninstall.sh`: 一键卸载脚本。
*   **安装位置**: `~/Library/Application Support/top.qiyuey.edge-copilot-helper/`
*   **日志位置**: `~/Library/Logs/top.qiyuey.edge-copilot-helper/`
    *   `service.log`: 标准输出日志。
    *   `service.err`: 错误日志。

## 查看状态

安装完成后，可以通过查看日志确认服务运行状态：

```bash
tail -f ~/Library/Logs/top.qiyuey.edge-copilot-helper/service.log
```

## 卸载

若要移除服务，请运行以下命令：

```bash
./uninstall.sh
```

