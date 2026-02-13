# Agent Notes (Current Mission)

## Mission scope (active)

Primary plan: `tasks/REBOOT-PLATFORM-PLAN.md`.

MVP platforms only:

- `platform_server`
- `platform_browser` (WOSIX)

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

WOSIX runtime/UI changes:

1. `scripts/run-wosix-playground.sh --build-only`
2. `WOSIX_E2E_INCLUDE_PHPX=1 ./scripts/test-wosix-playground-e2e.sh`

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
