# Edge Copilot Helper

跨平台工具，自动绕过 Microsoft Edge Copilot 的区域限制。监控 Edge 浏览器进程，在 Edge 退出时自动修补配置文件，确保 Copilot 功能不受地理区域限制。

## 工作原理

Edge 在本地配置文件中存储了区域信息。在某些地区（如中国），Copilot 功能会受到限制。本工具监听 Edge 进程退出事件，并自动修改两项配置：

1. 将 `Local State` 中的 `**variations_country**` 设为 `"SG"`
2. 将各 Profile 的 `Preferences` 中的 `**chat_ip_eligibility_status**` 设为 `true`

仅在需要时写入文件——如果值已正确，则不做任何改动。

## 支持平台


| 平台          | 监控方式                       | 服务类型                |
| ----------- | -------------------------- | ------------------- |
| **macOS**   | NSWorkspace 通知（零 CPU 空闲占用） | LaunchAgent         |
| **Windows** | 通过 `sysinfo` 每 2 秒轮询       | 注册表开机启动（`HKCU\Run`） |
| **Linux**   | 通过 `sysinfo` 每 2 秒轮询       | systemd 用户服务        |


支持所有 Edge 频道：**Stable**、**Beta**、**Dev** 和 **Canary**。

## 安装

### 从源码构建

```bash
cargo build --release
```

生成的二进制文件位于 `target/release/edge-copilot-helper`（Windows 上为 `.exe`）。

### 安装为系统服务

```bash
# macOS / Linux
./target/release/edge-copilot-helper install

# Windows (PowerShell)
.\target\release\edge-copilot-helper.exe install
```

执行后会：

- 将二进制文件复制到平台对应的安装目录
- 注册为登录时自动启动
- 立即启动服务（macOS/Linux）


| 平台      | 安装目录                                                            | 自启动机制                                                |
| ------- | --------------------------------------------------------------- | ---------------------------------------------------- |
| macOS   | `~/Library/Application Support/top.qiyuey.edge-copilot-helper/` | LaunchAgent plist                                    |
| Windows | `%LOCALAPPDATA%\top.qiyuey.edge-copilot-helper\`                | `HKCU\Software\Microsoft\Windows\CurrentVersion\Run` |
| Linux   | `~/.local/share/top.qiyuey.edge-copilot-helper/`                | systemd 用户服务                                         |


## 使用方法

```
edge-copilot-helper [命令]
```


| 命令          | 说明            |
| ----------- | ------------- |
| `help`      | 显示帮助信息（默认）    |
| `version`   | 显示版本号         |
| `run`       | 前台运行，日志输出到控制台 |
| `daemon`    | 后台运行，日志仅写入文件  |
| `install`   | 安装为系统服务       |
| `uninstall` | 卸载服务并删除所有文件   |


### 前台运行

```bash
edge-copilot-helper run
```

适合调试和测试，日志直接输出到控制台。

### 后台运行

```bash
edge-copilot-helper daemon
```

静默在后台运行，日志写入文件。安装为服务后使用的就是此模式。

## 日志

日志文件按日期存储在平台对应的日志目录中，超过 7 天的旧日志会自动清理。


| 平台      | 日志目录                                                  |
| ------- | ----------------------------------------------------- |
| macOS   | `~/Library/Logs/top.qiyuey.edge-copilot-helper/`      |
| Windows | `%LOCALAPPDATA%\top.qiyuey.edge-copilot-helper\logs\` |
| Linux   | `~/.local/share/top.qiyuey.edge-copilot-helper/logs/` |


**查看日志：**

```bash
# macOS
tail -f ~/Library/Logs/top.qiyuey.edge-copilot-helper/service.log

# Linux
journalctl --user -u top.qiyuey.edge-copilot-helper -f

# Windows (PowerShell)
Get-Content "$env:LOCALAPPDATA\top.qiyuey.edge-copilot-helper\logs\edge-copilot-helper-*.log" -Tail 50
```

## 卸载

```bash
# macOS / Linux
./target/release/edge-copilot-helper uninstall

# Windows (PowerShell)
.\target\release\edge-copilot-helper.exe uninstall
```

将停止运行中的服务、移除自启动注册，并删除所有已安装的文件（包括日志）。

## 构建

```bash
cargo build --release    # 构建 Release 二进制
cargo check              # 检查编译
cargo test               # 运行测试
cargo clippy             # 代码检查
cargo fmt                # 格式化代码
```

## 许可证

[Anti-996](https://github.com/996icu/996.ICU/blob/master/LICENSE)