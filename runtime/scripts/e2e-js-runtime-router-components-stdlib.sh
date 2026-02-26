#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEKA_BIN="${DEKA_BIN:-$ROOT/target/release/cli}"
PORT="${PORT:-8541}"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/deka-js-e2e.XXXXXX")"
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
import { Router } from 'component/router';
import { json_encode } from 'encoding/json';

$app = function($req) {
  $router_result = Router();
  return json_encode({ ok: true, runtime: 'js', router: $router_result });
};
PHPX

PORT="$PORT" "$DEKA_BIN" serve "$WORK/app/main.phpx" >"$LOG" 2>&1 &
PID=$!

for _ in {1..20}; do
  if curl -fsS "http://localhost:$PORT/" >/dev/null 2>&1; then
    break
  fi
  sleep 0.25
done

ROOT_BODY="$(curl -fsS "http://localhost:$PORT/")"

if [[ "$ROOT_BODY" != *'"ok":true'* ]] || [[ "$ROOT_BODY" != *'"runtime":"js"'* ]]; then
  echo "[e2e] unexpected / response: $ROOT_BODY"
  echo "[e2e] serve log:"
  cat "$LOG"
  exit 1
fi

if [[ "$ROOT_BODY" != *'"router":"Not Found"'* ]]; then
  echo "[e2e] expected router field to include Not Found fallback"
  echo "[e2e] actual: $ROOT_BODY"
  echo "[e2e] serve log:"
  cat "$LOG"
  exit 1
fi

echo "[e2e] js runtime router/components/stdlib passed"
