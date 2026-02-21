"ui";

// AutoX.js script
// Save as status-client-android/status.js

// ======= 配置 =======
const ENDPOINT = "http://you-host:7999/heartbeat";
const TOKEN = "your_token";
const DEVICE_ID = "android-phone";
const DEVICE_NAME = "Android";
const MUSIC_SOURCE = "netease-cloudmusic";

// 60s 心跳
const HEARTBEAT_INTERVAL_MS = 60 * 1000;
// 锁屏 5 分钟后上报离线
const OFFLINE_DELAY_MS = 5 * 60 * 1000;
// 通知抓取音乐状态（需要通知读取权限）
const ENABLE_MUSIC_NOTIFICATION = true;
const MUSIC_APP_PACKAGE = "com.netease.cloudmusic";
const MUSIC_STALE_MS = 3 * 60 * 1000;
const MUSIC_PUSH_MIN_INTERVAL_MS = 6 * 1000;

// ======= 前台保活 =======
try {
  if (typeof $app !== "undefined" && $app.setForegroundService) {
    $app.setForegroundService({
      title: "状态上报运行中",
      text: "每 60 秒心跳",
      ticker: "状态上报",
      ongoing: true
    });
  }
} catch (e) {
  // ignore
}

// ======= 状态 =======
let screenOffTimer = null;
let musicState = {
  playing: false,
  title: null,
  artist: null,
  source: MUSIC_SOURCE,
  updatedAt: 0
};
let lastMusicSignature = "";
let lastMusicPushAt = 0;

function cleanText(v) {
  if (!v) return null;
  const s = String(v).trim();
  return s.length > 0 ? s : null;
}

function updateMusicState(title, artist) {
  const cleanTitle = cleanText(title);
  const cleanArtist = cleanText(artist);
  musicState = {
    playing: !!(cleanTitle || cleanArtist),
    title: cleanTitle,
    artist: cleanArtist,
    source: MUSIC_SOURCE,
    updatedAt: Date.now()
  };
}

function musicSignature(s) {
  return `${s.playing ? 1 : 0}|${s.title || ""}|${s.artist || ""}|${s.source || ""}`;
}

function clearMusicState() {
  musicState = {
    playing: false,
    title: null,
    artist: null,
    source: MUSIC_SOURCE,
    updatedAt: Date.now()
  };
}

function readMusicSnapshot() {
  const fresh = Date.now() - musicState.updatedAt <= MUSIC_STALE_MS;
  if (!fresh) {
    return {
      playing: false,
      title: null,
      artist: null,
      source: MUSIC_SOURCE
    };
  }
  return {
    playing: musicState.playing,
    title: musicState.title,
    artist: musicState.artist,
    source: musicState.source || MUSIC_SOURCE
  };
}

function parseNeteaseNotification(notification) {
  const title = cleanText(notification.getTitle && notification.getTitle());
  const text = cleanText(notification.getText && notification.getText());
  const subText = cleanText(notification.getSubText && notification.getSubText());
  if (!title && !text && !subText) return;

  let songTitle = title;
  let songArtist = subText || null;
  if (!songArtist && text) {
    songArtist = text;
  }

  if (songTitle && songTitle.includes(" - ")) {
    const pair = songTitle.split(" - ");
    songTitle = cleanText(pair[0]);
    songArtist = cleanText(pair.slice(1).join(" - ")) || songArtist;
  }
  if ((!songTitle || !songArtist) && text && text.includes(" - ")) {
    const pair = text.split(" - ");
    songTitle = songTitle || cleanText(pair[0]);
    songArtist = songArtist || cleanText(pair.slice(1).join(" - "));
  }

  const before = musicSignature(readMusicSnapshot());
  updateMusicState(songTitle, songArtist);
  const after = musicSignature(readMusicSnapshot());
  if (before !== after) {
    pushMusicHeartbeat();
  }
}

function pushMusicHeartbeat() {
  const now = Date.now();
  const current = musicSignature(readMusicSnapshot());
  if (current === lastMusicSignature) return;
  if (lastMusicPushAt && now - lastMusicPushAt < MUSIC_PUSH_MIN_INTERVAL_MS) return;
  lastMusicSignature = current;
  lastMusicPushAt = now;
  sendHeartbeat(true, 0);
}

// ======= 心跳上报 =======
function sendHeartbeat(online, idleSeconds) {
  const music = readMusicSnapshot();
  const payload = {
    device_id: DEVICE_ID,
    device_name: DEVICE_NAME,
    online: online,
    idle_seconds: idleSeconds || 0,
    music_playing: music.playing,
    music_title: music.title,
    music_artist: music.artist,
    music_source: music.source
  };
  try {
    http.postJson(ENDPOINT, payload, {
      headers: { "x-token": TOKEN }
    });
  } catch (e) {
    // ignore
  }
}

// ======= 音乐通知监听 =======
if (ENABLE_MUSIC_NOTIFICATION) {
  try {
    events.observeNotification();
    events.on("notification", (notification) => {
      try {
        const pkg = notification.getPackageName && notification.getPackageName();
        if (pkg !== MUSIC_APP_PACKAGE) return;
        parseNeteaseNotification(notification);
      } catch (e) {
        // ignore
      }
    });
  } catch (e) {
    // ignore
  }
}

// ======= 锁屏处理 =======
try {
  events.observeScreen();
  events.on("screen_off", () => {
    if (screenOffTimer) clearTimeout(screenOffTimer);
    screenOffTimer = setTimeout(() => {
      sendHeartbeat(false, OFFLINE_DELAY_MS / 1000);
    }, OFFLINE_DELAY_MS);
  });

  events.on("screen_on", () => {
    if (screenOffTimer) clearTimeout(screenOffTimer);
    sendHeartbeat(true, 0);
  });
} catch (e) {
  // ignore
}

// ======= 定时心跳 =======
setInterval(() => {
  sendHeartbeat(true, 0);
}, HEARTBEAT_INTERVAL_MS);

// 启动时立即上报一次
clearMusicState();
lastMusicSignature = musicSignature(readMusicSnapshot());
sendHeartbeat(true, 0);
