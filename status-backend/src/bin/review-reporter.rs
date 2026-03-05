use reqwest::Url;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::{
    collections::{HashMap, HashSet},
    fs,
    path::PathBuf,
    time::Duration,
};

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

#[derive(Clone)]
struct SeoApiConfig {
    url: String,
    api_key: Option<String>,
    api_key_header: String,
    max_bonus: i32,
}

#[derive(Clone)]
struct SerpApiConfig {
    api_key: String,
    endpoint: String,
    engine: String,
    hl: String,
    gl: String,
    num: u8,
    max_bonus: i32,
}

#[derive(Clone)]
enum SeoProviderConfig {
    Generic(SeoApiConfig),
    SerpApi(SerpApiConfig),
}

#[derive(Clone)]
struct WorkerConfig {
    seo_provider: Option<SeoProviderConfig>,
}

#[tokio::main]
async fn main() {
    let base =
        std::env::var("REVIEW_API_BASE").unwrap_or_else(|_| "http://127.0.0.1:7999".to_string());
    let token = std::env::var("REVIEW_REPORT_TOKEN").unwrap_or_else(|_| "KFCVME50".to_string());
    let interval_secs = std::env::var("REVIEW_LOOP_INTERVAL_SEC")
        .ok()
        .and_then(|v| v.parse::<u64>().ok())
        .unwrap_or(300)
        .max(30);
    let state_file = std::env::var("REVIEW_LOCAL_STATE")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("review-worker-state.json"));
    let worker_config = WorkerConfig::from_env();
    let run_once = std::env::args().any(|arg| arg == "--once")
        || std::env::var("REVIEW_RUN_ONCE")
            .ok()
            .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
            .unwrap_or(false);

    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(15))
        .build()
        .expect("build client");
    eprintln!(
        "[review-worker] started: api_base={} interval_sec={} seo_provider={} run_once={}",
        base,
        interval_secs,
        worker_config.provider_label(),
        run_once
    );

    loop {
        if let Err(err) = run_once_cycle(&client, &base, &token, &state_file, &worker_config).await
        {
            eprintln!("[review-worker] run_once error: {}", err);
        }
        if run_once {
            eprintln!("[review-worker] run_once finished, exiting.");
            break;
        }
        tokio::time::sleep(Duration::from_secs(interval_secs)).await;
    }
}

