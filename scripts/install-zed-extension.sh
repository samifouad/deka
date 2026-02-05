#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ext_src="$repo_root/extensions/phpx"

zed_dir="${ZED_CONFIG_DIR:-$HOME/.config/zed}"
ext_dir="$zed_dir/extensions"
ext_dest="$ext_dir/phpx"

mkdir -p "$ext_dir"

if [ -L "$ext_dest" ]; then
  existing="$(readlink "$ext_dest")"
  if [ "$existing" = "$ext_src" ]; then
    echo "PHPX Zed extension already linked: $ext_dest"
    exit 0
  fi
  echo "Existing PHPX link points to $existing"
  echo "Remove $ext_dest and re-run to link to $ext_src"
  exit 1
fi

if [ -e "$ext_dest" ]; then
  echo "Path exists and is not a symlink: $ext_dest"
  echo "Move it aside or delete it, then re-run."
  exit 1
fi

ln -s "$ext_src" "$ext_dest"
echo "Linked PHPX Zed extension to $ext_dest"
echo "Next: configure phpx-lsp in $zed_dir/settings.json"
