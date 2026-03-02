mod admin_pages;
use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{Datelike, Utc};
use lettre::{
    message::Mailbox, transport::smtp::authentication::Credentials, AsyncSmtpTransport,
    AsyncTransport, Message, Tokio1Executor,
};
use reqwest::Url;
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
    review_report_token: String,
    notifier: Arc<Notifier>,
    auto_review: Arc<AutoReviewConfig>,
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
    content_md: String,
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
    content: Option<Vec<String>>,
    content_md: Option<String>,
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

#[derive(Serialize)]
struct FriendLink {
    id: String,
    name: String,
    url: String,
    avatar_url: Option<String>,
    description: Option<String>,
    tags: Option<String>,
    sort_order: i64,
    created_at: i64,
}

#[derive(Deserialize)]
struct LinkApplyPayload {
    site_name: String,
    site_url: String,
    avatar_url: Option<String>,
    description: Option<String>,
    email: Option<String>,
    note: Option<String>,
}

#[derive(Serialize)]
struct ApiMessage {
    message: String,
}

#[derive(Serialize)]
struct LinkApplication {
    id: i64,
    site_name: String,
    site_url: String,
    avatar_url: Option<String>,
    description: Option<String>,
    email: Option<String>,
    note: Option<String>,
    status: String,
    ip: Option<String>,
    user_agent: Option<String>,
    review_note: Option<String>,
    created_at: i64,
    updated_at: i64,
}

#[derive(Deserialize)]
struct LinkReviewPayload {
    application_id: i64,
    action: String,
    sort_order: Option<i64>,
    tags: Option<String>,
    review_note: Option<String>,
}

#[derive(Deserialize)]
struct LinkSortPayload {
    items: Vec<LinkSortItem>,
}

#[derive(Deserialize)]
struct LinkSortItem {
    id: String,
    sort_order: i64,
}

#[derive(Deserialize)]
struct LinkUpdatePayload {
    id: String,
    name: String,
    url: String,
    avatar_url: Option<String>,
    description: Option<String>,
    tags: Option<String>,
    sort_order: Option<i64>,
}

#[derive(Deserialize)]
struct LinkDeletePayload {
    id: String,
}

#[derive(Serialize)]
struct LinkSettingsResponse {
    tg_bot_token: Option<String>,
    tg_chat_id: Option<String>,
    smtp_host: Option<String>,
    smtp_port: Option<u16>,
    smtp_user: Option<String>,
    smtp_pass_set: bool,
    smtp_from: Option<String>,
    smtp_to: Option<String>,
    smtp_starttls: bool,
}

#[derive(Deserialize)]
struct LinkSettingsPayload {
    tg_bot_token: Option<String>,
    tg_chat_id: Option<String>,
    smtp_host: Option<String>,
    smtp_port: Option<u16>,
    smtp_user: Option<String>,
    smtp_pass: Option<String>,
    smtp_from: Option<String>,
    smtp_to: Option<String>,
    smtp_starttls: Option<bool>,
}

#[derive(Deserialize)]
struct SmtpTestPayload {
    recipient: Option<String>,
}

#[derive(Deserialize)]
struct ReviewDecisionReportPayload {
    application_id: i64,
    action: String,
    sort_order: Option<i64>,
    tags: Option<String>,
    review_note: Option<String>,
    send_email: Option<bool>,
}

#[derive(Deserialize)]
struct ReviewRemovalReportPayload {
    link_id: String,
    application_id: Option<i64>,
    app_status: Option<String>,
    reason: Option<String>,
    send_email: Option<bool>,
}

#[derive(Deserialize)]
struct ReviewManualReportPayload {
    application_id: i64,
    review_note: Option<String>,
    send_admin_notify: Option<bool>,
}

#[derive(Serialize)]
struct ReviewTasksResponse {
    pending_applications: Vec<PendingApplicationTask>,
    active_links: Vec<ActiveLinkTask>,
    now_ts: i64,
    backlink_target: String,
    backlink_enforce_hours: i64,
    unreachable_enforce_hours: i64,
}

#[derive(Serialize)]
struct PendingApplicationTask {
    id: i64,
    site_name: String,
    site_url: String,
    avatar_url: Option<String>,
    description: Option<String>,
    note: Option<String>,
}

#[derive(Serialize)]
struct ActiveLinkTask {
    id: String,
    url: String,
    application_id: Option<i64>,
    backlink_deadline: Option<i64>,
}

#[derive(Clone)]
struct SmtpConfig {
    host: String,
    port: u16,
    username: Option<String>,
    password: Option<String>,
    from: String,
    to: Vec<String>,
    use_starttls: bool,
}

#[derive(Clone)]
struct Notifier {
    tg_bot_token: Option<String>,
    tg_chat_id: Option<String>,
    smtp: Option<SmtpConfig>,
}

#[derive(Clone)]
struct RuntimeNotifyConfig {
    tg_bot_token: Option<String>,
    tg_chat_id: Option<String>,
    smtp: Option<SmtpConfig>,
}

