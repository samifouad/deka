#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
E2E_DIR="${ROOT_DIR}/tests/e2e"
PORT="${ADWA_E2E_PORT:-5173}"
HOST="${ADWA_E2E_HOST:-127.0.0.1}"
URL="http://${HOST}:${PORT}"
LOG_FILE="${ADWA_E2E_LOG:-/tmp/adwa-e2e.log}"
PLAYWRIGHT_OUTPUT_DIR="${PLAYWRIGHT_OUTPUT_DIR:-/tmp/adwa-playwright}"
PLAYWRIGHT_INSTALL_TIMEOUT_MS="${PLAYWRIGHT_INSTALL_TIMEOUT_MS:-120000}"
PLAYWRIGHT_DOWNLOAD_HOST="${PLAYWRIGHT_DOWNLOAD_HOST:-https://playwright.download.prss.microsoft.com}"

if ! command -v rg >/dev/null 2>&1; then
  echo "[adwa-e2e] ripgrep (rg) is required"
  exit 1
fi

echo "[adwa-e2e] building demo assets"
PORT="$PORT" "$ROOT_DIR/scripts/run-playground.sh" --build-only

if rg -n "vendor/php_rs|php_rs\\.js|php_rs_bg\\.wasm" "$ROOT_DIR/website/main.js" >/dev/null 2>&1; then
  echo "[adwa-e2e] architecture guard failed: browser demo imports php-rs directly."
  exit 1
fi

if rg -n "WebAssembly\\.instantiate|WebAssembly\\.instantiateStreaming" "$ROOT_DIR/website/main.js" >/dev/null 2>&1; then
  echo "[adwa-e2e] architecture guard failed: browser demo calls raw WebAssembly instantiate APIs."
  exit 1
fi

echo "[adwa-e2e] serving browser demo at $URL"
(
  cd "$ROOT_DIR/website"
  python3 -m http.server "$PORT" >"$LOG_FILE" 2>&1
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
  echo "[adwa-e2e] server did not start at $URL"
  tail -n 80 "$LOG_FILE" || true
  exit 1
fi

echo "[adwa-e2e] ensuring playwright chromium is installed"
(
  cd "$E2E_DIR"
  if [[ ! -d node_modules/@playwright/test ]]; then
    npm install --package-lock=false --no-fund --no-audit >/dev/null
  fi
  PLAYWRIGHT_DOWNLOAD_CONNECTION_TIMEOUT="$PLAYWRIGHT_INSTALL_TIMEOUT_MS" \
  PLAYWRIGHT_DOWNLOAD_HOST="$PLAYWRIGHT_DOWNLOAD_HOST" \
    npx playwright install chromium >/dev/null
)

echo "[adwa-e2e] running playwright browser check"
(
  cd "$E2E_DIR"
  ADWA_E2E_URL="$URL" \
    npx playwright test adwa_playground_e2e.spec.js --workers=1 --reporter=line --output="$PLAYWRIGHT_OUTPUT_DIR"
)

echo "[adwa-e2e] complete"
