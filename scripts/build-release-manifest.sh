#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "${ROOT_DIR}"

cargo build --release -p cli
cargo build -p php-rs --release --target wasm32-unknown-unknown --lib --no-default-features

CLI_BIN="${ROOT_DIR}/target/release/cli"
PHP_WASM="${ROOT_DIR}/target/wasm32-unknown-unknown/release/php_rs.wasm"
MANIFEST="${ROOT_DIR}/target/release/deka-manifest.json"

if [[ ! -f "${CLI_BIN}" ]]; then
  echo "missing artifact: ${CLI_BIN}" >&2
  exit 1
fi
if [[ ! -f "${PHP_WASM}" ]]; then
  echo "missing artifact: ${PHP_WASM}" >&2
  exit 1
fi

GIT_SHA="$(git rev-parse --short=12 HEAD 2>/dev/null || echo unknown)"
BUILD_UNIX="$(date +%s)"
TARGET="$(rustc -vV | awk '/^host:/ { print $2 }')"
CLI_META="$(${CLI_BIN} --version --verbose 2>&1)"
RUNTIME_ABI="$(printf '%s\n' "${CLI_META}" | awk -F': ' '/^runtime_abi:/ { print $2 }')"

CLI_SHA="$(shasum -a 256 "${CLI_BIN}" | awk '{print $1}')"
PHP_WASM_SHA="$(shasum -a 256 "${PHP_WASM}" | awk '{print $1}')"
CLI_SIZE="$(stat -f%z "${CLI_BIN}")"
PHP_WASM_SIZE="$(stat -f%z "${PHP_WASM}")"

cat > "${MANIFEST}" <<JSON
{
  "git_sha": "${GIT_SHA}",
  "build_unix": ${BUILD_UNIX},
  "target": "${TARGET}",
  "runtime_abi": "${RUNTIME_ABI}",
  "artifacts": {
    "cli": {
      "path": "target/release/cli",
      "sha256": "${CLI_SHA}",
      "size": ${CLI_SIZE}
    },
    "php_wasm": {
      "path": "target/wasm32-unknown-unknown/release/php_rs.wasm",
      "sha256": "${PHP_WASM_SHA}",
      "size": ${PHP_WASM_SIZE}
    }
  }
}
JSON

echo "wrote manifest: ${MANIFEST}"
