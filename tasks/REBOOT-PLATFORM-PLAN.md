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

- [ ] No Node/Bun compatibility layer.
- [x] No separate LSP runtime path from CLI (`deka lsp` now runs in-process).
- [ ] No CLI/Desktop platform runtime work.
- [ ] No JS/TS bundling pipeline in MVP (`crates/bundler` dropped for reboot scope).

## Locked decisions

- [ ] `introspect` is retained in reboot MVP.
- [x] `loop` crate is excluded from reboot MVP CLI surface (workspace removal tracked separately).
- [ ] `bundler` crate is dropped from reboot MVP.

## Hard workflow rules

- [ ] Every change must be committed before moving to the next task.
- [ ] Commit history is append-only notes (no amend/rewrite unless explicitly requested).
- [ ] Run relevant verification before each commit; include verification in commit message.
- [ ] Release profile only; avoid debug build drift.

## Phase 1: Foundation

- [ ] Create new repo/workspace scaffold for rebooted runtime.
- [ ] Keep crate-per-responsibility model and command registry pattern.
- [ ] Add `platform` abstraction crate with host contracts:
- [ ] `fs`, `env`, `io`, `process`, `time`, `random`, `net`, `ports`.
- [ ] Add architecture docs for platform contracts and crate boundaries.

## Phase 2: Runtime Core Extraction

- [ ] Extract host-agnostic runtime logic into `runtime_core`.
- [ ] Remove direct host/runtime globals from core execution path.
- [ ] Ensure PHP/PHPX pipeline can run through injected platform contracts.
- [ ] Keep module system behavior parity with current runtime where intended.

## Phase 3: Server Platform

- [ ] Implement `platform_server`.
- [ ] Update Deno dependencies to latest stable.
- [ ] Minimize direct Deno touchpoints to server platform adapter.
- [ ] Restore parity for `deka run` and `deka serve` on server.

## Phase 4: Browser Platform (WOSIX)

- [ ] Implement `platform_browser` on WOSIX primitives.
- [ ] Remove browser-side "serve magic" and command stubs.
- [ ] Run real `deka` process semantics in browser host.
- [ ] Keep strict module/network restrictions in browser adapter.

## Phase 5: CLI + LSP Unification

- [x] Move LSP implementation behind `deka lsp`.
- [ ] Remove separate LSP binary packaging path.
- [ ] Ensure same version lineage for runtime and LSP behavior.

## Phase 6: Parity and Verification

- [ ] Build behavior parity suite: old runtime vs rebooted runtime.
- [ ] Cover frontmatter, JSX/components, module loading, run/serve, diagnostics.
- [ ] Add required CI checks for parity gates.

## Phase 7: Artifact and Version Discipline

- [ ] Emit build manifest for each build artifact set:
- [ ] `git_sha`, build timestamp, target triple, runtime ABI version, artifact hashes.
- [ ] Add `deka --version --verbose` with manifest details.
- [ ] Add mismatch detection and fail fast on stale artifact combinations.

## Acceptance Criteria (MVP)

- [ ] Single `deka` binary ships runtime + LSP command.
- [ ] `platform_server` and `platform_browser` are functional and tested.
- [ ] No Node/Bun compatibility code remains in reboot repo.
- [ ] Runtime core contains no platform-specific glue.
- [ ] Artifact versioning and stale detection are enforced.
