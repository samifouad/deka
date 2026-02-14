#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
TARGET_DIR="$ROOT_DIR/target/wasm32-unknown-unknown/release"
PKG_DIR="$ROOT_DIR/crates/adwa-wasm/pkg"

cd "$ROOT_DIR"

if ! command -v wasm-bindgen >/dev/null 2>&1; then
  echo "wasm-bindgen not found. Install it with: cargo install wasm-bindgen-cli"
  exit 1
fi

cargo build --release -p adwa-wasm --target wasm32-unknown-unknown --features web

mkdir -p "$PKG_DIR"
wasm-bindgen \
  --target web \
  --out-dir "$PKG_DIR" \
  "$TARGET_DIR/adwa_wasm.wasm"

echo "WASM bindings written (release) to $PKG_DIR"
