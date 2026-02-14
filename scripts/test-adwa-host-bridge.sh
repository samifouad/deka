#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ADWA_JS_DIR="$ROOT_DIR/adwa/js"

if [ ! -d "$ADWA_JS_DIR/node_modules" ]; then
  echo "[adwa-host-bridge] installing adwa/js dependencies"
  (cd "$ADWA_JS_DIR" && npm install --no-audit --no-fund)
fi

echo "[adwa-host-bridge] running bridge smoke"
(cd "$ADWA_JS_DIR" && npm run smoke:phpx-bridge)

echo "[adwa-host-bridge] complete"