async fn run_once_cycle(
    client: &reqwest::Client,
    base: &str,
    token: &str,
    state_file: &PathBuf,
    worker_config: &WorkerConfig,
) -> Result<(), String> {
    let mut state = load_state(state_file);
    let tasks: ReviewTasksResponse = client
        .get(format!(
            "{}/links/review/report/tasks",
            base.trim_end_matches('/')
        ))
        .header("x-token", token)
        .send()
        .await
        .map_err(|e| format!("fetch tasks failed: {}", e))?
        .json()
        .await
        .map_err(|e| format!("decode tasks failed: {}", e))?;

    for app in &tasks.pending_applications {
        let decision =
            evaluate_application(client, app, &tasks.backlink_target, worker_config).await;
        eprintln!(
            "[review-worker] app#{} {} => action={} note={}",
            app.id, app.site_url, decision.action, decision.review_note
        );
        if decision.action == "pending" {
            let payload = json!({
                "application_id": app.id,
                "review_note": decision.review_note,
                "send_admin_notify": true
            });
            match client
                .post(format!(
                    "{}/links/review/report/manual",
                    base.trim_end_matches('/')
                ))
                .header("x-token", token)
                .json(&payload)
                .send()
                .await
            {
                Ok(resp) => {
                    let status = resp.status();
                    let body = resp.text().await.unwrap_or_default();
                    eprintln!(
                        "[review-worker] report manual app#{} status={} body={}",
                        app.id, status, body
                    );
                }
                Err(err) => {
                    eprintln!(
                        "[review-worker] report manual app#{} failed: {}",
                        app.id, err
                    );
                }
            }
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
        match client
            .post(format!(
                "{}/links/review/report/decision",
                base.trim_end_matches('/')
            ))
            .header("x-token", token)
            .json(&payload)
            .send()
            .await
        {
            Ok(resp) => {
                let status = resp.status();
                let body = resp.text().await.unwrap_or_default();
                eprintln!(
                    "[review-worker] report decision app#{} status={} body={}",
                    app.id, status, body
                );
            }
            Err(err) => {
                eprintln!(
                    "[review-worker] report decision app#{} failed: {}",
                    app.id, err
                );
            }
        }
    }

    let now = tasks.now_ts;
    for link in &tasks.active_links {
        let snap = fetch_site(client, &link.url).await;
        let accessible = snap.as_ref().map(|(ok, _)| *ok).unwrap_or(false);
        let has_backlink = find_backlink_in_site(
            client,
            &link.url,
            &tasks.backlink_target,
            snap.as_ref().map(|(_, html)| html.as_str()),
        )
        .await;

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
                        tasks.backlink_enforce_hours, tasks.backlink_target
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

        let entry = state
            .unreachable_since
            .entry(link.id.clone())
            .or_insert(now);
        if now - *entry >= tasks.unreachable_enforce_hours.max(1) * 3600 {
            report_removal(
                client,
                base,
                token,
                &link.id,
                link.application_id,
                "removed_unreachable",
                &format!(
                    "友链已下架：连续 {} 小时无法访问",
                    tasks.unreachable_enforce_hours
                ),
            )
            .await;
            state.unreachable_since.remove(&link.id);
        }
    }

    eprintln!(
        "[review-worker] loop done: pending_apps={} active_links={} unreachable_state={}",
        tasks.pending_applications.len(),
        tasks.active_links.len(),
        state.unreachable_since.len()
    );
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
    worker_config: &WorkerConfig,
) -> Decision {
    let mut score = 50;
    let mut reasons: Vec<String> = Vec::new();

    if looks_suspicious_domain(&app.site_url) {
        score -= 40;
        reasons.push("域名风险较高".to_string());
    }
    if is_disallowed_public_service_domain(&app.site_url) {
        return Decision {
            action: "reject",
            sort_order: None,
            review_note: "auto-score=0；学校/政府网站不接受友链互换申请".to_string(),
        };
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
    let desc_len = app.description.clone().unwrap_or_default().chars().count();
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
        let has_backlink = if contains_backlink(&lower, backlink_target) {
            true
        } else {
            find_backlink_in_site(client, &app.site_url, backlink_target, Some(&html)).await
        };
        if has_backlink {
            score += 10;
        } else {
            score -= 10;
            reasons.push("未检测到本站链接".to_string());
        }
    } else {
        score -= 25;
        reasons.push("站点主页抓取失败".to_string());
    }

    if let Some(seo_cfg) = &worker_config.seo_provider {
        match fetch_third_party_seo_score(client, seo_cfg, app).await {
            Ok((remote_score, reason)) => {
                let max_bonus = match seo_cfg {
                    SeoProviderConfig::Generic(cfg) => cfg.max_bonus,
                    SeoProviderConfig::SerpApi(cfg) => cfg.max_bonus,
                };
                let delta = map_remote_score_to_delta(remote_score, max_bonus);
                score += delta;
                reasons.push(format!(
                    "第三方SEO={}({:+}) {}",
                    remote_score,
                    delta,
                    reason.unwrap_or_default()
                ));
            }
            Err(err) => {
                reasons.push(format!("第三方SEO不可用({})", err));
            }
        }
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
        sort_order: if action == "approve" {
            Some(now_ts())
        } else {
            None
        },
        review_note: format!("auto-score={}；{}", score, reasons.join("；")),
    }
}

async fn fetch_third_party_seo_score(
    client: &reqwest::Client,
    cfg: &SeoProviderConfig,
    app: &PendingApplicationTask,
) -> Result<(i32, Option<String>), String> {
    match cfg {
        SeoProviderConfig::Generic(cfg) => fetch_generic_seo_score(client, cfg, app).await,
        SeoProviderConfig::SerpApi(cfg) => fetch_serpapi_seo_score(client, cfg, app).await,
    }
}

async fn fetch_generic_seo_score(
    client: &reqwest::Client,
    cfg: &SeoApiConfig,
    app: &PendingApplicationTask,
) -> Result<(i32, Option<String>), String> {
    let mut request = client.post(&cfg.url).json(&json!({
        "url": app.site_url,
        "site_name": app.site_name,
        "description": app.description.clone().unwrap_or_default(),
        "note": app.note.clone().unwrap_or_default()
    }));
    if let Some(api_key) = &cfg.api_key {
        request = request.header(&cfg.api_key_header, api_key);
    }
    let response = request
        .send()
        .await
        .map_err(|e| format!("request failed: {}", e))?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("http {} {}", status, body));
    }
    let value: Value = response
        .json()
        .await
        .map_err(|e| format!("decode failed: {}", e))?;
    let score = value
        .get("score")
        .and_then(|v| v.as_i64())
        .ok_or_else(|| "missing score field".to_string())? as i32;
    let reason = value
        .get("reason")
        .and_then(|v| v.as_str())
        .map(|v| v.to_string());
    Ok((score.clamp(0, 100), reason))
}

