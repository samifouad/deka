#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
ext_src="$repo_root/extensions/phpx"

if [ -n "${ZED_CONFIG_DIR:-}" ]; then
  zed_dir="$ZED_CONFIG_DIR"
elif [ "$(uname -s)" = "Darwin" ]; then
  zed_dir="$HOME/Library/Application Support/Zed"
else
  zed_dir="${XDG_CONFIG_HOME:-$HOME/.config}/zed"
fi
ext_dir="$zed_dir/extensions/work"
ext_dest="$ext_dir/phpx"
old_dest="$zed_dir/extensions/phpx"
use_symlink="${ZED_PHPX_SYMLINK:-0}"

mkdir -p "$ext_dir"

if [ -L "$old_dest" ]; then
  old_target="$(readlink "$old_dest")"
  if [ "$old_target" = "$ext_src" ]; then
    rm "$old_dest"
  fi
fi

if [ -L "$ext_dest" ]; then
  existing="$(readlink "$ext_dest")"
  if [ "$use_symlink" = "1" ]; then
    if [ "$existing" = "$ext_src" ]; then
      echo "PHPX Zed extension already linked: $ext_dest"
      exit 0
    fi
    echo "Existing PHPX link points to $existing"
    echo "Remove $ext_dest and re-run to link to $ext_src"
    exit 1
  fi
  rm "$ext_dest"
fi

if [ -e "$ext_dest" ]; then
  echo "Path exists and is not a symlink: $ext_dest"
  if [ "$use_symlink" = "1" ]; then
    echo "Move it aside or delete it, then re-run."
    exit 1
  fi
  rm -rf "$ext_dest"
fi

if [ "$use_symlink" = "1" ]; then
  ln -s "$ext_src" "$ext_dest"
  echo "Linked PHPX Zed extension to $ext_dest"
else
  mkdir -p "$ext_dest"
  rsync -a --delete "$ext_src/" "$ext_dest/"
  if [ -f "$ext_dest/extension.toml" ]; then
    repo_uri="file://$repo_root"
    perl -0pi -e 's#(\[grammars\.phpx_only\][^\[]*?repository = ")[^"]*(")#$1'"$repo_uri"'$2#s' \
      "$ext_dest/extension.toml"
    perl -0pi -e 's#(\[grammars\.phpx_only\][^\[]*?rev = ")[^"]*(")#$1HEAD$2#s' \
      "$ext_dest/extension.toml"
  fi
  echo "Copied PHPX Zed extension to $ext_dest"
fi
echo "Next: configure phpx-lsp in $zed_dir/settings.json"
