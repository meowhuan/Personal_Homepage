# Status Backend

Rust + SQLite 在线状态后端。
Rust + SQLite online status backend.

## 功能 / Features

- 心跳上报 / Heartbeat reporting
- 在线列表 / Status list
- 设备删除（鉴权）/ Delete device with token
- 全局手动离线开关（开启后心跳静默）/ Global manual-offline switch (heartbeat muted when enabled)
- 设备手动离线状态 / Per-device manual offline status

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

说明：删除接口为 `GET`，需要 `id` 与 `token`。`/schedule`、`/blog`、`/device/status`、`/status/manual` 的更新接口需在请求头中携带 `x-token` 或 `authorization: Bearer TOKEN`。全局手动离线开启后，`/heartbeat` 会直接返回 `200` 且不更新设备状态。
Note: delete API is `GET` and requires `id` and `token`.
