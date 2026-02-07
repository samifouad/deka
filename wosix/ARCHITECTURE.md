# Architecture

## Runtime model
- Browser-only host running in a dedicated Web Worker.
- WASM core (wosix-wasm) exposes a JS facade that mirrors WebContainers.
- wosix-core stays platform-agnostic and contains state machines, protocol parsing, and VFS logic.

## Subsystems
- Virtual FS: layered FS with in-memory, OPFS (persistent), and package cache.
- Process model: spawn/exec semantics mapped onto Web Workers and async tasks.
- Networking: outbound via fetch/WebSocket; inbound via port proxy (TBD).
- Module/package: npm registry fetch, tarball unpack, and lockfile resolution.

## JS API surface (parity targets)
- WebContainer.boot / mount / fs / spawn
- Port mapping and server-ready events
- Process I/O streams and TTY emulation

## Integration with Deka
- Deka runtime compiled to WASM and loaded as a "container image."
- Deka's module loader uses Wosix FS and process plumbing.

## Deka-in-browser architecture (proposed)
This mirrors the WebContainers idea: compile the core engine to WASM and replace
OS syscalls with browser adapters.

### Layers
- **Runtime core (WASM):** Deka scheduler, router, JS/TS module loading logic,
  validation, and other CPU-bound core logic.
- **Host adapters (JS):** FS, network, and process abstractions implemented
  with browser APIs (OPFS/IndexedDB, Fetch/WebSocket, Web Workers).
- **JS façade:** WebContainer-style API + Deka-specific bootstrap (serve/run).

### Mapping OS features to browser equivalents
- **Filesystem:** in-memory for ephemeral + OPFS for persistence; tarball
  extraction and npm cache are browser-managed.
- **Networking:** outbound via `fetch`/`WebSocket`; inbound is proxied through
  a "port publish" surface and fetch bridge.
- **Process model:** simulated "processes" in WASM with JS-driven scheduling
  (Workers for isolation if needed).
- **Timers/event loop:** use browser timers; keep Deka's scheduler in WASM.

### Constraints
- No embedded V8; browser JS engine hosts the runtime.
- No raw sockets or fork/exec; these stay as virtualized APIs.
- Threading is limited to Web Workers and WASM threads when available.

### PHP (WASM) integration
Deka can run PHP via WASM today; in-browser this becomes a multi-WASM setup.

- **PHP runtime (WASM):** A separate PHP WASM module is loaded by the JS host.
  It is treated as a "guest runtime" that Deka can invoke for PHP handlers.
- **FS bridge:** PHP runs against the same virtual FS as Deka so it can read
  user code, dependencies, and temp files.
- **Invocation path:** Deka routes a request to PHP by:
  1) marshalling the request (env, headers, body) into the PHP WASM runtime,
  2) executing the script, and
  3) collecting the response body/headers back into Deka.
- **Process model:** PHP runs as a logical "subprocess" inside the same worker
  (or a dedicated worker for isolation).
- **Limits:** No native extensions that require OS syscalls; only WASM-friendly
  extensions can be bundled.
