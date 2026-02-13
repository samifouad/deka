#!/usr/bin/env bash
set -euo pipefail

DEKA_NEW_BIN="${DEKA_NEW_BIN:-${HOME}/.local/bin/deka}"
DEKA_OLD_BIN="${DEKA_OLD_BIN:-${HOME}/.local/bin/dekav2}"

if [[ ! -x "${DEKA_NEW_BIN}" ]]; then
  echo "missing new binary: ${DEKA_NEW_BIN}" >&2
  exit 1
fi

COMPARE_WITH_OLD=1
if [[ ! -x "${DEKA_OLD_BIN}" ]]; then
  COMPARE_WITH_OLD=0
  DEKA_OLD_BIN="${DEKA_NEW_BIN}"
  echo "old binary not found; running smoke/snapshot mode against ${DEKA_NEW_BIN}" >&2
fi

TMP_DIR="$(mktemp -d)"
cleanup() {
  rm -rf "${TMP_DIR}"
}
trap cleanup EXIT

cat > "${TMP_DIR}/run.php" <<'PHP'
<?php
$val = "runtime-parity";
echo $val;
PHP

cat > "${TMP_DIR}/serve.php" <<'PHP'
<?php
echo "serve-parity";
PHP

cat > "${TMP_DIR}/run.phpx" <<'PHPX'
---
function Hello($props): string {
  return $props.name
}
---
<!doctype html>
<html>
  <body>
    <h1><Hello name="parity" /></h1>
  </body>
</html>
PHPX

cat > "${TMP_DIR}/serve.phpx" <<'PHPX'
---
function Hello($props): string {
  return $props.name
}
---
<!doctype html>
<html>
  <body>
    <h1><Hello name="parity" /></h1>
  </body>
</html>
PHPX

