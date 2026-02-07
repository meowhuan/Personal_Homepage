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
