#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
install_dir="${PHPX_LSP_INSTALL_DIR:-$HOME/.local/bin}"

cd "$repo_root"

echo "Building phpx_lsp (release)..."
cargo build --release -p phpx_lsp

mkdir -p "$install_dir"
install -m 755 target/release/phpx_lsp "$install_dir/phpx_lsp"

echo "Installed phpx_lsp to $install_dir/phpx_lsp"
if ! command -v phpx_lsp >/dev/null 2>&1; then
  echo "Note: add $install_dir to your PATH to run phpx_lsp directly."
fi
