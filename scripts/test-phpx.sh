#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEKA_BIN="${DEKA_BIN:-${ROOT_DIR}/target/release/cli}"

if [[ ! -x "$DEKA_BIN" ]]; then
  echo "deka binary not found at $DEKA_BIN" >&2
  echo "Build it first: cargo build -p cli --release" >&2
  exit 1
fi

run_case() {
  local label="$1"
  local cmd="$2"
  local expect="$3"

  echo "[$label]"
  local output
  if output=$(eval "$cmd" 2>&1); then
    if [[ -n "$expect" && "$output" != *"$expect"* ]]; then
      echo "Expected output to include: $expect" >&2
      echo "Got: $output" >&2
      exit 1
    fi
    echo "ok"
  else
    if [[ -n "$expect" && "$output" != *"$expect"* ]]; then
      echo "Expected error to include: $expect" >&2
      echo "Got: $output" >&2
      exit 1
    fi
    echo "ok"
  fi
}

run_case "modules" "$DEKA_BIN run examples/php/modules/index.php" "value: something"
run_case "modules-import" "$DEKA_BIN run examples/php/modules-import/index.php" "value: something"
run_case "modules-cycle" "$DEKA_BIN run examples/php/modules-cycle/index.php" "Cyclic phpx import detected: a"
run_case "modules-missing" "$DEKA_BIN run examples/php/modules-missing/index.php" "Missing export 'missing' in 'bar' (imported by 'foo')."
run_case "modules-types" "$DEKA_BIN run examples/php/modules-types/index.php" "hi deka"
run_case "modules-reexport" "$DEKA_BIN run examples/php/modules-reexport/index.php" "7"

printf "\nAll phpx checks passed.\n"
