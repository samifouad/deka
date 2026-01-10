// Minimal PHP runtime module - no heavy dependencies

use deno_core::op2;
use std::collections::HashMap;

/// Embedded PHP WASM binary produced by the `php-rs` crate.
static PHP_WASM_BYTES: &[u8] = include_bytes!(concat!(env!("OUT_DIR"), "/php_rs.wasm"));

#[op2]
#[buffer]
fn op_php_get_wasm() -> Vec<u8> {
    PHP_WASM_BYTES.to_vec()
}

#[op2]
#[buffer]
fn op_php_read_file_sync(#[string] path: String) -> Result<Vec<u8>, deno_core::error::CoreError> {
    std::fs::read(&path).map_err(|e| {
        deno_core::error::CoreError::from(std::io::Error::new(
            e.kind(),
            format!("Failed to read file '{}': {}", path, e),
        ))
    })
}

#[op2]
#[serde]
fn op_php_read_env() -> HashMap<String, String> {
    std::env::vars().collect()
}

#[op2]
#[string]
fn op_php_cwd() -> Result<String, deno_core::error::CoreError> {
    std::env::current_dir()
        .map(|p| p.to_string_lossy().to_string())
        .map_err(|e| deno_core::error::CoreError::from(e))
}

#[op2(fast)]
fn op_php_file_exists(#[string] path: String) -> bool {
    std::path::Path::new(&path).exists()
}

#[op2]
#[string]
fn op_php_path_resolve(#[string] base: String, #[string] path: String) -> String {
    let base_path = std::path::Path::new(&base);
    let target_path = std::path::Path::new(&path);

    let resolved = if target_path.is_absolute() {
        target_path.to_path_buf()
    } else {
        base_path.join(target_path)
    };

    resolved.to_string_lossy().to_string()
}

deno_core::extension!(
    php_core,
    ops = [
        op_php_get_wasm,
        op_php_read_file_sync,
        op_php_read_env,
        op_php_cwd,
        op_php_file_exists,
        op_php_path_resolve,
    ],
    esm_entry_point = "ext:php_core/php.js",
    esm = [dir "src/modules/php", "php.js"],
);

pub fn init() -> deno_core::Extension {
    php_core::init_ops_and_esm()
}
