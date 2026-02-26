#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

echo "[runtime-dev] cargo test"
(cd "${ROOT_DIR}" && cargo test)

echo "[runtime-dev] phpx runtime checks"
"${ROOT_DIR}/scripts/test-phpx.sh"

if [[ "${DEKA_RUN_DB_E2E:-0}" == "1" ]]; then
  echo "[runtime-dev] db e2e checks"
  "${ROOT_DIR}/scripts/test-phpx-db-e2e.sh"
else
  echo "[runtime-dev] skipping db e2e (set DEKA_RUN_DB_E2E=1 to enable)"
fi

echo "[runtime-dev] complete"
