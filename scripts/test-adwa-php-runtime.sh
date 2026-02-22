#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEFAULT_ADWA_JS_DIR="$ROOT_DIR/../adwa/js"
ADWA_JS_DIR="${ADWA_JS_DIR:-$DEFAULT_ADWA_JS_DIR}"
if [ ! -d "$ADWA_JS_DIR" ]; then
  ADWA_JS_DIR="$ROOT_DIR/adwa/js"
fi

if [ ! -d "$ADWA_JS_DIR" ]; then
  echo "[adwa-runtime] could not locate adwa/js (set ADWA_JS_DIR)" >&2
  exit 1
fi

if [ ! -d "$ADWA_JS_DIR/node_modules" ]; then
  echo "[adwa-runtime] installing adwa/js dependencies"
  (cd "$ADWA_JS_DIR" && npm install --no-audit --no-fund)
fi

echo "[adwa-runtime] running runtime adapter smoke"
(cd "$ADWA_JS_DIR" && npm run smoke:phpx-runtime-adapter)

echo "[adwa-runtime] running capability integration smoke"
(cd "$ADWA_JS_DIR" && npm run smoke:phpx-runtime-capability)

echo "[adwa-runtime] complete"
