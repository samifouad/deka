#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEKA_BIN="${DEKA_BIN:-${ROOT_DIR}/target/release/cli}"
DEKA_DOCS_OUT="${DEKA_DOCS_OUT:-${ROOT_DIR}/../deka-website/content/docs}"

if [[ ! -x "$DEKA_BIN" ]]; then
  echo "deka binary not found at $DEKA_BIN" >&2
  echo "Build it first: cargo build -p cli --release" >&2
  exit 1
fi

publish_docs() {
  if [[ "${DEKA_TEST_SKIP_DOCS:-0}" == "1" ]]; then
    echo "[docs] skipped (DEKA_TEST_SKIP_DOCS=1)"
    return
  fi
  if [[ ! -f "${ROOT_DIR}/scripts/publish-docs.js" ]]; then
    return
  fi
  if ! command -v node >/dev/null 2>&1; then
    echo "[docs] node is required to publish docs during tests" >&2
    exit 1
  fi
  echo "[docs] publishing docs -> ${DEKA_DOCS_OUT}"
  node "${ROOT_DIR}/scripts/publish-docs.js" --scan "${ROOT_DIR}" --out "${DEKA_DOCS_OUT}"
}

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

publish_docs

printf "\nAll phpx checks passed.\n"
