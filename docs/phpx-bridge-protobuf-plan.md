# PHPX Bridge Protobuf Migration Plan

## Goal
- Migrate PHPX host bridge transport from ad-hoc JSON payloads to Protobuf while keeping user-facing PHPX APIs stable (`db/*`, `fs`, `tcp`, `tls`).
- Keep bridge internals private (`__deka_*` not userland API).
- Preserve runtime execution model (no build step required for app code).

## Scope
- In scope: `db`, `fs`, `net` bridge kinds, Rust ops, JS host shim, PHPX core modules, tests/docs.
- Out of scope: changing public module function names/signatures during migration.

## Rollout Strategy
- Domain-by-domain cutover with fallback compatibility.
- First domain: `db`.
- Keep JSON fallback temporarily until parity tests pass, then remove.

## Tasks

### Phase 1: Schema and Generation
- [x] 1. Define bridge envelope and `db` action messages in `.proto`.
- [ ] 2. Add Rust protobuf codegen (build-time generation in `modules_php`).
- [ ] 3. Add JS protobuf encode/decode support in `deka_php/php.js`.
- [ ] 4. Document versioning policy for backward-compatible schema evolution.

### Phase 2: DB Bridge Migration
- [ ] 5. Implement Protobuf decode/dispatch in `op_php_db_call`.
- [ ] 6. Implement Protobuf encode responses for `open/query/query_one/exec/tx/close/stats`.
- [ ] 7. Keep temporary JSON fallback path behind runtime feature flag.
- [ ] 8. Add parity tests (`json` vs `protobuf`) for `postgres/mysql/sqlite`.

### Phase 3: FS Bridge Migration
- [ ] 9. Add `fs` schema messages (`open/read/write/close/read_file/write_file`).
- [ ] 10. Migrate `op_php_fs_call` to Protobuf transport.
- [ ] 11. Add binary integrity tests for file bytes round-trip.

### Phase 4: Net Bridge Migration
- [ ] 12. Add `net` schema messages (`connect/read/write/tls_upgrade/close/deadline`).
- [ ] 13. Migrate `op_php_net_call` to Protobuf transport.
- [ ] 14. Add TCP/TLS protocol sanity tests.

### Phase 5: Cleanup and Hardening
- [ ] 15. Remove JSON bridge fallback paths.
- [ ] 16. Add bridge fuzz/safety tests for malformed payloads.
- [ ] 17. Add bridge metrics for decode/encode overhead and payload size.
- [ ] 18. Update AGENTS/docs with mandatory validation test commands.

## Validation Gate Per Task
- Run `cargo check -p modules_php`.
- Run targeted runtime tests for affected bridge domain.
- Commit after passing checks.
