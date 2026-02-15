#!/usr/bin/env bash
set -euo pipefail

CLI_BIN="${DEKA_CLI_BIN:-$(pwd)/target/release/cli}"
if [[ ! -x "$CLI_BIN" ]]; then
  echo "missing cli binary: $CLI_BIN" >&2
  echo "build with: cargo build --release -p cli" >&2
  exit 1
fi

ROOT="$(mktemp -d /tmp/deka_islands_smoke.XXXXXX)"
cleanup() {
  pkill -f "$CLI_BIN serve $ROOT --port 8551" >/dev/null 2>&1 || true
  rm -rf "$ROOT"
}
trap cleanup EXIT

assert_contains() {
  local needle="$1"
  local body="$2"
  if ! grep -q "$needle" <<<"$body"; then
    echo "assertion failed: missing [$needle]" >&2
    echo "--- html ---" >&2
    echo "$body" >&2
    echo "--- server log ---" >&2
    cat /tmp/deka_islands_smoke.log >&2 || true
    exit 1
  fi
}

assert_regex() {
  local regex="$1"
  local body="$2"
  if ! grep -Eq "$regex" <<<"$body"; then
    echo "assertion failed: regex did not match [$regex]" >&2
    echo "--- html ---" >&2
    echo "$body" >&2
    echo "--- server log ---" >&2
    cat /tmp/deka_islands_smoke.log >&2 || true
    exit 1
  fi
}

cd "$ROOT"
"$CLI_BIN" init
rm -rf php_modules
cp -R "$(cd "$(dirname "$CLI_BIN")/../.." && pwd)/php_modules" php_modules

cat > app/page.phpx <<'PHPX'
---
import { jsx, jsxs } from 'component/core'
import { Hydration } from 'component/dom'

function LoadCard($props: object) {
  return <section>Load</section>
}

function AliasLoadCard($props: object) {
  return <section>Alias Load</section>
}

function IdleCard($props: object) {
  return <section>Idle</section>
}

function AliasIdleCard($props: object) {
  return <section>Alias Idle</section>
}

function VisibleCard($props: object) {
  return <section>Visible</section>
}

function AliasVisibleCard($props: object) {
  return <section>Alias Visible</section>
}

function MediaCard($props: object) {
  return <section>Media</section>
}

function AliasMediaCard($props: object) {
  return <section>Alias Media</section>
}

function OnlyCard($props: object) {
  return <section>Only</section>
}
---
<div id="app">
  <LoadCard client:load={true} />
  <AliasLoadCard clientLoad={true} />
  <IdleCard client:idle={true} />
  <AliasIdleCard clientIdle={true} />
  <VisibleCard client:visible={true} />
  <AliasVisibleCard clientVisible={true} />
  <MediaCard client:media="(min-width: 768px)" />
  <AliasMediaCard clientMedia="(max-width: 1024px)" />
  <OnlyCard client:only={true} />
</div>
<Hydration container="#app" layout="users" />
PHPX

"$CLI_BIN" serve "$ROOT" --port 8551 >/tmp/deka_islands_smoke.log 2>&1 &
sleep 1
HTML="$(curl -sS http://localhost:8551/)"

assert_contains 'data-deka-island="1"' "$HTML"
assert_contains 'data-client="load"' "$HTML"
assert_contains 'data-client="idle"' "$HTML"
assert_contains 'data-client="visible"' "$HTML"
assert_contains 'data-client="media"' "$HTML"
assert_contains 'data-media="(min-width: 768px)"' "$HTML"
assert_contains 'data-media="(max-width: 1024px)"' "$HTML"

assert_regex '<deka-island[^>]*data-component="[^"]*LoadCard"[^>]*data-client="load"' "$HTML"
assert_regex '<deka-island[^>]*data-component="[^"]*AliasLoadCard"[^>]*data-client="load"' "$HTML"
assert_regex '<deka-island[^>]*data-component="[^"]*IdleCard"[^>]*data-client="idle"' "$HTML"
assert_regex '<deka-island[^>]*data-component="[^"]*AliasIdleCard"[^>]*data-client="idle"' "$HTML"
assert_regex '<deka-island[^>]*data-component="[^"]*VisibleCard"[^>]*data-client="visible"' "$HTML"
assert_regex '<deka-island[^>]*data-component="[^"]*AliasVisibleCard"[^>]*data-client="visible"' "$HTML"
assert_regex '<deka-island[^>]*data-component="[^"]*MediaCard"[^>]*data-client="media"[^>]*data-media="\(min-width: 768px\)"' "$HTML"
assert_regex '<deka-island[^>]*data-component="[^"]*AliasMediaCard"[^>]*data-client="media"[^>]*data-media="\(max-width: 1024px\)"' "$HTML"

assert_contains '__dekaHydrateIslands' "$HTML"
assert_contains 'dekaIslandScheduled' "$HTML"
assert_contains 'requestIdleCallback' "$HTML"
assert_contains 'IntersectionObserver' "$HTML"

if ! grep -Eq '<deka-island[^>]*data-component="[^"]*OnlyCard"[^>]*>\s*<template shadowrootmode="open"></template>\s*</deka-island>' <<<"$HTML"; then
  echo "assertion failed: client:only island wrapper was not empty" >&2
  echo "--- html ---" >&2
  echo "$HTML" >&2
  exit 1
fi

echo "islands smoke: ok"
