#!/system/bin/sh
# Magisk service script
MODDIR=${0%/*}

# Shell heartbeat daemon (root)
CONFIG="/data/adb/meowra-status/config.env"

# defaults
ENDPOINT="http://your_host:7999/heartbeat"
TOKEN="your_token"
DEVICE_ID="$(getprop ro.product.device)"
DEVICE_NAME="$(getprop ro.product.model)"
HEARTBEAT_INTERVAL=60
OFFLINE_DELAY=300
MUSIC_POLL_INTERVAL=5
MUSIC_PUSH_MIN_INTERVAL=6
ENABLE_MUSIC_NOTIFICATION=1
MUSIC_PACKAGE="com.netease.cloudmusic"
MUSIC_SOURCE="netease-cloudmusic"

if [ -f "$CONFIG" ]; then
  . "$CONFIG"
fi

last_off=0
last_hb_ts=0
last_music_push_ts=0
last_music_sig=""

json_escape() {
  echo "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'
}

extract_music_from_notification() {
  music_playing=false
  music_title=""
  music_artist=""
  music_source="$MUSIC_SOURCE"

  [ "$ENABLE_MUSIC_NOTIFICATION" = "1" ] || return 0

  dump="$(dumpsys notification --noredact 2>/dev/null)"
  [ -n "$dump" ] || return 0

  block="$(echo "$dump" | awk -v pkg="$MUSIC_PACKAGE" '
    index($0, pkg) { in_pkg=1 }
    in_pkg { print }
    in_pkg && /^ *mSnoozeHelper:/ { exit }
  ')"
  [ -n "$block" ] || return 0

  music_title="$(echo "$block" | sed -n 's/.*android.title=\(.*\)$/\1/p' | head -n1 | tr -d '\r')"
  music_artist="$(echo "$block" | sed -n 's/.*android.subText=\(.*\)$/\1/p' | head -n1 | tr -d '\r')"
  if [ -z "$music_artist" ]; then
    music_artist="$(echo "$block" | sed -n 's/.*android.text=\(.*\)$/\1/p' | head -n1 | tr -d '\r')"
  fi

  # 兼容 “歌名 - 歌手” 这种通知格式
  if [ -n "$music_title" ] && [ -z "$music_artist" ] && echo "$music_title" | grep -q " - "; then
    music_artist="${music_title#* - }"
    music_title="${music_title%% - *}"
  fi

  [ -n "$music_title" ] || [ -n "$music_artist" ] || return 0
  music_playing=true
}

post_json() {
  local online="$1"
  local idle="$2"
  extract_music_from_notification

  local esc_device_id esc_device_name esc_music_source esc_music_title esc_music_artist
  esc_device_id="$(json_escape "$DEVICE_ID")"
  esc_device_name="$(json_escape "$DEVICE_NAME")"
  esc_music_source="$(json_escape "$music_source")"
  esc_music_title="$(json_escape "$music_title")"
  esc_music_artist="$(json_escape "$music_artist")"

  local title_json artist_json
  if [ -n "$music_title" ]; then
    title_json="\"$esc_music_title\""
  else
    title_json="null"
  fi
  if [ -n "$music_artist" ]; then
    artist_json="\"$esc_music_artist\""
  else
    artist_json="null"
  fi

  local payload="{\"device_id\":\"$esc_device_id\",\"device_name\":\"$esc_device_name\",\"online\":$online,\"idle_seconds\":$idle,\"music_playing\":$music_playing,\"music_title\":$title_json,\"music_artist\":$artist_json,\"music_source\":\"$esc_music_source\"}"
  if command -v curl >/dev/null 2>&1; then
    curl -s -m 5 -H "x-token: $TOKEN" -H "Content-Type: application/json" -d "$payload" "$ENDPOINT" >/dev/null 2>&1
  elif command -v wget >/dev/null 2>&1; then
    wget -q --timeout=5 --header="x-token: $TOKEN" --header="Content-Type: application/json" --post-data="$payload" "$ENDPOINT" -O /dev/null 2>&1
  fi
}

current_presence() {
  local now="$1"
  if screen_is_on; then
    last_off=0
    echo "true 0"
    return 0
  fi
  if [ "$last_off" -eq 0 ]; then
    last_off="$now"
  fi
  local idle=$((now - last_off))
  if [ "$idle" -ge "$OFFLINE_DELAY" ]; then
    echo "false $idle"
  else
    echo "true $idle"
  fi
}

current_music_sig() {
  extract_music_from_notification
  echo "${music_playing}|${music_title}|${music_artist}|${music_source}"
}

screen_is_on() {
  dumpsys power | grep -q "mScreenOn=true" && return 0
  dumpsys display | grep -q "mScreenState=ON" && return 0
  dumpsys power | grep -q "Display Power: state=ON" && return 0
  return 1
}

while true; do
  now=$(date +%s)

  # 常规心跳：保持原频率
  if [ $((now - last_hb_ts)) -ge "$HEARTBEAT_INTERVAL" ]; then
    set -- $(current_presence "$now")
    post_json "$1" "$2"
    last_hb_ts="$now"
  fi

  # 音乐变化：独立快速上报
  sig="$(current_music_sig)"
  if [ "$sig" != "$last_music_sig" ]; then
    if [ "$last_music_push_ts" -eq 0 ] || [ $((now - last_music_push_ts)) -ge "$MUSIC_PUSH_MIN_INTERVAL" ]; then
      set -- $(current_presence "$now")
      post_json "$1" "$2"
      last_music_push_ts="$now"
    fi
    last_music_sig="$sig"
  fi

  sleep "$MUSIC_POLL_INTERVAL"
done
