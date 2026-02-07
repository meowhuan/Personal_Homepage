# Status Client (Android)

Android 端提供两种方案：
Android offers two approaches:

- **AutoX.js 脚本**（非 root / non-root）
- **Magisk Shell 守护**（root）

> 说明：原生 App 方案已暂停，当前以 AutoX.js / Magisk 为主。
> Note: native app is paused. AutoX.js / Magisk are the primary options.

## 方案 A：AutoX.js

脚本位于 / Script path:

- `status-client-android/status.js`

修改脚本顶部配置 / Edit config at the top:

- `ENDPOINT`
- `TOKEN`
- `DEVICE_ID`
- `DEVICE_NAME`

功能 / Features:

- 60s 心跳 / heartbeat
- 锁屏 5 分钟离线 / offline after 5 min screen-off
- 前台保活（尽力）/ best-effort keep-alive

## 方案 B：Magisk Shell 守护（root）

模块目录 / Module dir:

- `status-client-android/magisk-module/`

配置文件 / Config file:

- `status-client-android/config.env`

示例 / Example:

```
ENDPOINT="http://your-host:7999/heartbeat"
TOKEN="your_token"
DEVICE_ID="android-root"
DEVICE_NAME="Android Root"
HEARTBEAT_INTERVAL=60
OFFLINE_DELAY=300
```

功能 / Features:

- 60s 心跳 / heartbeat
- 锁屏 5 分钟离线（通过 `dumpsys power/display`）
- 断网/关机交由后端超时处理

Offline/Power-off is handled by backend timeout.

## 打包模块 / Pack Module

将 `magisk-module/` 目录打包为 zip 后在 Magisk 安装。
Zip `magisk-module/` and install it via Magisk.
