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
music_poll_interval_secs = 5
music_push_min_interval_secs = 6
log_file = "status-client.log"
```

## 环境变量（覆盖配置） / Env (Override Config)

- `STATUS_ENDPOINT`
- `STATUS_TOKEN`
- `DEVICE_ID`
- `DEVICE_NAME`
- `IDLE_TIMEOUT_SECS`
- `HEARTBEAT_INTERVAL_SECS`
- `MUSIC_POLL_INTERVAL_SECS`
- `MUSIC_PUSH_MIN_INTERVAL_SECS`
- `STATUS_CONFIG` (path to config)
- `LOG_FILE` (log file path)
- `LOG_MAX_BYTES`

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
- 心跳会附带听歌状态：
  - Windows：SMTC（系统媒体会话）
  - Linux：MPRIS（依赖 `playerctl`）
  - 网易云：需安装[BetterNCM](https://github.com/std-microblock/chromatic)框架后安装[InfLink-rs](https://github.com/apoint123/inflink-rs)插件进行SMTC支持

This client runs silently in the background (no UI, no tray).
You can set it to auto-start or run as a service.
Heartbeat also includes music status:
- Windows: SMTC
- Linux: MPRIS (`playerctl` required)
- NetEase Cloud: SMTC support requires installing the [InfLink-rs](https://github.com/apoint123/inflink-rs) plugin after installing the [BetterNCM](https://github.com/std-microblock/chromatic) framework.
- 主心跳频率不变；音乐状态会单独快速轮询并仅在变化时触发额外上报。

## 排障 / Troubleshooting

- Windows 听歌识别不到：
  - 请确保客户端运行在“当前登录用户会话”中（不要以 SYSTEM 会话运行）。
  - 任务计划建议：使用当前用户、仅在用户登录时运行。
  - 某些播放器未暴露 SMTC 时会返回空。
- 日志文件没有生成：
  - 客户端会优先写入 `log_file` 配置路径。
  - 若路径不可写，会自动回退到系统临时目录 `status-client.log`。
  - 默认日志级别为 `info`；也可用 `RUST_LOG=debug` 提高详细度。
  - 可用 `LOG_MAX_BYTES`（默认 `2097152`，2MB）限制单文件大小，超限会自动截断。

### 最小 SMTC 测试 / Minimal SMTC Probe (Windows)

在 `status-client-rust` 目录运行：

```bash
cargo run --bin smtc-probe
```

输出会是 JSON（例如 `ok/playing/title/artist/source`），并同时写一份日志到系统临时目录：

`%TEMP%\smtc-probe.log`

如果 probe 能读到歌，但主程序读不到，优先检查任务计划运行账户/会话（是否为登录用户会话）。

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
