"ui";

// AutoX.js script
// Save as status-client-android/status.js

// ======= 配置 =======
const ENDPOINT = "http://you-host:7999/heartbeat";
const TOKEN = "your_token";
const DEVICE_ID = "android-phone";
const DEVICE_NAME = "Android";

// 60s 心跳
const HEARTBEAT_INTERVAL_MS = 60 * 1000;
// 锁屏 5 分钟后上报离线
const OFFLINE_DELAY_MS = 5 * 60 * 1000;

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

// ======= 心跳上报 =======
function sendHeartbeat(online, idleSeconds) {
  const payload = {
    device_id: DEVICE_ID,
    device_name: DEVICE_NAME,
    online: online,
    idle_seconds: idleSeconds || 0
  };
  try {
    http.postJson(ENDPOINT, payload, {
      headers: { "x-token": TOKEN }
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
sendHeartbeat(true, 0);
