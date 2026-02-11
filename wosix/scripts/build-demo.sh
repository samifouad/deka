#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEMO_DIR="$ROOT_DIR/examples/browser"
WASM_PKG_DIR="$ROOT_DIR/crates/wosix-wasm/pkg"
WOSIX_JS_DIST_DIR="$ROOT_DIR/js/dist"
WASM_VENDOR_DIR="$DEMO_DIR/vendor/wosix_wasm"
JS_VENDOR_DIR="$DEMO_DIR/vendor/wosix_js"

"$ROOT_DIR/scripts/build-wasm.sh"

cd "$ROOT_DIR/js"
if [ ! -d node_modules ]; then
  echo "node_modules missing; run 'npm install' in $ROOT_DIR/js first."
  exit 1
fi

npm run build

mkdir -p "$WASM_VENDOR_DIR" "$JS_VENDOR_DIR"
cp -f "$WASM_PKG_DIR"/wosix_wasm.js "$WASM_VENDOR_DIR"/
cp -f "$WASM_PKG_DIR"/wosix_wasm_bg.wasm "$WASM_VENDOR_DIR"/
if [ -f "$WASM_PKG_DIR/wosix_wasm_bg.js" ]; then
  cp -f "$WASM_PKG_DIR"/wosix_wasm_bg.js "$WASM_VENDOR_DIR"/
fi
if [ -f "$WASM_PKG_DIR/wosix_wasm.d.ts" ]; then
  cp -f "$WASM_PKG_DIR"/wosix_wasm.d.ts "$WASM_VENDOR_DIR"/
fi

cp -f "$WOSIX_JS_DIST_DIR"/*.js "$JS_VENDOR_DIR"/
cp -f "$WOSIX_JS_DIST_DIR"/*.d.ts "$JS_VENDOR_DIR"/

echo "Demo assets built. Serve $ROOT_DIR/examples/browser"
