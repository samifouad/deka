#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
E2E_DIR="${ROOT_DIR}/tests/manual/hmr_e2e"
DEKA_BIN="${PHPX_BIN:-${ROOT_DIR}/target/release/cli}"
ENTRY="${ROOT_DIR}/tests/manual/hmr_e2e/index.phpx"
PORT="${HMR_E2E_PORT:-8530}"
HOST="${HMR_E2E_HOST:-127.0.0.1}"
LOG_FILE="${HMR_E2E_LOG:-/tmp/deka-hmr-e2e.log}"
PLAYWRIGHT_OUTPUT_DIR="${PLAYWRIGHT_OUTPUT_DIR:-/tmp/deka-hmr-playwright}"
PLAYWRIGHT_INSTALL_TIMEOUT_MS="${PLAYWRIGHT_INSTALL_TIMEOUT_MS:-120000}"
PLAYWRIGHT_DOWNLOAD_HOST="${PLAYWRIGHT_DOWNLOAD_HOST:-https://playwright.download.prss.microsoft.com}"

is_port_busy() {
  local port="$1"
  lsof -nP -iTCP:"$port" -sTCP:LISTEN >/dev/null 2>&1
}

if [[ -z "${HMR_E2E_PORT:-}" ]]; then
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
  echo "[hmr-e2e] release cli missing at $DEKA_BIN; building"
  (cd "$ROOT_DIR" && cargo build --release -p cli >/dev/null)
fi

echo "[hmr-e2e] starting dev server: $DEKA_BIN serve --dev $ENTRY"
(
  cd "$ROOT_DIR"
  PORT="$PORT" "$DEKA_BIN" serve --dev "$ENTRY" >"$LOG_FILE" 2>&1
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
  echo "[hmr-e2e] server did not start at $URL"
  echo "[hmr-e2e] recent logs:"
  tail -n 80 "$LOG_FILE" || true
  exit 1
fi

echo "[hmr-e2e] ensuring playwright chromium is installed"
(
  cd "$E2E_DIR"
  if [[ ! -d node_modules/@playwright/test ]]; then
    npm install --package-lock=false --no-fund --no-audit >/dev/null
  fi
  PLAYWRIGHT_DOWNLOAD_CONNECTION_TIMEOUT="$PLAYWRIGHT_INSTALL_TIMEOUT_MS" \
  PLAYWRIGHT_DOWNLOAD_HOST="$PLAYWRIGHT_DOWNLOAD_HOST" \
    npx playwright install chromium >/dev/null
)

echo "[hmr-e2e] running playwright browser check"
(
  cd "$E2E_DIR"
  HMR_E2E_URL="$URL" HMR_E2E_ENTRY_FILE="$ENTRY" \
    npx playwright test hmr_e2e.spec.js --workers=1 --reporter=line --output="$PLAYWRIGHT_OUTPUT_DIR"
)

echo "[hmr-e2e] complete"
