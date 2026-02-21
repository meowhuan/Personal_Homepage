#![cfg_attr(windows, windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::{
    fs,
    fs::OpenOptions,
    io::Write,
    path::PathBuf,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
#[cfg(target_os = "linux")]
use std::process::Command;
use tracing_subscriber::EnvFilter;
use tracing_subscriber::fmt::writer::MakeWriterExt;

#[derive(Serialize)]
struct Heartbeat {
    device_id: String,
    device_name: String,
    online: bool,
    idle_seconds: Option<u64>,
    music_playing: bool,
    music_title: Option<String>,
    music_artist: Option<String>,
    music_source: Option<String>,
}

#[derive(Default, Clone, PartialEq, Eq, Debug)]
struct MusicState {
    playing: bool,
    title: Option<String>,
    artist: Option<String>,
    source: Option<String>,
}

#[derive(Default, Deserialize)]
struct ConfigFile {
    endpoint: Option<String>,
    token: Option<String>,
    device_id: Option<String>,
    device_name: Option<String>,
    idle_timeout_secs: Option<u64>,
    heartbeat_interval_secs: Option<u64>,
    music_poll_interval_secs: Option<u64>,
    music_push_min_interval_secs: Option<u64>,
    log_file: Option<String>,
    log_max_bytes: Option<u64>,
}

#[derive(Clone)]
struct Config {
    endpoint: String,
    token: String,
    device_id: String,
    device_name: String,
    idle_timeout_secs: u64,
    heartbeat_interval_secs: u64,
    music_poll_interval_secs: u64,
    music_push_min_interval_secs: u64,
    log_file: String,
    log_max_bytes: u64,
}

fn main() {
    let cfg = load_config();
    let (_log_guard, log_path) = init_logging(&cfg);
    tracing::info!("status-client starting: log={}", log_path.display());

    let status = Arc::new(Mutex::new(String::from("starting")));

    let hb_cfg = cfg.clone();
    let hb_status = status.clone();
    thread::spawn(move || heartbeat_loop(hb_cfg, hb_status));

    let music_cfg = cfg.clone();
    let music_status = status.clone();
    thread::spawn(move || music_push_loop(music_cfg, music_status));

    loop {
        thread::sleep(Duration::from_secs(3600));
    }
}

fn heartbeat_loop(cfg: Config, status: Arc<Mutex<String>>) {
    let client = reqwest::blocking::Client::new();
    loop {
        enforce_log_size(&resolve_path(&cfg.log_file), cfg.log_max_bytes);
        let tick = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let payload = build_payload(&cfg);
            send_payload(&client, &cfg, &status, payload, "heartbeat");
        }));
        if tick.is_err() {
            tracing::error!("heartbeat loop panic recovered");
        }

        thread::sleep(Duration::from_secs(cfg.heartbeat_interval_secs));
    }
}

fn music_push_loop(cfg: Config, status: Arc<Mutex<String>>) {
    let client = reqwest::blocking::Client::new();
    let mut last_music: Option<MusicState> = None;
    let mut last_push_ts: u64 = 0;
    loop {
        enforce_log_size(&resolve_path(&cfg.log_file), cfg.log_max_bytes);
        let tick = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let payload = build_payload(&cfg);
            let current = MusicState {
                playing: payload.music_playing,
                title: payload.music_title.clone(),
                artist: payload.music_artist.clone(),
                source: payload.music_source.clone(),
            };
            let changed = last_music.as_ref().map(|v| v != &current).unwrap_or(true);
            let now = chrono_like_ts();
            if changed {
                let can_push = last_push_ts == 0
                    || now.saturating_sub(last_push_ts) >= cfg.music_push_min_interval_secs;
                if can_push {
                    tracing::info!("music changed: {:?}", current);
                    send_payload(&client, &cfg, &status, payload, "music-change");
                    last_push_ts = now;
                }
                last_music = Some(current);
            }
        }));
        if tick.is_err() {
            tracing::error!("music push loop panic recovered");
        }
        thread::sleep(Duration::from_secs(cfg.music_poll_interval_secs));
    }
}