async fn fetch_serpapi_seo_score(
    client: &reqwest::Client,
    cfg: &SerpApiConfig,
    app: &PendingApplicationTask,
) -> Result<(i32, Option<String>), String> {
    let domain = Url::parse(&app.site_url)
        .ok()
        .and_then(|u| u.host_str().map(|v| v.to_lowercase()))
        .ok_or_else(|| "invalid site url for serpapi".to_string())?;
    let query = format!("site:{} {}", domain, app.site_name);
    let num_str = cfg.num.to_string();
    let params = [
        ("engine", cfg.engine.as_str()),
        ("q", query.as_str()),
        ("api_key", cfg.api_key.as_str()),
        ("hl", cfg.hl.as_str()),
        ("gl", cfg.gl.as_str()),
        ("num", num_str.as_str()),
    ];
    let response = client
        .get(&cfg.endpoint)
        .query(&params)
        .send()
        .await
        .map_err(|e| format!("request failed: {}", e))?;
    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        return Err(format!("http {} {}", status, body));
    }
    let value: Value = response
        .json()
        .await
        .map_err(|e| format!("decode failed: {}", e))?;

    let organic = value
        .get("organic_results")
        .and_then(|v| v.as_array())
        .cloned()
        .unwrap_or_default();
    let total_hits = organic.len() as i32;
    let first_match = organic.iter().take(3).any(|item| {
        item.get("link")
            .and_then(|v| v.as_str())
            .and_then(|link| Url::parse(link).ok())
            .and_then(|u| u.host_str().map(|h| h.to_lowercase()))
            .is_some_and(|h| h == domain || h.ends_with(&format!(".{}", domain)))
    });
    let has_any_match = organic.iter().any(|item| {
        item.get("link")
            .and_then(|v| v.as_str())
            .and_then(|link| Url::parse(link).ok())
            .and_then(|u| u.host_str().map(|h| h.to_lowercase()))
            .is_some_and(|h| h == domain || h.ends_with(&format!(".{}", domain)))
    });
    let mut score = 45;
    if has_any_match {
        score += 25;
    } else {
        score -= 15;
    }
    if first_match {
        score += 20;
    }
    if total_hits >= 5 {
        score += 10;
    } else if total_hits == 0 {
        score -= 15;
    }
    let reason = format!(
        "serpapi: domain={}, results={}, top3_match={}",
        domain, total_hits, first_match
    );
    Ok((score.clamp(0, 100), Some(reason)))
}

