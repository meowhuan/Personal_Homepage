mod admin_pages;
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{Datelike, Utc};
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
    music_playing: Option<bool>,
    music_title: Option<String>,
    music_artist: Option<String>,
    music_source: Option<String>,
}

#[derive(Serialize)]
struct DeviceStatus {
    device_id: String,
    device_name: String,
    online: bool,
    last_seen: i64,
    idle_seconds: Option<u64>,
    manual_offline: bool,
    global_manual_offline: bool,
    music_playing: bool,
    music_title: Option<String>,
    music_artist: Option<String>,
    music_source: Option<String>,
    music_updated_at: Option<i64>,
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

#[derive(Deserialize)]
struct VisitPayload {
    visitor_id: String,
}

#[derive(Deserialize)]
struct ManualStatusPayload {
    enabled: bool,
}

#[derive(Serialize)]
struct ManualStatusResponse {
    enabled: bool,
    updated_at: i64,
}

#[derive(Deserialize)]
struct DeviceStatusUpdatePayload {
    device_id: String,
    device_name: Option<String>,
    online: Option<bool>,
    manual_offline: Option<bool>,
    music_playing: Option<bool>,
    music_title: Option<String>,
    music_artist: Option<String>,
    music_source: Option<String>,
}

#[derive(Deserialize, Serialize, Clone)]
struct BlogPost {
    slug: String,
    title: String,
    date: String,
    tag: Option<String>,
    excerpt: String,
    content: Vec<String>,
    sort_order: i64,
    updated_at: i64,
}

#[derive(Serialize)]
struct BlogPostSummary {
    slug: String,
    title: String,
    date: String,
    tag: Option<String>,
    excerpt: String,
    sort_order: i64,
    updated_at: i64,
}

#[derive(Deserialize)]
struct BlogPayload {
    items: Vec<BlogPostInput>,
}

#[derive(Deserialize)]
struct BlogPostInput {
    slug: Option<String>,
    title: String,
    date: String,
    tag: Option<String>,
    excerpt: Option<String>,
    content: Vec<String>,
    sort_order: Option<i64>,
}

#[derive(Serialize)]
struct VisitorStats {
    today: i64,
    month: i64,
    total: i64,
    updated_at: i64,
}

#[derive(Serialize)]
struct VersionInfo {
    service: String,
    version: String,
    music_fields: bool,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let db_path = std::env::var("STATUS_DB").unwrap_or_else(|_| "status.db".to_string());
    let token = std::env::var("STATUS_TOKEN").unwrap_or_else(|_| "KFCVME50".to_string());
    let port = std::env::var("STATUS_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(7999);
    let build_version =
        std::env::var("STATUS_BUILD").unwrap_or_else(|_| "status-backend v1.1-music".to_string());

    let conn = Connection::open(db_path).expect("open db");
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS device_status (
            device_id TEXT PRIMARY KEY,
            device_name TEXT NOT NULL,
            online INTEGER NOT NULL,
            last_seen INTEGER NOT NULL,
            idle_seconds INTEGER,
            music_playing INTEGER NOT NULL DEFAULT 0,
            music_title TEXT,
            music_artist TEXT,
            music_source TEXT,
            music_updated_at INTEGER
        );
        CREATE TABLE IF NOT EXISTS status_control (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            global_manual_offline INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
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
        );
        CREATE TABLE IF NOT EXISTS visitor_visits (
            visitor_id TEXT NOT NULL,
            visit_date TEXT NOT NULL,
            created_at INTEGER NOT NULL,
            PRIMARY KEY (visitor_id, visit_date)
        );
        CREATE TABLE IF NOT EXISTS blog_posts (
            slug TEXT PRIMARY KEY,
            title TEXT NOT NULL,
            date TEXT NOT NULL,
            tag TEXT,
            excerpt TEXT NOT NULL,
            content_json TEXT NOT NULL,
            sort_order INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );",
    )
    .expect("init db");
    let _ = conn.execute(
        "ALTER TABLE device_status ADD COLUMN manual_offline INTEGER NOT NULL DEFAULT 0",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE device_status ADD COLUMN music_playing INTEGER NOT NULL DEFAULT 0",
        [],
    );
    let _ = conn.execute("ALTER TABLE device_status ADD COLUMN music_title TEXT", []);
    let _ = conn.execute("ALTER TABLE device_status ADD COLUMN music_artist TEXT", []);
    let _ = conn.execute("ALTER TABLE device_status ADD COLUMN music_source TEXT", []);
    let _ = conn.execute("ALTER TABLE device_status ADD COLUMN music_updated_at INTEGER", []);
    let _ = conn.execute(
        "INSERT INTO status_control (id, global_manual_offline, updated_at)
         VALUES (1, 0, ?1)
         ON CONFLICT(id) DO NOTHING",
        params![now_ts()],
    );

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
        .route(
            "/version",
            get(move || {
                let build_version = build_version.clone();
                async move {
                    Json(VersionInfo {
                        service: "status-backend".to_string(),
                        version: build_version,
                        music_fields: true,
                    })
                }
            }),
        )
        .route("/heartbeat", post(heartbeat))
        .route("/device", get(delete_device))
        .route("/device/status", post(device_status_update))
        .route("/status", get(status))
        .route(
            "/status/manual",
            get(get_manual_status).post(set_manual_status),
        )
        .route("/status/admin", get(admin_pages::status_admin_page))
        .route("/admin/common.css", get(admin_pages::admin_common_css))
        .route("/schedule", get(schedule_list).post(schedule_update))
        .route("/schedule/admin", get(admin_pages::schedule_admin_page))
        .route("/blog", get(blog_list).post(blog_update))
        .route("/blog/:slug", get(blog_detail))
        .route("/blog/admin", get(admin_pages::blog_admin_page))
        .route("/visitor", get(visitor_stats))
        .route("/visitor/visit", post(visitor_visit))
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

