use axum::{
    extract::{Query, State},
    http::{HeaderMap, StatusCode},
    response::{Html, IntoResponse},
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

#[derive(Deserialize)]
struct VisitPayload {
    visitor_id: String,
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

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let db_path = std::env::var("STATUS_DB").unwrap_or_else(|_| "status.db".to_string());
    let token = std::env::var("STATUS_TOKEN").unwrap_or_else(|_| "KFCVME50".to_string());
    let port = std::env::var("STATUS_PORT").ok().and_then(|v| v.parse().ok()).unwrap_or(7999);

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
        .route("/version", get(|| async { "status-backend v1.0" }))
        .route("/heartbeat", post(heartbeat))
        .route("/device", get(delete_device))
        .route("/status", get(status))
        .route("/schedule", get(schedule_list).post(schedule_update))
        .route("/schedule/admin", get(schedule_admin_page))
        .route("/blog", get(blog_list).post(blog_update))
        .route("/blog/:slug", get(blog_detail))
        .route("/blog/admin", get(blog_admin_page))
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
      textarea, input, select {
        width: 100%;
        box-sizing: border-box;
        border-radius: 12px;
        border: 1px solid #eadbea;
        padding: 10px 12px;
        font-size: 13px;
        background: rgba(255, 255, 255, 0.8);
      }
      .row {
        display: flex;
        gap: 12px;
        margin-bottom: 12px;
      }
      .row > div {
        flex: 1;
      }
      .toolbar {
        display: flex;
        gap: 10px;
        margin: 12px 0;
        flex-wrap: wrap;
      }
      .list {
        display: flex;
        flex-direction: column;
        gap: 12px;
      }
      .item {
        border-radius: 16px;
        border: 1px solid rgba(234, 219, 234, 0.9);
        background: rgba(255, 255, 255, 0.7);
        padding: 12px;
      }
      .item-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 8px;
        margin-bottom: 8px;
        font-size: 12px;
        color: #7b6b7a;
      }
      .item-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
        gap: 10px;
      }
      .item-note {
        margin-top: 10px;
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
      .ghost {
        background: rgba(255, 255, 255, 0.9);
        color: #2b1d2a;
        border: 1px solid #eadbea;
      }
      .danger {
        background: #a03555;
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
          <div class="toolbar">
            <button id="add">新增行程</button>
            <button id="load" class="ghost">加载</button>
            <button id="save">保存</button>
          </div>
          <div class="list" id="list"></div>
          <div class="hint">提示：时间/标题必填，其它可留空。拖拽排序暂不支持，可用“排序”字段。</div>
          <div class="status" id="status"></div>
        </div>
      </div>
    </div>
    <script>
      const statusEl = document.getElementById("status");
      const tokenEl = document.getElementById("token");
      const apiEl = document.getElementById("api");
      const listEl = document.getElementById("list");

      const setStatus = (text) => { statusEl.textContent = text; };

      const createItem = (item = {}) => {
        const wrap = document.createElement("div");
        wrap.className = "item";
        wrap.innerHTML = `
          <div class="item-header">
            <span>行程项</span>
            <button class="danger" data-remove>删除</button>
          </div>
          <div class="item-grid">
            <div>
              <label>时间 *</label>
              <input data-time placeholder="如：今晚 19:00" value="${item.time || ""}" />
            </div>
            <div>
              <label>标题 *</label>
              <input data-title placeholder="如：晚饭" value="${item.title || ""}" />
            </div>
            <div>
              <label>地点</label>
              <input data-location placeholder="如：咖啡馆" value="${item.location || ""}" />
            </div>
            <div>
              <label>标签</label>
              <input data-tag placeholder="如：私事" value="${item.tag || ""}" />
            </div>
            <div>
              <label>排序</label>
              <input data-sort type="number" placeholder="0" value="${item.sort_order ?? ""}" />
            </div>
          </div>
          <div class="item-note">
            <label>备注</label>
            <input data-note placeholder="可选" value="${item.note || ""}" />
          </div>
        `;
        wrap.querySelector("[data-remove]").addEventListener("click", () => {
          wrap.remove();
        });
        return wrap;
      };

      const readItems = () => {
        const items = [];
        listEl.querySelectorAll(".item").forEach((el, idx) => {
          const time = el.querySelector("[data-time]").value.trim();
          const title = el.querySelector("[data-title]").value.trim();
          if (!time || !title) return;
          const location = el.querySelector("[data-location]").value.trim();
          const tag = el.querySelector("[data-tag]").value.trim();
          const note = el.querySelector("[data-note]").value.trim();
          const sortRaw = el.querySelector("[data-sort]").value.trim();
          items.push({
            time,
            title,
            location: location || undefined,
            tag: tag || undefined,
            note: note || undefined,
            sort_order: sortRaw === "" ? idx : Number(sortRaw)
          });
        });
        return items;
      };

      const loadSchedule = async () => {
        try {
          setStatus("加载中...");
          const res = await fetch(apiEl.value);
          const items = await res.json();
          listEl.innerHTML = "";
          items.forEach((item) => listEl.appendChild(createItem(item)));
          if (items.length === 0) {
            listEl.appendChild(createItem());
          }
          setStatus("已加载");
        } catch (err) {
          setStatus("加载失败");
        }
      };

      const saveSchedule = async () => {
        try {
          const payload = { items: readItems() };
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

      document.getElementById("add").addEventListener("click", () => {
        listEl.appendChild(createItem());
      });
      document.getElementById("load").addEventListener("click", loadSchedule);
      document.getElementById("save").addEventListener("click", saveSchedule);
      loadSchedule();
    </script>
  </body>
</html>"#,
    )
}

async fn blog_admin_page() -> impl IntoResponse {
    Html(
        r#"<!doctype html>
<html lang="zh-CN">
  <head>
    <meta charset="UTF-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>Meow 博客管理</title>
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
        max-width: 980px;
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
        min-height: 88px;
        resize: vertical;
      }
      .row {
        display: flex;
        gap: 12px;
        margin-bottom: 12px;
      }
      .row > div {
        flex: 1;
      }
      .toolbar {
        display: flex;
        gap: 10px;
        margin: 12px 0;
        flex-wrap: wrap;
      }
      .list {
        display: flex;
        flex-direction: column;
        gap: 12px;
      }
      .item {
        border-radius: 16px;
        border: 1px solid rgba(234, 219, 234, 0.9);
        background: rgba(255, 255, 255, 0.7);
        padding: 12px;
      }
      .item-header {
        display: flex;
        align-items: center;
        justify-content: space-between;
        gap: 8px;
        margin-bottom: 8px;
        font-size: 12px;
        color: #7b6b7a;
      }
      .item-grid {
        display: grid;
        grid-template-columns: repeat(auto-fit, minmax(160px, 1fr));
        gap: 10px;
      }
      .item-note {
        margin-top: 10px;
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
      .ghost {
        background: rgba(255, 255, 255, 0.9);
        color: #2b1d2a;
        border: 1px solid #eadbea;
      }
      .danger {
        background: #a03555;
      }
    </style>
  </head>
  <body>
    <div class="wrap">
      <div class="window">
        <div class="titlebar">
          <span class="dot"></span>
          Meow Blog Admin
        </div>
        <div class="content">
          <div class="row">
            <div>
              <label>Token</label>
              <input id="token" type="password" placeholder="输入 STATUS_TOKEN" />
            </div>
            <div>
              <label>接口地址</label>
              <input id="api" type="text" value="/blog" />
            </div>
          </div>
          <div class="toolbar">
            <button id="add">新增文章</button>
            <button id="load" class="ghost">加载</button>
            <button id="save">保存</button>
          </div>
          <div class="list" id="list"></div>
          <div class="hint">提示：标题/日期必填。标签可用逗号分隔（如：碎碎念,开发,偶然）；正文每行一段，图片可用 ![说明](图片URL) 或直接填图片 URL。</div>
          <div class="status" id="status"></div>
        </div>
      </div>
    </div>
    <script>
      const statusEl = document.getElementById("status");
      const tokenEl = document.getElementById("token");
      const apiEl = document.getElementById("api");
      const listEl = document.getElementById("list");

      const setStatus = (text) => { statusEl.textContent = text; };
      const esc = (value = "") => value
        .replaceAll("&", "&amp;")
        .replaceAll("<", "&lt;")
        .replaceAll(">", "&gt;")
        .replaceAll("\"", "&quot;");

      const createItem = (item = {}) => {
        const wrap = document.createElement("div");
        wrap.className = "item";
        wrap.innerHTML = `
          <div class="item-header">
            <span>博客文章</span>
            <button class="danger" data-remove>删除</button>
          </div>
          <div class="item-grid">
            <div>
              <label>Slug</label>
              <input data-slug placeholder="如：my-first-post" value="${esc(item.slug || "")}" />
            </div>
            <div>
              <label>标题 *</label>
              <input data-title placeholder="如：博客开张" value="${esc(item.title || "")}" />
            </div>
            <div>
              <label>日期 *</label>
              <input data-date placeholder="如：2026-02-16" value="${esc(item.date || "")}" />
            </div>
            <div>
              <label>标签（逗号分隔）</label>
              <input data-tag placeholder="如：碎碎念,开发,偶然" value="${esc(item.tag || "")}" />
            </div>
            <div>
              <label>排序</label>
              <input data-sort type="number" placeholder="0" value="${item.sort_order ?? ""}" />
            </div>
          </div>
          <div class="item-note">
            <label>摘要</label>
            <textarea data-excerpt placeholder="列表展示摘要">${esc(item.excerpt || "")}</textarea>
          </div>
          <div class="item-note">
            <label>正文（每行一段）</label>
            <textarea data-content placeholder="第一段&#10;![配图说明](https://example.com/pic.jpg)&#10;或直接图片链接 https://example.com/pic.jpg">${esc((item.content || []).join("\n"))}</textarea>
          </div>
        `;
        wrap.querySelector("[data-remove]").addEventListener("click", () => {
          wrap.remove();
        });
        return wrap;
      };

      const readItems = () => {
        const items = [];
        listEl.querySelectorAll(".item").forEach((el, idx) => {
          const slug = el.querySelector("[data-slug]").value.trim();
          const title = el.querySelector("[data-title]").value.trim();
          const date = el.querySelector("[data-date]").value.trim();
          if (!title || !date) return;
          const tag = el.querySelector("[data-tag]").value.trim();
          const excerpt = el.querySelector("[data-excerpt]").value.trim();
          const sortRaw = el.querySelector("[data-sort]").value.trim();
          const content = el
            .querySelector("[data-content]")
            .value
            .split("\n")
            .map((v) => v.trim())
            .filter(Boolean);
          items.push({
            slug: slug || undefined,
            title,
            date,
            tag: tag || undefined,
            excerpt: excerpt || undefined,
            content,
            sort_order: sortRaw === "" ? idx : Number(sortRaw)
          });
        });
        return items;
      };

      const loadBlog = async () => {
        try {
          setStatus("加载中...");
          const listRes = await fetch(apiEl.value);
          if (!listRes.ok) throw new Error("list failed");
          const list = await listRes.json();
          const detailTasks = list.map(async (item) => {
            try {
              const detailRes = await fetch(`${apiEl.value}/${encodeURIComponent(item.slug)}`);
              if (!detailRes.ok) throw new Error("detail failed");
              return await detailRes.json();
            } catch (err) {
              return { ...item, content: item.excerpt ? [item.excerpt] : [] };
            }
          });
          const items = await Promise.all(detailTasks);
          listEl.innerHTML = "";
          items.forEach((item) => listEl.appendChild(createItem(item)));
          if (items.length === 0) {
            listEl.appendChild(createItem());
          }
          setStatus("已加载");
        } catch (err) {
          setStatus("加载失败");
        }
      };

      const saveBlog = async () => {
        try {
          const payload = { items: readItems() };
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

      document.getElementById("add").addEventListener("click", () => {
        listEl.appendChild(createItem());
      });
      document.getElementById("load").addEventListener("click", loadBlog);
      document.getElementById("save").addEventListener("click", saveBlog);
      loadBlog();
    </script>
  </body>
</html>"#,
    )
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

fn today_key() -> String {
    let now = Utc::now();
    format!("{:04}-{:02}-{:02}", now.year(), now.month(), now.day())
}

fn month_key() -> String {
    let now = Utc::now();
    format!("{:04}-{:02}", now.year(), now.month())
}
