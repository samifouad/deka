# Adwa JS Wrapper

This package provides a WebContainers-style API on top of the `adwa-wasm`
bindings. It is intentionally thin and forwards calls to the WASM exports.

## Usage
```ts
import * as wasm from "./path/to/adwa_wasm.js";
import { WebContainer } from "@deka/adwa-js";

const container = await WebContainer.boot(wasm, {
  init: wasm.default,
  nodeRuntime: "shim",
});
await container.mount({
  "index.js": {
    file: "console.log('hello');",
  },
});

const proc = await container.spawn("node", ["index.js"]);
const reader = proc.output.getReader();
const { value } = await reader.read();
```

## Notes
- `WebContainer.on()` registers a WASM callback (no polling required).
- Process stdio streams are synchronous wrappers over in-memory pipes.
- `spawn("node", ...)` uses a lightweight Node-like shim (CommonJS + minimal fs/path/process).
- `nodeRuntime: "wasm"` is a stub hook for integrating a real Node WASM build later.
- `DekaBrowserRuntime` provides a minimal, single-file CommonJS loader to test Deka-style handlers.
- See `NODE_WASM.md` for the adapter interface and wiring TODOs.

## Build
```
npm run build
```