    let conn = state.db.lock().unwrap();
    if is_global_manual_offline(&conn) {
        return StatusCode::OK;
    }
    let now = now_ts();
    let music_playing = payload.music_playing.unwrap_or(false);
    let music_title = payload.music_title;
    let music_artist = payload.music_artist;
    let music_source = payload.music_source;
    tracing::info!(
        "heartbeat recv: device_id={} online={} idle={:?} music_playing={} title={:?} artist={:?} source={:?}",
        payload.device_id,
        payload.online,
        payload.idle_seconds,
        music_playing,
        music_title,
        music_artist,
        music_source
    );
    let _ = conn.execute(
        "INSERT INTO device_status (
            device_id, device_name, online, last_seen, idle_seconds, manual_offline,
            music_playing, music_title, music_artist, music_source, music_updated_at
         )
         VALUES (
            ?1, ?2, ?3, ?4, ?5, COALESCE((SELECT manual_offline FROM device_status WHERE device_id = ?1), 0),
            ?6, ?7, ?8, ?9, ?10
         )
         ON CONFLICT(device_id) DO UPDATE SET
           device_name=excluded.device_name,
           online=excluded.online,
           last_seen=excluded.last_seen,
           idle_seconds=excluded.idle_seconds,
           music_playing=excluded.music_playing,
           music_title=excluded.music_title,
           music_artist=excluded.music_artist,
           music_source=excluded.music_source,
           music_updated_at=excluded.music_updated_at;",
        params![
            payload.device_id,
            payload.device_name,
            payload.online as i32,
            now,
            payload.idle_seconds.map(|v| v as i64),
            music_playing as i32,
            music_title,
            music_artist,
            music_source,
            now,
        ],
    );

    StatusCode::OK
}

