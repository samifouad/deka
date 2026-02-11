# PHPX Async Track

Status: active
Owner: runtime
Scope: PHPX language/runtime/LSP (not regular `.php`)

## Goals

- Make PHPX async-first on top of the existing Deka event loop.
- Keep sync APIs explicit (`*Sync`) for compatibility and predictable blocking paths.
- Add async JSX with `Suspense`.
- Keep PHP bridge sync-only.

## Decisions (locked)

- [x] Top-level await is allowed in `.phpx` module scope and frontmatter.
- [x] Top-level await is disallowed in `.php`.
- [x] Async module evaluation propagates to importers.
- [x] Cycles involving top-level await are hard errors.
- [x] PHPX standard guidance is async-first; sync is explicit (`*Sync`).
- [x] PHP surface remains sync-only.
- [x] Promise type is `Promise<T>`.
- [x] Recoverable async failures use `Promise<Result<T, E>>`.

## Phase A: Parser + AST

- [x] Add `async function` syntax in PHPX mode.
- [x] Add `await <expr>` expression in PHPX mode.
- [x] Allow top-level await in `.phpx` module scope.
- [x] Add parser errors for invalid await contexts in `.php` and non-async fn bodies.
- [x] Tests: valid/invalid parse cases for async/await/TLA.

## Phase B: Type System

- [x] Add `Promise<T>` to PHPX type model.
- [x] Type rule: `await Promise<T> -> T`.
- [x] Type rule: `await` on non-promise is error.
- [x] Type rule: async fn returns `Promise<T>`.
- [x] Inference: function containing `await` must be async (or emit error with fix).
- [x] Tests: `Promise<Result<T,E>>` handling and match-based flow.

## Phase C: Runtime + Module Loader

- [x] Add async completion path in internal bridge pipeline (no userland bridge exposure).
- [ ] Ensure module loader supports async initialization dependencies.
- [x] Add deterministic TLA cycle detection with path details.
- [x] `deka run` waits for module graph async completion.
- [x] `deka serve` resolves graph before serving first request.
- [ ] Tests: ordering, dependency waiting, cycle errors.
  Note: module-scope `await` inside cached `php_modules/.cache/phpx/*.php` still parses in PHP mode; loader-side parse mode/caching semantics need adjustment before this can be completed end-to-end.

## Phase D: Std Module API split (Node-style)

- [x] `fs`: async default + `*Sync` pair (`readFile`/`readFileSync`, etc.).
- [x] `db/*`: async default + `*Sync` pair.
- [x] `net`: async default + `*Sync` pair where meaningful.
- [x] Keep PHP-facing APIs sync-only.
- [x] Tests: API parity and behavior consistency.

## Phase E: Async JSX

- [ ] Permit components returning `Promise<VNode>`.
- [ ] Add `Suspense` component with typed `fallback`.
- [ ] Runtime behavior: fallback on unresolved subtree, resolved output on completion.
- [x] Validation: async component usage without `Suspense` errors with actionable hint.
- [ ] Tests: async component render, nested async trees, fallback correctness.

## Phase F: LSP + Validation

- [x] Diagnostics for invalid `await` usage/context.
- [x] Hovers/signatures for `async` and `Promise<T>`.
- [x] JSX diagnostics for async component + `Suspense` requirements.
- [x] Completion updates for async and sync API variants.
- [ ] VS Code regression checks.

## Phase G: Docs

- [ ] Add `docs/phpx/general/async-await.mdx`.
- [ ] Add top-level await section with cycle caveats and examples.
- [ ] Add async JSX + `Suspense` usage examples.
- [ ] Add PHP vs PHPX async behavior note.

## Exit Criteria

- [ ] End-to-end async sample app passes runtime + LSP checks.
- [ ] No `.php` behavior regressions.
- [ ] Async and sync module APIs documented and tested.