#[derive(Clone)]
struct AutoReviewConfig {
    backlink_window_secs: i64,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SmtpMode {
    StartTls,
    TlsWrapper,
    Plain,
}

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let db_path = std::env::var("STATUS_DB").unwrap_or_else(|_| "status.db".to_string());
    let token = std::env::var("STATUS_TOKEN").unwrap_or_else(|_| "KFCVME50".to_string());
    let review_report_token =
        std::env::var("LINK_REVIEW_REPORT_TOKEN").unwrap_or_else(|_| token.clone());
    let port = std::env::var("STATUS_PORT")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(7999);
    let build_version =
        std::env::var("STATUS_BUILD").unwrap_or_else(|_| "status-backend v1.2-music".to_string());
    let notifier = Arc::new(Notifier::from_env());
    let auto_review = Arc::new(AutoReviewConfig::from_env());

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
            content_md TEXT,
            sort_order INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS friend_links (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            url TEXT NOT NULL,
            avatar_url TEXT,
            description TEXT,
            tags TEXT,
            sort_order INTEGER NOT NULL DEFAULT 0,
            created_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS friend_link_applications (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            site_name TEXT NOT NULL,
            site_url TEXT NOT NULL,
            avatar_url TEXT,
            description TEXT,
            email TEXT,
            note TEXT,
            status TEXT NOT NULL DEFAULT 'pending',
            manual_notified INTEGER NOT NULL DEFAULT 0,
            ip TEXT,
            user_agent TEXT,
            review_note TEXT,
            created_at INTEGER NOT NULL,
            updated_at INTEGER NOT NULL
        );
        CREATE TABLE IF NOT EXISTS friend_link_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
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
    let _ = conn.execute("ALTER TABLE blog_posts ADD COLUMN content_md TEXT", []);
    let _ = conn.execute(
        "ALTER TABLE friend_link_applications ADD COLUMN review_note TEXT",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE friend_link_applications ADD COLUMN manual_notified INTEGER NOT NULL DEFAULT 0",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE friend_links ADD COLUMN application_id INTEGER",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE friend_links ADD COLUMN backlink_status TEXT NOT NULL DEFAULT 'pending'",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE friend_links ADD COLUMN backlink_deadline INTEGER",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE friend_links ADD COLUMN backlink_checked_at INTEGER",
        [],
    );
    let _ = conn.execute(
        "ALTER TABLE friend_links ADD COLUMN unreachable_since INTEGER",
        [],
    );
    let _ = conn.execute(
        "INSERT INTO status_control (id, global_manual_offline, updated_at)
         VALUES (1, 0, ?1)
         ON CONFLICT(id) DO NOTHING",
        params![now_ts()],
    );

    let state = AppState {
        db: Arc::new(Mutex::new(conn)),
        token,
        review_report_token,
        notifier,
        auto_review,
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
        .route("/links", get(links_list))
        .route("/links/apply", post(links_apply))
        .route("/links/applications", get(links_applications))
        .route("/links/review", post(links_review))
        .route("/links/sort", post(links_sort))
        .route("/links/update", post(links_update))
        .route("/links/delete", post(links_delete))
        .route("/links/settings", get(links_settings_get).post(links_settings_set))
        .route("/links/settings/test-smtp", post(links_settings_test_smtp))
        .route("/links/review/report/tasks", get(links_review_report_tasks))
        .route("/links/review/report/decision", post(links_review_report_decision))
        .route("/links/review/report/manual", post(links_review_report_manual))
        .route("/links/review/report/removal", post(links_review_report_removal))
        .route("/links/admin", get(admin_pages::links_admin_page))
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
        "SELECT slug, title, date, tag, excerpt, content_json, content_md, sort_order, updated_at
         FROM blog_posts
         WHERE slug = ?1
         LIMIT 1",
        params![slug],
        |row| {
            let content_json: String = row.get(5)?;
            let content = serde_json::from_str::<Vec<String>>(&content_json).unwrap_or_default();
            let content_md = row
                .get::<_, Option<String>>(6)?
                .unwrap_or_else(|| content.join("\n"));
            Ok(BlogPost {
                slug: row.get(0)?,
                title: row.get(1)?,
                date: row.get(2)?,
                tag: row.get(3)?,
                excerpt: row.get(4)?,
                content,
                content_md,
                sort_order: row.get(7)?,
                updated_at: row.get(8)?,
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
        let input_content = item.content.unwrap_or_default();
        let content_md = item
            .content_md
            .unwrap_or_else(|| input_content.join("\n"));
        let content: Vec<String> = if !input_content.is_empty() {
            input_content
        } else {
            content_md.split('\n').map(|v| v.to_string()).collect()
        };
        let excerpt = item
            .excerpt
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| {
                content
                    .iter()
                    .find(|line| !line.trim().is_empty())
                    .cloned()
                    .unwrap_or_default()
            });
        let content_json = match serde_json::to_string(&content) {
            Ok(v) => v,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR,
        };
        if tx
            .execute(
                "INSERT INTO blog_posts (slug, title, date, tag, excerpt, content_json, content_md, sort_order, updated_at)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
                params![
                    slug,
                    item.title,
                    item.date,
                    item.tag,
                    excerpt,
                    content_json,
                    content_md,
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

async fn links_list(State(state): State<AppState>) -> impl IntoResponse {
    let conn = state.db.lock().unwrap();
    let mut stmt = match conn.prepare(
        "SELECT id, name, url, avatar_url, description, tags, sort_order, created_at
         FROM friend_links
         ORDER BY sort_order ASC, created_at DESC",
    ) {
        Ok(stmt) => stmt,
        Err(_) => return Json(Vec::<FriendLink>::new()),
    };

    let rows = match stmt.query_map([], |row| {
        Ok(FriendLink {
            id: row.get(0)?,
            name: row.get(1)?,
            url: row.get(2)?,
            avatar_url: row.get(3)?,
            description: row.get(4)?,
            tags: row.get(5)?,
            sort_order: row.get(6)?,
            created_at: row.get(7)?,
        })
    }) {
        Ok(rows) => rows,
        Err(_) => return Json(Vec::<FriendLink>::new()),
    };

    Json(rows.filter_map(Result::ok).collect::<Vec<_>>())
}

async fn links_apply(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<LinkApplyPayload>,
) -> impl IntoResponse {
    let site_name = payload.site_name.trim();
    if site_name.is_empty() || site_name.chars().count() > 64 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiMessage {
                message: "站点名称长度需在 1-64 字符内".to_string(),
            }),
        )
            .into_response();
    }

    let site_url = payload.site_url.trim();
    if !is_valid_http_url(site_url) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiMessage {
                message: "站点地址必须是可访问的 http/https 链接".to_string(),
            }),
        )
            .into_response();
    }

    let avatar_url = normalize_optional(payload.avatar_url, 255);
    if avatar_url
        .as_deref()
        .is_some_and(|value| !is_valid_http_url(value))
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiMessage {
                message: "头像地址格式不正确（需为 http/https）".to_string(),
            }),
        )
            .into_response();
    }

    let description = normalize_optional(payload.description, 280);
    let email = normalize_optional(payload.email, 128);
    let note = normalize_optional(payload.note, 280);
    let ip = client_ip(&headers);
    let user_agent = headers
        .get("user-agent")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.to_string());
    let now = now_ts();
    let application_id: i64;

    {
        let conn = state.db.lock().unwrap();
        let pending_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM friend_link_applications
                 WHERE site_url = ?1 AND status = 'pending'",
                params![site_url],
                |row| row.get(0),
            )
            .unwrap_or(0);
        if pending_count > 0 {
            return (
                StatusCode::CONFLICT,
                Json(ApiMessage {
                    message: "该站点已有待处理申请，请勿重复提交".to_string(),
                }),
            )
                .into_response();
        }
        let linked_count: i64 = conn
            .query_row(
                "SELECT COUNT(*) FROM friend_links WHERE url = ?1",
                params![site_url],
                |row| row.get(0),
            )
            .unwrap_or(0);
        if linked_count > 0 {
            return (
                StatusCode::CONFLICT,
                Json(ApiMessage {
                    message: "该站点已在友链列表中".to_string(),
                }),
            )
                .into_response();
        }

        let inserted = conn.execute(
            "INSERT INTO friend_link_applications (
                site_name, site_url, avatar_url, description, email, note,
                status, ip, user_agent, created_at, updated_at
             ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, 'pending', ?7, ?8, ?9, ?10)",
            params![
                site_name,
                site_url,
                avatar_url,
                description,
                email,
                note,
                ip,
                user_agent,
                now,
                now
            ],
        );
        if inserted.is_err() {
            return (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(ApiMessage {
                    message: "提交失败，请稍后重试".to_string(),
                }),
            )
                .into_response();
        }
        application_id = conn.last_insert_rowid();
    }

    (
        StatusCode::CREATED,
        Json(ApiMessage {
            message: format!("友链申请已提交，编号 #{}，等待审核", application_id),
        }),
    )
        .into_response()
}

