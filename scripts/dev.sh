#!/usr/bin/env bash
set -euo pipefail
ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
"$ROOT_DIR/scripts/build-demo.sh"

SIDE_HOST="${PHPX_LSP_HOST:-127.0.0.1}"
SIDE_PORT="${PHPX_LSP_PORT:-8531}"
USE_LSP_SIDECAR="${PHPX_LSP_SIDECAR:-0}"
DEV_PORT="${ADWA_DEV_PORT:-8530}"

SIDE_PID=""
if [ "$USE_LSP_SIDECAR" = "1" ]; then
  ADWA_ROOT="$ROOT_DIR" node "$ROOT_DIR/scripts/lsp-sidecar.mjs" &
  SIDE_PID=$!
fi

cleanup() {
  if [ -n "$SIDE_PID" ]; then
    kill "$SIDE_PID" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT INT TERM

if [ "$USE_LSP_SIDECAR" = "1" ]; then
  for _ in $(seq 1 50); do
    if curl -fsS "http://${SIDE_HOST}:${SIDE_PORT}/ping" >/dev/null 2>&1; then
      break
    fi
    sleep 0.1
  done
fi

if ! command -v python3 >/dev/null 2>&1; then
  echo "python3 is required for adwa dev server"
  exit 1
fi

echo "[adwa-dev] serving $ROOT_DIR/examples/browser at http://localhost:${DEV_PORT}"
exec python3 -m http.server "$DEV_PORT" --directory "$ROOT_DIR/examples/browser"
