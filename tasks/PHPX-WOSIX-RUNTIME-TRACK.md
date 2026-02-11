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
  - Note: runtime gating is implemented; validation/LSP target-aware diagnostics are still pending.
- [x] Ensure unsupported operations return structured capability errors.

## Runtime Integration

- [ ] Build/pack php-rs runtime wasm artifact for `wosix` integration tests.
- [ ] Add host adapter layer mapping PHPX runtime ops to `wosix` interfaces.
- [ ] Verify fs path maps to `wosix` in-memory/mounted filesystem.
- [ ] Verify process I/O maps to `wosix` stdio channels.
- [ ] Verify module loading from virtual FS.

## Module System + Imports

- [ ] Support `@/` alias against virtual project root in `wosix`.
- [ ] Resolve `php_modules/` via mounted/virtual tree, not disk-only assumptions.
- [ ] Ensure generated cache/lock behaviors work in virtual FS mode.
- [ ] Add tests for module graph load under `wosix`.

## Validation + LSP target awareness

- [ ] Add target-mode setting for diagnostics: `phpx.target = server|wosix`.
- [ ] Emit capability-aware errors (forbidden modules/APIs in target host).
- [ ] Keep suggestions actionable (e.g. alternate API or host config).
- [ ] Add LSP tests for target-specific diagnostics.

## DX + Tooling

- [ ] Add developer script for `wosix` php runtime smoke run.
- [ ] Add playground boot script/example using `wosix` with PHPX file tree.
- [ ] Ensure docs explain host capabilities and limitations clearly.

## Testing

- [ ] Runtime smoke tests in `wosix`:
  - [ ] basic PHPX eval
  - [ ] imports/module graph
  - [ ] JSX render-to-string
  - [ ] filesystem read/write
- [ ] Negative tests: capability denied errors for blocked APIs.
- [ ] Browser e2e: edit -> run -> output updates in playground.

## Docs

- [ ] Add `docs/phpx/general/wosix-browser-runtime.mdx`.
- [ ] Document host model, capability matrix, and how to run demos.
- [ ] Include migration notes from server-only assumptions.

## Cross-track dependency

- [ ] Async track (`tasks/PHPX-ASYNC-TRACK.md`) feeds directly into `wosix` host event scheduling and top-level await behavior.

## Exit Criteria

- [ ] PHPX runs in browser via `wosix` with module imports and JSX SSR output.
- [ ] Capability errors are explicit and stable.
- [ ] LSP can validate against `wosix` target mode.
