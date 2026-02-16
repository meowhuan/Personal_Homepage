# Status Backend

Rust + SQLite 在线状态后端。
Rust + SQLite online status backend.

## 功能 / Features

- 心跳上报 / Heartbeat reporting
- 在线列表 / Status list
- 设备删除（鉴权）/ Delete device with token

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
- `GET /device?id=DEVICE_ID&token=TOKEN`
- `GET /schedule`
- `POST /schedule` (需要 token 鉴权)
- `GET /schedule/admin` (简易管理页面)
- `GET /blog`
- `GET /blog/:slug`
- `POST /blog` (需要 token 鉴权)
- `GET /blog/admin` (简易管理页面)

说明：删除接口为 `GET`，需要 `id` 与 `token`。`/schedule` 与 `/blog` 的更新接口需在请求头中携带 `x-token` 或 `authorization: Bearer TOKEN`。
Note: delete API is `GET` and requires `id` and `token`.
