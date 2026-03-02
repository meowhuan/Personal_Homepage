use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{collections::HashMap, fs, path::PathBuf, time::Duration};

#[derive(Deserialize)]
struct ReviewTasksResponse {
    pending_applications: Vec<PendingApplicationTask>,
    active_links: Vec<ActiveLinkTask>,
    now_ts: i64,
    backlink_target: String,
    backlink_enforce_hours: i64,
    unreachable_enforce_hours: i64,
}

#[derive(Deserialize)]
struct PendingApplicationTask {
    id: i64,
    site_name: String,
    site_url: String,
    avatar_url: Option<String>,
    description: Option<String>,
    note: Option<String>,
}

#[derive(Deserialize)]
struct ActiveLinkTask {
    id: String,
    url: String,
    application_id: Option<i64>,
    backlink_deadline: Option<i64>,
}

#[derive(Serialize, Deserialize, Default)]
struct LocalState {
    unreachable_since: HashMap<String, i64>,
}

#[tokio::main]
async fn main() {
    let base = std::env::var("REVIEW_API_BASE").unwrap_or_else(|_| "http://127.0.0.1:7999".to_string());
    let token = std::env::var("REVIEW_REPORT_TOKEN").unwrap_or_else(|_| "KFCVME50".to_string());
    let interval_secs = std::env::var("REVIEW_LOOP_INTERVAL_SEC")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(300)
        .max(30);
    let state_file = std::env::var("REVIEW_LOCAL_STATE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("review-worker-state.json"));

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .expect("build client");

    loop {
        if let Err(err) = run_once(&client, &base, &token, &state_file).await {
            eprintln!("[review-worker] run_once error: {}", err);
        }
        tokio::time::sleep(Duration::from_secs(interval_secs)).await;
    }
}

async fn run_once(
    client: &reqwest::Client,
    base: &str,
    token: &str,
    state_file: &PathBuf,
) -> Result<(), String> {
    let mut state = load_state(state_file);
    let tasks: ReviewTasksResponse = client
        .get(format!("{}/links/review/report/tasks", base.trim_end_matches('/')))
        .header("x-token", token)
        .send()
        .await
        .map_err(|e| format!("fetch tasks failed: {}", e))?
        .json()
        .await
        .map_err(|e| format!("decode tasks failed: {}", e))?;

    for app in &tasks.pending_applications {
        let decision = evaluate_application(client, app, &tasks.backlink_target).await;
        if decision.action == "pending" {
            continue;
        }
        let payload = json!({
            "application_id": app.id,
            "action": decision.action,
            "sort_order": decision.sort_order,
            "tags": Value::Null,
            "review_note": decision.review_note,
            "send_email": true
        });
        let _ = client
            .post(format!("{}/links/review/report/decision", base.trim_end_matches('/')))
            .header("x-token", token)
            .json(&payload)
            .send()
            .await;
    }

    let now = tasks.now_ts;
    for link in &tasks.active_links {
        let snap = fetch_site(client, &link.url).await;
        let accessible = snap.as_ref().map(|(ok, _)| *ok).unwrap_or(false);
        let page_lower = snap.map(|(_, html)| html.to_lowercase()).unwrap_or_default();
        let has_backlink = contains_backlink(&page_lower, &tasks.backlink_target);

        if let Some(deadline) = link.backlink_deadline {
            if now >= deadline && !has_backlink {
                report_removal(
                    client,
                    base,
                    token,
                    &link.id,
                    link.application_id,
                    "removed_no_backlink",
                    &format!(
                        "友链已下架：在通过后 {} 小时内未检测到本站链接 {}",
                        tasks.backlink_enforce_hours,
                        tasks.backlink_target
                    ),
                )
                .await;
                state.unreachable_since.remove(&link.id);
                continue;
            }
        }

        if accessible {
            state.unreachable_since.remove(&link.id);
            continue;
        }

        let entry = state.unreachable_since.entry(link.id.clone()).or_insert(now);
        if now - *entry >= tasks.unreachable_enforce_hours.max(1) * 3600 {
            report_removal(
                client,
                base,
                token,
                &link.id,
                link.application_id,
                "removed_unreachable",
                &format!("友链已下架：连续 {} 小时无法访问", tasks.unreachable_enforce_hours),
            )
            .await;
            state.unreachable_since.remove(&link.id);
        }
    }

    save_state(state_file, &state);
    Ok(())
}

struct Decision {
    action: &'static str,
    sort_order: Option<i64>,
    review_note: String,
}

async fn evaluate_application(
    client: &reqwest::Client,
    app: &PendingApplicationTask,
    backlink_target: &str,
) -> Decision {
    let mut score = 50;
    let mut reasons: Vec<String> = Vec::new();

    if looks_suspicious_domain(&app.site_url) {
        score -= 40;
        reasons.push("域名风险较高".to_string());
    }
    let full_text = format!(
        "{} {} {}",
        app.site_name,
        app.description.clone().unwrap_or_default(),
        app.note.clone().unwrap_or_default()
    );
    if contains_spam_keyword(&full_text) {
        score -= 80;
        reasons.push("命中垃圾关键词".to_string());
    }
    let desc_len = app
        .description
        .clone()
        .unwrap_or_default()
        .chars()
        .count();
    if desc_len >= 8 && desc_len <= 180 {
        score += 12;
    } else {
        score -= 10;
        reasons.push("简介信息不足".to_string());
    }
    if app.avatar_url.as_deref().is_some_and(is_valid_http_url) {
        score += 4;
    }

    if let Some((ok, html)) = fetch_site(client, &app.site_url).await {
        if ok {
            score += 18;
        } else {
            score -= 24;
            reasons.push("站点主页访问失败".to_string());
        }
        let lower = html.to_lowercase();
        if lower.contains("<title") {
            score += 5;
        }
        if lower.contains("description") {
            score += 5;
        }
        if lower.contains("meta") {
            score += 6;
        } else {
            score -= 6;
            reasons.push("SEO 基础信息偏弱".to_string());
        }
        if contains_backlink(&lower, backlink_target) {
            score += 10;
        } else {
            score -= 10;
            reasons.push("未检测到本站链接".to_string());
        }
    } else {
        score -= 25;
        reasons.push("站点主页抓取失败".to_string());
    }

    let action = if score >= 80 {
        "approve"
    } else if score < 40 {
        "reject"
    } else {
        "pending"
    };
    Decision {
        action,
        sort_order: if action == "approve" { Some(now_ts()) } else { None },
        review_note: format!("auto-score={}；{}", score, reasons.join("；")),
    }
}

async fn report_removal(
    client: &reqwest::Client,
    base: &str,
    token: &str,
    link_id: &str,
    application_id: Option<i64>,
    status: &str,
    reason: &str,
) {
    let payload = json!({
        "link_id": link_id,
        "application_id": application_id,
        "app_status": status,
        "reason": reason,
        "send_email": true
    });
    let _ = client
        .post(format!("{}/links/review/report/removal", base.trim_end_matches('/')))
        .header("x-token", token)
        .json(&payload)
        .send()
        .await;
}

async fn fetch_site(client: &reqwest::Client, site_url: &str) -> Option<(bool, String)> {
    let response = client
        .get(site_url)
        .header("user-agent", "MeowReviewWorker/1.0")
        .send()
        .await
        .ok()?;
    let ok = response.status().is_success() || response.status().is_redirection();
    let html = response.text().await.unwrap_or_default();
    Some((ok, html))
}

fn contains_backlink(page_lower: &str, backlink_target: &str) -> bool {
    let normalized = backlink_target.trim().to_lowercase();
    if normalized.is_empty() {
        return false;
    }
    let mut candidates = vec![normalized.clone()];
    if normalized.starts_with("https://") {
        candidates.push(normalized.replacen("https://", "http://", 1));
    }
    if normalized.ends_with('/') {
        candidates.push(normalized.trim_end_matches('/').to_string());
    } else {
        candidates.push(format!("{}/", normalized));
    }
    candidates.iter().any(|needle| page_lower.contains(needle))
}

fn contains_spam_keyword(content: &str) -> bool {
    let text = content.to_lowercase();
    let spam_words = ["博彩", "彩票", "娱乐城", "代刷", "赌博", "av", "色情", "vpn", "加速器", "私服"];
    spam_words.iter().any(|word| text.contains(word))
}

fn looks_suspicious_domain(site_url: &str) -> bool {
    let parsed = Url::parse(site_url);
    let host = parsed.ok().and_then(|url| url.host_str().map(|h| h.to_lowercase()));
    match host {
        Some(host) => {
            host == "localhost"
                || host.starts_with("127.")
                || host.starts_with("10.")
                || host.starts_with("192.168.")
                || host.ends_with(".local")
        }
        None => true,
    }
}

fn is_valid_http_url(value: &str) -> bool {
    Url::parse(value)
        .map(|url| matches!(url.scheme(), "http" | "https") && url.host_str().is_some())
        .unwrap_or(false)
}

fn now_ts() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs() as i64
}

fn load_state(path: &PathBuf) -> LocalState {
    let content = fs::read_to_string(path);
    match content {
        Ok(content) => serde_json::from_str::<LocalState>(&content).unwrap_or_default(),
        Err(_) => LocalState::default(),
    }
}

fn save_state(path: &PathBuf, state: &LocalState) {
    if let Ok(text) = serde_json::to_string_pretty(state) {
        let _ = fs::write(path, text);
    }
}
