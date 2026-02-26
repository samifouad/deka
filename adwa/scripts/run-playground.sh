#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PORT="${PORT:-5173}"
ADWA_JS_DIR="$ROOT_DIR/js"

if [ ! -d "$ADWA_JS_DIR/node_modules" ]; then
  echo "[adwa-playground] installing adwa/js dependencies"
  (cd "$ADWA_JS_DIR" && npm install --no-audit --no-fund)
fi

echo "[adwa-playground] building browser demo assets"
"$ROOT_DIR/scripts/build-demo.sh"

if [[ "${1:-}" == "--build-only" ]]; then
  echo "[adwa-playground] build-only mode complete"
  exit 0
fi

if command -v python3 >/dev/null 2>&1; then
  echo "[adwa-playground] serving $ROOT_DIR/website at http://localhost:${PORT}"
  exec python3 -m http.server "$PORT" --directory "$ROOT_DIR/website"
fi

echo "[adwa-playground] python3 is required to serve the demo (or use your own static server)"
exit 1
