# Adwa

Adwa is a Rust-first, open source WebContainers-compatible runtime designed to run Deka inside the browser.

## Goals
- Feature parity with WebContainers APIs (filesystem, processes, networking, ports, and package tooling).
- Runs entirely in the browser via WASM with a small JS bridge.
- General-purpose container runtime for web apps and tooling.
- Deterministic, reproducible environments (snapshots, caching).
- Tight integration with Deka runtime and tooling.

## Non-goals (for now)
- Native host runtime beyond test harnesses.
- Perfect Node/Bun parity in the first milestone.

## Project layout
- crates/adwa-core: platform-agnostic Rust APIs and core logic.
- crates/adwa-wasm: browser/WASM adapter and JS bindings.

## Status
This is a new subproject scaffold. See ARCHITECTURE.md for the intended shape,
API_MAPPING.md for WebContainers parity mapping, js/ for the JS wrapper,
scripts/ for build helpers, and examples/browser for a smoke test.

Node WASM integration is scaffolded in `js/NODE_WASM.md`.
