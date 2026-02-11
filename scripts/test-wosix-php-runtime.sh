#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT_DIR"

echo "[wosix-smoke] building php-rs wasm artifact (release)"
cargo build -p php-rs --release --target wasm32-unknown-unknown --lib --no-default-features

WASM_PATH="$ROOT_DIR/target/wasm32-unknown-unknown/release/php_rs.wasm"
if [ ! -f "$WASM_PATH" ]; then
  echo "[wosix-smoke] missing wasm artifact at $WASM_PATH"
  exit 1
fi
echo "[wosix-smoke] wasm artifact ready: $WASM_PATH"

echo "[wosix-smoke] validating runtime host profile gating tests"
cargo test -p php-rs --release runtime::context::tests::host_capabilities_for_wosix_limits_db_and_env -- --nocapture
cargo test -p phpx_lsp --release target_capability_diagnostics_block_db_modules_for_wosix -- --nocapture

echo "[wosix-smoke] running focused PHPX runtime smoke cases"
node tests/phpx/testrunner.js tests/phpx/objects/literals_basic.phpx
node tests/phpx/testrunner.js tests/phpx/modules/import_export.phpx
node tests/phpx/testrunner.js tests/phpx/jsx/render.phpx
node tests/phpx/testrunner.js tests/phpx/foundation/fs_bytes_roundtrip.phpx

echo "[wosix-smoke] complete"
