#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
DEKA_BIN="${DEKA_BIN:-$ROOT/target/release/cli}"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/deka-sec-e2e.XXXXXX")"
LOG_DEV="$WORK/dev.log"
LOG_DENY="$WORK/deny.log"

cleanup() {
  if [[ -n "${DEV_PID:-}" ]] && kill -0 "$DEV_PID" 2>/dev/null; then
    kill "$DEV_PID" 2>/dev/null || true
    wait "$DEV_PID" 2>/dev/null || true
  fi
  rm -rf "$WORK"
}
trap cleanup EXIT

echo "[e2e] init project"
"$DEKA_BIN" init "$WORK" >/dev/null

echo "[e2e] boot deka task dev without prompt cascade"
(
  cd "$WORK"
  "$DEKA_BIN" task dev --no-prompt >"$LOG_DEV" 2>&1
) &
DEV_PID=$!

for _ in {1..30}; do
  if curl -fsS "http://localhost:8530/" >/dev/null 2>&1; then
    break
  fi
  sleep 0.2
done

BODY="$(curl -fsS "http://localhost:8530/")"
if [[ "$BODY" != *"Deka App"* ]]; then
  echo "[e2e] expected init page from dev server"
  echo "[e2e] body: $BODY"
  echo "[e2e] log:"
  cat "$LOG_DEV"
  exit 1
fi

if grep -q "\\[security\\] allow .* for this process\\?" "$LOG_DEV"; then
  echo "[e2e] unexpected interactive security prompt detected"
  cat "$LOG_DEV"
  exit 1
fi

kill "$DEV_PID" 2>/dev/null || true
wait "$DEV_PID" 2>/dev/null || true
unset DEV_PID

echo "[e2e] verify third-party read is denied by default"
mkdir -p "$WORK/deps"
echo "evil" > "$WORK/deps/evil.txt"
cat > "$WORK/app/main.phpx" <<'PHPX'
import { read_file } from 'fs';

$res = read_file('./deps/evil.txt');
if (!$res->ok) {
  echo $res->error;
}
PHPX

(
  cd "$WORK"
  "$DEKA_BIN" run app/main.phpx --no-prompt >"$LOG_DENY" 2>&1 || true
)

if ! grep -q "SECURITY_CAPABILITY_DENIED" "$LOG_DENY"; then
  echo "[e2e] expected security denial for third-party path"
  cat "$LOG_DENY"
  exit 1
fi

echo "[e2e] security phase1 checks passed"
