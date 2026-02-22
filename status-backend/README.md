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
