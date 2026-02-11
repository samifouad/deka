#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
E2E_DIR="${ROOT_DIR}/tests/manual/hmr_e2e"
PORT="${WOSIX_E2E_PORT:-5173}"
HOST="${WOSIX_E2E_HOST:-127.0.0.1}"
URL="http://${HOST}:${PORT}"
LOG_FILE="${WOSIX_E2E_LOG:-/tmp/deka-wosix-e2e.log}"
PLAYWRIGHT_OUTPUT_DIR="${PLAYWRIGHT_OUTPUT_DIR:-/tmp/deka-wosix-playwright}"
PLAYWRIGHT_INSTALL_TIMEOUT_MS="${PLAYWRIGHT_INSTALL_TIMEOUT_MS:-120000}"
PLAYWRIGHT_DOWNLOAD_HOST="${PLAYWRIGHT_DOWNLOAD_HOST:-https://playwright.download.prss.microsoft.com}"

echo "[wosix-e2e] building demo assets"
PORT="$PORT" "${ROOT_DIR}/scripts/run-wosix-playground.sh" --build-only

echo "[wosix-e2e] serving browser demo at $URL"
(
  cd "${ROOT_DIR}/wosix/examples/browser"
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
  echo "[wosix-e2e] server did not start at $URL"
  echo "[wosix-e2e] recent logs:"
  tail -n 80 "$LOG_FILE" || true
  exit 1
fi

echo "[wosix-e2e] ensuring playwright chromium is installed"
(
  cd "$E2E_DIR"
  if [[ ! -d node_modules/@playwright/test ]]; then
    npm install --package-lock=false --no-fund --no-audit >/dev/null
  fi
  PLAYWRIGHT_DOWNLOAD_CONNECTION_TIMEOUT="$PLAYWRIGHT_INSTALL_TIMEOUT_MS" \
  PLAYWRIGHT_DOWNLOAD_HOST="$PLAYWRIGHT_DOWNLOAD_HOST" \
    npx playwright install chromium >/dev/null
)

echo "[wosix-e2e] running playwright browser check"
(
  cd "$E2E_DIR"
  WOSIX_E2E_URL="$URL" \
    npx playwright test wosix_playground_e2e.spec.js --workers=1 --reporter=line --output="$PLAYWRIGHT_OUTPUT_DIR"
)

echo "[wosix-e2e] complete"
