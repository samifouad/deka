#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ext_dir="$repo_root/extensions/vscode-phpx"

if ! command -v code >/dev/null 2>&1; then
  echo "VS Code 'code' CLI not found. Install it via VS Code:"
  echo "  Cmd+Shift+P -> Shell Command: Install 'code' command in PATH"
  exit 1
fi

if ! command -v vsce >/dev/null 2>&1; then
  echo "vsce not found; installing with npm..."
  npm install -g @vscode/vsce
fi

pushd "$ext_dir" >/dev/null
npm install
vsce package
vsix=$(ls -t *.vsix | head -n 1)
code --install-extension "$vsix" --force
popd >/dev/null

echo "Installed PHPX VS Code extension: $vsix"
