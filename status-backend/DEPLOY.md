# Deploy (Ubuntu 24.04)

示例文档，不包含真实地址或密钥。
Examples only. No real endpoints or tokens.

## 1) 准备环境 / Prerequisites

```bash
sudo apt update
sudo apt install -y build-essential pkg-config libssl-dev
```

如需 Playwright 渲染（内网审查用），还需要 Node.js。
If you need Playwright rendering (internal review), install Node.js as well.

## 2) 获取代码 / Get Code

将仓库放在例如 `/opt/status-backend`。
Place the repo at e.g. `/opt/status-backend`.

## 3) 配置 / Config

复制 `.env.example` 并填写：
Copy `.env.example` and fill in:

```bash
cp .env.example .env
```

最少配置：
Minimum required:

```
STATUS_TOKEN=your_token
```

常用可选项：
Common optional values:

```
STATUS_PORT=7999
STATUS_DB=/opt/status-backend/status.db
STATUS_BUILD=status-backend v1.x
RUST_LOG=info
```

### 3.1 ALTCHA 部署 / ALTCHA Setup

1. 准备 Sentinel 并创建 API Key。Widget 需要 `challengeurl` 指向 Sentinel 的 `/v1/challenge`，并携带 `apiKey` 参数（形如 `https://sentinel.example.com/v1/challenge?apiKey=key_...`）。
2. 后台配置方式（二选一）：
   - 管理后台：`Captcha Provider` 选 `ALTCHA`，`Captcha Site Key / Challenge URL` 填上一步的 `challengeurl`。
   - 环境变量：`LINK_CAPTCHA_PROVIDER=altcha`，`LINK_ALTCHA_CHALLENGE_URL=<challengeurl>`。
3. 本后端使用 ALTCHA 的 `POST /v1/verify/signature` HTTP API 验证 payload；响应中 `verified: true` 即验证通过。

## 4) 构建与运行 / Build & Run

```bash
cargo build --release
./target/release/status-backend
```

服务默认监听 `0.0.0.0:7999`。
The service listens on `0.0.0.0:7999` by default.

## 5) systemd 服务 / systemd Service

创建 `/etc/systemd/system/status-backend.service`：
Create `/etc/systemd/system/status-backend.service`:

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
Enable:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now status-backend
```

## 6) 防火墙 / Firewall

```bash
sudo ufw allow 7999/tcp
```

## 7) 内网审查服务（review-reporter）/ Internal Review Worker

用于在内网环境执行网站审查，再将结果上报公网后端，避免公网后端主动抓取外站。
Runs inside private network to review sites and report decisions to public backend.

### 7.1 构建发行版 / Build release binary

```bash
cargo build --release --bin review-reporter
```

发行版路径：
Release binary path:

- Linux: `./target/release/review-reporter`
- Windows: `./target/release/review-reporter.exe`

### 7.2 运行（Linux）/ Run on Linux

```bash
REVIEW_API_BASE="https://your-public-api.example.com" \
REVIEW_REPORT_TOKEN="your_report_token" \
./target/release/review-reporter
```

### 7.3 systemd（Linux）/ systemd Service

创建 `/etc/systemd/system/review-reporter.service`：
Create `/etc/systemd/system/review-reporter.service`:

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
# Optional: Playwright JS rendering fallback
# Environment=REVIEW_JS_RENDER=1
# Environment=REVIEW_JS_RENDER_SCRIPT=/opt/status-backend/scripts/playwright-fetch.mjs
# Environment=REVIEW_JS_RENDER_MAX_PAGES=2

[Install]
WantedBy=multi-user.target
```

启用：
Enable:

```bash
sudo systemctl daemon-reload
sudo systemctl enable --now review-reporter
```

### 7.4 可选环境变量 / Optional Env

- `REVIEW_LOOP_INTERVAL_SEC` (default `300`)
- `REVIEW_LOCAL_STATE` (default `review-worker-state.json`)
- `REVIEW_RUN_ONCE` (optional, `1/true` to run single cycle then exit)
- `REVIEW_SEO_PROVIDER` (optional: `none`/`generic`/`serpapi`, default `none`)
- `REVIEW_SEO_MAX_BONUS` (optional, default `12`, range `1~30`)
- `REVIEW_JS_RENDER` (optional, `1/true` to enable Playwright rendering)
- `REVIEW_JS_RENDER_CMD` (optional, default `node`)
- `REVIEW_JS_RENDER_SCRIPT` (optional, default `scripts/playwright-fetch.mjs`)
- `REVIEW_JS_RENDER_TIMEOUT_SEC` (optional, default `18`, range `5~60`)
- `REVIEW_JS_RENDER_WAIT_UNTIL` (optional, `load`/`domcontentloaded`/`networkidle`, default `networkidle`)
- `REVIEW_JS_RENDER_WAIT_AFTER_MS` (optional, default `800`, range `0~5000`)
- `REVIEW_JS_RENDER_MAX_PAGES` (optional, default `2`, range `1~8`)

当 `REVIEW_SEO_PROVIDER=generic` 时：
When `REVIEW_SEO_PROVIDER=generic`:

- `REVIEW_SEO_API_URL` (required)
- `REVIEW_SEO_API_KEY` (optional)
- `REVIEW_SEO_API_KEY_HEADER` (optional, default `Authorization`)

当 `REVIEW_SEO_PROVIDER=serpapi` 时：
When `REVIEW_SEO_PROVIDER=serpapi`:

- `REVIEW_SERPAPI_KEY` (required)
- `REVIEW_SERPAPI_ENDPOINT` (optional, default `https://serpapi.com/search.json`)
- `REVIEW_SERPAPI_ENGINE` (optional, default `google`)
- `REVIEW_SERPAPI_HL` (optional, default `zh-cn`)
- `REVIEW_SERPAPI_GL` (optional, default `cn`)
- `REVIEW_SERPAPI_NUM` (optional, default `10`, range `5~20`)

### 7.5 Playwright 依赖 / Playwright Dependency

若需要审查 JavaScript 动态加载页面，先安装依赖：
Install Playwright if you need JS-rendered pages:

```bash
npm install playwright
```

## 8) 公网部署建议 / Public Deployment Notes

- 公网后端建议开启 CAPTCHA 与限流配置（见 `status-backend/README.md`）。
  Enable CAPTCHA and rate limit settings for the public backend (see `status-backend/README.md`).
- `LINK_REVIEW_REPORT_TOKEN` 建议与 `STATUS_TOKEN` 分离，避免内网 token 泄漏风险。
  Keep `LINK_REVIEW_REPORT_TOKEN` separate from `STATUS_TOKEN` when possible.

单次排障运行示例：
One-shot debug run:

```bash
REVIEW_API_BASE="https://your-public-api.example.com" \
REVIEW_REPORT_TOKEN="your_report_token" \
REVIEW_RUN_ONCE=1 \
./target/release/review-reporter
```
