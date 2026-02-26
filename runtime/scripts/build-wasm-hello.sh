#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
crate="$root/examples/wasm_hello_rust"

out_dir="$root/target/wasm32-unknown-unknown/release"

cargo build --release --target wasm32-unknown-unknown --manifest-path "$crate/Cargo.toml"

cp "$out_dir/deka_wasm_hello.wasm" \
  "$root/php_modules/@user/hello/module.wasm"

echo "Wrote php_modules/@user/hello/module.wasm"
