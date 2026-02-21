#![cfg_attr(windows, windows_subsystem = "windows")]

use serde::{Deserialize, Serialize};
use std::{
    fs,
    path::PathBuf,
    process::Command,
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};
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

#[derive(Default)]
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
    log_file: Option<String>,
}

#[derive(Clone)]
struct Config {
    endpoint: String,
    token: String,
    device_id: String,
    device_name: String,
    idle_timeout_secs: u64,
    heartbeat_interval_secs: u64,
    log_file: String,
}

fn main() {
    let cfg = load_config();
    let _log_guard = init_logging(&cfg);
    tracing::info!("status-client starting: log={}", cfg.log_file);

    let status = Arc::new(Mutex::new(String::from("starting")));

    let hb_cfg = cfg.clone();
    let hb_status = status.clone();
    thread::spawn(move || heartbeat_loop(hb_cfg, hb_status));

    loop {
        thread::sleep(Duration::from_secs(3600));
    }
}

fn heartbeat_loop(cfg: Config, status: Arc<Mutex<String>>) {
    let client = reqwest::blocking::Client::new();
    loop {
        let idle = idle_seconds();
        let online = idle.map(|v| v < cfg.idle_timeout_secs).unwrap_or(true);

        let payload = Heartbeat {
            device_id: cfg.device_id.clone(),
            device_name: cfg.device_name.clone(),
            online,
            idle_seconds: idle,
            music_playing: false,
            music_title: None,
            music_artist: None,
            music_source: None,
        };
        let mut payload = payload;
        if let Some(music) = current_music() {
            payload.music_playing = music.playing;
            payload.music_title = music.title;
            payload.music_artist = music.artist;
            payload.music_source = music.source;
        }

        let res = client
            .post(&cfg.endpoint)
            .header("x-token", &cfg.token)
            .json(&payload)
            .send();

        if let Ok(resp) = res {
            let label: &str = if resp.status().is_success() { "online" } else { "error" };
            if let Ok(mut s) = status.lock() {
                *s = label.to_string();
            }
        }

        thread::sleep(Duration::from_secs(cfg.heartbeat_interval_secs));
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

    let log_file = std::env::var("LOG_FILE")
        .ok()
        .or(file_cfg.log_file)
        .unwrap_or_else(|| "status-client.log".to_string());

    Config {
        endpoint,
        token,
        device_id,
        device_name,
        idle_timeout_secs,
        heartbeat_interval_secs,
        log_file,
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

fn init_logging(cfg: &Config) -> tracing_appender::non_blocking::WorkerGuard {
    let log_path = resolve_path(&cfg.log_file);
    let file_appender = tracing_appender::rolling::never(
        log_path.parent().unwrap_or_else(|| std::path::Path::new(".")),
        log_path.file_name().unwrap_or_default(),
    );
    let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(non_blocking.and(std::io::stdout))
        .init();
    guard
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

fn clean_text(s: Option<&str>) -> Option<String> {
    s.map(|v| v.trim())
        .filter(|v| !v.is_empty())
        .map(|v| v.to_string())
}

#[cfg(windows)]
fn current_music() -> Option<MusicState> {
    let script = r#"
Add-Type -AssemblyName System.Runtime.WindowsRuntime
$null = [Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager, Windows.Media.Control, ContentType = WindowsRuntime]
$req = [Windows.Media.Control.GlobalSystemMediaTransportControlsSessionManager]::RequestAsync()
while ($req.Status -eq 0) { Start-Sleep -Milliseconds 12 }
if ($req.Status -ne 1) { return }
$manager = $req.GetResults()
$session = $manager.GetCurrentSession()
if ($null -eq $session) { return }
$mediaReq = $session.TryGetMediaPropertiesAsync()
while ($mediaReq.Status -eq 0) { Start-Sleep -Milliseconds 12 }
if ($mediaReq.Status -ne 1) { return }
$media = $mediaReq.GetResults()
$playback = $session.GetPlaybackInfo()
$status = [string]$playback.PlaybackStatus
[pscustomobject]@{
  playing = ($status -eq 'Playing')
  title = [string]$media.Title
  artist = [string]$media.Artist
  source = [string]$session.SourceAppUserModelId
} | ConvertTo-Json -Compress
"#;
    let out = Command::new("powershell")
        .args(["-NoProfile", "-NonInteractive", "-Command", script])
        .output()
        .ok()?;
    if !out.status.success() {
        return None;
    }
    let raw = String::from_utf8_lossy(&out.stdout);
    let raw = raw.trim();
    if raw.is_empty() {
        return None;
    }
    let value: serde_json::Value = serde_json::from_str(raw).ok()?;
    let title = clean_text(value.get("title").and_then(|v| v.as_str()));
    let artist = clean_text(value.get("artist").and_then(|v| v.as_str()));
    let source = clean_text(value.get("source").and_then(|v| v.as_str()));
    let playing = value
        .get("playing")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    if title.is_none() && artist.is_none() {
        return Some(MusicState {
            playing: false,
            title: None,
            artist: None,
            source,
        });
    }
    Some(MusicState {
        playing,
        title,
        artist,
        source,
    })
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
