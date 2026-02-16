#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEMO_DIR="$ROOT_DIR/website"
WASM_PKG_DIR="$ROOT_DIR/crates/adwa-wasm/pkg"
ADWA_JS_DIST_DIR="$ROOT_DIR/js/dist"
MVP_ROOT="${DEKA_MVP_ROOT:-$ROOT_DIR/../mvp}"
PHP_RS_WASM_PATH="$MVP_ROOT/target/wasm32-unknown-unknown/release/php_rs.wasm"
WASM_VENDOR_DIR="$DEMO_DIR/vendor/adwa_wasm"
JS_VENDOR_DIR="$DEMO_DIR/vendor/adwa_js"
EDITOR_VENDOR_DIR="$DEMO_DIR/vendor/adwa_editor"
PHP_RUNTIME_JS_SRC="$MVP_ROOT/crates/modules_php/src/modules/deka_php/php.js"
PHP_MODULES_SRC="$MVP_ROOT/php_modules"
DEKA_LOCK_SRC="$MVP_ROOT/deka.lock"
DEKA_CONFIG_SRC="$ROOT_DIR/website/project/deka.json"
INCLUDE_EDITOR_ASSETS="${ADWA_DEMO_INCLUDE_EDITOR:-0}"

"$ROOT_DIR/scripts/build-wasm.sh"

(
  cd "$MVP_ROOT"
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

# Prevent stale payload drift across rebuilds.
rm -rf "$WASM_VENDOR_DIR" "$JS_VENDOR_DIR" "$EDITOR_VENDOR_DIR" "$DEMO_DIR/vendor/php_rs"
mkdir -p "$WASM_VENDOR_DIR" "$JS_VENDOR_DIR"

cp -f "$WASM_PKG_DIR"/adwa_wasm.js "$WASM_VENDOR_DIR"/
cp -f "$WASM_PKG_DIR"/adwa_wasm_bg.wasm "$WASM_VENDOR_DIR"/
if [ -f "$WASM_PKG_DIR/adwa_wasm_bg.js" ]; then
  cp -f "$WASM_PKG_DIR"/adwa_wasm_bg.js "$WASM_VENDOR_DIR"/
fi
if [ -f "$WASM_PKG_DIR/adwa_wasm.d.ts" ]; then
  cp -f "$WASM_PKG_DIR"/adwa_wasm.d.ts "$WASM_VENDOR_DIR"/
fi

cp -f "$ADWA_JS_DIST_DIR"/*.js "$JS_VENDOR_DIR"/
cp -f "$ADWA_JS_DIST_DIR"/*.d.ts "$JS_VENDOR_DIR"/
cp -f "$PHP_RUNTIME_JS_SRC" "$JS_VENDOR_DIR/deka_php_runtime.js"

# Editor/LSP wasm assets are optional and excluded from default runtime payload.
if [ "$INCLUDE_EDITOR_ASSETS" = "1" ]; then
  mkdir -p "$EDITOR_VENDOR_DIR"
  if compgen -G "$JS_VENDOR_DIR/phpx_lsp_wasm*" >/dev/null; then
    cp -f "$JS_VENDOR_DIR"/phpx_lsp_wasm* "$EDITOR_VENDOR_DIR"/
  fi
  if [ -f "$JS_VENDOR_DIR/phpx_lsp_wasm_bg.wasm.d.ts" ]; then
    cp -f "$JS_VENDOR_DIR/phpx_lsp_wasm_bg.wasm.d.ts" "$EDITOR_VENDOR_DIR"/
  fi
fi
rm -f "$JS_VENDOR_DIR"/phpx_lsp_wasm*

wasm-bindgen \
  "$PHP_RS_WASM_PATH" \
  --target web \
  --out-dir "$JS_VENDOR_DIR" \
  --out-name php_runtime >/dev/null

PHP_RUNTIME_WASM_B64="$(base64 < "$JS_VENDOR_DIR/php_runtime_bg.wasm" | tr -d '\n')"
cat > "$JS_VENDOR_DIR/php_runtime_wasm_data.js" <<EOF2
export const phpRuntimeWasmDataUrl = "data:application/wasm;base64,${PHP_RUNTIME_WASM_B64}";
EOF2

PHP_RUNTIME_RAW_WASM_B64="$(base64 < "$PHP_RS_WASM_PATH" | tr -d '\n')"
cat > "$JS_VENDOR_DIR/php_runtime_raw_wasm_data.js" <<EOF2
export const phpRuntimeRawWasmDataUrl = "data:application/wasm;base64,${PHP_RUNTIME_RAW_WASM_B64}";
EOF2

ROOT_DIR_ENV="$ROOT_DIR" PHP_MODULES_SRC_ENV="$PHP_MODULES_SRC" DEKA_LOCK_SRC_ENV="$DEKA_LOCK_SRC" DEKA_CONFIG_SRC_ENV="$DEKA_CONFIG_SRC" JS_VENDOR_DIR_ENV="$JS_VENDOR_DIR" node <<'EOF2'
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
EOF2

echo "Demo assets built. Serve $ROOT_DIR/website"
