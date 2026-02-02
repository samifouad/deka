# PHPX DX + Syntax Summary

This is a concise, developer-facing summary of the PHPX experience and the
syntax additions we have introduced or locked in. It is meant to help future
contributors quickly understand the current behavior.

## File types + runtime behavior
- `.php`: Classic PHP. Module semantics only if the file starts with `import`.
- `.phpx`: PHPX mode is always on. Import/export is required for cross-module use.
- `.d.phpx`: Typed stub files (used for WASM/WIT stubs and IDE/typing only).

Runtime notes:
- `php_modules/deka.php` is auto-included by the runtime.
- Request prelude (superglobals, headers, body, etc.) is shared between `.php` and `.phpx`.
- Module prelude diverges: `.phpx` (or `.php` with top-level `import`) adds module
  registry, import wrappers, unused-import checks, and JSX auto-runtime injection.
- `.phpx` entry files execute inside `namespace __phpx_entry` to avoid leaking globals;
  the runtime injects a global prelude block (`namespace { ... }`) followed by an
  entry namespace block that contains the import wrappers plus user code.
- PHPX modules are compiled into per-module namespaces and register their
  exports with `__phpx_register`.
- `phpx_import($moduleId, $name)` loads a module and returns an export.
- A hand-maintained stdlib map is used for `.php` compatibility; `.phpx` should
  import stdlib functions explicitly.

## Module system (PHPX)
- `import { foo } from './bar.phpx'` and `export function foo()` are supported.
- Exports are explicit; non-exported functions are private to the module.
- Unused imports are rejected at runtime compile time.
- `.php` files opt in by placing `import` at the very top of the file.

## Objects + dot access
PHPX introduces JS-style object literals and tight-dot access:

```php
$cfg = { host: 'localhost', port: 5432 };
$host = $cfg.host;
```

Rules:
- Dot access only when there is no whitespace and RHS is an identifier.
- Keys: identifiers or quoted strings only (no computed keys yet).
- Dot access works for object literals and structs (not classes).
- `get_class()` returns `stdClass` for object literals and the struct name for structs.
- `property_exists()` checks object-literal keys and struct fields (including promoted).
- `method_exists()` works for structs; object literals always return false.
- `count()` returns the number of keys/fields for object literals and structs.

## Structs (value semantics)
Structs are nominal value types with Rust-style syntax:

```php
struct Position {
    $x: int = 0;
    $y: int = 0;
}

$p1 = Position { $x: 1, $y: 2 };
$p2 = $p1;
$p2.x++;
```

Notes:
- Structs reuse class metadata but are not regular PHP objects.
- `===` compares struct values, not handles.
- Struct composition uses `use` inside the struct body.
- `__construct` is not allowed in PHPX structs (use literals instead).
- `new` is not allowed in PHPX; use struct literals (`Point { ... }`).
- Shorthand field init is allowed: `Point { $x, $y }`.
- Struct field defaults must be constant expressions.

## Types + inference (PHPX)
PHPX has a strict type checker with hybrid nominal/structural behavior.

Key rules:
- Safe widening only (e.g. int -> float). No other implicit coercions.
- `Object<{...}>` is structural; `struct` is nominal.
- `type Name = ...;` creates compile-time-only aliases.
- Generic aliases and type params use Go-style constraints: `T: Reader`.
- `array<T>` is the canonical generic array spelling.

## Option/Result + panic (no exceptions in PHPX)
PHPX replaces exceptions with enums and explicit error handling.

```php
enum Result {
    case Ok(mixed $value);
    case Err(mixed $error);
}

enum Option {
    case Some(mixed $value);
    case None;
}
```

- `throw` and `try/catch` are banned in PHPX.
- `panic($message)` aborts execution (provided by `php_modules/deka.php`).
- `Option::unwrap()` / `Result::unwrap()` call `panic(...)` on failure.
- `null` literals and nullable types (`?T`, `T|null`) are rejected in PHPX.

## Enums + match
- Enums support payload cases: `enum Msg { case Text(string $body); }`.
- `match` is exhaustively checked when possible.
- Enum case access supports payload field narrowing within match arms.

## JSX + components (PHPX)
- JSX is PHPX-only and lowers to `jsx/jsxs` calls from `component/core`.
- JSX auto-injects the runtime import; user code should not import `jsx/jsxs`.
- `<>...</>` lowers to a special fragment tag.
- JSX outputs VNode values (renderer lives in `component/dom`).
- `{ ... }` accepts any PHPX expression (no statements). Object literals use `{ { ... } }`.

## PHP interoperability
- PHP can call PHPX exports via `phpx_import` or bridged helpers.
- PHPX compiles to valid PHP, and types are stripped before execution.
- Runtime values remain standard PHP values at the boundary.
- Boundary coercions are lenient for legacy PHP:
  - `null` -> `Option::None` for `Option<T>` params.
  - Arrays/stdClass are accepted for `Object`/object-shape + `struct` params
    (extra keys are ignored).
  - `Option<T>` return -> `null` (None) or inner value (Some).
  - `Result<T,E>` return -> `T` (Ok) or `['ok' => false, 'error' => ...]` (Err).

## Not supported in PHPX (by design)
- `class`, `trait`, `extends`, `implements`
- `namespace` and top-level `use` (use `import` instead)
- `throw` and `try/catch`

## Related docs
- `PHPX_TYPES.md` for detailed type mapping and rules.
- `docs/php-modules.md` for module system details.
- `docs/phpx-upgrade-plan.md` for phased status.