fn build_payload(cfg: &Config) -> Heartbeat {
    let idle = idle_seconds();
    let online = idle.map(|v| v < cfg.idle_timeout_secs).unwrap_or(true);
    let mut payload = Heartbeat {
        device_id: cfg.device_id.clone(),
        device_name: cfg.device_name.clone(),
        online,
        idle_seconds: idle,
        music_playing: false,
        music_title: None,
        music_artist: None,
        music_source: None,
    };
    if let Some(music) = current_music() {
        payload.music_playing = music.playing;
        payload.music_title = music.title;
        payload.music_artist = music.artist;
        payload.music_source = music.source;
    }
    payload
}

fn send_payload(
    client: &reqwest::blocking::Client,
    cfg: &Config,
    status: &Arc<Mutex<String>>,
    payload: Heartbeat,
    reason: &str,
) {
    tracing::info!(
        "{} build: online={} idle={:?} music_playing={} title={:?} artist={:?} source={:?}",
        reason,
        payload.online,
        payload.idle_seconds,
        payload.music_playing,
        payload.music_title,
        payload.music_artist,
        payload.music_source
    );
    let res = client
        .post(&cfg.endpoint)
        .header("x-token", &cfg.token)
        .json(&payload)
        .send();
    match res {
        Ok(resp) => {
            let label: &str = if resp.status().is_success() { "online" } else { "error" };
            if let Ok(mut s) = status.lock() {
                *s = label.to_string();
            }
            tracing::info!("{} sent: status={}", reason, resp.status());
            if !resp.status().is_success() {
                tracing::warn!("{} failed: status={}", reason, resp.status());
            }
        }
        Err(err) => {
            if let Ok(mut s) = status.lock() {
                *s = "error".to_string();
            }
            tracing::warn!("{} request error: {}", reason, err);
        }
    }
}

fn load_config() -> Config {
    let file_cfg = read_config_file();

    let endpoint = std::env::var("STATUS_ENDPOINT")
        .ok()
        .or(file_cfg.endpoint)
        .unwrap_or_else(|| "http://xxx.com:7999/heartbeat".to_string());

    let token = std::env::var("STATUS_TOKEN")
        .ok()
        .or(file_cfg.token)
        .unwrap_or_else(|| "you_token".to_string());

    let device_id = std::env::var("DEVICE_ID")
        .ok()
        .or(file_cfg.device_id)
        .unwrap_or_else(hostname);

    let device_name = std::env::var("DEVICE_NAME")
        .ok()
        .or(file_cfg.device_name)
        .unwrap_or_else(|| device_id.clone());

    let idle_timeout_secs = std::env::var("IDLE_TIMEOUT_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .or(file_cfg.idle_timeout_secs)
        .unwrap_or(300);

    let heartbeat_interval_secs = std::env::var("HEARTBEAT_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .or(file_cfg.heartbeat_interval_secs)
        .unwrap_or(60);
    let music_poll_interval_secs = std::env::var("MUSIC_POLL_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .or(file_cfg.music_poll_interval_secs)
        .unwrap_or(5);
    let music_push_min_interval_secs = std::env::var("MUSIC_PUSH_MIN_INTERVAL_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .or(file_cfg.music_push_min_interval_secs)
        .unwrap_or(6);

    let log_file = std::env::var("LOG_FILE")
        .ok()
        .or(file_cfg.log_file)
        .unwrap_or_else(|| "status-client.log".to_string());
    let log_max_bytes = std::env::var("LOG_MAX_BYTES")
        .ok()
        .and_then(|v| v.parse().ok())
        .or(file_cfg.log_max_bytes)
        .unwrap_or(2 * 1024 * 1024);

    Config {
        endpoint,
        token,
        device_id,
        device_name,
        idle_timeout_secs,
        heartbeat_interval_secs,
        music_poll_interval_secs,
        music_push_min_interval_secs,
        log_file,
        log_max_bytes,
    }
}

fn read_config_file() -> ConfigFile {
    let path = std::env::var("STATUS_CONFIG")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| config_path());

    let data = fs::read_to_string(path).unwrap_or_default();
    toml::from_str(&data).unwrap_or_default()
}

fn config_path() -> PathBuf {
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            return dir.join("status-client.toml");
        }
    }
    PathBuf::from("status-client.toml")
}

fn init_logging(cfg: &Config) -> (tracing_appender::non_blocking::WorkerGuard, PathBuf) {
    let preferred = resolve_path(&cfg.log_file);
    let log_path = open_log_file_path(&preferred).unwrap_or_else(|| {
        std::env::temp_dir().join("status-client.log")
    });

    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&log_path)
        .unwrap_or_else(|_| {
            let fallback = std::env::temp_dir().join("status-client.log");
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(fallback)
                .expect("open fallback log file")
        });
    let _ = writeln!(
        file,
        "[bootstrap] status-client logger init pid={} ts={}",
        std::process::id(),
        chrono_like_ts()
    );

    let (non_blocking, guard) = tracing_appender::non_blocking(file);
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    tracing_subscriber::fmt()
        .with_env_filter(filter)
        .with_writer(non_blocking.and(std::io::stdout))
        .init();
    (guard, log_path)
}