async fn links_applications(State(state): State<AppState>, headers: HeaderMap) -> impl IntoResponse {
    if !authorized(&headers, &state.token) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let conn = state.db.lock().unwrap();
    let mut stmt = match conn.prepare(
        "SELECT id, site_name, site_url, avatar_url, description, email, note, status,
                ip, user_agent, review_note, created_at, updated_at
         FROM friend_link_applications
         ORDER BY status = 'pending' DESC, created_at DESC",
    ) {
        Ok(stmt) => stmt,
        Err(_) => return (StatusCode::OK, Json(Vec::<LinkApplication>::new())).into_response(),
    };
    let rows = match stmt.query_map([], |row| {
        Ok(LinkApplication {
            id: row.get(0)?,
            site_name: row.get(1)?,
            site_url: row.get(2)?,
            avatar_url: row.get(3)?,
            description: row.get(4)?,
            email: row.get(5)?,
            note: row.get(6)?,
            status: row.get(7)?,
            ip: row.get(8)?,
            user_agent: row.get(9)?,
            review_note: row.get(10)?,
            created_at: row.get(11)?,
            updated_at: row.get(12)?,
        })
    }) {
        Ok(rows) => rows,
        Err(_) => return (StatusCode::OK, Json(Vec::<LinkApplication>::new())).into_response(),
    };

    (StatusCode::OK, Json(rows.filter_map(Result::ok).collect::<Vec<_>>())).into_response()
}

async fn links_review(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<LinkReviewPayload>,
) -> impl IntoResponse {
    if !authorized(&headers, &state.token) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    match perform_review_decision(&state, payload, true, false).await {
        Ok(message) => (StatusCode::OK, Json(ApiMessage { message })).into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(ApiMessage {
                message: err.to_string(),
            }),
        )
            .into_response(),
    }
}

async fn links_review_report_decision(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ReviewDecisionReportPayload>,
) -> impl IntoResponse {
    if !authorized(&headers, &state.review_report_token) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let review_payload = LinkReviewPayload {
        application_id: payload.application_id,
        action: payload.action,
        sort_order: payload.sort_order,
        tags: payload.tags,
        review_note: payload.review_note,
    };
    match perform_review_decision(
        &state,
        review_payload,
        payload.send_email.unwrap_or(true),
        true,
    )
    .await
    {
        Ok(message) => (StatusCode::OK, Json(ApiMessage { message })).into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(ApiMessage {
                message: err.to_string(),
            }),
        )
            .into_response(),
    }
}

async fn links_review_report_manual(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ReviewManualReportPayload>,
) -> impl IntoResponse {
    if !authorized(&headers, &state.review_report_token) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let now = now_ts();
    let review_note = normalize_optional(payload.review_note, 280);
    let app_row = {
        let conn = state.db.lock().unwrap();
        conn.query_row(
            "SELECT site_name, site_url, email, status, manual_notified
             FROM friend_link_applications WHERE id = ?1",
            params![payload.application_id],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, Option<String>>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, i32>(4)?,
                ))
            },
        )
        .ok()
    };
    let (site_name, site_url, _email, status, manual_notified) = match app_row {
        Some(v) => v,
        None => {
            return (
                StatusCode::NOT_FOUND,
                Json(ApiMessage {
                    message: "申请记录不存在".to_string(),
                }),
            )
                .into_response()
        }
    };
    if status != "pending" {
        return (
            StatusCode::OK,
            Json(ApiMessage {
                message: "申请状态已非 pending，跳过手动提醒".to_string(),
            }),
        )
            .into_response();
    }

    {
        let conn = state.db.lock().unwrap();
        let _ = conn.execute(
            "UPDATE friend_link_applications
             SET review_note = ?1, updated_at = ?2
             WHERE id = ?3",
            params![review_note, now, payload.application_id],
        );
    }

    let should_notify = payload.send_admin_notify.unwrap_or(true) && manual_notified == 0;
    if should_notify {
        let notify_cfg = {
            let conn = state.db.lock().unwrap();
            state.notifier.runtime_config(&conn)
        };
        let subject = "New friend-link application (manual review required)".to_string();
        let msg = format!(
            "New friend-link application (manual review required)\nsite: {}\nurl: {}\nreview_note: {}",
            site_name,
            site_url,
            review_note.as_deref().unwrap_or("-")
        );
        let send_result = if let Some(smtp_cfg) = notify_cfg.smtp.as_ref() {
            if smtp_cfg.to.is_empty() {
                Err("smtp to recipients is empty".to_string())
            } else {
                state
                    .notifier
                    .send_smtp(Some(smtp_cfg), &subject, &msg, None)
                    .await
            }
        } else {
            Err("smtp not configured".to_string())
        };
        if let Err(err) = send_result {
            tracing::warn!("manual review smtp notify failed: {}", err);
        } else {
            let conn = state.db.lock().unwrap();
            let _ = conn.execute(
                "UPDATE friend_link_applications
                 SET manual_notified = 1, updated_at = ?1
                 WHERE id = ?2",
                params![now, payload.application_id],
            );
        }
    }

    (
        StatusCode::OK,
        Json(ApiMessage {
            message: if should_notify {
                "已标记为待人工审核并推送提醒".to_string()
            } else {
                "已标记为待人工审核（提醒已发送过）".to_string()
            },
        }),
    )
        .into_response()
}

