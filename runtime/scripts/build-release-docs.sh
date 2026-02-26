#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WEBSITE_ROOT="${DEKA_WEBSITE_ROOT:-${REPO_ROOT}/../deka-website}"
DOCS_OUT="${DEKA_DOCS_OUT:-${WEBSITE_ROOT}/content/docs}"
SECTIONS="${DEKA_DOCS_SECTIONS:-phpx}"
VERSION="${DEKA_DOCS_VERSION:-latest}"

if ! command -v node >/dev/null 2>&1; then
  echo "node is required" >&2
  exit 1
fi

if [[ ! -f "${REPO_ROOT}/scripts/publish-docs.js" ]]; then
  echo "missing scripts/publish-docs.js" >&2
  exit 1
fi

echo "[build] cargo build --release -p cli"
(cd "${REPO_ROOT}" && cargo build --release -p cli)

echo "[docs] publish -> ${DOCS_OUT}"
node "${REPO_ROOT}/scripts/publish-docs.js" \
  --manual "${REPO_ROOT}/docs/phpx" \
  --scan "${REPO_ROOT}" \
  --map "${REPO_ROOT}/docs/docmap.json" \
  --examples "${REPO_ROOT}/examples" \
  --sections "${SECTIONS}" \
  --version "${VERSION}" \
  --out "${DOCS_OUT}" \
  --force

if [[ -d "${WEBSITE_ROOT}" ]] && command -v bun >/dev/null 2>&1; then
  echo "[docs] bundle runtime docs"
  (cd "${WEBSITE_ROOT}" && bun scripts/bundle-runtime.ts --source content/docs --lang en)
else
  echo "[docs] skipped runtime bundle (set DEKA_WEBSITE_ROOT and install bun to enable)"
fi

echo "[ok] release build + docs publish complete"