cat > "${TMP_DIR}/invalid.phpx" <<'PHPX'
---
function Broken($props): string {
  return $props.name
---
<!doctype html>
<html><body>broken</body></html>
PHPX

run_once() {
  local bin="$1"
  local file="$2"
  local out_file="$3"
  local err_file="$4"
  set +e
  env -u PHPX_MODULE_ROOT "${bin}" run "${file}" >"${out_file}" 2>"${err_file}"
  local code=$?
  set -e
  echo "${code}"
}

compare_files() {
  local label="$1"
  local old_file="$2"
  local new_file="$3"
  if ! cmp -s "${new_file}" "${old_file}"; then
    echo "${label} mismatch" >&2
    diff -u "${old_file}" "${new_file}" || true
    exit 1
  fi
}

run_parity_case() {
  local label="$1"
  local file="$2"
  local new_code
  local old_code
  new_code="$(run_once "${DEKA_NEW_BIN}" "${file}" "${TMP_DIR}/new.${label}.out" "${TMP_DIR}/new.${label}.err")"
  old_code="$(run_once "${DEKA_OLD_BIN}" "${file}" "${TMP_DIR}/old.${label}.out" "${TMP_DIR}/old.${label}.err")"

  if [[ "${new_code}" != "${old_code}" ]]; then
    echo "${label} exit code mismatch: new=${new_code} old=${old_code}" >&2
    exit 1
  fi

  if [[ "${COMPARE_WITH_OLD}" == "1" ]]; then
    compare_files "${label} stdout" "${TMP_DIR}/old.${label}.out" "${TMP_DIR}/new.${label}.out"
    compare_files "${label} stderr" "${TMP_DIR}/old.${label}.err" "${TMP_DIR}/new.${label}.err"
  fi
}

run_diagnostics_case() {
  local new_code
  local old_code
  new_code="$(run_once "${DEKA_NEW_BIN}" "${TMP_DIR}/invalid.phpx" "${TMP_DIR}/new.invalid.out" "${TMP_DIR}/new.invalid.err")"
  old_code="$(run_once "${DEKA_OLD_BIN}" "${TMP_DIR}/invalid.phpx" "${TMP_DIR}/old.invalid.out" "${TMP_DIR}/old.invalid.err")"

  if [[ "${new_code}" == "0" || "${old_code}" == "0" ]]; then
    echo "diagnostics case expected non-zero exit for both binaries" >&2
    exit 1
  fi

  sed -E $'s/\x1b\[[0-9;]*m//g' "${TMP_DIR}/new.invalid.err" >"${TMP_DIR}/new.invalid.err.clean"
  sed -E $'s/\x1b\[[0-9;]*m//g' "${TMP_DIR}/old.invalid.err" >"${TMP_DIR}/old.invalid.err.clean"

  for marker in "Validation Error" "Syntax Error"; do
    if ! grep -q "${marker}" "${TMP_DIR}/new.invalid.err.clean"; then
      echo "new binary missing diagnostics marker: ${marker}" >&2
      exit 1
    fi
    if ! grep -q "${marker}" "${TMP_DIR}/old.invalid.err.clean"; then
      echo "old binary missing diagnostics marker: ${marker}" >&2
      exit 1
    fi
  done
}

start_serve() {
  local bin="$1"
  local file="$2"
  local port="$3"
  local log_out="$4"
  local log_err="$5"
  env -u PHPX_MODULE_ROOT PORT="${port}" "${bin}" serve "${file}" >"${log_out}" 2>"${log_err}" &
  echo $!
}

wait_http_ready() {
  local port="$1"
  local attempts=60
  while (( attempts > 0 )); do
    if curl -fsS "http://127.0.0.1:${port}" >/dev/null 2>&1; then
      return 0
    fi
    attempts=$((attempts - 1))
    sleep 0.1
  done
  return 1
}

fetch_with_retry() {
  local url="$1"
  local out_file="$2"
  local attempts=20
  while (( attempts > 0 )); do
    if curl -fsS "${url}" >"${out_file}" 2>/dev/null; then
      return 0
    fi
    attempts=$((attempts - 1))
    sleep 0.1
  done
  return 1
}

run_serve_case() {
  local label="$1"
  local file="$2"
  local new_port="$3"
  local old_port="$4"

  local new_pid
  local old_pid
  new_pid="$(start_serve "${DEKA_NEW_BIN}" "${file}" "${new_port}" "${TMP_DIR}/new.${label}.serve.out" "${TMP_DIR}/new.${label}.serve.err")"
  old_pid="$(start_serve "${DEKA_OLD_BIN}" "${file}" "${old_port}" "${TMP_DIR}/old.${label}.serve.out" "${TMP_DIR}/old.${label}.serve.err")"

  stop_servers() {
    kill "${new_pid}" "${old_pid}" >/dev/null 2>&1 || true
    wait "${new_pid}" >/dev/null 2>&1 || true
    wait "${old_pid}" >/dev/null 2>&1 || true
  }

  if ! wait_http_ready "${new_port}"; then
    stop_servers
    echo "new ${label} server did not become ready" >&2
    exit 1
  fi
  if ! wait_http_ready "${old_port}"; then
    stop_servers
    echo "old ${label} server did not become ready" >&2
    exit 1
  fi

  if ! fetch_with_retry "http://127.0.0.1:${new_port}" "${TMP_DIR}/new.${label}.serve.body"; then
    stop_servers
    echo "failed to fetch new ${label} response body" >&2
    exit 1
  fi
  if ! fetch_with_retry "http://127.0.0.1:${old_port}" "${TMP_DIR}/old.${label}.serve.body"; then
    stop_servers
    echo "failed to fetch old ${label} response body" >&2
    exit 1
  fi

  if [[ "${COMPARE_WITH_OLD}" == "1" ]]; then
    compare_files "${label} serve body" "${TMP_DIR}/old.${label}.serve.body" "${TMP_DIR}/new.${label}.serve.body"
  fi

  stop_servers
}

assert_snapshot_outputs() {
  compare_files "snapshot run_php stdout" <(printf "runtime-parity") "${TMP_DIR}/new.run_php.out"
  compare_files "snapshot run_phpx stdout" <(printf "<!doctype html>\n<html><body><h1>parity</h1></body></html>") "${TMP_DIR}/new.run_phpx.out"
  compare_files "snapshot serve_php body" <(printf "serve-parity") "${TMP_DIR}/new.serve_php.serve.body"
  compare_files "snapshot serve_phpx body" <(printf "<!doctype html>\n<html><body><h1>parity</h1></body></html>") "${TMP_DIR}/new.serve_phpx.serve.body"
}

run_parity_case "run_php" "${TMP_DIR}/run.php"
run_parity_case "run_phpx" "${TMP_DIR}/run.phpx"
run_diagnostics_case
run_serve_case "serve_php" "${TMP_DIR}/serve.php" 18631 18632
run_serve_case "serve_phpx" "${TMP_DIR}/serve.phpx" 18633 18634

if [[ "${COMPARE_WITH_OLD}" == "0" ]]; then
  assert_snapshot_outputs
fi

echo "parity ok: run/serve + frontmatter/jsx + diagnostics (mode=$([[ "${COMPARE_WITH_OLD}" == "1" ]] && echo compare || echo smoke))"
