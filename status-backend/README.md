# Status Backend

Rust + SQLite 在线状态与内容后端（状态/日程/博客/友链/访客统计）。
Rust + SQLite backend for status, schedule, blog, friend links, and visitor stats.

## 功能 / Features

- 设备心跳与在线状态（含 idle 秒数）/ Device heartbeat and online status (with idle seconds)
- 全局手动离线与单设备手动离线 / Global manual-offline and per-device manual-offline
- 听歌状态字段（播放中/歌名/作者/来源）/ Music status fields (playing/title/artist/source)
- 日程列表与管理页 / Schedule list and admin page
- 博客列表/详情与管理页（支持 Markdown）/ Blog list/detail and admin page (Markdown supported)
- 友链公开列表、申请、审核、验证与管理 / Friend links list, apply, review, verify, and admin
- 访客统计（今日/本月/累计）/ Visitor stats (today/month/total)
- 内网审查 worker（自动审核、回链检查、可访问性下架）/ Internal review worker (auto review, backlink check, unreachable removal)

## 运行 / Run

```bash
cargo run
```

发布构建：
Release build:

```bash
cargo build --release
./target/release/status-backend
```

## 配置 / Config

优先使用 `status-backend/.env`，示例见 `.env.example`。
Configuration is read from `status-backend/.env` (see `.env.example`).

### 基础 / Basics

- `STATUS_PORT` (default `7999`)
- `STATUS_DB` (default `status.db`)
- `STATUS_TOKEN` (required for protected APIs)
- `STATUS_BUILD` (optional, shown in `/version`)
- `RUST_LOG` (optional, e.g. `info`)

### 通知 / Notifications

- `LINK_TG_BOT_TOKEN` (optional, Telegram Bot Token)
- `LINK_TG_CHAT_ID` (optional, Telegram chat/user id)
- `LINK_SMTP_HOST` (optional)
- `LINK_SMTP_PORT` (optional, default `587`)
- `LINK_SMTP_USER` (optional)
- `LINK_SMTP_PASS` (optional)
- `LINK_SMTP_FROM` (optional, e.g. `bot@example.com`)
- `LINK_SMTP_TO` (optional, receiver list separated by comma)
- `LINK_SMTP_STARTTLS` (optional, default `true`, set `0`/`false` to disable)

### 友链风控 / Link Anti-abuse

- `LINK_CAPTCHA_PROVIDER` (optional: `none`/`turnstile`/`hcaptcha`, default `none`)
- `LINK_TURNSTILE_SITE_KEY` (required if provider is `turnstile`)
- `LINK_TURNSTILE_SECRET` (required if provider is `turnstile`)
- `LINK_HCAPTCHA_SITE_KEY` (required if provider is `hcaptcha`)
- `LINK_HCAPTCHA_SECRET` (required if provider is `hcaptcha`)
- `LINK_APPLY_RATE_LIMIT_WINDOW_SEC` (optional, default `3600`)
- `LINK_APPLY_RATE_LIMIT_MAX` (optional, default `3`, per IP)
- `LINK_APPLY_RATE_LIMIT_PREFIX_MAX` (optional, default `8`, per IP prefix)
- `LINK_APPLY_RATE_LIMIT_EMAIL_DOMAIN_MAX` (optional, default `6`, per email domain)
- `LINK_APPLY_RATE_LIMIT_SITE_HOST_MAX` (optional, default `3`, per site host)
- `LINK_BLOCK_DISPOSABLE_EMAIL` (optional, default `true`)
- `LINK_BLOCK_EDU_GOV_EMAIL` (optional, default `true`)
- `LINK_APPLY_DENY_HOSTS` (optional, blocked host list, comma/semicolon separated)

### 友链验证 / Link Verification

