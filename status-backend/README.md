# Status Backend

Rust + SQLite 在线状态后端。

## 运行 / Run

```bash
cargo run
```

## 环境变量 / Env

- `STATUS_PORT` (default 7999)
- `STATUS_DB` (default `status.db`)
- `STATUS_TOKEN` (set in `.env`)

## 接口 / API

- `POST /heartbeat`
- `GET /status`
- `GET /device?id=DEVICE_ID&token=TOKEN`
