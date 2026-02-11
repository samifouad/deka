#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WOSIX_DIR="$ROOT_DIR/wosix"
PORT="${PORT:-5173}"
WOSIX_JS_DIR="$WOSIX_DIR/js"

if [ ! -d "$WOSIX_JS_DIR/node_modules" ]; then
  echo "[wosix-playground] installing wosix/js dependencies"
  (cd "$WOSIX_JS_DIR" && npm install --no-audit --no-fund)
fi

echo "[wosix-playground] building browser demo assets"
"$WOSIX_DIR/scripts/build-demo.sh"

if [[ "${1:-}" == "--build-only" ]]; then
  echo "[wosix-playground] build-only mode complete"
  exit 0
fi

if command -v python3 >/dev/null 2>&1; then
  echo "[wosix-playground] serving $WOSIX_DIR/examples/browser at http://localhost:${PORT}"
  exec python3 -m http.server "$PORT" --directory "$WOSIX_DIR/examples/browser"
fi

echo "[wosix-playground] python3 is required to serve the demo (or use your own static server)"
exit 1
