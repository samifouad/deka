#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEMO_DIR="$ROOT_DIR/examples/browser"
WASM_PKG_DIR="$ROOT_DIR/crates/wosix-wasm/pkg"
WOSIX_JS_DIST_DIR="$ROOT_DIR/js/dist"
PHP_RS_WASM_PATH="$ROOT_DIR/../target/wasm32-unknown-unknown/release/php_rs.wasm"
PHPX_LSP_WASM_PATH="$ROOT_DIR/../target/wasm32-unknown-unknown/release/phpx_lsp_wasm.wasm"
WASM_VENDOR_DIR="$DEMO_DIR/vendor/wosix_wasm"
JS_VENDOR_DIR="$DEMO_DIR/vendor/wosix_js"
PHP_RUNTIME_JS_SRC="$ROOT_DIR/../crates/modules_php/src/modules/deka_php/php.js"
PHP_MODULES_SRC="$ROOT_DIR/../php_modules"
DEKA_LOCK_SRC="$ROOT_DIR/../deka.lock"
DEKA_CONFIG_SRC="$ROOT_DIR/deka.json"

"$ROOT_DIR/scripts/build-wasm.sh"

(
  cd "$ROOT_DIR/.."
  cargo build -p php-rs --release --target wasm32-unknown-unknown --lib --no-default-features >/dev/null
  cargo build -p phpx_lsp_wasm --release --target wasm32-unknown-unknown --lib >/dev/null
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
cp -f "$PHP_RUNTIME_JS_SRC" "$JS_VENDOR_DIR/deka_php_runtime.js"

wasm-bindgen \
  "$PHP_RS_WASM_PATH" \
  --target web \
  --out-dir "$JS_VENDOR_DIR" \
  --out-name php_runtime >/dev/null

wasm-bindgen \
  "$PHPX_LSP_WASM_PATH" \
  --target web \
  --out-dir "$JS_VENDOR_DIR" \
  --out-name phpx_lsp_wasm >/dev/null

PHP_RUNTIME_WASM_B64="$(base64 < "$JS_VENDOR_DIR/php_runtime_bg.wasm" | tr -d '\n')"
cat > "$JS_VENDOR_DIR/php_runtime_wasm_data.js" <<EOF
export const phpRuntimeWasmDataUrl = "data:application/wasm;base64,${PHP_RUNTIME_WASM_B64}";
EOF

PHP_RUNTIME_RAW_WASM_B64="$(base64 < "$PHP_RS_WASM_PATH" | tr -d '\n')"
cat > "$JS_VENDOR_DIR/php_runtime_raw_wasm_data.js" <<EOF
export const phpRuntimeRawWasmDataUrl = "data:application/wasm;base64,${PHP_RUNTIME_RAW_WASM_B64}";
EOF

ROOT_DIR_ENV="$ROOT_DIR" PHP_MODULES_SRC_ENV="$PHP_MODULES_SRC" DEKA_LOCK_SRC_ENV="$DEKA_LOCK_SRC" DEKA_CONFIG_SRC_ENV="$DEKA_CONFIG_SRC" JS_VENDOR_DIR_ENV="$JS_VENDOR_DIR" node <<'EOF'
const fs = require("node:fs");
const path = require("node:path");

const phpModulesRoot = process.env.PHP_MODULES_SRC_ENV;
const dekaLockPath = process.env.DEKA_LOCK_SRC_ENV;
const dekaConfigPath = process.env.DEKA_CONFIG_SRC_ENV;
const outPath = path.join(process.env.JS_VENDOR_DIR_ENV, "php_project_bundle.js");

const files = {};

function walk(dir) {
  const entries = fs.readdirSync(dir, { withFileTypes: true });
  for (const entry of entries) {
    const full = path.join(dir, entry.name);
    if (entry.isDirectory()) {
      if (entry.name === ".cache") continue;
      walk(full);
      continue;
    }
    if (entry.name === ".DS_Store") continue;
    const rel = path.relative(phpModulesRoot, full).replace(/\\/g, "/");
    if (rel.startsWith(".cache/")) continue;
    const target = `/php_modules/${rel}`;
    files[target] = fs.readFileSync(full, "utf8");
  }
}

if (fs.existsSync(phpModulesRoot)) {
  walk(phpModulesRoot);
}
if (fs.existsSync(dekaLockPath)) {
  files["/deka.lock"] = fs.readFileSync(dekaLockPath, "utf8");
}
if (fs.existsSync(dekaConfigPath)) {
  files["/deka.json"] = fs.readFileSync(dekaConfigPath, "utf8");
}

const content =
  `export const bundledProjectFiles = ${JSON.stringify(files)};\n` +
  `export const bundledProjectVersion = "php-bundle-v1";\n`;
fs.writeFileSync(outPath, content, "utf8");
EOF

echo "Demo assets built. Serve $ROOT_DIR/examples/browser"
