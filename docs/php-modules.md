# PHP Module System (php_modules + .phpx)

This document defines the intended behavior for the PHP module system in deka.

## Goals
- Keep default PHP behavior intact for normal `.php` files.
- Add a separate, ESM-style module system for `.phpx`.
- Expose a single, explicit bridge to userland via `php_modules/deka.php`.
- Allow privileged APIs only inside `.phpx`.
- Make tree-shaking possible by relying on static `import`/`export`.

## Non-goals
- No Composer-style autoloading inside `php_modules/`.
- No implicit scanning of userland `.php` for module exports.

## Terms
- **Userland**: traditional `.php` executed by the runtime.
- **Module world**: `.phpx` files under `php_modules/`.
- **Bridge**: `php_modules/deka.php`, the only public entry for userland.
- **Privileged APIs**: host-backed APIs available only to `.phpx`.

## Runtime behavior
1) The runtime checks for `php_modules/deka.php`.
   - If missing, fail fast with a message that suggests running `deka init`.
2) The runtime auto-includes `php_modules/deka.php` before user code.
   - This is the only magic. No other autoload system exists.
3) The runtime pre-processes `php_modules/` (compile + cache).
   - `.phpx` files are parsed and compiled into a module registry.
   - The registry maps module exports to callable functions.

## Module resolution (phpx only)
- Relative paths: `import { foo } from './string.phpx'`
- Package-style: `import { foo } from 'string'` resolves to
  `php_modules/string/index.phpx`
- Import/export is only valid in `.phpx`.
- Re-exports are supported: `export { foo } from './bar.phpx'`.

## Bridge behavior (deka.php)
`deka.php` defines public namespaces and functions that delegate to `.phpx`.

Example sketch:
```php
<?php
// php_modules/deka.php
namespace Deka\String;

function get_str_func($value) {
    return __deka_call('string/get_str_func', [$value]);
}
```

The `__deka_call` builtin is only available inside the bridge or in `.phpx`.

## Privileged APIs
Only `.phpx` can access privileged host APIs. Normal `.php` cannot.

Initial categories (Node-like):
- fs
- net/http
- process
- env
- timers
- crypto

## Error behavior
- Missing `php_modules/deka.php` => hard error + `deka init` hint.
- `import`/`export` in `.php` => syntax error.
- Privileged API from `.php` => runtime error.

## Build/dev pipeline (intended)
- Dev run: compile `.phpx` on first use with a file-hash cache.
- Build: precompile `.phpx` and ship a module registry alongside runtime.

## Example layout
```
project/
  index.php
  php_modules/
    deka.php
    string/
      index.phpx
```

## Prototype notes (current)
- Module discovery scans `php_modules/` for `.phpx` files at runtime.
- .phpx supports `import { foo } from './bar.phpx'` and `export function foo()`.
- Imports are validated and a simple dependency graph is built (cycles error).
- .phpx exports compile into global functions for now (php-rs namespaces are not ready).
- The bridge file `php_modules/deka.php` is auto-included but should not redeclare exported names yet.
- The JSON extension is now provided in `php_modules/json/index.phpx` with a pure phpx parser/encoder.
- Result helpers live in `php_modules/core/result.phpx`, and `json_decode_result` uses them.
- Core primitives are now available in `php_modules/core/` (reader, byte, bytes, num).
- phpx function signatures may include type annotations, which are stripped at runtime (see `PHPX_TYPES.md`).
