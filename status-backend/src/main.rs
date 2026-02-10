use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse},
    routing::{get, post},
    Json, Router,
};
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::{
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::EnvFilter;

#[derive(Clone)]
struct AppState {
    db: Arc<Mutex<Connection>>,
    token: String,
}

#[derive(Deserialize)]
struct Heartbeat {
    device_id: String,
    device_name: String,
    online: bool,
    idle_seconds: Option<u64>,
}

#[derive(Serialize)]
struct DeviceStatus {
    device_id: String,
    device_name: String,
    online: bool,
    last_seen: i64,
    idle_seconds: Option<u64>,
}

#[derive(Deserialize, Serialize, Clone)]
struct ScheduleItem {
    id: String,
    title: String,
    time: String,
    note: Option<String>,
    location: Option<String>,
    tag: Option<String>,
    sort_order: i64,
    updated_at: i64,
}

#[derive(Deserialize)]
struct SchedulePayload {
    items: Vec<ScheduleItemInput>,
}

#[derive(Deserialize)]
struct ScheduleItemInput {
    id: Option<String>,
    title: String,
    time: String,
    note: Option<String>,
    location: Option<String>,
    tag: Option<String>,
    sort_order: Option<i64>,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let db_path = std::env::var("STATUS_DB").unwrap_or_else(|_| "status.db".to_string());
    let token = std::env::var("STATUS_TOKEN").unwrap_or_else(|_| "KFCVME50".to_string());
    let port = std::env::var("STATUS_PORT").ok().and_then(|v| v.parse().ok()).unwrap_or(799);

    let conn = Connection::open(db_path).expect("open db");
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS device_status (
            device_id TEXT PRIMARY KEY,
            device_name TEXT NOT NULL,
            online INTEGER NOT NULL,
            last_seen INTEGER NOT NULL,
            idle_seconds INTEGER
        );
        CREATE TABLE IF NOT EXISTS schedule_items (
            id TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            time TEXT NOT NULL,
            note TEXT,
            location TEXT,
            tag TEXT,
            sort_order INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );",
    )
    .expect("init db");

    let state = AppState {
        db: Arc::new(Mutex::new(conn)),
        token,
    };

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = Router::new()
        .route("/", get(|| async { "ok" }))
        .route("/heartbeat", post(heartbeat))
        .route("/device", get(delete_device))
        .route("/status", get(status))
        .route("/schedule", get(schedule_list).post(schedule_update))
        .route("/schedule/admin", get(schedule_admin_page))
        .with_state(state)
        .layer(cors);

    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("listening on {}", addr);
    axum::serve(tokio::net::TcpListener::bind(addr).await.unwrap(), app)
        .await
        .unwrap();
}

async fn heartbeat(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<Heartbeat>,
) -> impl IntoResponse {
    if !authorized(&headers, &state.token) {
        return StatusCode::UNAUTHORIZED;
    }

    let now = now_ts();
    let conn = state.db.lock().unwrap();
    let _ = conn.execute(
        "INSERT INTO device_status (device_id, device_name, online, last_seen, idle_seconds)
         VALUES (?1, ?2, ?3, ?4, ?5)
         ON CONFLICT(device_id) DO UPDATE SET
           device_name=excluded.device_name,
           online=excluded.online,
           last_seen=excluded.last_seen,
           idle_seconds=excluded.idle_seconds;",
        params![
            payload.device_id,
            payload.device_name,
            payload.online as i32,
            now,
            payload.idle_seconds.map(|v| v as i64),
        ],
    );

    StatusCode::OK
}

