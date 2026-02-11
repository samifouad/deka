#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WOSIX_JS_DIR="$ROOT_DIR/wosix/js"

if [ ! -d "$WOSIX_JS_DIR/node_modules" ]; then
  echo "[wosix-host-bridge] installing wosix/js dependencies"
  (cd "$WOSIX_JS_DIR" && npm install --no-audit --no-fund)
fi

echo "[wosix-host-bridge] running bridge smoke"
(cd "$WOSIX_JS_DIR" && npm run smoke:phpx-bridge)

echo "[wosix-host-bridge] complete"
