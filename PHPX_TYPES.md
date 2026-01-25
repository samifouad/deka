# PHPX Types and Interop

This document describes how phpx types map to runtime PHP values and how they
interact with the Rust/WASM bridge.

## Design goals
- phpx types are compile-time only (TypeScript-style).
- phpx compiles to valid PHP for execution in the runtime.
- Runtime values stay compatible with normal PHP userland.

## Current compiler behavior
- Type annotations are stripped from phpx function signatures before execution.
- This applies to parameter types and return types.
- Inline variable annotations are stripped:
  - `int $count = 1;` → `$count = 1;`
  - `$count: int = 1;` → `$count = 1;`
- Typed properties are stripped:
  - `public int $count;` → `public $count;`

## Runtime mapping (PHP)
- `int`, `float`, `string`, `bool`, `array`, `object`, `callable`, `mixed`,
  `null` map directly to PHP equivalents.
- `byte` is a phpx-only alias for an integer in the range 0..255.
- `Option<T>` is represented as `T|null` (no runtime enforcement).
- `Result<T, E>` is represented as a tagged array:
  - ok: `['ok' => true, 'value' => <T>]`
  - err: `['ok' => false, 'error' => <E>]`

## Rust/WASM bridge impact
- All values crossing the boundary are still PHP values.
- phpx types are not enforced across the boundary; they exist for tooling.
- `byte` should be treated as a u8 in host ops, but is passed as a PHP int.
- If an op needs strict validation, perform it in the op itself or in a phpx
  wrapper before calling the op.

## PHP interop
- phpx compiles into plain PHP functions.
- Userland PHP can call phpx exports normally (no special casing needed).
- The only magic is auto-including `php_modules/deka.php` before user code.

## TODO
- Add typed properties and inline type annotations to the stripper.
- Introduce a dedicated phpx type-checker (build-time).
- Consider a standard library for runtime validation helpers.