async fn links_review_report_tasks(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !authorized(&headers, &state.review_report_token) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let conn = state.db.lock().unwrap();

    let pending_applications = {
        let mut stmt = match conn.prepare(
            "SELECT id, site_name, site_url, avatar_url, description, note
             FROM friend_link_applications
             WHERE status = 'pending'
             ORDER BY created_at ASC",
        ) {
            Ok(stmt) => stmt,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };
        let rows = match stmt.query_map([], |row| {
            Ok(PendingApplicationTask {
                id: row.get(0)?,
                site_name: row.get(1)?,
                site_url: row.get(2)?,
                avatar_url: row.get(3)?,
                description: row.get(4)?,
                note: row.get(5)?,
            })
        }) {
            Ok(rows) => rows,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };
        rows.filter_map(Result::ok).collect::<Vec<_>>()
    };

    let active_links = {
        let mut stmt = match conn.prepare(
            "SELECT id, url, application_id, backlink_deadline
             FROM friend_links
             ORDER BY created_at ASC",
        ) {
            Ok(stmt) => stmt,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };
        let rows = match stmt.query_map([], |row| {
            Ok(ActiveLinkTask {
                id: row.get(0)?,
                url: row.get(1)?,
                application_id: row.get(2)?,
                backlink_deadline: row.get(3)?,
            })
        }) {
            Ok(rows) => rows,
            Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
        };
        rows.filter_map(Result::ok).collect::<Vec<_>>()
    };

    (
        StatusCode::OK,
        Json(ReviewTasksResponse {
            pending_applications,
            active_links,
            now_ts: now_ts(),
            backlink_target: std::env::var("LINK_BACKLINK_TARGET")
                .ok()
                .filter(|v| !v.trim().is_empty())
                .unwrap_or_else(|| "https://www.meowra.cn/".to_string()),
            backlink_enforce_hours: state.auto_review.backlink_window_secs / 3600,
            unreachable_enforce_hours: std::env::var("LINK_UNREACHABLE_ENFORCE_HOURS")
                .ok()
                .and_then(|v| v.parse::<i64>().ok())
                .unwrap_or(72),
        }),
    )
        .into_response()
}

async fn links_review_report_removal(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<ReviewRemovalReportPayload>,
) -> impl IntoResponse {
    if !authorized(&headers, &state.review_report_token) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let now = now_ts();
    let app_status = payload
        .app_status
        .unwrap_or_else(|| "removed_external_review".to_string());
    let reason = payload
        .reason
        .unwrap_or_else(|| "由内网审查服务判定下架".to_string());
    match remove_link_and_notify(
        &state,
        payload.link_id,
        payload.application_id,
        &app_status,
        &reason,
        now,
        payload.send_email.unwrap_or(true),
    )
    .await
    {
        Ok(_) => (
            StatusCode::OK,
            Json(ApiMessage {
                message: "已执行下架并同步状态".to_string(),
            }),
        )
            .into_response(),
        Err(err) => (
            StatusCode::BAD_REQUEST,
            Json(ApiMessage {
                message: format!("下架失败: {}", err),
            }),
        )
            .into_response(),
    }
}

