#!/usr/bin/env bash
set -euo pipefail

PHP_BIN=${PHP_BIN:-php}
PHP_ROUTER_BIN=${PHP_NATIVE_BIN:-target/release/php}

if [ ! -x "$PHP_BIN" ]; then
  echo "official PHP binary not found at $PHP_BIN" >&2
  exit 1
fi

if [ ! -x "$PHP_ROUTER_BIN" ]; then
  PHP_ROUTER_BIN=target/debug/php
fi

if [ ! -x "$PHP_ROUTER_BIN" ]; then
  echo "php router native PHP binary not found; run \"cargo build --bin php\" or set PHP_NATIVE_BIN" >&2
  exit 1
fi

SCRIPT=${1:-tests/php/test-001/hello.php}
shift || true

if [ ! -f "$SCRIPT" ]; then
  echo "script not found: $SCRIPT" >&2
  exit 1
fi

OUTDIR=$(mktemp -d)
trap 'rm -rf "$OUTDIR"' EXIT

run_script() {
  local label=$1
  local bin=$2
  local out="$OUTDIR/${label}.txt"
  "$bin" "$SCRIPT" "$@" >"$out" 2>&1
  echo "$out"
}

official_out=$(run_script official "$PHP_BIN" "$@")
php_router_out=$(run_script php-router "$PHP_ROUTER_BIN" "$@")

echo "=== Official PHP ($PHP_BIN) ==="
cat "$official_out"
echo
echo "=== php-router executable ($PHP_ROUTER_BIN) ==="
cat "$php_router_out"
echo

if diff -u "$official_out" "$php_router_out"; then
  echo "Outputs match." >&2
else
  echo "Outputs differ: compare $official_out vs $php_router_out" >&2
  exit 1
fi
