#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEKA_BIN="${DEKA_BIN:-$ROOT/target/release/cli}"
PORT="${PORT:-8542}"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/deka-fallback-e2e.XXXXXX")"
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
type Alias = string;

$app = function($req) {
  return "fallback-ok";
};
PHPX

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

curl -fsS "http://localhost:$PORT/" >/dev/null
curl -fsS "http://localhost:$PORT/" >/dev/null

COUNT="$(grep -c "\\[phpx-js\\] subset transpile fallback:" "$LOG" || true)"
if [[ "$COUNT" -ne 1 ]]; then
  echo "[e2e] expected exactly one fallback diagnostic, got $COUNT"
  echo "[e2e] log:"
  cat "$LOG"
  exit 1
fi

echo "[e2e] js runtime fallback diagnostics passed"
