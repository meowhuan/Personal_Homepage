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

if [ -f "$CONFIG" ]; then
  . "$CONFIG"
fi

last_off=0

post_json() {
  local online="$1"
  local idle="$2"
  local payload="{\"device_id\":\"$DEVICE_ID\",\"device_name\":\"$DEVICE_NAME\",\"online\":$online,\"idle_seconds\":$idle}"
  if command -v curl >/dev/null 2>&1; then
    curl -s -m 5 -H "x-token: $TOKEN" -H "Content-Type: application/json" -d "$payload" "$ENDPOINT" >/dev/null 2>&1
  elif command -v wget >/dev/null 2>&1; then
    wget -q --timeout=5 --header="x-token: $TOKEN" --header="Content-Type: application/json" --post-data="$payload" "$ENDPOINT" -O /dev/null 2>&1
  fi
}

screen_is_on() {
  dumpsys power | grep -q "mScreenOn=true" && return 0
  dumpsys display | grep -q "mScreenState=ON" && return 0
  dumpsys power | grep -q "Display Power: state=ON" && return 0
  return 1
}

while true; do
  now=$(date +%s)
  if screen_is_on; then
    last_off=0
    post_json true 0
  else
    if [ "$last_off" -eq 0 ]; then
      last_off=$now
    fi
    idle=$((now - last_off))
    if [ "$idle" -ge "$OFFLINE_DELAY" ]; then
      post_json false "$idle"
    else
      post_json true "$idle"
    fi
  fi
  sleep "$HEARTBEAT_INTERVAL"
done
