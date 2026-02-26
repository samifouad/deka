#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEKA_BIN="${DEKA_BIN:-$ROOT/target/release/cli}"
PORT="${PORT:-8543}"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/deka-mvp2-smoke.XXXXXX")"
LOG="$WORK/serve.log"

cleanup() {
  if [[ -n "${PID:-}" ]] && kill -0 "$PID" 2>/dev/null; then
    kill "$PID" 2>/dev/null || true
    wait "$PID" 2>/dev/null || true
  fi
  rm -rf "$WORK"
}
trap cleanup EXIT

"$DEKA_BIN" init "$WORK" >/dev/null

cat > "$WORK/app/main.phpx" <<'PHPX'
import { json_encode } from 'encoding/json';

$app = function($req) {
  return json_encode({ mode: 'serve', ok: true });
};

echo json_encode({ mode: 'run', ok: true });
PHPX

RUN_OUT="$("$DEKA_BIN" run "$WORK/app/main.phpx" 2>/dev/null || true)"
if [[ "$RUN_OUT" != *'{"mode":"run","ok":true}'* ]]; then
  echo "[e2e] run output mismatch"
  echo "$RUN_OUT"
  exit 1
fi

(
  cd "$WORK"
  PORT="$PORT" "$DEKA_BIN" serve app/main.phpx >"$LOG" 2>&1
) &
PID=$!

for _ in {1..30}; do
  if curl -fsS "http://localhost:$PORT/" >/dev/null 2>&1; then
    break
  fi
  sleep 0.2
done

SERVE_OUT="$(curl -fsS "http://localhost:$PORT/")"
if [[ "$SERVE_OUT" != '{"mode":"serve","ok":true}' ]]; then
  echo "[e2e] serve output mismatch"
  echo "$SERVE_OUT"
  exit 1
fi

(
  cd "$WORK"
  "$DEKA_BIN" build app/main.phpx --bundle --out dist/main.bundle.js >/dev/null
)
if [[ ! -s "$WORK/dist/main.bundle.js" ]]; then
  echo "[e2e] expected dist/main.bundle.js"
  exit 1
fi

echo "[e2e] mvp2 run/serve/build smoke passed"
