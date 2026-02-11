# Wosix Browser Demo

This demo runs the Node-like shim in the browser using the WASM bindings and
`@deka/wosix-js` wrapper.

The Node WASM runtime hook is scaffolded but not wired yet, so the demo uses
the shim by default. A minimal Deka browser stub is available via the
`?demo=deka` query param.

## Build steps
1. Build the WASM bindings + JS wrapper:
   ```sh
   cd ../..
   ./scripts/build-wasm.sh
   cd js
   npm install
   npm run build
   ```
   Alternatively:
   ```sh
   cd ../..
   ./scripts/build-demo.sh
   ```
2. Serve the demo:
   ```sh
   cd ../examples/browser
   python3 -m http.server 5173
   ```
3. Open `http://localhost:5173`.

## Deka stub testing
Open:
```
http://localhost:5173/?demo=deka
```

The stub loads a single CommonJS handler module and invokes `fetch()` on it.

## PHPX in-browser testing
Open:
```
http://localhost:5173/?demo=phpx
```

This path executes PHPX through the runtime adapter layer in `@deka/wosix-js`.
The browser entry (`main.js`) does not import or instantiate `php-rs` directly;
all runtime access goes through adapter APIs.

Boundary and guardrails are tracked in:
`tasks/PHPX-WOSIX-RUNTIME-BOUNDARY.md`.

## Node WASM testing
Place a `node.wasm` file next to `index.html` and open:
```
http://localhost:5173/?node=wasm
```

The current integration loads the module but does not execute Node yet; this is
scaffolded for wiring in `js/NODE_WASM.md`.

## Expected output
The log panel should show:
```
Mounted /index.js
hello from wosix node shim
```