- `LINK_VERIFY_WINDOW_MINUTES` (optional, default `120`)
- `LINK_VERIFY_WINDOW_HOURS` (optional legacy alias)
- `LINK_PUBLIC_BASE_URL` (optional, base url for email verification links)
- `LINK_VERIFY_EMAIL_RATE_LIMIT_WINDOW_SEC` (optional, default `1800`)
- `LINK_VERIFY_EMAIL_RATE_LIMIT_MAX` (optional, default `3`, per IP)
- `LINK_VERIFY_EMAIL_RATE_LIMIT_APP_MAX` (optional, default `2`, per application)
- `LINK_VERIFY_EMAIL_COOLDOWN_SEC` (optional, default `600`)

### 审查上报 / Review Reporting

- `LINK_REVIEW_REPORT_TOKEN` (optional, default same as `STATUS_TOKEN`)
- `LINK_BACKLINK_TARGET` (optional, backlink target, default `https://www.meowra.cn/`)
- `LINK_BACKLINK_ENFORCE_HOURS` (optional, default `24`)
- `LINK_UNREACHABLE_ENFORCE_HOURS` (optional, default `72`)

## 鉴权 / Auth

- 需要 token 的接口支持请求头 `x-token` 或 `authorization: Bearer TOKEN`。
  Protected endpoints accept `x-token` or `authorization: Bearer TOKEN`.
- `GET /device` 使用 query 参数 `token` 进行鉴权。
  `GET /device` uses the query param `token` for auth.

## 接口 / API

- `GET /` (health)
- `GET /version` (version info)
- `POST /heartbeat` (token)
- `GET /status`
- `GET /status/manual`
- `POST /status/manual` (token)
- `GET /status/admin` (admin page)
- `GET /admin/common.css` (admin CSS)
- `GET /device?id=DEVICE_ID&token=TOKEN`
- `POST /device/status` (token)
- `GET /schedule`
- `POST /schedule` (token)
- `GET /schedule/admin` (admin page)
- `GET /blog`
- `GET /blog/:slug`
- `POST /blog` (token)
- `GET /blog/admin` (admin page)
- `GET /links` (public list)
- `POST /links/apply` (public apply)
- `GET /links/apply/config` (public config: captcha provider/site key)
- `POST /links/verify/http` (public verify)
- `POST /links/verify/email/send` (public verify)
- `GET /links/verify/email?token=...` (public verify)
- `POST /links/verify/reset` (token)
- `GET /links/applications` (token)
- `POST /links/review` (token)
- `POST /links/review/stage/cancel` (token, skip backlink stage)
- `POST /links/sort` (token)
- `POST /links/update` (token)
- `POST /links/delete` (token)
- `GET /links/settings` (token)
- `POST /links/settings` (token)
- `POST /links/settings/test-smtp` (token)
- `GET /links/review/report/tasks` (review token)
- `POST /links/review/report/decision` (review token)
- `POST /links/review/report/manual` (review token)
- `POST /links/review/report/removal` (review token)
- `GET /links/admin` (admin page)
- `GET /visitor`
- `POST /visitor/visit`

## 说明 / Notes

- 在线状态默认 5 分钟未上报视为离线。
  Devices are marked offline after 5 minutes without heartbeat.
- 开启全局手动离线后，`/heartbeat` 直接返回 `200` 且不更新设备状态。
  When global manual-offline is enabled, `/heartbeat` returns `200` without updating status.
- `POST /links/apply` 的 `verify_status` 初始为 `verify_pending`，完成 HTTP / DNS TXT / 首页 meta / 邮箱验证任意一种后进入 `pending` 审核队列。
  `POST /links/apply` starts as `verify_pending`; HTTP / DNS TXT / homepage meta / email verify moves it to `pending`.
- 公网后端不主动抓取外站，审查与回链检查由内网 `review-reporter` 完成。
  Public backend does not crawl external sites; review/backlink checks are done by internal `review-reporter`.
- 若申请记录包含 `email` 且 SMTP 可用，审核结果会自动邮件通知申请者。
  If an application has `email` and SMTP is configured, review results are emailed automatically.

