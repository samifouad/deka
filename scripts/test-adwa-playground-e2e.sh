#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEFAULT_ADWA_DIR="$ROOT_DIR/../adwa"
ADWA_DIR="${ADWA_DIR:-$DEFAULT_ADWA_DIR}"
if [ ! -d "$ADWA_DIR" ]; then
  ADWA_DIR="$ROOT_DIR/adwa"
fi
E2E_DIR="${ROOT_DIR}/tests/manual/hmr_e2e"
PORT="${ADWA_E2E_PORT:-5173}"
HOST="${ADWA_E2E_HOST:-127.0.0.1}"
URL="http://${HOST}:${PORT}"
LOG_FILE="${ADWA_E2E_LOG:-/tmp/deka-adwa-e2e.log}"
PLAYWRIGHT_OUTPUT_DIR="${PLAYWRIGHT_OUTPUT_DIR:-/tmp/deka-adwa-playwright}"
PLAYWRIGHT_INSTALL_TIMEOUT_MS="${PLAYWRIGHT_INSTALL_TIMEOUT_MS:-120000}"
PLAYWRIGHT_DOWNLOAD_HOST="${PLAYWRIGHT_DOWNLOAD_HOST:-https://playwright.download.prss.microsoft.com}"
INCLUDE_PHPX="${ADWA_E2E_INCLUDE_PHPX:-0}"

echo "[adwa-e2e] building demo assets"
PORT="$PORT" "${ROOT_DIR}/scripts/run-adwa-playground.sh" --build-only

if rg -n "vendor/php_rs|php_rs\\.js|php_rs_bg\\.wasm" "${ADWA_DIR}/website/main.js" >/dev/null 2>&1; then
  echo "[adwa-e2e] architecture guard failed: browser demo imports php-rs directly."
  echo "[adwa-e2e] use runtime adapter APIs instead of direct php-rs wasm wiring."
  exit 1
fi

if rg -n "WebAssembly\\.instantiate|WebAssembly\\.instantiateStreaming" "${ADWA_DIR}/website/main.js" >/dev/null 2>&1; then
  echo "[adwa-e2e] architecture guard failed: browser demo calls raw WebAssembly instantiate APIs."
  echo "[adwa-e2e] wasm startup must stay behind runtime adapter abstractions."
  exit 1
fi

echo "[adwa-e2e] serving browser demo at $URL"
(
  cd "${ADWA_DIR}/website"
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
  echo "[adwa-e2e] recent logs:"
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
  if [[ "$INCLUDE_PHPX" == "1" ]]; then
    SPECS=("adwa_playground_e2e.spec.js" "adwa_phpx_e2e.spec.js")
  else
    SPECS=("adwa_playground_e2e.spec.js")
    echo "[adwa-e2e] skipping phpx browser spec (set ADWA_E2E_INCLUDE_PHPX=1 to include it)"
  fi
  ADWA_E2E_URL="$URL" \
    npx playwright test "${SPECS[@]}" --workers=1 --reporter=line --output="$PLAYWRIGHT_OUTPUT_DIR"
)

echo "[adwa-e2e] complete"