fn chrono_like_ts() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn enforce_log_size(path: &std::path::Path, max_bytes: u64) {
    if max_bytes == 0 {
        return;
    }
    let Ok(meta) = fs::metadata(path) else { return };
    if meta.len() <= max_bytes {
        return;
    }
    if let Ok(mut file) = OpenOptions::new().write(true).truncate(true).open(path) {
        let _ = writeln!(
            file,
            "[bootstrap] log truncated at ts={} max_bytes={}",
            chrono_like_ts(),
            max_bytes
        );
    }
}

fn hostname() -> String {
    hostname::get()
        .ok()
        .and_then(|v| v.into_string().ok())
        .unwrap_or_else(|| "unknown-device".to_string())
}

fn resolve_path(p: &str) -> PathBuf {
    let path = PathBuf::from(p);
    if path.is_absolute() {
        return path;
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(dir) = exe.parent() {
            return dir.join(path);
        }
    }
    path
}

fn open_log_file_path(path: &std::path::Path) -> Option<PathBuf> {
    if let Some(parent) = path.parent() {
        if fs::create_dir_all(parent).is_err() {
            return None;
        }
    }
    Some(path.to_path_buf())
}

fn clean_text(s: Option<&str>) -> Option<String> {
    s.map(|v| v.trim())
        .filter(|v| !v.is_empty())
        .map(|v| v.to_string())
}

#[cfg(windows)]
fn current_music() -> Option<MusicState> {
    use pollster::block_on;
    use windows::Media::Control::{
        GlobalSystemMediaTransportControlsSession,
        GlobalSystemMediaTransportControlsSessionManager,
        GlobalSystemMediaTransportControlsSessionPlaybackStatus,
    };

    let result = block_on(async {
        let manager_op = GlobalSystemMediaTransportControlsSessionManager::RequestAsync()?;
        let manager = manager_op.await?;

        let mut chosen: Option<GlobalSystemMediaTransportControlsSession> = None;
        if let Ok(sessions) = manager.GetSessions() {
            let len = sessions.Size().unwrap_or(0);
            for idx in 0..len {
                let Ok(session) = sessions.GetAt(idx) else {
                    continue;
                };
                let Ok(playback) = session.GetPlaybackInfo() else {
                    continue;
                };
                let Ok(status) = playback.PlaybackStatus() else {
                    continue;
                };
                if status == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing {
                    chosen = Some(session);
                    break;
                }
            }
        }

        let session = chosen.or_else(|| manager.GetCurrentSession().ok());
        let Some(session) = session else {
            return Ok::<Option<MusicState>, windows::core::Error>(None);
        };

        let playing = session
            .GetPlaybackInfo()
            .and_then(|p| p.PlaybackStatus())
            .map(|s| s == GlobalSystemMediaTransportControlsSessionPlaybackStatus::Playing)
            .unwrap_or(false);

        let media_op = session.TryGetMediaPropertiesAsync()?;
        let media = media_op.await?;

        let title_owned = media.Title().ok().map(|s| s.to_string());
        let artist_owned = media.Artist().ok().map(|s| s.to_string());
        let source_owned = session.SourceAppUserModelId().ok().map(|s| s.to_string());

        let title = clean_text(title_owned.as_deref());
        let artist = clean_text(artist_owned.as_deref());
        let source = clean_text(source_owned.as_deref());
        Ok(Some(MusicState {
            playing,
            title,
            artist,
            source,
        }))
    });

    let state = match result {
        Ok(v) => v,
        Err(err) => {
            tracing::warn!("smtc unavailable: {}", err);
            return None;
        }
    }?;
    let title = state.title.clone();
    let artist = state.artist.clone();
    let source = state.source.clone();
    let playing = state.playing;
    if title.is_none() && artist.is_none() {
        tracing::info!("smtc found session but no title/artist");
        return Some(MusicState { playing: false, title, artist, source });
    }
    tracing::info!(
        "smtc ok: playing={} title={:?} artist={:?} source={:?}",
        playing,
        title,
        artist,
        source
    );
    Some(state)
}

