# Agent Notes (Current Mission)

## Canonical Repo Policy (non-negotiable)

- **ACTIVE IMPLEMENTATION REPO:** `~/Projects/deka/mvp`
- **ARCHIVE SOURCE REPO (read-only):** `~/Projects/deka/deka-ARCHIVE`
- **Browser host substrate repo:** `~/Projects/deka/adwa`

You must only implement, test, and commit runtime/CLI/LSP MVP work in `~/Projects/deka/mvp`.

`~/Projects/deka/deka-ARCHIVE` is migration source only.
Do not run active task work there.

## Mission scope (active)

Primary plan: `tasks/REBOOT-PLATFORM-PLAN.md`.

MVP platforms only:

- `platform_server`
- `platform_browser` (ADWA)

Deferred platforms (post-MVP):

- `platform_multi_tenant`
- `platform_cli`
- `platform_desktop`

Do not add Node/Bun compatibility work in this mission.

## Commit policy (mandatory)

- Every change, including small fixes, must be committed before starting the next task.
- Use commit messages as append-only notes.
- Do not amend/rewrite commit history unless explicitly requested.
- Include verification summary in each commit message.

## Checkpoint process (mandatory)

`checkpoint` is a required quality gate before continuing major work.

- Trigger checkpoint when:
  - finishing a phase milestone,
  - switching repos/runtime tracks,
  - touching runtime execution, CLI behavior, LSP behavior, or ADWA host behavior,
  - before handing off to another agent/session.
- Checkpoint steps:
  - Run release builds/tests for touched areas.
  - Verify local artifact wiring and lineage (`deka`, `deka lsp`, manifest).
  - Execute basic human validation flow:
    - `deka run` smoke on a minimal PHPX file.
    - `deka serve` smoke and confirm endpoint behavior.
    - `scripts/test-islands-smoke.sh` for islands SSR/hydration metadata + directive alias checks.
    - ADWA build/e2e checks for browser platform updates.
  - Record a short checkpoint summary in commit message or task notes:
    - what passed,
    - what failed,
    - what was deferred and why.
- Do not proceed to the next major task until checkpoint results are captured.

## Build and verification policy

- Build release artifacts only.
- Do not rely on debug binaries for validation.
- Run relevant tests/checks before commit.
- Use `scripts/build-release-manifest.sh` to produce release artifacts + `target/release/deka-manifest.json`.
- Use `scripts/verify-release-manifest.sh` to fail fast on stale/mismatched `cli` and `php_rs.wasm` artifacts.
- Keep local PATH wiring pinned to this repo's release binaries:
  - `~/.local/bin/deka -> ~/Projects/deka/mvp/target/release/cli`
  - Do not wire a separate `phpx_lsp` binary; use `deka lsp`.

ADWA runtime/UI changes (current script names still use `adwa`):

1. `scripts/run-adwa-playground.sh --build-only`
2. `ADWA_E2E_INCLUDE_PHPX=1 ./scripts/test-adwa-playground-e2e.sh`

## Artifact/version discipline

- Track artifact freshness explicitly.
- Prefer a build manifest with artifact hashes and git SHA.
- Surface runtime lineage in CLI/version output when available.
- Avoid stale binary/wasm drift across runtime, browser assets, and LSP.

## Architecture direction

- Keep crate-per-responsibility organization.
- Keep command registry pattern in CLI.
- Move toward a host abstraction (`platform`) so runtime core stays host-agnostic.
- Keep LSP integrated under `deka lsp` direction; avoid separate lifecycle drift.

## Docs and tasks policy

- Keep active plans and checklists in `tasks/`.
- Keep user-facing docs in `docs/phpx/`.
- Keep internal plans/devlogs/design notes in `tasks/phpx/` (never under `docs/phpx/`).
- If runtime behavior changes, update relevant docs in the same task.
- `php_modules` exported APIs must include `/// docid:` blocks; docs publish/build must fail when coverage is missing.
- Use `scripts/build-release-docs.sh` as the default release pipeline (build `cli` + publish/bundle docs).
- CI docs gates live in `.github/workflows/phpx-docs.yml` (`scripts/check-module-docs.sh` and `scripts/build-release-docs.sh`).


## Runtime language support (explicit)

- Dynamic runtime execution supports **PHPX only** (`.phpx`).
- Do not implement or preserve dynamic execution support for `.php`, `.js`, `.jsx`, `.ts`, or `.tsx`.
- File-based routing under `app/` and `api/` must resolve `.phpx` route files only.
- Static assets (HTML/CSS/JS files) may be served as static files; this is distinct from dynamic handler execution.

## deka.json project contract (build/runtime)

- `deka.json` is required at project root.
- Standard key: `type`
  - `type: "lib"` => library/module package (no runnable app entry).
  - `type: "serve"` => runnable app package.
- For runnable apps (`type: "serve"`), `serve.entry` is required and must point to the runtime entry file.
- `deka build` web-project mode requires:
  - `type: "serve"`
  - `serve.entry` set to a `.phpx` file under `app/`
  - `deka.lock` present
  - `public/index.html` present
- Do not infer project kind from folder shape alone when `deka.json` explicitly defines `type`.

## PHPX module resolution contract (MVP)

- Resolver order is deterministic:
  1) local `<project>/php_modules` (requires local `deka.lock`)
  2) global `<PHPX_MODULE_ROOT>/php_modules` (requires global `deka.lock`)
- `deka.lock` is source of truth for package-style PHPX module resolution.
- Runtime must reject drift:
  - lock entry missing for requested package module
  - lock entry points to missing bytes
  - lock hash/integrity mismatch
- Import shorthand/index behavior must be consistent for both `@/...` and package paths (`Foo.phpx` and `Foo/index.phpx`).
- Compiled module cache keys must include lock identity (`lockfileVersion` + lock hash).
- Only `PHPX_MODULE_ROOT` is supported for global root overrides.
- Each module-resolution task update requires tests and a commit before moving to the next task.

## Introspect metrics (quick reality check)

- The `introspect` crate is primarily a CLI/UI client; core op timing collection lives in runtime pool internals.
- Deno op metrics are tracked in `crates/pool/src/isolate_pool.rs` via `OpMetricsEvent` (`Dispatched`, `Completed`, `CompletedAsync`, `ErrorAsync`, etc.).
- Per-op summaries include `in_flight` counts (`OpTimingSummary.in_flight`) and request traces include per-request `op_timings`.
- `crates/modules_php` bridge stats (`op_php_bridge_proto_stats`) provide transport-level metrics (calls/bytes/time), not full isolate op scheduling metrics.
- If async behavior is in question, validate with runtime op timing output (or introspect debug views) rather than only module-level wrappers.