async fn status(State(state): State<AppState>) -> impl IntoResponse {
    let now = now_ts();
    let conn = state.db.lock().unwrap();
    let global_manual_offline = is_global_manual_offline(&conn);
    let mut stmt = conn
        .prepare(
            "SELECT device_id, device_name, online, last_seen, idle_seconds, manual_offline,
                    music_playing, music_title, music_artist, music_source, music_updated_at
             FROM device_status
             ORDER BY device_id ASC",
        )
        .unwrap();

    let rows = stmt
        .query_map([], |row| {
            let last_seen: i64 = row.get(3)?;
            let online_flag: i32 = row.get(2)?;
            let manual_offline: i32 = row.get(5)?;
            let music_playing: i32 = row.get(6)?;
            let stale = now.saturating_sub(last_seen) > 300;
            let device_manual_offline = manual_offline == 1;
            let online =
                !global_manual_offline && !device_manual_offline && online_flag == 1 && !stale;
            Ok(DeviceStatus {
                device_id: row.get(0)?,
                device_name: row.get(1)?,
                online,
                last_seen,
                idle_seconds: row.get::<_, Option<i64>>(4)?.map(|v| v as u64),
                manual_offline: device_manual_offline,
                global_manual_offline,
                music_playing: music_playing == 1,
                music_title: row.get(7)?,
                music_artist: row.get(8)?,
                music_source: row.get(9)?,
                music_updated_at: row.get(10)?,
            })
        })
        .unwrap();

    let list: Vec<DeviceStatus> = rows.filter_map(Result::ok).collect();

    Json(list)
}

