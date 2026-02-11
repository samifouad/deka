#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WOSIX_JS_DIR="$ROOT_DIR/wosix/js"

if [ ! -d "$WOSIX_JS_DIR/node_modules" ]; then
  echo "[wosix-module-graph] installing wosix/js dependencies"
  (cd "$WOSIX_JS_DIR" && npm install --no-audit --no-fund)
fi

echo "[wosix-module-graph] running module graph smoke"
(cd "$WOSIX_JS_DIR" && npm run smoke:module-graph)

echo "[wosix-module-graph] complete"
