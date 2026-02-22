# Test 001: Wasmtime + HTTP Bridge

This folder proves we can build the slim `php-wasm` binary, emit a WASI module, and execute a PHP-like script with Wasmtime. It also ships a tiny HTTP bridge that defaults to running the native `php` binary so we can avoid the Wasmtime locals limit.

## 1. Build the WASM binary

```bash
cargo build --bin php-wasm --target wasm32-wasip1 --no-default-features --features wasm-target --release
```

The optimized output lives at `target/wasm32-wasip1/release/php-wasm.wasm`. Debug builds currently hit the Wasm-local limit and cannot be executed by Wasmtime, so prefer release mode.

## 2. Run the script directly

```bash
wasmtime target/wasm32-wasip1/debug/php-wasm.wasm -- tests/php/test-001/hello.php
```

If you prefer a helper:

```bash
./tests/php/test-001/run-wasm.sh
```

## 3. Spin up an HTTP router (Rust)

```bash
PHP_DOC_ROOT=tests/php/test-001 \\
cargo run --bin php-router --features router
```

The router now defaults to `PHP_ROUTER_MODE=native` and executes the native `php` binary (`target/release/php` by default, or override with `PHP_NATIVE_BIN`). Set `PHP_ROUTER_MODE=wasm` to reconnect the Wasmtime path; it still prefers `target/wasm32-wasip1/release/php-wasm.wasm` unless `PHP_WASM_PATH` is set.

By default the router listens on `http://localhost:8541` and proxies any `GET /foo.php` to the native `php` binary (switch to `PHP_ROUTER_MODE=wasm` to route through Wasmtime/php-wasm instead). For example:

```bash
curl http://localhost:8541/hello.php
curl http://localhost:8541/another.php
```

The router prevents directory traversal, rejects non-`.php` files, and returns Wasmtime's stdout/stderr (logged via `tracing`) in the HTTP response. Set `PHP_ROUTER_WASMTIME` if you need a custom `wasmtime` executable path.

## Notes

- Build `php-wasm` in release mode if you plan to run the router under `PHP_ROUTER_MODE=wasm`; debug builds still exceed Wasmtime's locals limit.
- The router now runs `target/release/php` by default, so no Wasmtime binaries are needed unless you explicitly switch modes. Set `RUST_LOG=php_router=info` if you want to see request or Wasmtime logging.
 
## 4. Compare official PHP vs php-router behavior

Use `bun` from the repo root to drive the entire `tests/` directory so you can see which files match PHP’s output and which need attention:

```bash
bun tests/php/testrunner.js
```

Pass a directory path if you want to narrow the run (e.g., `bun tests/php/testrunner.js tests/php/basic_examples`). The runner compares the official PHP CLI (`PHP_BIN`) against `target/release/php` (or `PHP_NATIVE_BIN`) on every `.php` file it discovers.

- `PHP_BIN` (defaults to `php`) selects the official CLI to compare against.
- `PHP_NATIVE_BIN` overrides the target release/debug binary path when it is built elsewhere.
- The runner prints each file name, reports mismatches, and exits non-zero when any script diverges so you can inspect the printed stdout/stderr differences.
