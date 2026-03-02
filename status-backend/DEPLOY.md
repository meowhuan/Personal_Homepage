# Deploy (Ubuntu 24.04)

示例文档，不包含真实地址或密钥。
Examples only. No real endpoints or tokens.

## 1) 准备环境 / Prerequisites

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev
```

## 2) 配置 / Config

在 `status-backend/.env` 中设置：

```
STATUS_TOKEN=your_token
STATUS_PORT=7999
STATUS_DB=status.db
```

## 3) 构建与运行 / Build & Run

```bash
cd status-backend
cargo build --release
./target/release/status-backend
```

服务默认监听 `0.0.0.0:7999`。

## 4) systemd 服务 / systemd service

创建 `/etc/systemd/system/status-backend.service`：

```
[Unit]
Description=Status Backend
After=network.target

[Service]
WorkingDirectory=/opt/status-backend
ExecStart=/opt/status-backend/target/release/status-backend
Restart=always
RestartSec=5
EnvironmentFile=/opt/status-backend/.env

[Install]
WantedBy=multi-user.target
```

启用：

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now status-backend
```

## 5) 防火墙 / Firewall

```bash
sudo ufw allow 7999/tcp
```

## 6) 内网审查服务（review-reporter）/ Internal Review Worker

用于在内网环境执行网站审查，再将结果上报公网后端，避免公网后端主动抓取外站。

### 6.1 构建发行版 / Build release binary

```bash
cd status-backend
cargo build --release --bin review-reporter
```

发行版路径：
- Linux: `./target/release/review-reporter`
- Windows: `.\target\release\review-reporter.exe`

### 6.2 运行（Linux）/ Run on Linux

```bash
cd /opt/status-backend
REVIEW_API_BASE="https://your-public-api.example.com" \
REVIEW_REPORT_TOKEN="your_report_token" \
./target/release/review-reporter
```

### 6.3 systemd（Linux）/ systemd service

创建 `/etc/systemd/system/review-reporter.service`：

```
[Unit]
Description=Review Reporter (Internal Link Review Worker)
After=network.target

[Service]
WorkingDirectory=/opt/status-backend
ExecStart=/opt/status-backend/target/release/review-reporter
Restart=always
RestartSec=5
Environment=REVIEW_API_BASE=https://your-public-api.example.com
Environment=REVIEW_REPORT_TOKEN=your_report_token
Environment=REVIEW_LOOP_INTERVAL_SEC=300
Environment=REVIEW_LOCAL_STATE=/opt/status-backend/review-worker-state.json

[Install]
WantedBy=multi-user.target
```

启用：

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now review-reporter
```

### 6.4 可选环境变量 / Optional env

- `REVIEW_LOOP_INTERVAL_SEC` (default `300`)
- `REVIEW_LOCAL_STATE` (default `review-worker-state.json`)