## 内网审查计算 / Internal Review Scoring

以下为内网 `review-reporter` 当前自动审核规则摘要（实现见 `status-backend/src/bin/review-reporter.rs`）。  
The following is a summary of current auto-review rules in internal `review-reporter`.

### 1) 评分制自动审核（pending applications）

- 初始分：`50`
- 域名风险（`localhost` / 内网段 / `.local`）：`-40`
- 命中垃圾关键词（博彩/代刷/色情等）：`-80`
- 简介长度：
- `8~180` 字：`+12`
- 其它：`-10`
- 头像 URL 合法（http/https）：`+4`
- 抓取主页：
- 请求成功且状态码 2xx/3xx：`+18`
- 请求成功但状态码异常：`-24`
- 请求失败：`-25`
- 页面基础 SEO：
- 含 `<title>`：`+5`
- 含 `description`：`+5`
- 含 `meta`：`+6`（否则 `-6`）
- 包含本站链接（`LINK_BACKLINK_TARGET`）：`+10`，否则 `-10`
- 第三方 SEO 接口（可选，内网 worker 配置后生效）：
- `REVIEW_SEO_PROVIDER=generic`：调用自定义评分接口，返回 `score (0~100)`
- `REVIEW_SEO_PROVIDER=serpapi`：调用 SerpAPI 搜索结果进行扩展评分
- 按 `REVIEW_SEO_MAX_BONUS` 折算为加减分
- 折算公式：`delta = ((score - 50) * max_bonus) / 50`

决策阈值（当前内网 worker 固定）：

- `score >= 80`：自动 `approve`
- `score < 40`：自动 `reject`
- 其余：保持 `pending`（人工审核）

### 1) Score-based Auto Review (pending applications)

- Base score: `50`
- Domain risk (`localhost` / private IP / `.local`): `-40`
- Spam keyword hit (gambling/boost/NSFW etc.): `-80`
- Description length:
- `8~180` chars: `+12`
- otherwise: `-10`
- Avatar URL valid (http/https): `+4`
- Fetch homepage:
- success 2xx/3xx: `+18`
- success but bad status: `-24`
- request failed: `-25`
- Basic SEO:
- has `<title>`: `+5`
- has `description`: `+5`
- has `meta`: `+6` (otherwise `-6`)
- contains backlink (`LINK_BACKLINK_TARGET`): `+10`, else `-10`
- Third-party SEO (optional, enabled in internal worker):
- `REVIEW_SEO_PROVIDER=generic`: call custom scoring API returning `score (0~100)`
- `REVIEW_SEO_PROVIDER=serpapi`: use SerpAPI for extra scoring
- Converted to delta using `REVIEW_SEO_MAX_BONUS`
- Formula: `delta = ((score - 50) * max_bonus) / 50`

Decision thresholds (current internal worker defaults):

- `score >= 80`: auto `approve`
- `score < 40`: auto `reject`
- otherwise: keep `pending` (manual review)

### 2) 通过后的回查与下架

- 回链检查（默认 24h，`LINK_BACKLINK_ENFORCE_HOURS`）：
- 若到期仍未检测到本站链接，自动下架（`removed_no_backlink`）
- 默认使用 HTTP 源码抓取；可选 Playwright 渲染抓取（用于 JavaScript 动态页面）
- 可访问性检查（默认 72h，`LINK_UNREACHABLE_ENFORCE_HOURS`）：
- 连续不可访问达到阈值后自动下架（`removed_unreachable`）

### 2) Post-approval Checks

- Backlink check (default 24h, `LINK_BACKLINK_ENFORCE_HOURS`):
- if still missing by deadline, auto remove (`removed_no_backlink`)
- default HTTP fetch; optional Playwright rendering for JS pages
- Reachability check (default 72h, `LINK_UNREACHABLE_ENFORCE_HOURS`):
- auto remove after consecutive unreachable (`removed_unreachable`)
