# PHP Module Examples

This folder contains small fixtures for the `.phpx` module system.

## modules
- Purpose: baseline module loading from `php_modules/`.
- Run: `deka run examples/php/modules/index.php`
- Expected: `value: something`

## modules-import
- Purpose: package-style import (`import { foo } from 'string'`).
- Run: `deka run examples/php/modules-import/index.php`
- Expected: `value: something`

## modules-cycle
- Purpose: cycle detection across `.phpx` imports.
- Run: `deka run examples/php/modules-cycle/index.php`
- Expected: error `Cyclic phpx import detected: a`

## modules-missing
- Purpose: missing export validation.
- Run: `deka run examples/php/modules-missing/index.php`
- Expected: error `Missing export 'missing' in 'bar' (imported by 'foo').`

## modules-types
- Purpose: type-stripping in `.phpx` function signatures.
- Run: `deka run examples/php/modules-types/index.php`
- Expected: `5` then `hi deka`

## modules-reexport
- Purpose: re-export support (`export { foo } from ...`).
- Run: `deka run examples/php/modules-reexport/index.php`
- Expected: `7`

## json
- Purpose: JSON decode/encode/validate with the phpx parser.
- Run: `deka run examples/php/json/index.php`
