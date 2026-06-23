# Edge Copilot Helper

跨平台工具，自动绕过 Microsoft Edge Copilot 的区域限制。监控 Edge 浏览器进程，在 Edge 退出时自动修补配置文件，确保 Copilot 功能不受地理区域限制。

## 前置条件

本工具仅处理 Edge 本地配置文件。要使 Copilot 正常工作，还需要：

- **代理/VPN**：Copilot 依赖 Microsoft 海外服务器，需确保相关流量可达（建议全局模式）
- **DNS 配置**：建议使用 `8.8.8.8` 或 `1.1.1.1`，避免 DNS 污染
- **系统区域**：建议将 Windows 区域设置改为目标国家（如美国、新加坡等）

## 工作原理

Edge 通过多层机制限制特定区域的 Copilot 功能。本工具监听 Edge 进程退出事件，自动修补以下配置：

**种子文件清理（User Data 目录）：**

1. 删除 `VariationsSeedV2`、`VariationsSafeSeedV2`、`VariationsRuntimeSeedV2` — 这些二进制文件内嵌了国家代码，会导致 Edge 不断重新生成 Copilot 禁用标志。删除后 Edge 会通过网络重新获取种子。

**Local State 文件：**

2. 将 **`variations_country`** 设为目标国家代码
3. 将 **`variations_safe_seed_session_consistency_country`** 设为目标国家代码
4. 从 **`variations_config_ids`** 中移除 `disablecopilotmodeenp` 等服务端下发的 Copilot 禁用标志
5. 清除种子元数据（`variations_seed_date`、`variations_seed_etag` 等），确保 Edge 不会尝试加载已删除的种子

**各 Profile 的 Preferences 文件：**

6. 将 **`chat_ip_eligibility_status`** 设为 `true`
7. 将 Copilot 侧边栏应用从隐藏状态恢复为可见

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
- macOS 上会主动执行 `request-permissions`，打开隐私设置并在 Finder 中定位已安装的 helper，方便授予 Full Disk Access


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
| `request-permissions` | macOS 上主动检查并引导授予 Edge 应用数据访问权限 |


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

### macOS 27 权限

macOS 27 会限制后台工具读取其他 App 的数据。Edge 配置位于 `~/Library/Application Support/Microsoft Edge/`，因此 LaunchAgent 需要 Full Disk Access 才能在 Edge 退出后修补 `Local State` 和 `Preferences`。

安装命令会自动运行一次权限引导：

```bash
./target/release/edge-copilot-helper install
```

也可以手动触发：

```bash
~/Library/Application\ Support/top.qiyuey.edge-copilot-helper/edge-copilot-helper request-permissions
```

该命令会尝试读取 Edge 数据以触发 macOS/TCC 记录，打开 **System Settings → Privacy & Security → Full Disk Access**，并用 Finder 选中需要授权的二进制文件：

```text
~/Library/Application Support/top.qiyuey.edge-copilot-helper/edge-copilot-helper
```

授予权限后重启服务，或重新运行 `install`。完整验证方式是退出一次 Edge，然后检查当天日志是否出现 `Edge Copilot region fix applied`。

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
tail -f ~/Library/Logs/top.qiyuey.edge-copilot-helper/edge-copilot-helper-$(date +%Y%m%d).log

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