async fn status(State(state): State<AppState>) -> impl IntoResponse {
    let now = now_ts();
    let conn = state.db.lock().unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT device_id, device_name, online, last_seen, idle_seconds
             FROM device_status
             ORDER BY device_id ASC",
        )
        .unwrap();

    let rows = stmt
        .query_map([], |row| {
            let last_seen: i64 = row.get(3)?;
            let online_flag: i32 = row.get(2)?;
            let stale = now.saturating_sub(last_seen) > 300;
            let online = online_flag == 1 && !stale;
            Ok(DeviceStatus {
                device_id: row.get(0)?,
                device_name: row.get(1)?,
                online,
                last_seen,
                idle_seconds: row.get::<_, Option<i64>>(4)?.map(|v| v as u64),
            })
        })
        .unwrap();

    let list: Vec<DeviceStatus> = rows.filter_map(Result::ok).collect();

    Json(list)
}

async fn schedule_list(State(state): State<AppState>) -> impl IntoResponse {
    let mut conn = state.db.lock().unwrap();
    let mut stmt = conn
        .prepare(
            "SELECT id, title, time, note, location, tag, sort_order, updated_at
             FROM schedule_items
             ORDER BY sort_order ASC, updated_at DESC",
        )
        .unwrap();

    let rows = stmt
        .query_map([], |row| {
            Ok(ScheduleItem {
                id: row.get(0)?,
                title: row.get(1)?,
                time: row.get(2)?,
                note: row.get(3)?,
                location: row.get(4)?,
                tag: row.get(5)?,
                sort_order: row.get(6)?,
                updated_at: row.get(7)?,
            })
        })
        .unwrap();

    let list: Vec<ScheduleItem> = rows.filter_map(Result::ok).collect();
    Json(list)
}

async fn schedule_update(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SchedulePayload>,
) -> impl IntoResponse {
    if !authorized(&headers, &state.token) {
        return StatusCode::UNAUTHORIZED;
    }

    let now = now_ts();
    let conn = state.db.lock().unwrap();
    let tx = match conn.transaction() {
        Ok(tx) => tx,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    if tx.execute("DELETE FROM schedule_items", []).is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    for (idx, item) in payload.items.into_iter().enumerate() {
        let id = item
            .id
            .unwrap_or_else(|| format!("schedule-{}-{}", now, idx));
        let sort_order = item.sort_order.unwrap_or(idx as i64);
        if tx
            .execute(
                "INSERT INTO schedule_items (id, title, time, note, location, tag, sort_order, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    id,
                    item.title,
                    item.time,
                    item.note,
                    item.location,
                    item.tag,
                    sort_order,
                    now
                ],
            )
            .is_err()
        {
            return StatusCode::INTERNAL_SERVER_ERROR;
        }
    }

    if tx.commit().is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}

