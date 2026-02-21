# Personal Homepage + Status System

本仓库包含个人主页与在线状态系统（后端 + 客户端）。

This repo contains a personal homepage and an online status system (backend + clients).

## 结构 / Structure

- `src/`：个人主页（Vite + Vue3 + UnoCSS）
- `status-backend/`：在线状态后端（Rust + SQLite）
- `status-client-rust/`：Windows / Linux 客户端（静默心跳)
- `status-client-android/`：Android 方案（AutoX.js / Magisk shell）

- 注：Linux客户端并未进行测试，目前能够完成使用的仅Windows客户端，Linux可用性待取证

## 快速开始 / Quick Start

### 1) 主页 / Homepage

```bash
npm install
npm run dev
```

### 2) 后端 / Backend

见 / See `status-backend/README.md` + `status-backend/DEPLOY.md`

### 3) 客户端 / Clients

- Windows / Linux：`status-client-rust/README.md`
- Android：`status-client-android/README.md`

## 说明 / Notes

- 后端默认 5 分钟未上报判离线。
- Android 高频心跳需要前台服务或 root shell 守护（见 Android README）。
- 所有文档内仅示例，不包含真实地址或密钥。

Backend marks offline after 5 minutes without heartbeat by default.
Android frequent heartbeats require a foreground service or root shell daemon.
All docs are examples only and do not include real endpoints or tokens.