async fn perform_review_decision(
    state: &AppState,
    payload: LinkReviewPayload,
    send_email: bool,
    send_admin_smtp_notify: bool,
) -> Result<String, String> {
    let action = payload.action.trim().to_lowercase();
    if action != "approve" && action != "reject" {
        return Err("action 仅支持 approve/reject".to_string());
    }

    let now = now_ts();
    let review_note = normalize_optional(payload.review_note, 280);
    let (site_name, site_url, applicant_email) = {
        let mut conn = state.db.lock().map_err(|_| "db lock failed".to_string())?;
        let tx = match conn.transaction() {
            Ok(tx) => tx,
            Err(_) => return Err("db transaction failed".to_string()),
        };
        let app_row = tx
            .query_row(
                "SELECT site_name, site_url, avatar_url, description, email
                 FROM friend_link_applications WHERE id = ?1",
                params![payload.application_id],
                |row| {
                    Ok((
                        row.get::<_, String>(0)?,
                        row.get::<_, String>(1)?,
                        row.get::<_, Option<String>>(2)?,
                        row.get::<_, Option<String>>(3)?,
                        row.get::<_, Option<String>>(4)?,
                    ))
                },
            )
            .ok();
        let (site_name, site_url, avatar_url, description, applicant_email) = match app_row {
            Some(v) => v,
            None => return Err("申请记录不存在".to_string()),
        };

        if action == "approve" {
            let link_id = format!("link-{}-{}", slugify_ascii(&site_name), payload.application_id);
            let tags = normalize_optional(payload.tags, 120);
            let sort_order = payload.sort_order.unwrap_or(now);
            let backlink_window = state.auto_review.backlink_window_secs.max(3600);
            if tx
                .execute(
                    "INSERT INTO friend_links (
                        id, name, url, avatar_url, description, tags, sort_order, created_at,
                        application_id, backlink_status, backlink_deadline, backlink_checked_at, unreachable_since
                     )
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, 'pending', ?10, NULL, NULL)
                     ON CONFLICT(id) DO UPDATE SET
                       name = excluded.name,
                       url = excluded.url,
                       avatar_url = excluded.avatar_url,
                       description = excluded.description,
                       tags = excluded.tags,
                       sort_order = excluded.sort_order,
                       application_id = excluded.application_id,
                       backlink_status = excluded.backlink_status,
                       backlink_deadline = excluded.backlink_deadline,
                       backlink_checked_at = excluded.backlink_checked_at,
                       unreachable_since = excluded.unreachable_since",
                    params![
                        link_id,
                        site_name,
                        site_url,
                        avatar_url,
                        description,
                        tags,
                        sort_order,
                        now,
                        payload.application_id,
                        now + backlink_window,
                    ],
                )
                .is_err()
            {
                return Err("写入友链失败".to_string());
            }
        }

        if tx
            .execute(
                "UPDATE friend_link_applications
                 SET status = ?1, review_note = ?2, updated_at = ?3
                 WHERE id = ?4",
                params![action, review_note.clone(), now, payload.application_id],
            )
            .is_err()
        {
            return Err("更新申请状态失败".to_string());
        }
        if tx.commit().is_err() {
            return Err("事务提交失败".to_string());
        }
        (site_name, site_url, applicant_email)
    };

    let applicant_email = normalize_optional(applicant_email, 128);
    let mail_note: String;
    if let Some(email) = applicant_email {
        if !send_email {
            mail_note = "（邮件通知已跳过）".to_string();
        } else {
        let notify_cfg = {
            let conn = state.db.lock().map_err(|_| "db lock failed".to_string())?;
            state.notifier.runtime_config(&conn)
        };
        if let Err(err) = state
            .notifier
            .notify_review_result_email(
                notify_cfg.smtp.as_ref(),
                &email,
                &site_name,
                &site_url,
                &action,
                review_note.as_deref(),
            )
            .await
        {
            tracing::warn!("review result mail failed: {}", err);
            mail_note = "（邮件通知失败）".to_string();
        } else {
            mail_note = "（邮件通知已发送）".to_string();
        }
        }
    } else {
        mail_note = "（申请方未提供邮箱）".to_string();
    }

    if send_admin_smtp_notify {
        let notify_cfg = {
            let conn = state.db.lock().map_err(|_| "db lock failed".to_string())?;
            state.notifier.runtime_config(&conn)
        };
        if let Some(smtp_cfg) = notify_cfg.smtp.as_ref() {
            if !smtp_cfg.to.is_empty() {
                let subject = format!(
                    "Auto review result: {} ({})",
                    if action == "approve" { "APPROVE" } else { "REJECT" },
                    site_name
                );
                let body = format!(
                    "Auto review result\napplication_id: {}\naction: {}\nsite: {}\nurl: {}\nreview_note: {}",
                    payload.application_id,
                    action,
                    site_name,
                    site_url,
                    review_note.as_deref().unwrap_or("-")
                );
                if let Err(err) = state
                    .notifier
                    .send_smtp(Some(smtp_cfg), &subject, &body, None)
                    .await
                {
                    tracing::warn!("auto review admin smtp notify failed: {}", err);
                }
            }
        }
    }

    Ok(if action == "approve" {
        format!("已通过并加入友链列表{}", mail_note)
    } else {
        format!("已拒绝该申请{}", mail_note)
    })
}

async fn links_sort(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<LinkSortPayload>,
) -> impl IntoResponse {
    if !authorized(&headers, &state.token) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let mut conn = state.db.lock().unwrap();
    let tx = match conn.transaction() {
        Ok(tx) => tx,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    for item in payload.items {
        if tx
            .execute(
                "UPDATE friend_links SET sort_order = ?1 WHERE id = ?2",
                params![item.sort_order, item.id],
            )
            .is_err()
        {
            return StatusCode::INTERNAL_SERVER_ERROR.into_response();
        }
    }
    if tx.commit().is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }
    (
        StatusCode::OK,
        Json(ApiMessage {
            message: "排序已更新".to_string(),
        }),
    )
        .into_response()
}

async fn links_update(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<LinkUpdatePayload>,
) -> impl IntoResponse {
    if !authorized(&headers, &state.token) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let id = payload.id.trim();
    if id.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiMessage {
                message: "id 不能为空".to_string(),
            }),
        )
            .into_response();
    }
    let name = payload.name.trim();
    if name.is_empty() || name.chars().count() > 64 {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiMessage {
                message: "名称长度需在 1-64 字符内".to_string(),
            }),
        )
            .into_response();
    }
    let url = payload.url.trim();
    if !is_valid_http_url(url) {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiMessage {
                message: "URL 需为 http/https".to_string(),
            }),
        )
            .into_response();
    }
    let avatar_url = normalize_optional(payload.avatar_url, 255);
    if avatar_url
        .as_deref()
        .is_some_and(|value| !is_valid_http_url(value))
    {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiMessage {
                message: "头像 URL 需为 http/https".to_string(),
            }),
        )
            .into_response();
    }
    let description = normalize_optional(payload.description, 280);
    let tags = normalize_optional(payload.tags, 120);
    let sort_order = payload.sort_order.unwrap_or(0);

    let conn = state.db.lock().unwrap();
    let updated = conn
        .execute(
            "UPDATE friend_links
             SET name = ?1, url = ?2, avatar_url = ?3, description = ?4, tags = ?5, sort_order = ?6
             WHERE id = ?7",
            params![name, url, avatar_url, description, tags, sort_order, id],
        )
        .unwrap_or(0);
    if updated == 0 {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiMessage {
                message: "友链不存在".to_string(),
            }),
        )
            .into_response();
    }
    (
        StatusCode::OK,
        Json(ApiMessage {
            message: "友链已更新".to_string(),
        }),
    )
        .into_response()
}

