//! deka/sqlite Module
//!
//! SQLite database with Bun-compatible API
//!
//! Features:
//! - File-based and in-memory databases (`:memory:`)
//! - Prepared statements for performance
//! - Parameterized queries (? placeholders)
//! - Type-safe conversions (JSON ↔ SQLite)

mod db;
mod ops;

deno_core::extension!(
    deka_sqlite,
    ops = [
        ops::op_sqlite_open,
        ops::op_sqlite_close,
        ops::op_sqlite_prepare,
        ops::op_sqlite_query_all,
        ops::op_sqlite_query_get,
        ops::op_sqlite_execute,
    ],
    esm_entry_point = "ext:deka_sqlite/sqlite.js",
    esm = [ dir "src/modules/sqlite", "sqlite.js" ],
);

/// Register the SQLite module extension
pub fn register_ops() -> deno_core::Extension {
    deka_sqlite::init_ops_and_esm()
}
