# MVP2 JS Runtime Plan

Status: Planned
Owner: Runtime

## Mission
Move PHPX execution to a JS-first runtime: transpile PHPX to JS and execute via ESM with Rust ops providing host capabilities.

## Goals
- PHPX -> JS is the default execution model.
- ESM module loading is the runtime source of truth.
- Rust ops provide the host boundary for non-standard behavior.
- Dev: on-the-fly transpile + cache. Prod: build output only.

## Non-goals
- Keep PHPX/wasm execution in the hot path.
- Reintroduce Node/Bun compatibility layers.
- Add a parallel JS stdlib tree (we transpile PHPX stdlib instead).

## Workstreams
1. **Runtime prelude for JS execution**
   - Provide `bridge(...)` + core helpers expected by transpiled stdlib.
   - Map ops to existing Rust host behavior.

2. **JS module loading**
   - ESM loader in runtime consumes transpiled JS.
   - Cache transpiled output in `php_modules/.cache` and project `.cache`.

3. **Transpilation pipeline**
   - Use PHPX -> JS emit for all modules (stdlib + user code).
   - Ensure module graph integrity and hashing remains stable.

4. **Dev workflow**
   - Hot transpile on file change.
   - Clear errors if transpile fails.

5. **Build workflow**
   - `deka build` emits JS bundles for server/client.
   - Single output file for server/client with SWC bundler.

6. **Compatibility + tests**
   - Conformance tests updated to run via JS runtime.
   - E2E tests for router + components + stdlib.

## Acceptance Criteria
- `deka run app/main.phpx` executes through JS runtime path by default.
- `deka serve app/main.phpx` serves via JS runtime path.
- Stdlib modules execute via transpiled JS + ops.
- Bundle outputs run under Node without missing stdlib symbols (where supported).

## Verification
- `cargo build --release -p cli`
- PHPX conformance suite: `PHPX_BIN=target/release/cli PHPX_BIN_ARGS=run bun tests/phpx/testrunner.js`
- At least one JS runtime E2E (router + stdlib JSON).
