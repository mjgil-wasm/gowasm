#!/usr/bin/env bash
set -e

cd "$(dirname "$0")"

# echo "🔧 Building wasm..."
# ./scripts/build-web.sh

echo ""

cd web

PID=""
PORTLESS_BIN=""
PORTLESS_NODE_BIN=""
PORTLESS_STATE_DIR="${PORTLESS_STATE_DIR:-$HOME/.portless}"
WEB_DIR="$(pwd)"
PORTLESS_APP_ARGS=("$@")

cleanup() {
  echo -e "\n🛑 Stopping server..."
  kill "$PID" 2>/dev/null || true
  exit 0
}
trap cleanup SIGINT SIGTERM

run_portless() {
  if [[ -n "$PORTLESS_NODE_BIN" ]]; then
    env PORTLESS_STATE_DIR="$PORTLESS_STATE_DIR" "$PORTLESS_NODE_BIN" "$PORTLESS_BIN" "$@"
    return
  fi
  env PORTLESS_STATE_DIR="$PORTLESS_STATE_DIR" "$PORTLESS_BIN" "$@"
}

try_portless_runner() {
  local candidate_node="$1"
  if [[ -n "$candidate_node" ]]; then
    "$candidate_node" "$PORTLESS_BIN" list >/dev/null 2>&1
    return
  fi
  "$PORTLESS_BIN" list >/dev/null 2>&1
}

select_portless_runner() {
  local current_node candidate version node_path
  current_node=$(command -v node 2>/dev/null || true)

  if try_portless_runner ""; then
    return 0
  fi

  if [[ -x "${PORTLESS_NODE:-}" ]] && try_portless_runner "$PORTLESS_NODE"; then
    PORTLESS_NODE_BIN="$PORTLESS_NODE"
    return 0
  fi

  if command -v asdf >/dev/null 2>&1; then
    while IFS= read -r version; do
      version=${version// /}
      version=${version#\*}
      [[ -z "$version" ]] && continue
      node_path="$HOME/.asdf/installs/nodejs/$version/bin/node"
      [[ ! -x "$node_path" ]] && continue
      [[ -n "$current_node" && "$node_path" == "$current_node" ]] && continue
      if try_portless_runner "$node_path"; then
        PORTLESS_NODE_BIN="$node_path"
        return 0
      fi
    done < <(asdf list nodejs 2>/dev/null)
  fi

  return 1
}

ensure_portless_proxy() {
  local route host_port

  proxy_is_standard() {
    route="$(run_portless get gowasm 2>/dev/null || true)"
    [[ -n "$route" ]] || return 1
    [[ "$route" == http://* || "$route" == https://* ]] || return 1
    host_port="${route#*://}"
    host_port="${host_port%%/*}"
    [[ "$host_port" == *":"* ]] && return 1
    return 0
  }

  run_portless proxy start --no-tls >/dev/null 2>&1 || true
  if proxy_is_standard; then
    return 0
  fi

  run_portless proxy start --force --no-tls >/dev/null 2>&1 || true
  if proxy_is_standard; then
    return 0
  fi

  run_portless proxy start >/dev/null 2>&1 || true
  if proxy_is_standard; then
    return 0
  fi

  echo "ℹ️  Portless could not bind port 80/443 without elevation; retrying on :1355..."
  run_portless proxy stop -p 1355 >/dev/null 2>&1 || true
  run_portless proxy start --port 1355 --no-tls >/dev/null
}

# Detect whether portless is available and not explicitly disabled.
USE_PORTLESS=false
if [[ "${PORTLESS:-}" != "0" ]] && command -v portless >/dev/null 2>&1; then
  PORTLESS_BIN=$(command -v portless)
  USE_PORTLESS=true
fi

if [[ "$USE_PORTLESS" == "true" ]]; then
  if ! select_portless_runner; then
    echo "⚠️  Portless is installed but could not start with any available Node runtime."
    echo "   Falling back to localhost. Set PORTLESS=0 to skip the Portless probe."
    USE_PORTLESS=false
  elif [[ -n "$PORTLESS_NODE_BIN" ]]; then
    echo "ℹ️  Portless is using alternate Node runtime: $PORTLESS_NODE_BIN"
  fi
fi

if [[ "$USE_PORTLESS" == "true" ]]; then
  echo "🔄 Ensuring the Portless proxy is running..."
  echo "   (Tip: Set PORTLESS=0 to skip portless and use localhost instead.)"
  ensure_portless_proxy
fi

if [[ "$USE_PORTLESS" == "true" ]]; then
  echo "🚀 Starting gowasm IDE via Portless..."
  run_portless gowasm "${PORTLESS_APP_ARGS[@]}" env GOWASM_WEB_DIR="$WEB_DIR" bash -c 'python3 -m http.server "$PORT" --directory "$GOWASM_WEB_DIR"' &
  PID=$!
  PORTLESS_URL="$(run_portless get gowasm)"
  PORTLESS_SCHEME="${PORTLESS_URL%%://*}"
  PORTLESS_HOST_PORT="${PORTLESS_URL#*://}"
  PORTLESS_HOST="${PORTLESS_HOST_PORT%%/*}"
  PORTLESS_HOST_NO_PORT="${PORTLESS_HOST%%:*}"
  URL="${PORTLESS_SCHEME}://${PORTLESS_HOST_NO_PORT}/ide/index.html"
  HEALTH_URL="${PORTLESS_URL}/ide/index.html"
  OPEN_URL="$HEALTH_URL"
else
  PORT="${PORT:-8080}"

  if lsof -i":$PORT" >/dev/null 2>&1; then
    echo "⚠️  Port $PORT is already in use."
    echo "   Set a different port with: PORT=9000 ./run.sh"
    exit 1
  fi

  echo "🚀 Starting gowasm IDE on localhost:$PORT..."
  python3 -m http.server "$PORT" --directory "$WEB_DIR" &
  PID=$!
  URL="http://localhost:$PORT/ide/index.html"
  HEALTH_URL="$URL"
  OPEN_URL="$URL"
fi

# Verify the server is actually responding before opening the browser.
HEALTH_RETRIES=0
MAX_RETRIES=15
while ! curl -s --fail "$HEALTH_URL" >/dev/null 2>&1; do
  HEALTH_RETRIES=$((HEALTH_RETRIES + 1))
  if [[ "$HEALTH_RETRIES" -ge "$MAX_RETRIES" ]]; then
    echo "❌ Server did not start after $MAX_RETRIES attempts."
    kill "$PID" 2>/dev/null || true
    exit 1
  fi
  sleep 1
done

echo ""
echo "🌐 gowasm IDE is running at:"
echo "   $URL"
echo "   (Classic shell available at ${URL%ide/index.html}index.html)"
echo ""
echo "Press Ctrl+C to stop the server"

# Try to open browser, but don't fail if unavailable.
if command -v xdg-open >/dev/null 2>&1; then
  xdg-open "$OPEN_URL" 2>/dev/null || true
elif command -v open >/dev/null 2>&1; then
  open "$OPEN_URL" 2>/dev/null || true
fi

wait "$PID"
