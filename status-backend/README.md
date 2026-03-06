# Status Backend

Rust + SQLite 在线状态后端。
Rust + SQLite online status backend.

## 功能 / Features

- 心跳上报 / Heartbeat reporting
- 在线列表 / Status list
- 设备删除（鉴权）/ Delete device with token
- 全局手动离线开关（开启后心跳静默）/ Global manual-offline switch (heartbeat muted when enabled)
- 设备手动离线状态 / Per-device manual offline status
- 听歌状态字段（是否在听、歌曲名、作者、来源）/ Music status fields (playing/title/artist/source)

## 运行 / Run

```bash
cargo run
```

## 环境变量 / Env

- `STATUS_PORT` (default `7999`)
- `STATUS_DB` (default `status.db`)
- `STATUS_TOKEN` (set in `.env`)
- `LINK_TG_BOT_TOKEN` (optional, Telegram Bot Token)
- `LINK_TG_CHAT_ID` (optional, Telegram chat/user id)
- `LINK_SMTP_HOST` (optional, SMTP host)
- `LINK_SMTP_PORT` (optional, default `587`)
- `LINK_SMTP_USER` (optional, SMTP username)
- `LINK_SMTP_PASS` (optional, SMTP password)
- `LINK_SMTP_FROM` (optional, e.g. `bot@example.com`)
- `LINK_SMTP_TO` (optional, receiver list separated by comma)
- `LINK_SMTP_STARTTLS` (optional, default `true`, set `0`/`false` to disable)
- `LINK_REVIEW_REPORT_TOKEN` (optional, internal review reporter token; default same as `STATUS_TOKEN`)
- `LINK_BACKLINK_ENFORCE_HOURS` (optional, default `24`, backlink grace window when approved)
- `LINK_CAPTCHA_PROVIDER` (optional: `none`/`turnstile`/`hcaptcha`, default `none`)
- `LINK_TURNSTILE_SITE_KEY` (required if `LINK_CAPTCHA_PROVIDER=turnstile`)
- `LINK_TURNSTILE_SECRET` (required if `LINK_CAPTCHA_PROVIDER=turnstile`)
- `LINK_HCAPTCHA_SITE_KEY` (required if `LINK_CAPTCHA_PROVIDER=hcaptcha`)
- `LINK_HCAPTCHA_SECRET` (required if `LINK_CAPTCHA_PROVIDER=hcaptcha`)
- `LINK_APPLY_RATE_LIMIT_WINDOW_SEC` (optional, default `3600`)
- `LINK_APPLY_RATE_LIMIT_MAX` (optional, default `3`, max applies per IP within window)
- `LINK_APPLY_RATE_LIMIT_PREFIX_MAX` (optional, default `8`, max applies per IP prefix within window)
- `LINK_APPLY_RATE_LIMIT_EMAIL_DOMAIN_MAX` (optional, default `6`, max applies per email domain within window)
- `LINK_APPLY_RATE_LIMIT_SITE_HOST_MAX` (optional, default `3`, max applies per site host within window)
- `LINK_BLOCK_DISPOSABLE_EMAIL` (optional, default `true`)
- `LINK_BLOCK_EDU_GOV_EMAIL` (optional, default `true`)
- `LINK_APPLY_DENY_HOSTS` (optional, additional blocked host list; separated by comma/semicolon)
- `LINK_VERIFY_WINDOW_MINUTES` (optional, default `120`, verification window before review)
- `LINK_PUBLIC_BASE_URL` (optional, used for email verification link base url)
- `LINK_VERIFY_EMAIL_RATE_LIMIT_WINDOW_SEC` (optional, default `1800`, rate limit window)
- `LINK_VERIFY_EMAIL_RATE_LIMIT_MAX` (optional, default `3`, max sends per IP within window)
- `LINK_VERIFY_EMAIL_RATE_LIMIT_APP_MAX` (optional, default `2`, max sends per application within window)
- `LINK_VERIFY_EMAIL_COOLDOWN_SEC` (optional, default `600`, cooldown between sends per application)

`.env` 示例 / Example:

```
STATUS_TOKEN=your_token
```

## 接口 / API

- `POST /heartbeat`
- `GET /status`
- `GET /status/admin` (在线状态管理页面)
- `GET /device?id=DEVICE_ID&token=TOKEN`
- `POST /device/status` (需要 token 鉴权，用于手动设置单设备在线/离线状态)
- `GET /status/manual`
- `POST /status/manual` (需要 token 鉴权，用于开启/关闭全局手动离线)
- `GET /schedule`
- `POST /schedule` (需要 token 鉴权)
- `GET /schedule/admin` (简易管理页面)
- `GET /blog`
- `GET /blog/:slug`
- `POST /blog` (需要 token 鉴权)
- `GET /blog/admin` (简易管理页面)
- `GET /links` (公开友链列表)
- `POST /links/apply` (友链申请，无需 token，支持 TG/SMTP 通知)
- `GET /links/apply/config` (公开申请配置，返回 captcha provider/site key)
- `POST /links/verify/http` (公开验证接口：检测 `/.well-known/meow-links.txt` token)
- `POST /links/verify/email/send` (公开验证接口：发送邮箱验证链接，需携带申请时的 `verify_token`，captcha 开启时需带 `captcha_token`)
- `GET /links/verify/email?token=...` (公开验证接口：点击后进入审核队列)
- `GET /links/applications` (需要 token，申请列表)
- `POST /links/review` (需要 token，审核通过/拒绝)
- `POST /links/sort` (需要 token，更新已收录友链排序)
- `POST /links/update` (需要 token，编辑单条友链)
- `POST /links/delete` (需要 token，删除单条友链)
- `GET /links/settings` (需要 token，读取 TG/SMTP 配置)
- `POST /links/settings` (需要 token，保存 TG/SMTP 配置)
- `POST /links/settings/test-smtp` (需要 token，发送 SMTP 测试邮件)
- `POST /links/review/report/decision` (需要 `LINK_REVIEW_REPORT_TOKEN`，内网上报审核结果)
- `POST /links/review/report/manual` (需要 `LINK_REVIEW_REPORT_TOKEN`，内网上报“待人工审核”并触发新申请提醒)
- `POST /links/review/report/removal` (需要 `LINK_REVIEW_REPORT_TOKEN`，内网上报下架结果)
- `GET /links/review/report/tasks` (需要 `LINK_REVIEW_REPORT_TOKEN`，内网拉取待审任务)
- `GET /links/admin` (友链管理页面)

