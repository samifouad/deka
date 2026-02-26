# WIT hello (planned)

This folder shows the **planned** WIT/component-model shape for PHPX wasm
extensions. The runtime still uses the JSON ABI today; these files are for
previewing the DX and generating type stubs.

The example is structured like a real project:
- `php_modules/@user/hello/` contains `deka.json`, `module.wit`, `module.d.phpx`,
  and a placeholder `module.wasm` (replace it with a real build).
- `app.phpx` imports from `@user/hello` and calls the wasm functions.

## Generate PHPX stubs
```sh
cargo run -p wit-phpx -- php_modules/@user/hello/module.wit \
  --out php_modules/@user/hello/module.d.phpx \
  --records struct --no-interface-prefix
```

By default, exported interfaces are prefixed (e.g. `api__greet`). Use
`--no-interface-prefix` for flatter names.

## Import from PHPX
```php
import { greet, get_position } from '@user/hello' as wasm;

$pos = get_position();
echo $pos.x;
```

## Notes
- `module.d.phpx` is type-only.
- `module.wasm` is a placeholder to satisfy validation; replace it with a real build.
- When `abi: "wit"` is supported, the same `module.wit` will drive runtime bindings.
