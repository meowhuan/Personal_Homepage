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

若需要审查 JavaScript 动态加载页面，先安装 Node.js 与 Playwright（一次性）：

```bash
cd /opt/status-backend
npm install playwright
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
# Optional: Playwright JS rendering fallback
# Environment=REVIEW_JS_RENDER=1
# Environment=REVIEW_JS_RENDER_SCRIPT=/opt/status-backend/scripts/playwright-fetch.mjs
# Environment=REVIEW_JS_RENDER_MAX_PAGES=2

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
- `REVIEW_RUN_ONCE` (optional, `1/true` to run single cycle then exit, useful for debugging)
- `REVIEW_SEO_PROVIDER` (optional: `none`/`generic`/`serpapi`, default `none`)
- `REVIEW_SEO_MAX_BONUS` (optional, default `12`, range `1~30`)
- `REVIEW_TITLE_SIM_PENDING_BELOW` (optional, default `0.35`)
- `REVIEW_TITLE_SIM_REJECT_BELOW` (optional, default `0.18`)
- `REVIEW_JS_RENDER` (optional, `1/true` to enable Playwright rendering fallback for backlink detection)
- `REVIEW_JS_RENDER_CMD` (optional, default `node`)
- `REVIEW_JS_RENDER_SCRIPT` (optional, default `scripts/playwright-fetch.mjs`)
- `REVIEW_JS_RENDER_TIMEOUT_SEC` (optional, default `18`, range `5~60`)
- `REVIEW_JS_RENDER_WAIT_UNTIL` (optional, `load`/`domcontentloaded`/`networkidle`, default `networkidle`)
- `REVIEW_JS_RENDER_WAIT_AFTER_MS` (optional, default `800`, range `0~5000`)
- `REVIEW_JS_RENDER_MAX_PAGES` (optional, max JS-render pages per backlink scan, default `2`, range `1~8`)

当 `REVIEW_SEO_PROVIDER=generic` 时：
- `REVIEW_SEO_API_URL` (required)
- `REVIEW_SEO_API_KEY` (optional)
- `REVIEW_SEO_API_KEY_HEADER` (optional, default `Authorization`)

当 `REVIEW_SEO_PROVIDER=serpapi` 时：
- `REVIEW_SERPAPI_KEY` (required)
- `REVIEW_SERPAPI_ENDPOINT` (optional, default `https://serpapi.com/search.json`)
- `REVIEW_SERPAPI_ENGINE` (optional, default `google`)
- `REVIEW_SERPAPI_HL` (optional, default `zh-cn`)
- `REVIEW_SERPAPI_GL` (optional, default `cn`)
- `REVIEW_SERPAPI_NUM` (optional, default `10`, range `5~20`)

公网后端申请接口风控（`status-backend`）推荐配置：
- `LINK_CAPTCHA_PROVIDER=turnstile` 或 `hcaptcha`
- `LINK_TURNSTILE_SITE_KEY` / `LINK_HCAPTCHA_SITE_KEY`（前端渲染 key）
- `LINK_TURNSTILE_SECRET` / `LINK_HCAPTCHA_SECRET`（服务端校验 secret）
- `LINK_APPLY_RATE_LIMIT_WINDOW_SEC=3600`
- `LINK_APPLY_RATE_LIMIT_MAX=3`
- `LINK_APPLY_RATE_LIMIT_PREFIX_MAX=8`
- `LINK_APPLY_RATE_LIMIT_EMAIL_DOMAIN_MAX=6`
- `LINK_APPLY_RATE_LIMIT_SITE_HOST_MAX=3`
- `LINK_BLOCK_DISPOSABLE_EMAIL=true`
- `LINK_BLOCK_EDU_GOV_EMAIL=true`
- `LINK_APPLY_DENY_HOSTS=aliyun.com,qq.com,baidu.com`（按需扩展）
- `LINK_VERIFY_WINDOW_MINUTES=120`
- `LINK_PUBLIC_BASE_URL=https://m.ratf.cn`
- `LINK_VERIFY_EMAIL_RATE_LIMIT_WINDOW_SEC=1800`
- `LINK_VERIFY_EMAIL_RATE_LIMIT_MAX=3`
- `LINK_VERIFY_EMAIL_RATE_LIMIT_APP_MAX=2`
- `LINK_VERIFY_EMAIL_COOLDOWN_SEC=600`

说明：申请页前端会通过 `GET /links/apply/config` 自动获取 captcha provider 与 site key。
验证方式说明：`/.well-known/meow-links.txt`、DNS TXT `_meow-links`、首页 meta `name="meow-links"` 或邮箱验证任意一种通过即可进入审核。

第三方 SEO API（`generic` provider）返回 JSON 约定（最小）：

```json
{
  "score": 78,
  "reason": "index coverage and keyword profile look good"
}
```

单次排障运行示例：

```bash
REVIEW_API_BASE="https://your-public-api.example.com" \
REVIEW_REPORT_TOKEN="your_report_token" \
REVIEW_RUN_ONCE=1 \
./target/release/review-reporter
```
