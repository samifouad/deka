#!/usr/bin/env bash
set -euo pipefail

DEKA_NEW_BIN="${DEKA_NEW_BIN:-${HOME}/.local/bin/deka}"
DEKA_OLD_BIN="${DEKA_OLD_BIN:-${HOME}/.local/bin/dekav2}"

if [[ ! -x "${DEKA_NEW_BIN}" ]]; then
  echo "missing new binary: ${DEKA_NEW_BIN}" >&2
  exit 1
fi
if [[ ! -x "${DEKA_OLD_BIN}" ]]; then
  echo "missing old binary: ${DEKA_OLD_BIN}" >&2
  exit 1
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

NEW_RUN_CODE="$(run_once "${DEKA_NEW_BIN}" "${TMP_DIR}/run.php" "${TMP_DIR}/new.run.out" "${TMP_DIR}/new.run.err")"
OLD_RUN_CODE="$(run_once "${DEKA_OLD_BIN}" "${TMP_DIR}/run.php" "${TMP_DIR}/old.run.out" "${TMP_DIR}/old.run.err")"

if [[ "${NEW_RUN_CODE}" != "${OLD_RUN_CODE}" ]]; then
  echo "run exit code mismatch: new=${NEW_RUN_CODE} old=${OLD_RUN_CODE}" >&2
  exit 1
fi

if ! cmp -s "${TMP_DIR}/new.run.out" "${TMP_DIR}/old.run.out"; then
  echo "run stdout mismatch" >&2
  diff -u "${TMP_DIR}/old.run.out" "${TMP_DIR}/new.run.out" || true
  exit 1
fi

if ! cmp -s "${TMP_DIR}/new.run.err" "${TMP_DIR}/old.run.err"; then
  echo "run stderr mismatch" >&2
  diff -u "${TMP_DIR}/old.run.err" "${TMP_DIR}/new.run.err" || true
  exit 1
fi

start_serve() {
  local bin="$1"
  local port="$2"
  local log_out="$3"
  local log_err="$4"
  env -u PHPX_MODULE_ROOT PORT="${port}" "${bin}" serve "${TMP_DIR}/serve.php" >"${log_out}" 2>"${log_err}" &
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

NEW_PORT=18631
OLD_PORT=18632
NEW_PID="$(start_serve "${DEKA_NEW_BIN}" "${NEW_PORT}" "${TMP_DIR}/new.serve.out" "${TMP_DIR}/new.serve.err")"
OLD_PID="$(start_serve "${DEKA_OLD_BIN}" "${OLD_PORT}" "${TMP_DIR}/old.serve.out" "${TMP_DIR}/old.serve.err")"

stop_servers() {
  kill "${NEW_PID}" "${OLD_PID}" >/dev/null 2>&1 || true
  wait "${NEW_PID}" >/dev/null 2>&1 || true
  wait "${OLD_PID}" >/dev/null 2>&1 || true
}
trap 'stop_servers; cleanup' EXIT

if ! wait_http_ready "${NEW_PORT}"; then
  echo "new server did not become ready" >&2
  exit 1
fi
if ! wait_http_ready "${OLD_PORT}"; then
  echo "old server did not become ready" >&2
  exit 1
fi

curl -fsS "http://127.0.0.1:${NEW_PORT}" >"${TMP_DIR}/new.serve.body"
curl -fsS "http://127.0.0.1:${OLD_PORT}" >"${TMP_DIR}/old.serve.body"

if ! cmp -s "${TMP_DIR}/new.serve.body" "${TMP_DIR}/old.serve.body"; then
  echo "serve body mismatch" >&2
  diff -u "${TMP_DIR}/old.serve.body" "${TMP_DIR}/new.serve.body" || true
  exit 1
fi

stop_servers
trap cleanup EXIT

echo "parity ok: run + serve"
