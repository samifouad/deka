# Deka MVP

Deka MVP is the production-facing CLI/runtime workspace for PHP/PHPX execution, build, serve, and tooling.

## Release build

```sh
cargo build --release -p cli
```

Primary binary:
- `target/release/cli` (invoked as `deka` via local alias/symlink)

## Deliverables

### Dev deliverables
- `target/release/cli`
- Rust crate artifacts under `target/release/deps` and `target/release/*.rlib` (build-only; not shipped)

### Prod deliverables (ship list)
Minimum package:
- `target/release/cli`

Optional co-shipped utilities (only if your distribution needs them):
- `target/release/php`
- `target/release/php-fpm`
- `target/release/wit-phpx`

Do not ship:
- `target/release/deps/*`
- `target/release/*.d`
- `target/release/*.rlib`
- `target/release/build/*`
- `target/release/.fingerprint/*`

## Notes
- LSP is exposed via `deka lsp` from the main CLI path.
- Build policy is release-first to avoid debug/release drift.
