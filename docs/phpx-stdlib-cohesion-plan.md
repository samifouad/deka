# PHPX Cohesive Stdlib Plan

## Purpose
This plan defines a cohesive PHPX standard library architecture modeled after Go-style module layering:
- explicit low-level primitives
- stable high-level contracts
- internal runtime internals hidden from userland

This is the execution tracker for replacing patchwork internals with first-class user-facing modules.

## Design Goals
- Remove userland dependency on `__deka_wasm_call`.
- Keep runtime internals private/undocumented/internal-only.
- Provide explicit low-level networking and binary building blocks.
- Build database drivers on those building blocks.
- Keep API surface consistent, typed, and composable.

## Target Layering
1. Foundation:
- `bytes`
- `buffer`
- `tcp`
- `tls`
- `encoding/*` (`encoding/json`, `encoding/binary`, future `encoding/toml`, `encoding/cbor`)

2. Database facade:
- `db` contract (`Open`, `Query`, `Exec`, `Begin`, `Commit`, `Rollback`, `Close`)

3. Driver packages:
- `db/postgres`
- `db/mysql`
- `db/sqlite`

4. Runtime internals:
- internal ops and transport primitives only
- no direct userland access to internal bridge hooks

## Required Runtime Ops (Minimum)
1. `tcp_connect(host, port, options) -> socket_id`
2. `tcp_read(socket_id, max_bytes?) -> bytes`
3. `tcp_read_exact(socket_id, n) -> bytes`
4. `tcp_write(socket_id, bytes) -> written`
5. `tcp_close(socket_id)`
6. `tcp_set_deadline(socket_id, millis)`
7. `tls_upgrade(socket_id, options) -> tls_socket_id`
8. `tls_read(tls_socket_id, max_bytes?) -> bytes`
9. `tls_write(tls_socket_id, bytes) -> written`
10. `tls_close(tls_socket_id)`
11. `random_bytes(n) -> bytes`
12. `time_now_millis() -> int`

## Phases

### Phase 0: Guardrails (Security + DX)
1. [x] Make `__deka_wasm_call` inaccessible from non-internal userland modules.
2. [x] Add parser/typechecker hard error for direct use outside internal module roots.
3. [x] Add runtime enforcement guard (defense in depth).
4. [x] Remove/avoid documentation of internal bridge hook.
5. [x] Add tests proving external modules cannot call internal bridge hook.

### Phase 1: Foundation Modules
1. [x] Finalize `bytes` API (read/write primitives, conversion helpers).
2. [x] Implement `buffer` module over `bytes` with cursor/framing helpers.
3. [x] Implement `tcp` module with `Connect` and `TcpSocket` methods.
4. [x] Implement `tls` module with socket upgrade and shared IO API.
5. [x] Add runtime smoke tests for tcp/tls/bytes/buffer.

### Phase 2: Encoding Namespace
1. [x] Add `encoding/json` package (move current `json` surface).
2. [x] Keep temporary compatibility re-export from root `json`.
3. [x] Add `encoding/binary` for endian and framing helpers.
4. [x] Update internal modules to prefer `encoding/json`.
5. [x] Add migration note and deprecation window for root `json`.

### Phase 2A: JSON Migration (Explicit Tracker)
1. [x] Create `php_modules/encoding/json/index.phpx` as canonical JSON API location.
2. [x] Create `php_modules/encoding/json/module.d.phpx` with canonical type declarations.
3. [x] Keep `php_modules/json/index.phpx` as compatibility proxy re-exporting `encoding/json`.
4. [x] Keep `php_modules/json/module.d.phpx` as compatibility proxy declarations.
5. [x] Update runtime-owned modules to import from `encoding/json` only.
6. [x] Update Linkhash and example apps to import from `encoding/json`.
7. [x] Add tests covering both import paths during transition:
- `import { ... } from 'encoding/json'`
- `import { ... } from 'json'`
8. [x] Add warning/deprecation note in docs for root `json`.
9. [x] Define removal milestone for root `json` compatibility module (post-MVP window).

### Phase 3: DB Facade Standardization
1. [x] Define stable `db` facade API and type declarations.
2. [x] Implement shared row/result abstractions.
3. [ ] Normalize parameter and row decode behavior across drivers.
: `db` and postgres wire now normalize params (`null|array|object|scalar` -> positional list). mysql wire parameter parity remains pending.
4. [x] Add contract tests shared by postgres/mysql/sqlite.

### Phase 4: Driver Refactor to Foundation
1. [ ] Implement `db/postgres` on top of `tcp` + `buffer` (+ `tls`).
: Wire path now supports startup/auth/query plus parameterized queries via extended protocol (`Parse/Bind/Execute/Sync`) for `auth=ok|cleartext|md5`, with automatic native fallback for unsupported auth modes (e.g. sasl).
2. [ ] Implement `db/mysql` on top of `tcp` + `buffer` (+ `tls`).
: Initial wire path now supports TCP handshake/auth for `mysql_native_password` and text-protocol `COM_QUERY`, with automatic native fallback for unsupported auth/plugin modes.
3. [x] Keep `db/sqlite` file-backed with same facade contract.
4. [x] Maintain optional native acceleration paths behind same public API.
5. [x] Add driver compliance tests and perf baseline tests.

### Phase 5: Legacy Bridge Retirement
1. [x] Remove userland-facing dependency on bridge-specific packages.
2. [x] Keep internal fallback paths only where strictly needed.
3. [ ] Remove dead bridge wrappers once parity confirmed.
4. [x] Freeze public API contracts and publish migration guide.

## Current State (Snapshot)
1. [x] Generic `db` bridge exists with `postgres/sqlite/mysql` host routes.
2. [x] `postgres`, `sqlite`, `mysql` PHPX wrappers exist.
3. [x] Linkhash uses Postgres package and live data is visible on homepage/API.
4. [x] `tcp`/`tls` foundation modules implemented via internal `__deka_net` host bridge.
5. [x] `buffer` and `bytes` userland foundation modules implemented with smoke tests.
6. [x] `encoding/json` namespace migration completed (compat proxy kept at `json`).
7. [x] `db` facade now exposes shared `open_handle/query_one/rows/affected_rows` helpers.
8. [x] DB contract tests added for postgres/mysql/sqlite wrappers.
9. [x] Internal bridge hook restrictions enforced (typecheck + runtime).
10. [x] Canonical driver module paths now available under `db/postgres`, `db/mysql`, `db/sqlite`.
11. [x] Legacy top-level `postgres`/`mysql`/`sqlite` are compatibility proxy modules.
12. [x] `encoding/binary` now available for endian encode/decode + append/read helpers.

## Validation Requirements Per Phase
1. Unit tests for module APIs.
2. Runtime smoke tests in `tests/phpx`.
3. End-to-end app check (Linkhash route + DB query path).
4. Release build check (`cargo build --release`).
5. Commit only focused files per phase.

## Related Docs
- `docs/phpx-db-plan.md`
- `docs/phpx-upgrade-plan.md`
- `docs/TASKS.md`
- `docs/UNIFIED-TASKS.md`
- `docs/phpx-stdlib-migration.md`
