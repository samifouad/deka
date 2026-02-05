# PHPX Fixtures

This folder contains runtime fixtures for PHPX-only syntax.
Use the testrunner to execute `.phpx` scripts against the Deka PHP runtime.

## Test Header Requirement
Every PHPX fixture must start with a short comment block describing what the test covers.
This keeps intent obvious and makes regressions easy to diagnose.

Example header:
```
/*
TEST: Short title
Covers: feature list, behavior expectations, edge cases.
*/
```
For frontmatter files (`---`), place the header inside the frontmatter block.

## Running
```
bun tests/phpx/testrunner.js
bun tests/phpx/testrunner.js tests/phpx/objects
bun tests/phpx/testrunner.js tests/phpx --skip=tests/phpx/modules/import_export.phpx
```

When you need module-aware PHPX execution (imports/exports), run through the Deka CLI:
```
PHPX_BIN=target/debug/cli PHPX_BIN_ARGS=run bun tests/phpx/testrunner.js tests/phpx/modules
```

For PHP <-> PHPX bridge fixtures (PHP files that import PHPX modules), run:
```
PHPX_BIN=target/release/cli PHPX_BIN_ARGS=run bun tests/phpx/bridge/testrunner.js
```

## Mandatory When
After any major runtime or parser/compiler change, run the full suite:
```
PHPX_BIN=target/debug/cli PHPX_BIN_ARGS=run bun tests/phpx/testrunner.js
```

## Expectations
Each `.phpx` file can have optional sidecar expectation files:
- `.out`: expected stdout
- `.err`: expected stderr
- `.code`: expected exit code (integer)

If no expectation files exist, the test passes as long as the exit code is 0.

Directories prefixed with `_` are skipped.

### Regex expectations
If an expectation file starts with `re:`, the remainder is treated as a JavaScript regex.
Example:
```
re:^Exception\\(Handle\\(\\d+\\)\\)$
```

## Conformance Checklist
See `tests/phpx/CONFORMANCE.md` for the feature-to-fixture mapping and remaining gaps.
