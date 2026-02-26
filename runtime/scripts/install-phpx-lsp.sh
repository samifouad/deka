#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
install_dir="${PHPX_LSP_INSTALL_DIR:-$HOME/.local/bin}"
project_root="$(pwd)"
setup_zed=0
setup_vscode=0
print_neovim=0

while [[ $# -gt 0 ]]; do
  case "$1" in
    --setup-zed)
      setup_zed=1
      shift
      ;;
    --setup-vscode)
      setup_vscode=1
      shift
      ;;
    --print-neovim)
      print_neovim=1
      shift
      ;;
    --project-root)
      project_root="${2:-}"
      if [[ -z "$project_root" ]]; then
        echo "error: --project-root requires a path argument" >&2
        exit 1
      fi
      shift 2
      ;;
    *)
      echo "error: unknown argument: $1" >&2
      echo "usage: $0 [--setup-zed] [--setup-vscode] [--print-neovim] [--project-root <path>]" >&2
      exit 1
      ;;
  esac
done

cd "$repo_root"

echo "Building phpx_lsp (release)..."
cargo build --release -p phpx_lsp

mkdir -p "$install_dir"
install -m 755 target/release/phpx_lsp "$install_dir/phpx_lsp"

echo "Installed phpx_lsp to $install_dir/phpx_lsp"
if ! command -v phpx_lsp >/dev/null 2>&1; then
  echo "Note: add $install_dir to your PATH to run phpx_lsp directly."
fi

if [[ "$setup_zed" -eq 1 ]]; then
  zed_settings_dir="$project_root/.zed"
  zed_settings_file="$zed_settings_dir/settings.json"
  mkdir -p "$zed_settings_dir"
  if [[ -f "$zed_settings_file" ]]; then
    echo "Skipped Zed config (already exists): $zed_settings_file"
    echo "Add this block manually:"
  else
    cat >"$zed_settings_file" <<EOF
{
  "lsp": {
    "phpx-lsp": {
      "binary": {
        "path": "$install_dir/phpx_lsp"
      }
    }
  }
}
EOF
    echo "Wrote Zed project settings: $zed_settings_file"
  fi
  cat <<EOF
{
  "lsp": {
    "phpx-lsp": {
      "binary": {
        "path": "$install_dir/phpx_lsp"
      }
    }
  }
}
EOF
fi

if [[ "$setup_vscode" -eq 1 ]]; then
  vscode_settings_dir="$project_root/.vscode"
  vscode_settings_file="$vscode_settings_dir/settings.json"
  mkdir -p "$vscode_settings_dir"
  if [[ -f "$vscode_settings_file" ]]; then
    echo "Skipped VS Code config (already exists): $vscode_settings_file"
    echo "Add these keys manually:"
  else
    cat >"$vscode_settings_file" <<EOF
{
  "phpx.lsp.path": "$install_dir/phpx_lsp",
  "phpx.lsp.args": []
}
EOF
    echo "Wrote VS Code project settings: $vscode_settings_file"
  fi
  cat <<EOF
{
  "phpx.lsp.path": "$install_dir/phpx_lsp",
  "phpx.lsp.args": []
}
EOF
fi

if [[ "$print_neovim" -eq 1 ]]; then
  cat <<EOF
Neovim (nvim-lspconfig) snippet:
require('lspconfig').phpx_lsp.setup({
  cmd = { '$install_dir/phpx_lsp' },
  filetypes = { 'phpx' },
  root_dir = require('lspconfig.util').root_pattern('deka.lock', 'php_modules', '.git'),
})
EOF
fi
