# Deploy (Ubuntu 24.04)

## 1) Build

```bash
cd status-backend
cargo build --release
```

## 2) Create service user + dirs

```bash
sudo useradd -r -s /usr/sbin/nologin status || true
sudo mkdir -p /opt/status
sudo chown -R status:status /opt/status
```

## 3) Install binary

```bash
sudo cp target/release/status-backend /opt/status/status-backend
sudo chown status:status /opt/status/status-backend
sudo chmod +x /opt/status/status-backend
```

## 4) Env file

Create `/opt/status/status.env`:

```
STATUS_PORT=7999
STATUS_DB=/opt/status/status.db
STATUS_TOKEN=your_token_here
RUST_LOG=info
```

## 5) Systemd

Create `/etc/systemd/system/status-backend.service`:

```
[Unit]
Description=Status Backend
After=network.target

[Service]
User=status
WorkingDirectory=/opt/status
EnvironmentFile=/opt/status/status.env
ExecStart=/opt/status/status-backend
Restart=always
RestartSec=3

[Install]
WantedBy=multi-user.target
```

Enable:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now status-backend
sudo systemctl status status-backend
```

## 6) Nginx (optional reverse proxy)

Example `/etc/nginx/sites-available/status.meowra.cn`:

```
server {
    listen 80;
    server_name status.meowra.cn;

    location / {
        proxy_pass http://127.0.0.1:7999;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
    }
}
```

Enable:

```bash
sudo ln -s /etc/nginx/sites-available/status.meowra.cn /etc/nginx/sites-enabled/
sudo nginx -t && sudo systemctl reload nginx
```

## 7) UFW

```bash
sudo ufw allow 7999/tcp
sudo ufw allow 80/tcp
```
