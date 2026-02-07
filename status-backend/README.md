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

说明：删除接口为 `GET`，需要 `id` 与 `token`。
Note: delete API is `GET` and requires `id` and `token`.
