#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_DIR="$ROOT_DIR/target/wasm32-unknown-unknown"
PKG_DIR="$ROOT_DIR/crates/wosix-wasm/pkg"

cd "$ROOT_DIR"

if ! command -v wasm-bindgen >/dev/null 2>&1; then
  echo "wasm-bindgen not found. Install it with: cargo install wasm-bindgen-cli"
  exit 1
fi

cargo build -p wosix-wasm --release --target wasm32-unknown-unknown --features web

mkdir -p "$PKG_DIR"
wasm-bindgen \
  --target web \
  --out-dir "$PKG_DIR" \
  "$TARGET_DIR/release/wosix_wasm.wasm"

echo "WASM bindings written to $PKG_DIR"
