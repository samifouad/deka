# PHPX Fixtures

This folder contains runtime fixtures for PHPX-only syntax.
Use the testrunner to execute `.phpx` scripts against the Deka PHP runtime.

## Running
```
bun tests/phpx/testrunner.js
bun tests/phpx/testrunner.js tests/phpx/objects
bun tests/phpx/testrunner.js tests/phpx --skip=tests/phpx/modules/import_export.phpx
```

## Expectations
Each `.phpx` file can have optional sidecar expectation files:
- `.out`: expected stdout
- `.err`: expected stderr
- `.code`: expected exit code (integer)

If no expectation files exist, the test passes as long as the exit code is 0.

Directories prefixed with `_` are skipped.
