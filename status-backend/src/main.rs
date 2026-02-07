use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
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
