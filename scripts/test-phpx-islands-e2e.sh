#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
E2E_DIR="${ROOT_DIR}/tests/manual/hmr_e2e"
DEKA_BIN="${PHPX_BIN:-${ROOT_DIR}/target/release/cli}"
ENTRY="${ROOT_DIR}/tests/manual/hmr_e2e/islands_directives.phpx"
PORT="${ISLANDS_E2E_PORT:-8541}"
HOST="${ISLANDS_E2E_HOST:-127.0.0.1}"
LOG_FILE="${ISLANDS_E2E_LOG:-/tmp/deka-islands-e2e.log}"
PLAYWRIGHT_OUTPUT_DIR="${PLAYWRIGHT_OUTPUT_DIR:-/tmp/deka-islands-playwright}"

is_port_busy() {
  local port="$1"
  lsof -nP -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1
}

if [[ -z "${ISLANDS_E2E_PORT:-}" ]]; then
  base_port="$PORT"
  for candidate in $(seq "$base_port" $((base_port + 50))); do
    if ! is_port_busy "$candidate"; then
      PORT="$candidate"
      break
    fi
  done
fi

URL="http://${HOST}:${PORT}"

if [[ ! -x "$DEKA_BIN" ]]; then
  echo "[islands-e2e] release cli missing at $DEKA_BIN; building"
  (cd "$ROOT_DIR" && cargo build --release -p cli >/dev/null)
fi

echo "[islands-e2e] starting server: $DEKA_BIN serve $ENTRY"
(
  cd "$ROOT_DIR"
  PORT="$PORT" "$DEKA_BIN" serve "$ENTRY" >"$LOG_FILE" 2>&1
) &
SERVER_PID=$!

cleanup() {
  if kill -0 "$SERVER_PID" >/dev/null 2>&1; then
    kill "$SERVER_PID" >/dev/null 2>&1 || true
    wait "$SERVER_PID" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT

for _ in {1..80}; do
  if curl -fsS "$URL" >/dev/null 2>&1; then
    break
  fi
  sleep 0.25
done

if ! curl -fsS "$URL" >/dev/null 2>&1; then
  echo "[islands-e2e] server did not start at $URL"
  tail -n 80 "$LOG_FILE" || true
  exit 1
fi

(
  cd "$E2E_DIR"
  if [[ ! -d node_modules/@playwright/test ]]; then
    npm install --package-lock=false --no-fund --no-audit >/dev/null
  fi
  npx playwright install chromium >/dev/null
  HMR_E2E_URL="$URL" \
    npx playwright test islands_directives_e2e.spec.js --workers=1 --reporter=line --output="$PLAYWRIGHT_OUTPUT_DIR"
)

echo "[islands-e2e] complete"
