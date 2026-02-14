# Adwa

Adwa is a Rust-first browser runtime and UI playground for Deka's web-host execution model.

## Goals
- Browser-first host environment for Deka runtime behavior.
- Linux-like default FS/env model for predictable tooling behavior.
- Stable adapter boundary between UI, host capabilities, and PHP/PHPX runtime integration.
- Fast local iteration loop for runtime + editor UX in the browser.

## Non-goals (MVP)
- Node/Bun compatibility mode.
- Native desktop/host runtime parity in this repository.
- Owning package-manager/runtime logic that belongs in `mvp`.

## Project layout
- `crates/adwa-core`: platform-agnostic Rust core APIs and host model.
- `crates/adwa-wasm`: browser/WASM adapter and JS bindings.
- `js/`: browser-side host bridge and runtime adapter utilities.
- `examples/browser`: interactive browser playground (editor + render + console).
- `scripts/`: build/dev helpers.

## Runtime payload policy
- Default demo build is runtime-first and excludes browser LSP wasm assets.
- Editor/LSP wasm is optional and loaded lazily only when present.
- To include editor wasm assets in demo output:

```sh
ADWA_DEMO_INCLUDE_EDITOR=1 ./scripts/build-demo.sh
```

This keeps normal runtime delivery smaller and avoids shipping editor-only binaries by default.

## Current status
- ADWA naming migration is complete (no WOSIX compatibility layer).
- Browser host boots with Linux-like defaults (`/home/user`, `/tmp`, `PATH`, etc.).
- Playground path is active and used for checkpoint validation.
- Build scripts are release-first for artifact consistency.

See `ARCHITECTURE.md` and `API_MAPPING.md` for deeper details.

## Binaries And Artifacts (Dev vs Prod)

### Dev deliverables
- Rust outputs in `target/release/*` during local iteration.
- Browser demo payload under `examples/browser/vendor/*` after:

```sh
./scripts/build-demo.sh
```

### Prod deliverables (ship list)
Required runtime payload:
- `crates/adwa-wasm/pkg/adwa_wasm_bg.wasm`
- `crates/adwa-wasm/pkg/adwa_wasm.js`
- `crates/adwa-wasm/pkg/adwa_wasm.d.ts`
- Browser app files in `examples/browser/` (`index.html`, `main.js`, `styles.css`, `core/*`, `ui/*`)
- Runtime vendor bundle in `examples/browser/vendor/adwa_wasm/*`
- Runtime vendor bundle in `examples/browser/vendor/adwa_js/*`

Optional editor payload (only when enabled):
- `examples/browser/vendor/adwa_editor/*`

To include optional editor payload:

```sh
ADWA_DEMO_INCLUDE_EDITOR=1 ./scripts/build-demo.sh
```

Do not ship:
- `target/release/deps/*`
- `target/release/*.d`
- `target/release/*.rlib`
- `target/release/build/*`
- `target/release/.fingerprint/*`
