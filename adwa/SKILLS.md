# Adwa Agent Skills

This file is for future agents and subagents working in `adwa/`.
It documents how the Adwa browser demo is wired today, where command behavior differs from host Deka CLI, and how to debug common failures quickly.

## 1) What Adwa Is

Adwa is a browser runtime + IDE playground. It is not just a static site, and it is not just a Deka CLI project.

- UI/host shell layer: JS/TS (compiled into `website/vendor/adwa_js/*`)
- Browser runtime core: WASM (`adwa-wasm`) + WebContainer facade
- App/demo payload: bundled PHPX project files (`/deka.json`, `/deka.lock`, `php_modules/*`) injected into browser VFS

Important:
- There *is* PHPX in this repo (`website/phpx/*` and `website/project/php_modules/adwa/*`).
- There *are* `deka.json` files at multiple levels (`adwa/`, `website/`, `website/project/`).

## 2) Config Split (Critical)

These configs serve different purposes:

- `deka.json` (repo root): script entrypoints only (`dev`, `test`)
- `website/deka.json`: static website serve config (entry `index.html`)
- `website/project/deka.json`: bundled in-browser project config (entry `app/home.phpx`)

If behavior seems contradictory, confirm which config path is in play.

## 3) Canonical Run Paths

Use these commands from `adwa/`:

- Dev serve (8530): `./scripts/dev.sh`
- Playground serve (5173): `./scripts/run-playground.sh`
- Build only: `./scripts/run-playground.sh --build-only`
- E2E: `./scripts/test-playground-e2e.sh`

Why this matters:
- `scripts/dev.sh` and `scripts/run-playground.sh` are the source-of-truth runtime entrypoints now.
- Some older docs mention `deka run dev`; treat that as potentially stale unless verified against current scripts.

## 4) Build Pipeline (What Gets Bundled)

`scripts/build-demo.sh` does the heavy lift:

- Builds WASM host bindings
- Builds `js/` package
- Copies runtime JS into `website/vendor/adwa_js`
- Bundles project files into `website/vendor/adwa_js/php_project_bundle.js`
- Emits embedded wasm data URLs used by browser bootstrap

Current wasm fallback behavior:
- It first targets `../mvp2/target/.../php_rs.wasm`
- If missing required exports (`php_alloc/php_free/php_run`), it falls back to `../mvp-ARCHIVE/.../php_rs.wasm`

This fallback is intentional for current Adwa compatibility.

## 5) Browser VFS + Snapshot Model

Browser state persists via localStorage snapshots.

- Snapshot key: `adwa.vfs.snapshot.v1`
- On boot, `website/main.js`:
  - loads snapshot
  - seeds bundled files
  - force-refreshes bundled project files (including `php_modules`, `deka.lock`, `deka.json`)

If users report stale module errors or old imports reappearing:

```js
localStorage.removeItem("adwa.vfs.snapshot.v1");
location.reload();
```

## 6) Browser Terminal Command Model

Do not assume host CLI parity in the browser terminal.

- `website/main.js` intercepts commands
- `deka` is forwarded through browser shell container spawn
- command resolution for CLI-style modules comes from `bin` entries in bundled `php_modules/*/deka.json`

This is why:
- some commands work (`ls`, `cat`, etc. from `cli-adwa` packages)
- host-style commands like `deka task dev` may fail with non-intuitive exit codes inside browser terminal

Also note:
- shebang `#!/usr/bin/deka ...` path only supports `run` in browser demo mode

## 7) Known Pitfalls

1. Server “not up” after restart:
- Build step may still be running or process was backgrounded incorrectly.
- Use foreground first to verify:
  - `./scripts/run-playground.sh`

2. “Unused import … ContextProvider” or lock integrity mismatch:
- Usually stale snapshot + stale bundled payload mismatch.
- Rebuild + clear snapshot.

3. Assuming `deka run dev` works the same everywhere:
- Host shell and browser terminal are different execution environments.

## 8) Quick Triage Checklist

From `adwa/`:

1. `./scripts/run-playground.sh --build-only`
2. `curl -fsS http://127.0.0.1:5173` (or start server with `./scripts/run-playground.sh`)
3. If browser runtime errors persist, clear snapshot key.
4. Run `./scripts/test-playground-e2e.sh` to validate end-to-end.

If E2E passes and user still sees old behavior, suspect cached browser state first.

## 9) Files To Read First

- `README.md`
- `ARCHITECTURE.md`
- `scripts/build-demo.sh`
- `scripts/dev.sh`
- `scripts/run-playground.sh`
- `scripts/test-playground-e2e.sh`
- `website/main.js`
- `website/project/deka.json`

## 10) Historical Context (Recent)

Key commits worth reading when confused:

- `252a952`: serve browser demo entry instead of TS handler
- `e7d74a5`: split website serve config vs bundled project config
- `5c6a27d`: local playground harness + e2e
- `6f27ac9`, `0ed0a21`, `62f8632`, `4ced045`, `93e3687`: runtime integrity/snapshot hardening

