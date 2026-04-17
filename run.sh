#!/usr/bin/env bash
set -euo pipefail

root_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)
web_dir="$root_dir/web"
host="${HOST:-127.0.0.1}"
port="${PORT:-8000}"
url="http://$host:$port/"
server_pid=""

cleanup() {
  if [[ -n "$server_pid" ]] && kill -0 "$server_pid" 2>/dev/null; then
    kill "$server_pid" 2>/dev/null || true
    wait "$server_pid" 2>/dev/null || true
  fi
}

open_browser() {
  local target_url="$1"

  if [[ -n "${BROWSER:-}" ]] && "$BROWSER" "$target_url" >/dev/null 2>&1; then
    return 0
  fi

  if command -v xdg-open >/dev/null 2>&1 && xdg-open "$target_url" >/dev/null 2>&1; then
    return 0
  fi

  if command -v open >/dev/null 2>&1 && open "$target_url" >/dev/null 2>&1; then
    return 0
  fi

  if command -v sensible-browser >/dev/null 2>&1 && sensible-browser "$target_url" >/dev/null 2>&1; then
    return 0
  fi

  python3 -m webbrowser "$target_url" >/dev/null 2>&1
}

trap cleanup EXIT INT TERM

"$root_dir/scripts/build-web.sh"

python3 -m http.server "$port" --bind "$host" --directory "$web_dir" >/dev/null 2>&1 &
server_pid=$!

sleep 1
if ! kill -0 "$server_pid" 2>/dev/null; then
  echo "failed to start local web server for $web_dir" >&2
  wait "$server_pid"
  exit 1
fi

if ! open_browser "$url"; then
  echo "failed to launch a web browser automatically; open $url manually" >&2
  exit 1
fi

echo "serving $web_dir at $url"
echo "press Ctrl+C to stop"
wait "$server_pid"
