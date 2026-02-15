# Islands Implementation Tasks

Status: Active
Owner: Runtime team

## Goal

Implement and harden PHPX islands architecture for production SSR + selective hydration, with explicit directives and stable HMR behavior.

## Scope

- Dynamic runtime: `.phpx` only.
- Islands are hydration scheduling primitives, not full-page boundaries.
- Non-island HTML must continue to patch via HMR.

## Phase 1: Baseline and Validation

- [x] 1.1 Create dedicated islands task tracker.
- [x] 1.2 Add islands runtime smoke script for directive behavior.
- [ ] 1.3 Add islands smoke script to checkpoint guidance.
- [ ] 1.4 Add CI-friendly islands smoke invocation.

## Phase 2: Directive Runtime Completeness

- [ ] 2.1 Verify SSR metadata parity for `client:load`.
- [x] 2.2 Verify SSR metadata parity for `client:idle`.
- [x] 2.3 Verify SSR metadata parity for `client:visible`.
- [x] 2.4 Verify SSR metadata parity for `client:media`.
- [x] 2.5 Verify `client:only` server behavior (no SSR body, wrapper only).
- [ ] 2.6 Ensure directive aliases (`clientLoad`, etc.) are normalized consistently.

## Phase 3: Hydration Client Hardening

- [ ] 3.1 Confirm scheduler behavior across directives in browser runtime.
- [ ] 3.2 Ensure hydration idempotence (`dekaIslandHydrated`) across partial swaps.
- [ ] 3.3 Ensure `window.__dekaHydrateIslands` is stable and safe to call repeatedly.
- [ ] 3.4 Add fallback behavior for browsers lacking `IntersectionObserver` and `requestIdleCallback`.

## Phase 4: HMR + Islands Interop

- [ ] 4.1 Ensure island-id stability across unchanged structure renders.
- [ ] 4.2 Confirm island-scoped patching path when full structure diff fails.
- [ ] 4.3 Preserve active input/focus state during island patch updates.
- [ ] 4.4 Add regression tests for island patch fallback in HMR websocket layer.

## Phase 5: DX + Docs

- [ ] 5.1 Document canonical islands usage patterns (`load/idle/visible/media/only`).
- [ ] 5.2 Document anti-patterns and fallback semantics.
- [ ] 5.3 Add minimal cookbook examples (SSR + islands + Link + Hydration).

## Exit Criteria

- [ ] Islands directives pass runtime smoke and targeted regressions.
- [ ] HMR + islands interop is stable in manual browser verification.
- [ ] Docs and task status are fully updated.
