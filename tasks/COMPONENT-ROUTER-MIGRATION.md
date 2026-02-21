# Component Router Migration (Userland App Routing)

Goal: Move app routing + page resolution out of runtime magic and into userland `php_modules/component/router.phpx`, while keeping a clean, NextJS-like DX.

## Task 1: Inventory + Baseline
- [ ] Document current routing behavior and entry resolution.
- [ ] Capture current behavior and error cases in notes.
- [ ] Add a regression fixture for `deka init` + `deka task dev` 404 case.

## Task 2: Userland Router API (PHPX)
- [ ] Define `component/router.phpx` public API:
  - `resolve($request, $root, $opts)`
  - `Router($props)` component
  - `route_manifest($root, $opts)`
- [ ] Port routing logic from `deka_php/php.js` into PHPX.
- [ ] Ensure compatibility with existing `app/` conventions (`page.phpx`, `layout.phpx`, dynamic tokens).

## Task 3: Userland Serve Entry
- [ ] Update `deka init` template to generate `app/main.phpx` that calls the router.
- [ ] Ensure `renderToString` is default (alias `render()`), and `renderToStream` optional.
- [ ] Provide explicit error messages when `app/` is missing or empty.

## Task 4: Runtime De‑Magic (Phase 1)
- [ ] Gate the old runtime router with a warning.
- [ ] Prefer explicit `serve.entry` and userland router.
- [ ] Add a migration warning when runtime app routing is used.

## Task 5: Tests + Docs
- [ ] Add PHPX fixtures for router behavior (static, dynamic, layout).
- [ ] Update docs: “App Routing” and “Router Component” usage.
- [ ] Run PHPX conformance test suite.
