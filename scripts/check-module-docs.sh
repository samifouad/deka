#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="${DEKA_DOCS_CHECK_OUT:-${REPO_ROOT}/target/.docs-check}"

if ! command -v node >/dev/null 2>&1; then
  echo "node is required" >&2
  exit 1
fi

if [[ ! -f "${REPO_ROOT}/scripts/publish-docs.js" ]]; then
  echo "missing scripts/publish-docs.js" >&2
  exit 1
fi

rm -rf "${OUT_DIR}"
mkdir -p "${OUT_DIR}"

echo "[docs-check] validating php_modules doccomment coverage"
node "${REPO_ROOT}/scripts/publish-docs.js" \
  --manual "${REPO_ROOT}/docs/phpx" \
  --scan "${REPO_ROOT}" \
  --map "${REPO_ROOT}/docs/docmap.json" \
  --examples "${REPO_ROOT}/examples" \
  --sections "phpx" \
  --version "latest" \
  --out "${OUT_DIR}" \
  --dry-run

echo "[docs-check] ok"
