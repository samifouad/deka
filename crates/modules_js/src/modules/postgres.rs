//! deka/postgres module
//!
//! Provides PostgreSQL database operations
//!
//! TypeScript usage:
//! ```typescript
//! import { query, execute } from 'deka/postgres';
//!
//! const users = await query('SELECT * FROM users WHERE id = $1', [userId]);
//! await execute('INSERT INTO logs (message) VALUES ($1)', ['hello']);
//! ```

use deno_core::{error::CoreError, op2};

deno_core::extension!(
    deka_postgres,
    ops = [
        op_postgres_query,
        op_postgres_execute,
    ],
    esm_entry_point = "ext:deka_postgres/postgres.js",
    esm = [ dir "src/modules/postgres", "postgres.js" ],
);

/// Register all postgres operations
pub fn register_ops() -> deno_core::Extension {
    deka_postgres::init_ops_and_esm()
}

/// Execute a query and return rows
#[op2(async)]
#[serde]
async fn op_postgres_query(
    #[string] sql: String,
    #[serde] params: Vec<serde_json::Value>,
) -> Result<Vec<serde_json::Value>, CoreError> {
    let message = format!(
        "Postgres module not implemented for query: {} ({} params)",
        sql,
        params.len()
    );
    Err(CoreError::from(std::io::Error::new(
        std::io::ErrorKind::Other,
        message,
    )))
}

/// Execute a statement (INSERT, UPDATE, DELETE)
#[op2(async)]
#[bigint]
async fn op_postgres_execute(
    #[string] sql: String,
    #[serde] params: Vec<serde_json::Value>,
) -> Result<u64, CoreError> {
    let message = format!(
        "Postgres module not implemented for execute: {} ({} params)",
        sql,
        params.len()
    );
    Err(CoreError::from(std::io::Error::new(
        std::io::ErrorKind::Other,
        message,
    )))
}
