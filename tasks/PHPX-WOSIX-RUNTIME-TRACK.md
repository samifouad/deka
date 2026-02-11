# PHPX Wosix Runtime Track

Status: active
Owner: runtime + wosix
Scope: run PHPX inside browser via `wosix` host profile

## Goals

- Treat `wosix` as the primary browser host target for PHPX.
- Keep a single PHPX language model across server and browser hosts.
- Gate capabilities by host contract instead of ad-hoc browser checks.
- Enable playground/repl workflows with realistic module/runtime behavior.

## Host Alignment (locked)

- [x] Browser target is `wosix`-first (not generic browser stubs).
- [x] Capability-driven runtime behavior by host profile.
- [x] Same module/type semantics across `server` and `wosix`.
- [x] Internal bridge remains private; host adapters implement capabilities.

## Host Profiles

- [x] Define explicit host profile enum/config: `server`, `wosix`.
- [ ] Add capability matrix consumed by runtime + validation:
  - [x] `fs`
  - [x] `net`
  - [x] `process/env`
  - [x] `clock/random`
  - [x] `db`
  - [x] `wasm imports`
  - Note: runtime gating and LSP target-aware diagnostics for restricted imports are implemented; compiler validation pass is still pending.
- [x] Ensure unsupported operations return structured capability errors.

## Runtime Integration

- [x] Build/pack php-rs runtime wasm artifact for `wosix` integration tests.
- [x] Add host adapter layer mapping PHPX runtime ops to `wosix` interfaces.
- [x] Verify fs path maps to `wosix` in-memory/mounted filesystem.
- [x] Verify process I/O maps to `wosix` stdio channels.
- [x] Verify module loading from virtual FS.

## Module System + Imports

- [x] Support `@/` alias against virtual project root in `wosix`.
- [x] Resolve `php_modules/` via mounted/virtual tree, not disk-only assumptions.
- [x] Ensure generated cache/lock behaviors work in virtual FS mode.
- [x] Add tests for module graph load under `wosix`.

## Validation + LSP target awareness

- [x] Add target-mode setting for diagnostics: `phpx.target = server|wosix`.
- [x] Emit capability-aware errors (forbidden modules/APIs in target host).
- [x] Keep suggestions actionable (e.g. alternate API or host config).
- [x] Add LSP tests for target-specific diagnostics.

## DX + Tooling

- [x] Add developer script for `wosix` php runtime smoke run.
- [x] Add playground boot script/example using `wosix` with PHPX file tree.
- [x] Ensure docs explain host capabilities and limitations clearly.

## Testing

- [x] Runtime smoke tests in `wosix`:
  - [x] basic PHPX eval
  - [x] imports/module graph
  - [x] JSX render-to-string
  - [x] filesystem read/write
- [x] Negative tests: capability denied errors for blocked APIs.
- [ ] Browser e2e: edit -> run -> output updates in playground.

## Docs

- [x] Add `docs/phpx/general/wosix-browser-runtime.mdx`.
- [x] Document host model, capability matrix, and how to run demos.
- [x] Include migration notes from server-only assumptions.

## Cross-track dependency

- [ ] Async track (`tasks/PHPX-ASYNC-TRACK.md`) feeds directly into `wosix` host event scheduling and top-level await behavior.

## Exit Criteria

- [ ] PHPX runs in browser via `wosix` with module imports and JSX SSR output.
- [x] Capability errors are explicit and stable.
- [x] LSP can validate against `wosix` target mode.
