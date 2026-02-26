---
category: "php-wasm"
categoryLabel: "General"
categoryOrder: 90
---
# PHP Wasm Extensions (Host-Managed, Per-Request)

This document describes the current (MVP) Wasm extension ABI for PHP/PHPX
running inside the V8 host. The host loads user Wasm modules and provides
per-request isolation (a fresh instance per call).

## High-level flow
1) PHP/PHPX imports a Wasm module:
   - `import { greet } from '@user/hello' as wasm;`
2) The PHP runtime invokes an internal host bridge call (not exposed for userland use).
3) The host instantiates the Wasm module, calls the export, and returns the
   JSON result back to PHP.

## Module layout
```
php_modules/
  @user/
    hello/
      deka.json
      module.wasm
      module.wit   # reserved for future WIT/component model typing
      module.d.phpx # optional type stubs for PHPX tooling
```

### deka.json
```json
{
  "module": "module.wasm",
  "wit": "module.wit",
  "instance": "per_request",
  "abi": "deka-json",
  "records": "struct",
  "world": "hello",
  "interfacePrefix": false,
  "stubs": "module.d.phpx"
}
```

**Runtime fields (used by the host loader):**
- `module`: wasm filename (default `module.wasm`).
- `instance`: only `per_request` is supported right now.
- `abi`: currently `deka-json` (JSON-in/JSON-out). WIT/component model will
  become the default later. `abi: "wit"` is supported with a limited type set
  (see `docs/php/php-wasm-wit.md`).

**Tooling fields (used by stub generator + PHPX typecheck):**
- `wit`: WIT file path (for planned component model + stub generation).
- `records`: `struct` or `object` for WIT record mapping in stubs.
- `world`: select a WIT world when multiple exist.
- `interfacePrefix`: when false, exported interface functions are flattened.
- `stubs`: output path for `.d.phpx` (relative to module root).
- `crate`: wasm guest crate directory (used by `deka wasm build`).
- `crateName`: wasm guest crate name (used by `deka wasm build`).

## Rust example (JSON ABI)

### Cargo.toml
```toml
[package]
name = "hello"
version = "0.1.0"
edition = "2021"

[lib]
crate-type = ["cdylib"]

[dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
```

### src/lib.rs
```rust
use serde::Deserialize;

#[repr(C)]
struct WasmResult {
    ptr: u32,
    len: u32,
}

#[no_mangle]
pub extern "C" fn deka_alloc(size: u32) -> *mut u8 {
    let mut buf = Vec::with_capacity(size as usize);
    let ptr = buf.as_mut_ptr();
    std::mem::forget(buf);
    ptr
}

#[no_mangle]
pub extern "C" fn deka_free(ptr: *mut u8, size: u32) {
    if ptr.is_null() || size == 0 {
        return;
    }
    unsafe {
        drop(Vec::from_raw_parts(ptr, size as usize, size as usize));
    }
}

fn read_string(ptr: *const u8, len: u32) -> String {
    if ptr.is_null() || len == 0 {
        return String::new();
    }
    let bytes = unsafe { std::slice::from_raw_parts(ptr, len as usize) };
    String::from_utf8_lossy(bytes).to_string()
}

fn write_result(value: &str) -> *mut WasmResult {
    let bytes = value.as_bytes();
    let ptr = deka_alloc(bytes.len() as u32);
    unsafe {
        std::ptr::copy_nonoverlapping(bytes.as_ptr(), ptr, bytes.len());
    }
    let result = Box::new(WasmResult {
        ptr: ptr as u32,
        len: bytes.len() as u32,
    });
    Box::into_raw(result)
}

#[derive(Deserialize)]
struct CallArgs(Vec<String>);

#[no_mangle]
pub extern "C" fn deka_call(
    name_ptr: *const u8,
    name_len: u32,
    args_ptr: *const u8,
    args_len: u32,
) -> *mut WasmResult {
    let name = read_string(name_ptr, name_len);
    let args_json = read_string(args_ptr, args_len);
    let args: Vec<String> = serde_json::from_str(&args_json)
        .unwrap_or_default();

    let result = match name.as_str() {
        "greet" => {
            let who = args.get(0).cloned().unwrap_or_else(|| "world".to_string());
            format!("Hello, {}!", who)
        }
        _ => "unknown export".to_string(),
    };

    let json = serde_json::to_string(&result).unwrap_or_else(|_| "null".to_string());
    write_result(&json)
}
```

### Build
```sh
cargo build --release --target wasm32-unknown-unknown
```
Copy `target/wasm32-unknown-unknown/release/hello.wasm` to
`php_modules/@user/hello/module.wasm`.

### PHP usage
```php
import { greet } from '@user/hello' as wasm;

echo greet('Sami'); // "Hello, Sami!"
```

## Notes
- The host runs a fresh Wasm instance **per call**, so state does not leak.
- The current ABI uses JSON for arguments/results; it is simple and stable,
  but not the final design.
- WIT/component model support will replace the JSON ABI in a later phase;
  keep `module.wit` alongside your wasm module so we can migrate cleanly.
- Planned WIT/component model typing is documented in `docs/php/php-wasm-wit.md`.

## Developer workflow
If you want the canonical ABI (`abi: "wit"`), use the WIT workflow documented
in `docs/php/php-wasm-wit.md` (recommended, includes `deka wasm` CLI).

For the JSON ABI (`abi: "deka-json"`), the minimal flow is:
1) Provide `module.wasm` + `deka.json` under `php_modules/@user/<name>/`.
2) Import in PHPX: `import { fn } from '@user/name' as wasm;`
3) The module is instantiated per call and receives JSON args.

## Example crate in-repo
- Rust source: `examples/wasm_hello_rust/`
- Build script: `scripts/build-wasm-hello.sh`