fn map_remote_score_to_delta(remote_score: i32, max_bonus: i32) -> i32 {
    let centered = remote_score.clamp(0, 100) - 50;
    (centered * max_bonus) / 50
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
    match client
        .post(format!(
            "{}/links/review/report/removal",
            base.trim_end_matches('/')
        ))
        .header("x-token", token)
        .json(&payload)
        .send()
        .await
    {
        Ok(resp) => {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            eprintln!(
                "[review-worker] report removal link={} status={} body={}",
                link_id, status, body
            );
        }
        Err(err) => {
            eprintln!(
                "[review-worker] report removal link={} failed: {}",
                link_id, err
            );
        }
    }
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

async fn find_backlink_in_site(
    client: &reqwest::Client,
    site_url: &str,
    backlink_target: &str,
    homepage_html: Option<&str>,
) -> bool {
    let base = match Url::parse(site_url) {
        Ok(v) => v,
        Err(_) => return false,
    };
    let home_html_owned = if let Some(html) = homepage_html {
        html.to_string()
    } else {
        match fetch_site(client, site_url).await {
            Some((_, html)) => html,
            None => return false,
        }
    };
    let home_lower = home_html_owned.to_lowercase();
    if contains_backlink(&home_lower, backlink_target) {
        return true;
    }

    let mut candidates = collect_friend_page_candidates(&base, &home_html_owned);
    let mut dynamic_candidates = collect_dynamic_page_candidates(&base, &home_html_owned);
    candidates.append(&mut dynamic_candidates);
    if let Some(mut sitemap_urls) = load_sitemap_candidates(client, &base).await {
        candidates.append(&mut sitemap_urls);
    }

    let mut visited = HashSet::new();
    let mut checked = 0usize;
    for url in candidates {
        if checked >= 8 {
            break;
        }
        if !visited.insert(url.clone()) {
            continue;
        }
        if let Some((_, html)) = fetch_site(client, &url).await {
            checked += 1;
            if contains_backlink(&html.to_lowercase(), backlink_target) {
                return true;
            }
        }
    }
    false
}

fn collect_friend_page_candidates(base: &Url, html: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let keywords = [
        "friend",
        "friends",
        "link",
        "links",
        "blogroll",
        "友链",
        "友情链接",
    ];
    for href in extract_hrefs(html) {
        if is_non_web_href(&href) {
            continue;
        }
        let lower = href.to_lowercase();
        if !keywords.iter().any(|k| lower.contains(k)) {
            continue;
        }
        if let Some(url) = resolve_same_site_url(base, &href) {
            if seen.insert(url.clone()) {
                out.push(url);
            }
        }
    }
    for path in [
        "/friends",
        "/friend",
        "/links",
        "/link",
        "/friend-links",
        "/blogroll",
        "/youqing",
    ] {
        if let Ok(url) = base.join(path) {
            let s = normalize_url_for_fetch(url);
            if seen.insert(s.clone()) {
                out.push(s);
            }
        }
    }
    out
}

async fn load_sitemap_candidates(client: &reqwest::Client, base: &Url) -> Option<Vec<String>> {
    let sitemap_url = base.join("/sitemap.xml").ok()?;
    let (_, xml) = fetch_site(client, sitemap_url.as_str()).await?;
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let keywords = [
        "friend",
        "friends",
        "link",
        "links",
        "blogroll",
        "友链",
        "友情链接",
    ];
    for loc in extract_sitemap_locs(&xml) {
        let lower = loc.to_lowercase();
        if !keywords.iter().any(|k| lower.contains(k)) {
            continue;
        }
        if let Some(url) = resolve_same_site_url(base, &loc) {
            if seen.insert(url.clone()) {
                out.push(url);
            }
        }
        if out.len() >= 6 {
            break;
        }
    }
    Some(out)
}

fn extract_hrefs(html: &str) -> Vec<String> {
    let lower = html.to_lowercase();
    let bytes = lower.as_bytes();
    let mut out = Vec::new();
    let mut i = 0usize;
    while i + 5 < bytes.len() {
        if &bytes[i..i + 5] != b"href=" {
            i += 1;
            continue;
        }
        i += 5;
        while i < bytes.len() && bytes[i].is_ascii_whitespace() {
            i += 1;
        }
        if i >= bytes.len() {
            break;
        }
        let quote = bytes[i];
        let (start, end) = if quote == b'"' || quote == b'\'' {
            let start = i + 1;
            let mut j = start;
            while j < bytes.len() && bytes[j] != quote {
                j += 1;
            }
            (start, j)
        } else {
            let start = i;
            let mut j = start;
            while j < bytes.len()
                && !bytes[j].is_ascii_whitespace()
                && bytes[j] != b'>'
                && bytes[j] != b'"'
                && bytes[j] != b'\''
            {
                j += 1;
            }
            (start, j)
        };
        if start < end && end <= html.len() {
            let candidate = html[start..end].trim();
            if !candidate.is_empty() {
                out.push(candidate.to_string());
            }
        }
        i = end.saturating_add(1);
    }
    out
}

fn extract_sitemap_locs(xml: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut rest = xml;
    while let Some(start) = rest.find("<loc>") {
        let after = &rest[start + 5..];
        if let Some(end) = after.find("</loc>") {
            let loc = after[..end].trim();
            if !loc.is_empty() {
                out.push(loc.to_string());
            }
            rest = &after[end + 6..];
        } else {
            break;
        }
    }
    out
}

fn collect_dynamic_page_candidates(base: &Url, html: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut seen = HashSet::new();
    let keywords = [
        "friend",
        "friends",
        "link",
        "links",
        "blogroll",
        "youqing",
        "友链",
        "友情链接",
    ];

    for raw in extract_quoted_candidates(html) {
        let lower = raw.to_lowercase();
        if !keywords.iter().any(|k| lower.contains(k)) {
            continue;
        }
        if let Some(url) = resolve_same_site_url(base, &raw) {
            if seen.insert(url.clone()) {
                out.push(url);
            }
        }
    }

    out
}

fn extract_quoted_candidates(content: &str) -> Vec<String> {
    let mut out = Vec::new();
    let bytes = content.as_bytes();
    let mut i = 0usize;
    while i < bytes.len() {
        let ch = bytes[i];
        if ch != b'"' && ch != b'\'' {
            i += 1;
            continue;
        }
        let quote = ch;
        let start = i + 1;
        let mut j = start;
        while j < bytes.len() {
            if bytes[j] == quote && bytes[j.saturating_sub(1)] != b'\\' {
                break;
            }
            j += 1;
        }
        if j <= bytes.len() && start < j {
            let candidate = content[start..j].trim();
            if looks_like_route_or_url(candidate) {
                out.push(candidate.to_string());
            }
        }
        i = j.saturating_add(1);
    }
    out
}

fn looks_like_route_or_url(value: &str) -> bool {
    let v = value.trim();
    if v.len() < 2 || v.len() > 240 {
        return false;
    }
    if v.starts_with("http://") || v.starts_with("https://") {
        return true;
    }
    if !v.starts_with('/') {
        return false;
    }
    !is_non_web_href(v)
}

fn is_non_web_href(href: &str) -> bool {
    let lower = href.trim().to_lowercase();
    lower.starts_with('#')
        || lower.starts_with("javascript:")
        || lower.starts_with("mailto:")
        || lower.starts_with("tel:")
}

fn resolve_same_site_url(base: &Url, raw: &str) -> Option<String> {
    let parsed = Url::parse(raw).or_else(|_| base.join(raw)).ok()?;
    if !matches!(parsed.scheme(), "http" | "https") {
        return None;
    }
    let base_host = base.host_str()?.to_lowercase();
    let host = parsed.host_str()?.to_lowercase();
    if host != base_host {
        return None;
    }
    Some(normalize_url_for_fetch(parsed))
}

fn normalize_url_for_fetch(mut url: Url) -> String {
    url.set_fragment(None);
    url.to_string()
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
    let spam_words = [
        "博彩",
        "彩票",
        "娱乐城",
        "代刷",
        "赌博",
        "av",
        "色情",
        "vpn",
        "加速器",
        "私服",
    ];
    spam_words.iter().any(|word| text.contains(word))
}

fn looks_suspicious_domain(site_url: &str) -> bool {
    let parsed = Url::parse(site_url);
    let host = parsed
        .ok()
        .and_then(|url| url.host_str().map(|h| h.to_lowercase()));
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

fn is_disallowed_public_service_domain(site_url: &str) -> bool {
    let host = Url::parse(site_url)
        .ok()
        .and_then(|url| url.host_str().map(|h| h.to_lowercase()));
    match host {
        Some(host) => {
            host == "edu"
                || host.ends_with(".edu")
                || host.ends_with(".edu.cn")
                || host == "gov"
                || host.ends_with(".gov")
                || host.ends_with(".gov.cn")
        }
        None => false,
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

impl WorkerConfig {
    fn from_env() -> Self {
        let provider = std::env::var("REVIEW_SEO_PROVIDER")
            .ok()
            .map(|v| v.trim().to_lowercase())
            .unwrap_or_else(|| "none".to_string());
        let max_bonus = std::env::var("REVIEW_SEO_MAX_BONUS")
            .ok()
            .and_then(|v| v.parse::<i32>().ok())
            .unwrap_or(12)
            .clamp(1, 30);
        let seo_provider = match provider.as_str() {
            "generic" => std::env::var("REVIEW_SEO_API_URL")
                .ok()
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
                .map(|url| {
                    SeoProviderConfig::Generic(SeoApiConfig {
                        url,
                        api_key: std::env::var("REVIEW_SEO_API_KEY")
                            .ok()
                            .map(|v| v.trim().to_string())
                            .filter(|v| !v.is_empty()),
                        api_key_header: std::env::var("REVIEW_SEO_API_KEY_HEADER")
                            .ok()
                            .map(|v| v.trim().to_string())
                            .filter(|v| !v.is_empty())
                            .unwrap_or_else(|| "Authorization".to_string()),
                        max_bonus,
                    })
                }),
            "serpapi" => std::env::var("REVIEW_SERPAPI_KEY")
                .ok()
                .map(|v| v.trim().to_string())
                .filter(|v| !v.is_empty())
                .map(|api_key| {
                    SeoProviderConfig::SerpApi(SerpApiConfig {
                        api_key,
                        endpoint: std::env::var("REVIEW_SERPAPI_ENDPOINT")
                            .ok()
                            .map(|v| v.trim().to_string())
                            .filter(|v| !v.is_empty())
                            .unwrap_or_else(|| "https://serpapi.com/search.json".to_string()),
                        engine: std::env::var("REVIEW_SERPAPI_ENGINE")
                            .ok()
                            .map(|v| v.trim().to_string())
                            .filter(|v| !v.is_empty())
                            .unwrap_or_else(|| "google".to_string()),
                        hl: std::env::var("REVIEW_SERPAPI_HL")
                            .ok()
                            .map(|v| v.trim().to_string())
                            .filter(|v| !v.is_empty())
                            .unwrap_or_else(|| "zh-cn".to_string()),
                        gl: std::env::var("REVIEW_SERPAPI_GL")
                            .ok()
                            .map(|v| v.trim().to_string())
                            .filter(|v| !v.is_empty())
                            .unwrap_or_else(|| "cn".to_string()),
                        num: std::env::var("REVIEW_SERPAPI_NUM")
                            .ok()
                            .and_then(|v| v.parse::<u8>().ok())
                            .map(|v| v.clamp(5, 20))
                            .unwrap_or(10),
                        max_bonus,
                    })
                }),
            _ => None,
        };
        Self { seo_provider }
    }

    fn provider_label(&self) -> &'static str {
        match self.seo_provider {
            Some(SeoProviderConfig::Generic(_)) => "generic",
            Some(SeoProviderConfig::SerpApi(_)) => "serpapi",
            None => "none",
        }
    }
}
