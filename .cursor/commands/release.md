# 更新版本号并创建标签

## 任务描述

当需要发布新版本时，执行以下步骤来更新版本号并创建 Git 标签：

## 操作步骤

1. **预检查（推送前必须通过）**
   - 执行：`cargo fmt --check`（检查代码格式）
   - 执行：`cargo check`（检查编译错误）
   - 执行：`cargo clippy -- -D warnings`（检查代码质量，警告视为错误）
   - 如果任何检查失败，先修复问题再继续

2. **读取当前版本号**
   - 打开 `Cargo.toml` 文件
   - 找到 `version = "x.y.z"` 这一行
   - 记录当前版本号（例如：0.1.4）

3. **计算新版本号**
   - 将补丁版本号（第三位数字）加 1
   - 例如：0.1.4 → 0.1.5
   - 例如：0.2.3 → 0.2.4

4. **提交当前更改（如有）**
   - 执行：`git status` 检查是否有未提交的更改
   - 如有更改，先提交：`git add .` 然后 `git commit -m "描述更改内容"`

5. **更新 Cargo.toml**
   - 将 `version = "旧版本号"` 修改为 `version = "新版本号"`
   - 保存文件

6. **提交版本号更新**
   - 执行：`git add Cargo.toml`
   - 执行：`git commit -m "更新版本号到 x.y.z"`（使用新版本号）

7. **创建 Git 标签**
   - 执行：`git tag -a vx.y.z -m "vx.y.z: 发布新版本"`（使用新版本号，例如：v0.1.5）

8. **推送到远程仓库**
   - 执行：`git push origin main`
   - 执行：`git push origin vx.y.z`（使用新标签名）

## 示例

假设当前版本是 0.1.4：

1. 预检查：
   - `cargo fmt --check` ✓
   - `cargo check` ✓
   - `cargo clippy -- -D warnings` ✓
2. 新版本号：0.1.5
3. 更新 `Cargo.toml`：`version = "0.1.5"`
4. `git add Cargo.toml`
5. `git commit -m "更新版本号到 0.1.5"`
6. `git tag -a v0.1.5 -m "v0.1.5: 发布新版本"`
7. `git push origin main`
8. `git push origin v0.1.5`

## 注意事项

- **预检查必须全部通过后才能继续**，否则 CI 会失败且无法删除已推送的标签
- 确保所有其他更改已提交（除了版本号更新）
- 确保已配置 Git 远程仓库
- 需要有推送权限
- 标签格式：`v` + 版本号（例如：v0.1.5）