async fn schedule_list(State(state): State<AppState>) -> impl IntoResponse {
    let conn = state.db.lock().unwrap();
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
    let mut conn = state.db.lock().unwrap();
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

async fn blog_list(State(state): State<AppState>) -> impl IntoResponse {
    let conn = state.db.lock().unwrap();
    let mut stmt = match conn.prepare(
        "SELECT slug, title, date, tag, excerpt, sort_order, updated_at
         FROM blog_posts
         ORDER BY sort_order ASC, date DESC, updated_at DESC",
    ) {
        Ok(stmt) => stmt,
        Err(_) => return Json(Vec::<BlogPostSummary>::new()),
    };

    let rows = match stmt.query_map([], |row| {
        Ok(BlogPostSummary {
            slug: row.get(0)?,
            title: row.get(1)?,
            date: row.get(2)?,
            tag: row.get(3)?,
            excerpt: row.get(4)?,
            sort_order: row.get(5)?,
            updated_at: row.get(6)?,
        })
    }) {
        Ok(rows) => rows,
        Err(_) => return Json(Vec::<BlogPostSummary>::new()),
    };

    let list: Vec<BlogPostSummary> = rows.filter_map(Result::ok).collect();
    Json(list)
}

async fn get_manual_status(State(state): State<AppState>) -> impl IntoResponse {
    let conn = state.db.lock().unwrap();
    let result = conn.query_row(
        "SELECT global_manual_offline, updated_at FROM status_control WHERE id = 1",
        [],
        |row| {
            Ok(ManualStatusResponse {
                enabled: row.get::<_, i32>(0)? == 1,
                updated_at: row.get(1)?,
            })
        },
    );
    let payload = result.unwrap_or(ManualStatusResponse {
        enabled: false,
        updated_at: 0,
    });
    Json(payload)
}

async fn set_manual_status(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ManualStatusPayload>,
) -> impl IntoResponse {
    if !authorized(&headers, &state.token) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let now = now_ts();
    let conn = state.db.lock().unwrap();
    if conn
        .execute(
            "INSERT INTO status_control (id, global_manual_offline, updated_at)
             VALUES (1, ?1, ?2)
             ON CONFLICT(id) DO UPDATE SET
               global_manual_offline = excluded.global_manual_offline,
               updated_at = excluded.updated_at",
            params![payload.enabled as i32, now],
        )
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    (
        StatusCode::OK,
        Json(ManualStatusResponse {
            enabled: payload.enabled,
            updated_at: now,
        }),
    )
        .into_response()
}

async fn device_status_update(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<DeviceStatusUpdatePayload>,
) -> impl IntoResponse {
    if !authorized(&headers, &state.token) {
        return StatusCode::UNAUTHORIZED;
    }

    let now = now_ts();
    let conn = state.db.lock().unwrap();
    let existing = conn
        .query_row(
            "SELECT device_name, online, last_seen, idle_seconds, manual_offline,
                    music_playing, music_title, music_artist, music_source, music_updated_at
             FROM device_status
             WHERE device_id = ?1",
            params![payload.device_id.as_str()],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, i32>(1)?,
                    row.get::<_, i64>(2)?,
                    row.get::<_, Option<i64>>(3)?,
                    row.get::<_, i32>(4)?,
                    row.get::<_, i32>(5)?,
                    row.get::<_, Option<String>>(6)?,
                    row.get::<_, Option<String>>(7)?,
                    row.get::<_, Option<String>>(8)?,
                    row.get::<_, Option<i64>>(9)?,
                ))
            },
        )
        .ok();

    let device_name = payload
        .device_name
        .or_else(|| existing.as_ref().map(|v| v.0.clone()))
        .unwrap_or_else(|| payload.device_id.clone());
    let online = payload
        .online
        .unwrap_or_else(|| existing.as_ref().map(|v| v.1 == 1).unwrap_or(false));
    let last_seen = if payload.online.is_some() {
        now
    } else {
        existing.as_ref().map(|v| v.2).unwrap_or(now)
    };
    let idle_seconds = if payload.online.is_some() {
        None
    } else {
        existing.as_ref().and_then(|v| v.3)
    };
    let manual_offline = payload
        .manual_offline
        .unwrap_or_else(|| existing.as_ref().map(|v| v.4 == 1).unwrap_or(false));
    let has_music_update = payload.music_playing.is_some()
        || payload.music_title.is_some()
        || payload.music_artist.is_some()
        || payload.music_source.is_some();
    let music_playing = payload
        .music_playing
        .unwrap_or_else(|| existing.as_ref().map(|v| v.5 == 1).unwrap_or(false));
    let music_title = payload
        .music_title
        .or_else(|| existing.as_ref().and_then(|v| v.6.clone()));
    let music_artist = payload
        .music_artist
        .or_else(|| existing.as_ref().and_then(|v| v.7.clone()));
    let music_source = payload
        .music_source
        .or_else(|| existing.as_ref().and_then(|v| v.8.clone()));
    let music_updated_at = if has_music_update {
        Some(now)
    } else {
        existing.as_ref().and_then(|v| v.9)
    };

    if conn
        .execute(
            "INSERT INTO device_status (
                device_id, device_name, online, last_seen, idle_seconds, manual_offline,
                music_playing, music_title, music_artist, music_source, music_updated_at
             )
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)
             ON CONFLICT(device_id) DO UPDATE SET
               device_name = excluded.device_name,
               online = excluded.online,
               last_seen = excluded.last_seen,
               idle_seconds = excluded.idle_seconds,
               manual_offline = excluded.manual_offline,
               music_playing = excluded.music_playing,
               music_title = excluded.music_title,
               music_artist = excluded.music_artist,
               music_source = excluded.music_source,
               music_updated_at = excluded.music_updated_at",
            params![
                payload.device_id,
                device_name,
                online as i32,
                last_seen,
                idle_seconds,
                manual_offline as i32,
                music_playing as i32,
                music_title,
                music_artist,
                music_source,
                music_updated_at,
            ],
        )
        .is_err()
    {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    StatusCode::OK
}

