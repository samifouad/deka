# Reboot Platform Plan (MVP)

Status: Active
Owner: Runtime team

## Mission

Reboot Deka around PHP/PHPX only, with a clean platform abstraction and a single CLI binary.

MVP platforms:

- [ ] `platform_server`
- [ ] `platform_browser` (WOSIX)

Deferred (post-MVP):

- [ ] `platform_multi_tenant` (Tana-specific hardening/orchestration)
- [ ] `platform_cli` (embedded VFS executable publishing)
- [ ] `platform_desktop` (desktop packaging)

## Non-goals (MVP)

- [x] No Node/Bun compatibility layer (runtime run/serve paths stripped).
- [x] No separate LSP runtime path from CLI (`deka lsp` now runs in-process).
- [x] No CLI/Desktop platform runtime work.
- [x] No JS/TS bundling pipeline in MVP (`deka build` removed; `crates/bundler` dropped from workspace).

## Locked decisions

- [x] `introspect` is retained in reboot MVP.
- [x] `loop` crate is excluded from reboot MVP.
- [x] `bundler` crate is dropped from reboot MVP.

## Hard workflow rules

- [x] Every change must be committed before moving to the next task.
- [x] Commit history is append-only notes (no amend/rewrite unless explicitly requested).
- [x] Run relevant verification before each commit; include verification in commit message.
- [x] Release profile only; avoid debug build drift.

## Phase 1: Foundation

- [x] Create new repo/workspace scaffold for rebooted runtime.
- [x] Keep crate-per-responsibility model and command registry pattern.
- [x] Add `platform` abstraction crate with host contracts.
- [x] `fs`, `env`, `io`, `process`, `time`, `random`, `net`, `ports`.
- [x] Add architecture docs for platform contracts and crate boundaries.

## Phase 2: Runtime Core Extraction

- [x] Extract host-agnostic runtime logic into `runtime_core` (initial shared utilities).
- [ ] Remove direct host/runtime globals from core execution path.
- [ ] Ensure PHP/PHPX pipeline can run through injected platform contracts.
- [ ] Keep module system behavior parity with current runtime where intended.

## Phase 3: Server Platform

- [x] Implement `platform_server` (initial adapter scaffold).
- [ ] Update Deno dependencies to latest stable.
- [x] Minimize direct Deno touchpoints to server platform adapter.
- [x] Restore parity for `deka run` and `deka serve` on server.

## Phase 4: Browser Platform (WOSIX)

- [x] Implement `platform_browser` on WOSIX primitives (initial adapter scaffold).
- [ ] Remove browser-side "serve magic" and command stubs.
- [ ] Run real `deka` process semantics in browser host.
- [ ] Keep strict module/network restrictions in browser adapter.

## Phase 5: CLI + LSP Unification

- [x] Move LSP implementation behind `deka lsp`.
- [x] Remove separate LSP binary packaging path.
- [x] Ensure same version lineage for runtime and LSP behavior.

## Phase 6: Parity and Verification

- [x] Build behavior parity suite baseline: old runtime vs rebooted runtime (`scripts/parity-runtime.sh` run+serve).
- [x] Cover frontmatter, JSX/components, module loading, run/serve, diagnostics.
- [ ] Add required CI checks for parity gates.

## Phase 7: Artifact and Version Discipline

- [ ] Emit build manifest for each build artifact set:
- [ ] `git_sha`, build timestamp, target triple, runtime ABI version, artifact hashes.
- [x] Add `deka --version --verbose` with manifest details.
- [ ] Add mismatch detection and fail fast on stale artifact combinations.

## Acceptance Criteria (MVP)

- [x] Single `deka` binary ships runtime + LSP command.
- [ ] `platform_server` and `platform_browser` are functional and tested.
- [x] No Node/Bun compatibility code remains in reboot repo.
- [ ] Runtime core contains no platform-specific glue.
- [ ] Artifact versioning and stale detection are enforced.