async fn schedule_admin_page() -> impl IntoResponse {
    Html(
        r#"<!doctype html>
<html lang="zh-CN">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Meow 行程表管理</title>
    <style>
      :root {
        color-scheme: light;
      }
      body {
        margin: 0;
        min-height: 100vh;
        font-family: "Kalam", "Segoe UI", sans-serif;
        background: linear-gradient(140deg, #fff1f7, #f5f4ff);
        color: #2b1d2a;
      }
      .wrap {
        max-width: 900px;
        margin: 24px auto;
        padding: 24px;
      }
      .window {
        border-radius: 20px;
        background: rgba(255, 255, 255, 0.75);
        box-shadow: 0 16px 36px rgba(47, 20, 47, 0.12);
        border: 1px solid rgba(234, 219, 234, 0.9);
        overflow: hidden;
      }
      .titlebar {
        display: flex;
        align-items: center;
        gap: 10px;
        padding: 12px 16px;
        background: linear-gradient(90deg, rgba(255, 219, 235, 0.9), rgba(209, 244, 255, 0.9));
        font-size: 14px;
        letter-spacing: 2px;
        text-transform: uppercase;
      }
      .dot {
        width: 10px;
        height: 10px;
        border-radius: 50%;
        background: #f0b1c9;
        box-shadow: 16px 0 0 #f6d48f, 32px 0 0 #b9efdf;
      }
      .content {
        padding: 18px;
      }
      textarea, input {
        width: 100%;
        box-sizing: border-box;
        border-radius: 12px;
        border: 1px solid #eadbea;
        padding: 10px 12px;
        font-size: 13px;
        background: rgba(255, 255, 255, 0.8);
      }
      textarea {
        min-height: 260px;
        font-family: "JetBrains Mono", "Consolas", monospace;
      }
      .row {
        display: flex;
        gap: 12px;
        margin-bottom: 12px;
      }
      .row > div {
        flex: 1;
      }
      button {
        border: 0;
        padding: 10px 16px;
        border-radius: 999px;
        background: #2b1d2a;
        color: #fff;
        cursor: pointer;
        font-size: 13px;
      }
      .hint {
        font-size: 12px;
        color: #7b6b7a;
        margin-top: 8px;
      }
      .status {
        margin-top: 8px;
        font-size: 12px;
        color: #7b6b7a;
      }
    </style>
  </head>
  <body>
    <div class="wrap">
      <div class="window">
        <div class="titlebar">
          <span class="dot"></span>
          Meow Schedule Admin
        </div>
        <div class="content">
          <div class="row">
            <div>
              <label>Token</label>
              <input id="token" type="password" placeholder="输入 STATUS_TOKEN" />
            </div>
            <div>
              <label>接口地址</label>
              <input id="api" type="text" value="/schedule" />
            </div>
          </div>
          <label>行程表 JSON</label>
          <textarea id="payload"></textarea>
          <div class="row">
            <button id="load">加载</button>
            <button id="save">保存</button>
          </div>
          <div class="hint">格式：{ "items": [ { "title": "...", "time": "...", "note": "...", "location": "...", "tag": "...", "sort_order": 0 } ] }</div>
          <div class="status" id="status"></div>
        </div>
      </div>
    </div>
    <script>
      const statusEl = document.getElementById("status");
      const payloadEl = document.getElementById("payload");
      const tokenEl = document.getElementById("token");
      const apiEl = document.getElementById("api");

      const setStatus = (text) => { statusEl.textContent = text; };

      const loadSchedule = async () => {
        try {
          setStatus("加载中...");
          const res = await fetch(apiEl.value);
          const items = await res.json();
          payloadEl.value = JSON.stringify({ items }, null, 2);
          setStatus("已加载");
        } catch (err) {
          setStatus("加载失败");
        }
      };

      const saveSchedule = async () => {
        try {
          const payload = JSON.parse(payloadEl.value);
          setStatus("保存中...");
          const res = await fetch(apiEl.value, {
            method: "POST",
            headers: {
              "content-type": "application/json",
              "x-token": tokenEl.value
            },
            body: JSON.stringify(payload)
          });
          setStatus(res.ok ? "保存成功" : "保存失败");
        } catch (err) {
          setStatus("保存失败");
        }
      };

      document.getElementById("load").addEventListener("click", loadSchedule);
      document.getElementById("save").addEventListener("click", saveSchedule);
      loadSchedule();
    </script>
  </body>
</html>"#,
    )
}

#[derive(Deserialize)]
struct DeleteQuery {
    id: String,
    token: String,
}

async fn delete_device(
    State(state): State<AppState>,
    Query(q): Query<DeleteQuery>,
) -> impl IntoResponse {
    if q.token != state.token {
        return StatusCode::UNAUTHORIZED;
    }
    let conn = state.db.lock().unwrap();
    let _ = conn.execute("DELETE FROM device_status WHERE device_id = ?1", params![q.id]);
    StatusCode::OK
}

fn authorized(headers: &HeaderMap, token: &str) -> bool {
    if let Some(value) = headers.get("x-token") {
        if value.to_str().ok() == Some(token) {
            return true;
        }
    }
    if let Some(value) = headers.get("authorization") {
        if let Ok(s) = value.to_str() {
            if let Some(stripped) = s.strip_prefix("Bearer ") {
                return stripped == token;
            }
        }
    }
    false
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}
