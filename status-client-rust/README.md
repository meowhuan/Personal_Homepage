# Status Client (Rust)

Rust 客户端（Windows/Linux 心跳上报）。
Rust client for Windows/Linux heartbeat reporting.

## 配置文件 / Config File

将 `status-client.toml` 放在可执行文件同目录（双击运行会优先读取 EXE 同目录）：

Place `status-client.toml` next to the executable (double-click prefers the EXE directory):

```toml
endpoint = "http://your-host:7999/heartbeat"
token = "your_token"
device_id = "pc-main"
device_name = "PC Main"
idle_timeout_secs = 300
heartbeat_interval_secs = 60
log_file = "status-client.log"
```

## 环境变量（覆盖配置） / Env (Override Config)

- `STATUS_ENDPOINT`
- `STATUS_TOKEN`
- `DEVICE_ID`
- `DEVICE_NAME`
- `IDLE_TIMEOUT_SECS`
- `HEARTBEAT_INTERVAL_SECS`
- `STATUS_CONFIG` (path to config)
- `LOG_FILE` (log file path)

## 构建 / Build

Windows：

```bash
cargo build --release
```

Linux（.deb）：

```bash
cargo install cargo-deb
cargo deb
```

`.deb` 会生成在 `target/debian/`。

## 说明 / Notes

- 该客户端为后台静默运行（无 UI、无托盘）。
- 可配置为开机自启或作为服务运行。

This client runs silently in the background (no UI, no tray).
You can set it to auto-start or run as a service.

## 开机自启 / Auto-start

### Windows（任务计划程序）

1. 打开 **任务计划程序** → **创建任务**。
2. **常规**：名称 `StatusClient`，勾选 **使用最高权限运行**。
3. **触发器**：新建 → **登录时**。
4. **操作**：新建 → **启动程序**。
   - 程序：`status-client.exe`
   - 起始于：放有 `status-client.exe` 与 `status-client.toml` 的目录
5. 保存。

### Linux（systemd 用户服务）

创建 `~/.config/systemd/user/status-client.service`：

```
[Unit]
Description=Status Client

[Service]
ExecStart=/path/to/status-client
WorkingDirectory=/path/to/
Restart=always
RestartSec=5

[Install]
WantedBy=default.target
```

启用：

```
systemctl --user daemon-reload
systemctl --user enable --now status-client
```
