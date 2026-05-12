#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

RUN_CMD="${GOWASM_RUN_CMD:-$REPO_ROOT/run.sh}"
RUN_TIMEOUT_SECONDS="${GOWASM_SMOKE_RUN_TIMEOUT_SECONDS:-120}"
NODE_TIMEOUT_MS="${GOWASM_SMOKE_TIMEOUT_MS:-20000}"
FORCED_URL="${1:-${GOWASM_SMOKE_URL:-}}"

cleanup() {
  if [[ -n "${RUN_PID:-}" ]] && ps -p "$RUN_PID" >/dev/null 2>&1; then
    kill "$RUN_PID" 2>/dev/null || true
    wait "$RUN_PID" 2>/dev/null || true
  fi
  [[ -f "${LOG_FILE:-}" ]] && rm -f "$LOG_FILE"
}

trap cleanup EXIT INT TERM

if [[ -z "$FORCED_URL" ]]; then
  LOG_FILE="$(mktemp)"
  "$RUN_CMD" >"$LOG_FILE" 2>&1 &
  RUN_PID=$!

  echo "[smoke] started: $RUN_CMD (pid $RUN_PID)"
  deadline=$((SECONDS + RUN_TIMEOUT_SECONDS))
  while ((SECONDS < deadline)); do
    FORCED_URL="$(awk '/gowasm is running at:/{getline; print $1}' "$LOG_FILE" | tr -d '\r' | head -n1 || true)"
    if [[ -n "$FORCED_URL" ]]; then
      break
    fi
    sleep 1
  done

  if [[ -z "$FORCED_URL" ]]; then
    echo "[smoke] failed to read URL from run.sh output in ${RUN_TIMEOUT_SECONDS}s" >&2
    echo "--- run.sh log ---" >&2
    cat "$LOG_FILE" >&2
    exit 1
  fi
fi

echo "[smoke] checking $FORCED_URL"
GOWASM_SMOKE_TIMEOUT_MS="$NODE_TIMEOUT_MS" node "$SCRIPT_DIR/smoke-run.js" "$FORCED_URL"
