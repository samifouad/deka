# PHPX ORM Plan

## Goals

- [ ] Use PHPX `struct` models as the schema source of truth.
- [ ] Avoid a separate Prisma-like schema DSL.
- [ ] Generate app-owned ORM client files into project `db/` (not `php_modules/`).
- [ ] Support Drizzle-style query builder DX.
- [ ] Support first-class IDE feedback (parser, validator, LSP).

## Canonical Flow

- [ ] User defines models in `types/index.phpx` (or another file).
- [x] User runs `deka db generate types/index.phpx` or `deka db generate types/`.
- [x] CLI resolves/validates model input and generates initial `db/*`.
- [x] App imports generated client via `@/db`.
- [x] User applies migrations with `deka db migrate`.
- [x] User inspects generation status with `deka db info` (initial implementation).

## Generated Layout

- [x] `db/index.phpx` public client entrypoint (initial).
- [x] `db/client.phpx` generated client scaffold.
- [x] `db/meta.phpx` generated model metadata (initial).
- [x] `db/migrations/*.sql` migration files (initial `0001_init.sql` generation).
- [x] `db/.generated/*` internal generated helpers (initial `schema.json`).
- [x] `db/_state.json` generator state (initial).
- [x] Generated PHPX files include `AUTO-GENERATED` warning header.

## CLI Commands

- [x] `deka db generate [path]` implemented for initial artifact generation.
- [x] `deka db migrate` implemented for initial Postgres SQL migration apply.
- [x] `deka db info` implemented for generation + migration status summary.
- [x] `deka db flush` (dev-only) command scaffold added.
- [x] Implement generation/migration/info/flush internals (initial Postgres + filesystem state implementation).

## Generate Path Resolution Contract

- [x] File input uses exact file.
- [x] Directory input resolves `<dir>/index.phpx`.
- [x] Missing input defaults to `types/index.phpx` when present.
- [x] Invalid path yields actionable error with attempted resolution.
- [x] Prefer `.phpx` for model entry.

## Language + Tooling

- [x] Add struct field annotations (`@id`, `@unique`, etc.) in PHPX parser.
- [x] Store annotations in AST as metadata (not runtime object lowering).
- [x] Add annotation validation rules (unknown, duplicate, target mismatch, bad args).
- [x] Add LSP completions/signature help/hover for annotations.
- [x] Add LSP diagnostics for annotation errors (via typechecker diagnostics pipeline).
- [x] Add `@/` alias support in typechecker and LSP.
- [x] Add `@/` alias support in runtime resolver and CLI.

## Drizzle-Style Client DX

- [x] `db.insert(Model)->values(...)->returning()->one()`
- [x] `db.select()->from(Model)->where(...)->many()`
- [x] `db.update(Model)->set(...)->where(...)->returning()->many()`
- [x] `db.delete(Model)->where(...)->exec()`
- [x] Generated client includes runnable non-fluent helpers: `selectMany`, `selectOne`, `insertOne`, `updateWhere`, `deleteWhere`, `transaction`.
- [x] Predicates/helpers scaffolded in generated client: `eq`, `and`, `or` (+ shape placeholders for richer query builder).
- [x] Predicates/helpers: `asc`, `desc`, `limit`, `offset`.
- [x] Transaction API (non-fluent helper in generated client).
- [x] Bound client supports fluent API via `createClient($meta, $handle)` and rebinding via `withHandle($handle)`.

## Rollout

- [x] Phase 1: CLI command scaffolding (`db generate/migrate/info/flush`) and path resolution.
- [x] Phase 2: Annotation parser/AST support + parser tests.
- [ ] Phase 3: Validation + LSP support for annotations and model relations.
- [ ] Phase 4: ORM IR + Postgres migration generator.
- [x] Phase 5: Generated client implementation + `@/db` import path.
- [x] Phase 6: Linkhash migration to generated client.
- [ ] Phase 7: Test hardening across parser/validation/generator/CLI/runtime.

## Acceptance Criteria

- [x] `deka db generate types/` resolves and validates `types/index.phpx`.
- [x] Generated client imports via `@/db` in app code.
- [x] `deka db migrate` applies pending migrations and records state.
- [ ] LSP shows clear diagnostics/completions for model annotations.
- [x] Linkhash reads/writes via generated Postgres client in primary flows.