async fn links_delete(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<LinkDeletePayload>,
) -> impl IntoResponse {
    if !authorized(&headers, &state.token) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let id = payload.id.trim();
    if id.is_empty() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiMessage {
                message: "id 不能为空".to_string(),
            }),
        )
            .into_response();
    }
    let conn = state.db.lock().unwrap();
    let deleted = conn
        .execute("DELETE FROM friend_links WHERE id = ?1", params![id])
        .unwrap_or(0);
    if deleted == 0 {
        return (
            StatusCode::NOT_FOUND,
            Json(ApiMessage {
                message: "友链不存在".to_string(),
            }),
        )
            .into_response();
    }
    (
        StatusCode::OK,
        Json(ApiMessage {
            message: "友链已删除".to_string(),
        }),
    )
        .into_response()
}

async fn links_settings_get(
    State(state): State<AppState>,
    headers: HeaderMap,
) -> impl IntoResponse {
    if !authorized(&headers, &state.token) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let conn = state.db.lock().unwrap();
    let settings = state.notifier.resolved_settings(&conn);
    (StatusCode::OK, Json(settings)).into_response()
}

async fn links_settings_set(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<LinkSettingsPayload>,
) -> impl IntoResponse {
    if !authorized(&headers, &state.token) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let now = now_ts();
    let mut conn = state.db.lock().unwrap();
    let tx = match conn.transaction() {
        Ok(tx) => tx,
        Err(_) => return StatusCode::INTERNAL_SERVER_ERROR.into_response(),
    };
    let updates = [
        (
            "tg_bot_token",
            payload.tg_bot_token.map(|v| v.trim().chars().take(256).collect::<String>()),
        ),
        (
            "tg_chat_id",
            payload.tg_chat_id.map(|v| v.trim().chars().take(128).collect::<String>()),
        ),
        (
            "smtp_host",
            payload.smtp_host.map(|v| v.trim().chars().take(255).collect::<String>()),
        ),
        (
            "smtp_port",
            payload.smtp_port.map(|v| v.to_string()),
        ),
        (
            "smtp_user",
            payload.smtp_user.map(|v| v.trim().chars().take(255).collect::<String>()),
        ),
        ("smtp_pass", payload.smtp_pass.map(|v| v.trim().to_string())),
        (
            "smtp_from",
            payload.smtp_from.map(|v| v.trim().chars().take(255).collect::<String>()),
        ),
        (
            "smtp_to",
            payload.smtp_to.map(|v| v.trim().chars().take(512).collect::<String>()),
        ),
        (
            "smtp_starttls",
            payload.smtp_starttls.map(|v| if v { "1".to_string() } else { "0".to_string() }),
        ),
    ];
    for (key, value) in updates {
        if let Some(value) = value {
            if value.is_empty() {
                if tx
                    .execute(
                        "DELETE FROM friend_link_settings WHERE key = ?1",
                        params![key],
                    )
                    .is_err()
                {
                    return StatusCode::INTERNAL_SERVER_ERROR.into_response();
                }
            } else if tx
                .execute(
                    "INSERT INTO friend_link_settings (key, value, updated_at)
                     VALUES (?1, ?2, ?3)
                     ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
                    params![key, value, now],
                )
                .is_err()
            {
                return StatusCode::INTERNAL_SERVER_ERROR.into_response();
            }
        }
    }
    if tx.commit().is_err() {
        return StatusCode::INTERNAL_SERVER_ERROR.into_response();
    }

    let settings = state.notifier.resolved_settings(&conn);
    (StatusCode::OK, Json(settings)).into_response()
}

async fn links_settings_test_smtp(
    State(state): State<AppState>,
    headers: HeaderMap,
    Json(payload): Json<SmtpTestPayload>,
) -> impl IntoResponse {
    if !authorized(&headers, &state.token) {
        return StatusCode::UNAUTHORIZED.into_response();
    }
    let recipient = normalize_optional(payload.recipient, 255);
    let notify_cfg = {
        let conn = state.db.lock().unwrap();
        state.notifier.runtime_config(&conn)
    };
    if notify_cfg.smtp.is_none() {
        return (
            StatusCode::BAD_REQUEST,
            Json(ApiMessage {
                message: "SMTP 未配置完整，请先保存 SMTP Host/From/To".to_string(),
            }),
        )
            .into_response();
    }

    let now_local = chrono::Local::now().format("%Y-%m-%d %H:%M:%S").to_string();
    let subject = "SMTP test from Meow Links Admin";
    let body = format!(
        "这是一封 SMTP 测试邮件。\n\n发送时间：{}\n服务：status-backend\n说明：若收到此邮件，表示当前 SMTP 配置可用。",
        now_local
    );
    let override_to = recipient.map(|v| vec![v.clone()]);
    let send_result = state
        .notifier
        .send_smtp(
            notify_cfg.smtp.as_ref(),
            subject,
            &body,
            override_to,
        )
        .await;
    match send_result {
        Ok(()) => (
            StatusCode::OK,
            Json(ApiMessage {
                message: "SMTP 测试邮件发送成功".to_string(),
            }),
        )
            .into_response(),
        Err(err) => (
            StatusCode::BAD_GATEWAY,
            Json(ApiMessage {
                message: format!("SMTP 测试失败: {}", err),
            }),
        )
            .into_response(),
    }
}


