//! deka/t4 Module
//!
//! HTTP client for T4 object storage with S3-compatible API.
//!
//! Features:
//! - Lazy file references (no network call until read/write)
//! - JWT authentication
//! - GET/PUT/DELETE/HEAD operations
//! - Works with any T4 server (t4.deka.gg, self-hosted, etc.)

mod client;
mod ops;

deno_core::extension!(
    deka_t4,
    ops = [
        ops::op_t4_create_client,
        ops::op_t4_get_text,
        ops::op_t4_get_buffer,
        ops::op_t4_put,
        ops::op_t4_delete,
        ops::op_t4_exists,
        ops::op_t4_stat,
    ],
    esm_entry_point = "ext:deka_t4/t4.js",
    esm = [ dir "src/modules/t4", "t4.js" ],
);

/// Register the T4 module extension
pub fn register_ops() -> deno_core::Extension {
    deka_t4::init_ops_and_esm()
}
