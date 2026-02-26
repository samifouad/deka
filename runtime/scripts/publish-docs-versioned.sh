#!/usr/bin/env bash
set -euo pipefail

REPO_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
PUBLISHER="${REPO_ROOT}/scripts/publish-docs.js"
WEBSITE_ROOT="${DEKA_WEBSITE_ROOT:-${REPO_ROOT}/../deka-website}"
LATEST_OUT="${DEKA_DOCS_LATEST_OUT:-${WEBSITE_ROOT}/content/docs}"
VERSIONED_OUT="${DEKA_DOCS_VERSIONED_OUT:-${WEBSITE_ROOT}/content/docs-versioned}"
TAG_PATTERN="${DEKA_DOCS_TAG_PATTERN:-v*}"
MAX_TAGS="${DEKA_DOCS_MAX_TAGS:-10}"
INCLUDE_LATEST="${DEKA_DOCS_INCLUDE_LATEST:-1}"

if ! command -v node >/dev/null 2>&1; then
  echo "node is required" >&2
  exit 1
fi
if ! command -v git >/dev/null 2>&1; then
  echo "git is required" >&2
  exit 1
fi
if [[ ! -f "${PUBLISHER}" ]]; then
  echo "publisher missing: ${PUBLISHER}" >&2
  exit 1
fi

mkdir -p "${VERSIONED_OUT}"

if [[ "${INCLUDE_LATEST}" == "1" ]]; then
  echo "[docs] publishing latest -> ${LATEST_OUT}"
  node "${PUBLISHER}" \
    --manual "${REPO_ROOT}/docs/phpx" \
    --scan "${REPO_ROOT}" \
    --map "${REPO_ROOT}/docs/docmap.json" \
    --examples "${REPO_ROOT}/examples" \
    --sections "phpx" \
    --version "latest" \
    --out "${LATEST_OUT}" \
    --force
fi

mapfile -t TAGS < <(git -C "${REPO_ROOT}" tag --list "${TAG_PATTERN}" --sort=-v:refname | head -n "${MAX_TAGS}")
versions_json='["latest"'

if [[ ${#TAGS[@]} -eq 0 ]]; then
  echo "[docs] no tags matched pattern '${TAG_PATTERN}'"
  printf '%s\n' "${versions_json}]" > "${VERSIONED_OUT}/versions.json"
  echo "[docs] wrote ${VERSIONED_OUT}/versions.json"
  exit 0
fi

for tag in "${TAGS[@]}"; do
  echo "[docs] snapshot ${tag}"
  tmp="$(mktemp -d)"

  git -C "${REPO_ROOT}" archive "${tag}" | tar -x -C "${tmp}"

  out_dir="${VERSIONED_OUT}/${tag}"
  mkdir -p "${out_dir}"

  node "${PUBLISHER}" \
    --manual "${tmp}/docs/phpx" \
    --scan "${tmp}" \
    --map "${tmp}/docs/docmap.json" \
    --examples "${tmp}/examples" \
    --sections "phpx" \
    --version "${tag}" \
    --out "${out_dir}" \
    --force

  versions_json+=" , \"${tag}\""
  rm -rf "${tmp}"
done

versions_json+=']'
printf '%s\n' "${versions_json}" > "${VERSIONED_OUT}/versions.json"

echo "[docs] wrote ${VERSIONED_OUT}/versions.json"