async fn remove_link_and_notify(
    state: &AppState,
    link_id: String,
    application_id: Option<i64>,
    app_status: &str,
    review_note: &str,
    now: i64,
    send_email: bool,
) -> Result<(), String> {
    let applicant = if let Some(app_id) = application_id {
        let conn = state.db.lock().map_err(|_| "db lock failed".to_string())?;
        conn.query_row(
            "SELECT email, site_name, site_url FROM friend_link_applications WHERE id = ?1",
            params![app_id],
            |row| {
                Ok((
                    row.get::<_, Option<String>>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                ))
            },
        )
        .ok()
    } else {
        None
    };

    {
        let conn = state.db.lock().map_err(|_| "db lock failed".to_string())?;
        let _ = conn.execute("DELETE FROM friend_links WHERE id = ?1", params![link_id]);
        if let Some(app_id) = application_id {
            let _ = conn.execute(
                "UPDATE friend_link_applications
                 SET status = ?1, review_note = ?2, updated_at = ?3
                 WHERE id = ?4",
                params![app_status, review_note, now, app_id],
            );
        }
    }

    if send_email {
        if let Some((email, app_name, app_url)) = applicant {
        if let Some(email) = normalize_optional(email, 128) {
            let notify_cfg = {
                let conn = state.db.lock().map_err(|_| "db lock failed".to_string())?;
                state.notifier.runtime_config(&conn)
            };
            if let Err(err) = state
                .notifier
                .notify_review_result_email(
                    notify_cfg.smtp.as_ref(),
                    &email,
                    &app_name,
                    &app_url,
                    "reject",
                    Some(review_note),
                )
                .await
            {
                tracing::warn!("link removal mail failed: {}", err);
            }
        }
    }
    }
    Ok(())
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

impl Notifier {
    fn from_env() -> Self {
        let tg_bot_token = normalize_env("LINK_TG_BOT_TOKEN");
        let tg_chat_id = normalize_env("LINK_TG_CHAT_ID");

        let smtp = {
            let host = normalize_env("LINK_SMTP_HOST");
            let from = normalize_env("LINK_SMTP_FROM");
            let to = normalize_env("LINK_SMTP_TO");
            match (host, from) {
                (Some(host), Some(from)) => {
                    let port = std::env::var("LINK_SMTP_PORT")
                        .ok()
                        .and_then(|v| v.parse::<u16>().ok())
                        .unwrap_or(587);
                    let username = normalize_env("LINK_SMTP_USER");
                    let password = normalize_env("LINK_SMTP_PASS");
                    let use_starttls = std::env::var("LINK_SMTP_STARTTLS")
                        .ok()
                        .map(|v| v != "0" && v.to_lowercase() != "false")
                        .unwrap_or(true);
                    let recipients = to.map(|v| split_recipients(&v)).unwrap_or_default();
                    Some(SmtpConfig {
                        host,
                        port,
                        username,
                        password,
                        from,
                        to: recipients,
                        use_starttls,
                    })
                }
                _ => None,
            }
        };

        Self {
            tg_bot_token,
            tg_chat_id,
            smtp,
        }
    }

    fn runtime_config(&self, conn: &Connection) -> RuntimeNotifyConfig {
        let tg_bot_token = read_setting(conn, "tg_bot_token").or_else(|| self.tg_bot_token.clone());
        let tg_chat_id = read_setting(conn, "tg_chat_id").or_else(|| self.tg_chat_id.clone());

        let smtp_host = read_setting(conn, "smtp_host")
            .or_else(|| self.smtp.as_ref().map(|v| v.host.clone()));
        let smtp_from = read_setting(conn, "smtp_from")
            .or_else(|| self.smtp.as_ref().map(|v| v.from.clone()));
        let smtp_to = read_setting(conn, "smtp_to").or_else(|| {
            self.smtp
                .as_ref()
                .map(|v| v.to.join(","))
        });
        let smtp_port = read_setting(conn, "smtp_port")
            .and_then(|v| v.parse::<u16>().ok())
            .or_else(|| self.smtp.as_ref().map(|v| v.port));
        let smtp_user = read_setting(conn, "smtp_user")
            .or_else(|| self.smtp.as_ref().and_then(|v| v.username.clone()));
        let smtp_pass = read_setting(conn, "smtp_pass")
            .or_else(|| self.smtp.as_ref().and_then(|v| v.password.clone()));
        let smtp_starttls = read_setting(conn, "smtp_starttls")
            .map(|v| v != "0" && v.to_lowercase() != "false")
            .or_else(|| self.smtp.as_ref().map(|v| v.use_starttls))
            .unwrap_or(true);

        let smtp = match (smtp_host, smtp_from) {
            (Some(host), Some(from)) => {
                let recipients = smtp_to
                    .as_deref()
                    .map(split_recipients)
                    .unwrap_or_default();
                Some(SmtpConfig {
                    host,
                    port: smtp_port.unwrap_or(587),
                    username: smtp_user,
                    password: smtp_pass,
                    from,
                    to: recipients,
                    use_starttls: smtp_starttls,
                })
            }
            _ => None,
        };

        RuntimeNotifyConfig {
            tg_bot_token,
            tg_chat_id,
            smtp,
        }
    }

    fn resolved_settings(&self, conn: &Connection) -> LinkSettingsResponse {
        let cfg = self.runtime_config(conn);
        LinkSettingsResponse {
            tg_bot_token: cfg.tg_bot_token.clone(),
            tg_chat_id: cfg.tg_chat_id.clone(),
            smtp_host: cfg.smtp.as_ref().map(|v| v.host.clone()),
            smtp_port: cfg.smtp.as_ref().map(|v| v.port),
            smtp_user: cfg.smtp.as_ref().and_then(|v| v.username.clone()),
            smtp_pass_set: cfg
                .smtp
                .as_ref()
                .and_then(|v| v.password.clone())
                .is_some(),
            smtp_from: cfg.smtp.as_ref().map(|v| v.from.clone()),
            smtp_to: cfg.smtp.as_ref().map(|v| v.to.join(",")),
            smtp_starttls: cfg.smtp.as_ref().map(|v| v.use_starttls).unwrap_or(true),
        }
    }

    async fn notify_review_result_email(
        &self,
        smtp_cfg: Option<&SmtpConfig>,
        applicant_email: &str,
        site_name: &str,
        site_url: &str,
        action: &str,
        review_note: Option<&str>,
    ) -> Result<(), String> {
        let status_text = if action == "approve" {
            "已通过"
        } else {
            "未通过"
        };
        let backlink_reminder = review_note
            .map(|v| v.to_lowercase())
            .filter(|v| v.contains("未检测到本站链接") || v.contains("no_backlink"))
            .map(|_| {
                "\n\n提醒：当前未检测到你的网站包含本站链接。\n请在站点首页添加本站友链后再提交/等待复核：\nhttps://www.meowra.cn/"
            })
            .unwrap_or("");
        let subject = format!("友链申请审核结果：{}", status_text);
        let body = format!(
            "你好，\n\n你提交的友链申请已完成审核。\n\n站点名称：{}\n站点地址：{}\n审核结果：{}\n审核备注：{}{}\n\n此邮件由系统自动发送，请勿直接回复。",
            site_name,
            site_url,
            status_text,
            review_note.unwrap_or("-"),
            backlink_reminder
        );
        self.send_smtp(
            smtp_cfg,
            &subject,
            &body,
            Some(vec![applicant_email.to_string()]),
        )
        .await
    }

    async fn send_smtp(
        &self,
        cfg: Option<&SmtpConfig>,
        subject: &str,
        message: &str,
        override_to: Option<Vec<String>>,
    ) -> Result<(), String> {
        let cfg = cfg.ok_or_else(|| "smtp config missing".to_string())?;
        let recipients = override_to.unwrap_or_else(|| cfg.to.clone());
        if recipients.is_empty() {
            return Err("smtp recipients missing".to_string());
        }
        let from: Mailbox = cfg
            .from
            .parse()
            .map_err(|err| format!("smtp from invalid: {}", err))?;
        let mut builder = Message::builder().from(from).subject(subject);
        for recipient in &recipients {
            let mailbox: Mailbox = recipient
                .parse()
                .map_err(|err| format!("smtp to invalid: {}", err))?;
            builder = builder.to(mailbox);
        }
        let email = builder
            .body(message.to_string())
            .map_err(|err| format!("smtp message build failed: {}", err))?;

        let primary = if cfg.use_starttls {
            SmtpMode::StartTls
        } else if cfg.port == 465 {
            SmtpMode::TlsWrapper
        } else {
            SmtpMode::Plain
        };
        let mut modes = vec![primary];
        for mode in [SmtpMode::StartTls, SmtpMode::TlsWrapper, SmtpMode::Plain] {
            if !modes.contains(&mode) {
                modes.push(mode);
            }
        }

        let mut errors = Vec::new();
        for mode in modes {
            let transport_builder = match mode {
                SmtpMode::StartTls => AsyncSmtpTransport::<Tokio1Executor>::starttls_relay(&cfg.host)
                    .map_err(|err| format!("starttls config failed: {}", err))
                    .map(|b| b.port(cfg.port)),
                SmtpMode::TlsWrapper => AsyncSmtpTransport::<Tokio1Executor>::relay(&cfg.host)
                    .map_err(|err| format!("tls config failed: {}", err))
                    .map(|b| b.port(cfg.port)),
                SmtpMode::Plain => Ok(
                    AsyncSmtpTransport::<Tokio1Executor>::builder_dangerous(&cfg.host).port(cfg.port),
                ),
            };
            let mut transport_builder = match transport_builder {
                Ok(v) => v,
                Err(err) => {
                    errors.push(format!("mode {}: {}", smtp_mode_label(mode), err));
                    continue;
                }
            };
            if let (Some(username), Some(password)) = (&cfg.username, &cfg.password) {
                transport_builder = transport_builder
                    .credentials(Credentials::new(username.clone(), password.clone()));
            }
            let transport = transport_builder.build();
            match transport.send(email.clone()).await {
                Ok(_) => return Ok(()),
                Err(err) => {
                    errors.push(format!("mode {}: {}", smtp_mode_label(mode), err));
                }
            }
        }
        Err(format!("smtp send failed: {}", errors.join(" | ")))
    }
}

impl AutoReviewConfig {
    fn from_env() -> Self {
        let backlink_window_secs = std::env::var("LINK_BACKLINK_ENFORCE_HOURS")
            .ok()
            .and_then(|v| v.parse::<i64>().ok())
            .map(|hours| hours.max(1) * 3600)
            .unwrap_or(24 * 3600);

        Self {
            backlink_window_secs,
        }
    }
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

fn normalize_env(key: &str) -> Option<String> {
    std::env::var(key)
        .ok()
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn normalize_optional(value: Option<String>, max_len: usize) -> Option<String> {
    value
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .map(|v| {
            if v.chars().count() > max_len {
                v.chars().take(max_len).collect::<String>()
            } else {
                v
            }
        })
}

fn is_valid_http_url(value: &str) -> bool {
    Url::parse(value)
        .map(|url| matches!(url.scheme(), "http" | "https") && url.host_str().is_some())
        .unwrap_or(false)
}

fn client_ip(headers: &HeaderMap) -> Option<String> {
    if let Some(value) = headers.get("x-forwarded-for").and_then(|v| v.to_str().ok()) {
        let ip = value
            .split(',')
            .next()
            .map(|v| v.trim().to_string())
            .filter(|v| !v.is_empty());
        if ip.is_some() {
            return ip;
        }
    }
    headers
        .get("x-real-ip")
        .and_then(|v| v.to_str().ok())
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
}

fn read_setting(conn: &Connection, key: &str) -> Option<String> {
    conn.query_row(
        "SELECT value FROM friend_link_settings WHERE key = ?1",
        params![key],
        |row| row.get::<_, String>(0),
    )
    .ok()
    .map(|v| v.trim().to_string())
    .filter(|v| !v.is_empty())
}

fn split_recipients(raw: &str) -> Vec<String> {
    raw.split(|c| c == ',' || c == '，' || c == ';' || c == '；' || c == '\n' || c == '\r')
        .map(|v| v.trim().to_string())
        .filter(|v| !v.is_empty())
        .collect()
}

fn slugify_ascii(value: &str) -> String {
    let mut out = value
        .trim()
        .to_lowercase()
        .replace(' ', "-")
        .chars()
        .filter(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-')
        .collect::<String>();
    if out.is_empty() {
        out = "friend".to_string();
    }
    out
}

fn smtp_mode_label(mode: SmtpMode) -> &'static str {
    match mode {
        SmtpMode::StartTls => "starttls",
        SmtpMode::TlsWrapper => "tls",
        SmtpMode::Plain => "plain",
    }
}
