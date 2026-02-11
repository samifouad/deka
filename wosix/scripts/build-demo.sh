#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEMO_DIR="$ROOT_DIR/examples/browser"
WASM_PKG_DIR="$ROOT_DIR/crates/wosix-wasm/pkg"
WOSIX_JS_DIST_DIR="$ROOT_DIR/js/dist"
PHP_RS_WASM_PATH="$ROOT_DIR/../target/wasm32-unknown-unknown/release/php_rs.wasm"
WASM_VENDOR_DIR="$DEMO_DIR/vendor/wosix_wasm"
JS_VENDOR_DIR="$DEMO_DIR/vendor/wosix_js"

"$ROOT_DIR/scripts/build-wasm.sh"

(
  cd "$ROOT_DIR/.."
  cargo build -p php-rs --release --target wasm32-unknown-unknown --lib --no-default-features >/dev/null
)

if ! command -v wasm-bindgen >/dev/null 2>&1; then
  echo "wasm-bindgen not found. Install it with: cargo install wasm-bindgen-cli"
  exit 1
fi

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

wasm-bindgen \
  "$PHP_RS_WASM_PATH" \
  --target web \
  --out-dir "$JS_VENDOR_DIR" \
  --out-name php_runtime >/dev/null

echo "Demo assets built. Serve $ROOT_DIR/examples/browser"
