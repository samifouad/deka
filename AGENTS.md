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

## Build and verification policy

- Build release artifacts only.
- Do not rely on debug binaries for validation.
- Run relevant tests/checks before commit.
- Use `scripts/build-release-manifest.sh` to produce release artifacts + `target/release/deka-manifest.json`.
- Use `scripts/verify-release-manifest.sh` to fail fast on stale/mismatched `cli` and `php_rs.wasm` artifacts.
- Keep local PATH wiring pinned to this repo's release binaries:
  - `~/.local/bin/deka -> ~/Projects/deka/mvp/target/release/cli`
  - `~/.local/bin/phpx_lsp -> ~/Projects/deka/mvp/target/release/phpx_lsp`

ADWA runtime/UI changes:

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
- Keep user-facing docs in `docs/php/` and `docs/phpx/`.
- If runtime behavior changes, update relevant docs in the same task.