说明：若申请记录包含 `email` 且 SMTP 已配置，`/links/review` 完成后会自动给申请者邮箱发送审核结果通知。
`/links/apply` 风控：支持 captcha（Turnstile/hCaptcha）、按 IP 与网段/IP前缀限流、按邮箱域与站点域限流、一次性邮箱拦截、`edu/gov` 邮箱拦截、站点域名黑名单拦截。申请提交后默认状态为 `verify_pending`，完成以下任一验证后才进入 `pending` 审核队列：
- HTTP 文件：`/.well-known/meow-links.txt` 内容包含 token
- DNS TXT：`_meow-links.<domain>` 记录包含 token
- 首页 meta：在首页 `<head>` 加入 `<meta name="meow-links" content="TOKEN">`（TOKEN 为验证 token）
- 邮箱验证：点击验证邮件链接
审查拆分：公网后端不再主动抓取外站（避免暴露公网服务器 IP）。请将审查任务部署在内网服务，由内网服务调用 `.../review/report/...` 接口将审核/下架结果上报回公网后端。
内网审查服务（`review-reporter`）的发行版部署与 systemd 常驻配置见 `status-backend/DEPLOY.md`。
排障可使用单次模式：`review-reporter --once` 或设置 `REVIEW_RUN_ONCE=1`。

## 审核规则（当前实现）

当前自动审核由内网 `review-reporter` 执行，公网后端仅接收上报结果。

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
- 名称一致性检查（`site_name` vs `<title>`）：
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

### 2) 通过后的回查与下架

- 回链检查（默认 24h，`LINK_BACKLINK_ENFORCE_HOURS`）：
  - 若到期仍未检测到本站链接，自动下架（`removed_no_backlink`）
  - 默认使用 HTTP 源码抓取；可选开启 Playwright 渲染抓取（用于 JavaScript 动态页面）
- 可访问性检查（默认 72h，由内网 worker 侧 `unreachable` 逻辑控制）：
  - 连续不可访问达到阈值后自动下架（`removed_unreachable`）

Playwright 渲染抓取可选环境变量（`review-reporter`）：
- `REVIEW_JS_RENDER`（`1/true` 启用，默认关闭）
- `REVIEW_JS_RENDER_CMD`（默认 `node`）
- `REVIEW_JS_RENDER_SCRIPT`（默认 `scripts/playwright-fetch.mjs`）
- `REVIEW_JS_RENDER_TIMEOUT_SEC`（默认 `18`，范围 `5~60`）
- `REVIEW_JS_RENDER_WAIT_UNTIL`（`load`/`domcontentloaded`/`networkidle`，默认 `networkidle`）
- `REVIEW_JS_RENDER_WAIT_AFTER_MS`（默认 `800`，范围 `0~5000`）
- `REVIEW_JS_RENDER_MAX_PAGES`（每次回链检测最多渲染页数，默认 `2`，范围 `1~8`）

### 3) 邮件通知触发

- 通过/拒绝/自动下架后：
  - 若申请记录有 `email` 且 SMTP 可用，则向申请者发送结果邮件
- 自动审核 `approve/reject` 上报时：
  - 若 SMTP `To` 已配置，会向站长邮箱推送自动审核结果摘要
- 自动审核结果为 `pending`（需人工）上报时：
  - 仅此时向 SMTP `To` 推送“新申请待人工审核”提醒（同申请只推送一次）

`/blog` 正文字段支持两种写法（兼容）：
- `content_md`: Markdown 原文（推荐）
- `content`: 字符串数组（旧格式，仍可用）

当前 Markdown 渲染支持（管理页预览 + 前台详情页）：
- 标题、段落、粗体、斜体、删除线、行内代码
- 代码块（```）
- 无序/有序列表、任务列表（`- [ ]` / `- [x]`）
- 引用（`>`）、分隔线（`---` / `***` / `___`）
- 链接、图片
- 简单表格（`|` 语法）

说明：删除接口为 `GET`，需要 `id` 与 `token`。`/schedule`、`/blog`、`/device/status`、`/status/manual` 的更新接口需在请求头中携带 `x-token` 或 `authorization: Bearer TOKEN`。全局手动离线开启后，`/heartbeat` 会直接返回 `200` 且不更新设备状态。
Note: delete API is `GET` and requires `id` and `token`.
