# PHPX DB System Plan

## Goals
- Keep PHPX user experience clean and typed.
- Keep host/runtime primitives loosely coupled from any single database.
- Ship `postgres` first for `linkhash`.
- Prove the architecture can support `sqlite` and `mysql` next.

## Architecture
- Host layer: generic DB capability endpoint.
- PHPX layer:
  - `db` package provides generic primitives.
  - canonical driver packages live under `db/*` (`db/postgres`, `db/mysql`, `db/sqlite`).
- Bridge:
  - PHPX public packages call internal modules only.
  - internal modules call the runtime host bridge.
  - Host handles action dispatch and returns JSON-compatible payloads.

## Host Contract (v1)
- `open`: `{ driver, config } -> { ok, handle, reused }`
- `query`: `{ handle, sql, params } -> { ok, rows }`
- `exec`: `{ handle, sql, params } -> { ok, affected_rows }`
- `begin`: `{ handle } -> { ok }`
- `commit`: `{ handle } -> { ok }`
- `rollback`: `{ handle } -> { ok }`
- `close`: `{ handle } -> { ok }`

Errors use:
- `{ ok: false, error: string }`

## Implementation Status (Current)
- Implemented: `op_php_db_call` host action bridge in `crates/modules_php/src/modules/php/mod.rs`.
- Implemented: host driver routes:
  - `postgres` (sync `postgres` crate)
  - `sqlite` (sync `rusqlite` crate)
  - `mysql` (sync `mysql` crate)
- Implemented: PHPX packages:
  - `php_modules/db` (generic primitives)
  - `php_modules/db/postgres` (driver wrapper)
  - `php_modules/db/sqlite` (driver wrapper)
  - `php_modules/db/mysql` (driver wrapper)
  - `php_modules/postgres|sqlite|mysql` retained as compatibility proxies
- Implemented: prefixed exports to avoid import alias requirements in current parser:
  - `db_open`, `db_query`, `db_exec`, `db_begin`, `db_commit`, `db_rollback`, `db_close`
  - `pg_connect`, `pg_query`, `pg_query_one`, `pg_exec`, `pg_begin`, `pg_commit`, `pg_rollback`, `pg_close`
- Verified with `deka run` smoke scripts:
  - sqlite end-to-end `connect/exec/query/close` succeeds.
  - mysql module loads and returns structured errors when query fails.
  - postgres powers live `linkhash` API routes.

## PHPX API (v1)
- `db`:
  - `open(driver, config): Result<DbOpenOk, string>`
  - `query(handle, sql, params): Result<DbQueryOk, string>`
  - `exec(handle, sql, params): Result<DbExecOk, string>`
  - `begin/commit/rollback/close(handle): Result<DbUnitOk, string>`
- `postgres`:
  - Canonical: `connect/query/query_one/exec/begin/commit/rollback/close`
  - Compatibility aliases: `pg_connect/pg_query/pg_query_one/pg_exec/pg_begin/pg_commit/pg_rollback/pg_close`
  - Both route to the same implementation; unprefixed is preferred for PHPX module-style usage.

## PHPX Usage Example
```php
import { connect, query_one, close } from 'postgres';
import { result_is_ok } from 'core/result';

$conn = connect({
  host: '127.0.0.1',
  port: 5432,
  database: 'app_db',
  user: 'postgres',
  password: 'secret'
});

if (!result_is_ok($conn)) {
  echo $conn->error;
  return;
}

$row = query_one($conn->value, 'select 1 as ok');
if (result_is_ok($row)) {
  echo 'ok';
}

close($conn->value);
```

## Linkhash Integration Pattern
- Keep Linkhash-specific repository code as a project module under `linkhash/php_modules/linkhash_db`.
- Depend only on `postgres` package from that module.
- Return `Result` for connect/query_one style calls, and arrays for list-returning queries.

## Pooling/Reuse
- Host keeps process-local connection state keyed by normalized config.
- `open` reuses existing handle when the same key is already active.
- v1 is intentionally simple to validate UX and runtime behavior quickly.

## Why This Is Loosely Coupled
- No `postgres_*` host op surface.
- Host op is action-based and driver-keyed.
- Driver-specific behavior remains inside driver package and host action handlers.
- Same contract can be extended for `sqlite` and `mysql`.

## Next Steps
1. Add statement prepare/cache and richer type decoding.
2. Add metrics/introspection for active handles and query timings.
3. Expand `linkhash_db` from read-path methods to full write-path repository methods.