#[cfg(target_os = "linux")]
fn current_music() -> Option<MusicState> {
    let out = Command::new("playerctl")
        .args(["-a", "metadata", "--format", "{{status}}\t{{title}}\t{{artist}}\t{{playerName}}"])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let text = String::from_utf8_lossy(&out.stdout);
    let mut first_non_empty: Option<MusicState> = None;
    for line in text.lines().map(str::trim).filter(|v| !v.is_empty()) {
        let mut parts = line.split('\t');
        let status = parts.next().unwrap_or_default();
        let title = clean_text(parts.next());
        let artist = clean_text(parts.next());
        let source = clean_text(parts.next());
        let playing = status.eq_ignore_ascii_case("playing");
        let state = MusicState {
            playing,
            title,
            artist,
            source,
        };
        if state.playing {
            return Some(state);
        }
        if first_non_empty.is_none() {
            first_non_empty = Some(state);
        }
    }
    first_non_empty
}

#[cfg(not(any(windows, target_os = "linux")))]
fn current_music() -> Option<MusicState> {
    None
}

#[cfg(windows)]
fn idle_seconds() -> Option<u64> {
    use windows_sys::Win32::Foundation::BOOL;
    use windows_sys::Win32::UI::Input::KeyboardAndMouse::{GetLastInputInfo, LASTINPUTINFO};
    use windows_sys::Win32::System::SystemInformation::GetTickCount;

    unsafe {
        let mut info = LASTINPUTINFO {
            cbSize: std::mem::size_of::<LASTINPUTINFO>() as u32,
            dwTime: 0,
        };
        let ok: BOOL = GetLastInputInfo(&mut info);
        if ok == 0 {
            return None;
        }
        let tick = GetTickCount();
        let elapsed_ms = tick.wrapping_sub(info.dwTime);
        Some((elapsed_ms / 1000) as u64)
    }
}

#[cfg(target_os = "linux")]
fn idle_seconds() -> Option<u64> {
    use std::ffi::CString;
    use x11::xlib::{XCloseDisplay, XDefaultRootWindow, XOpenDisplay};
    use x11::xss::{XScreenSaverAllocInfo, XScreenSaverQueryInfo};

    let display_name = std::env::var("DISPLAY").unwrap_or_default();
    let cstr = CString::new(display_name).ok()?;

    unsafe {
        let display = XOpenDisplay(cstr.as_ptr());
        if display.is_null() {
            return None;
        }
        let root = XDefaultRootWindow(display);
        let info = XScreenSaverAllocInfo();
        if info.is_null() {
            XCloseDisplay(display);
            return None;
        }
        XScreenSaverQueryInfo(display, root, info);
        let idle_ms = (*info).idle;
        XCloseDisplay(display);
        Some((idle_ms / 1000) as u64)
    }
}

#[cfg(not(any(windows, target_os = "linux")))]
fn idle_seconds() -> Option<u64> {
    None
}