async fn blog_detail(
    State(state): State<AppState>,
    axum::extract::Path(slug): axum::extract::Path<String>,
) -> impl IntoResponse {
    let conn = state.db.lock().unwrap();
    let row = conn.query_row(
        "SELECT slug, title, date, tag, excerpt, content_json, sort_order, updated_at
         FROM blog_posts
         WHERE slug = ?1
         LIMIT 1",
        params![slug],
        |row| {
            let content_json: String = row.get(5)?;
            let content = serde_json::from_str::<Vec<String>>(&content_json).unwrap_or_default();
            Ok(BlogPost {
                slug: row.get(0)?,
                title: row.get(1)?,
                date: row.get(2)?,
                tag: row.get(3)?,
                excerpt: row.get(4)?,
                content,
                sort_order: row.get(6)?,
                updated_at: row.get(7)?,
            })
        },
    );

    match row {
        Ok(post) => (StatusCode::OK, Json(post)).into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn blog_update(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<BlogPayload>,
) -> impl IntoResponse {
    if !authorized(&headers, &state.token) {
        return StatusCode::UNAUTHORIZED;
    }

    let now = now_ts();
    let mut conn = state.db.lock().unwrap();
    let tx = match conn.transaction() {
        Ok(tx) => tx,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
    };

    if tx.execute("DELETE FROM blog_posts", []).is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR;
    }

    for (idx, item) in payload.items.into_iter().enumerate() {
        let mut slug = item.slug.unwrap_or_else(|| format!("post-{}-{}", now, idx));
        slug = slug
            .trim()
            .to_lowercase()
            .replace(' ', "-")
            .chars()
            .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-')
            .collect::<String>();
        if slug.is_empty() {
            slug = format!("post-{}-{}", now, idx);
        }
        let sort_order = item.sort_order.unwrap_or(idx as i64);
        let excerpt = item
            .excerpt
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| item.content.first().cloned().unwrap_or_default());
        let content_json = match serde_json::to_string(&item.content) {
            Ok(v) => v,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
        };
        if tx
            .execute(
                "INSERT INTO blog_posts (slug, title, date, tag, excerpt, content_json, sort_order, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                params![
                    slug,
                    item.title,
                    item.date,
                    item.tag,
                    excerpt,
                    content_json,
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

async fn visitor_visit(
    State(state): State<AppState>,
    Json(payload): Json<VisitPayload>,
) -> impl IntoResponse {
    let now = now_ts();
    let today = today_key();
    let conn = state.db.lock().unwrap();
    let _ = conn.execute(
        "INSERT OR IGNORE INTO visitor_visits (visitor_id, visit_date, created_at)
         VALUES (?1, ?2, ?3)",
        params![payload.visitor_id, today, now],
    );
    StatusCode::OK
}

async fn visitor_stats(State(state): State<AppState>) -> impl IntoResponse {
    let now = now_ts();
    let today = today_key();
    let month_prefix = month_key();
    let conn = state.db.lock().unwrap();
    let today_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM visitor_visits WHERE visit_date = ?1",
            params![today],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let month_count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM visitor_visits WHERE visit_date LIKE ?1",
            params![format!("{}-%", month_prefix)],
            |row| row.get(0),
        )
        .unwrap_or(0);
    let total_count: i64 = conn
        .query_row("SELECT COUNT(*) FROM visitor_visits", [], |row| row.get(0))
        .unwrap_or(0);
    Json(VisitorStats {
        today: today_count,
        month: month_count,
        total: total_count,
        updated_at: now,
    })
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
    let _ = conn.execute(
        "DELETE FROM device_status WHERE device_id = ?1",
        params![q.id],
    );
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

fn is_global_manual_offline(conn: &Connection) -> bool {
    conn.query_row(
        "SELECT global_manual_offline FROM status_control WHERE id = 1",
        [],
        |row| row.get::<_, i32>(0),
    )
    .map(|v| v == 1)
    .unwrap_or(false)
}

fn now_ts() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn today_key() -> String {
    let now = Utc::now();
    format!("{:04}-{:02}-{:02}", now.year(), now.month(), now.day())
}

fn month_key() -> String {
    let now = Utc::now();
    format!("{:04}-{:02}", now.year(), now.month())
}
