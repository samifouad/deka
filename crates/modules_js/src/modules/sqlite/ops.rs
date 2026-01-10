//! SQLite Operations
//!
//! Deno ops for database operations

use super::db::{DatabaseOptions, REGISTRY};
use deno_core::{error::CoreError, op2};
use rusqlite::params_from_iter;
use std::io::{Error as IoError, ErrorKind};

/// Open a database
#[op2]
#[smi]
pub fn op_sqlite_open(
    #[string] path: String,
    #[serde] options: DatabaseOptions,
) -> Result<u32, CoreError> {
    Ok(REGISTRY.open_database(&path, options)?)
}

/// Close a database
#[op2(async)]
pub async fn op_sqlite_close(#[smi] db_id: u32) -> Result<(), CoreError> {
    Ok(REGISTRY.close_database(db_id)?)
}

/// Prepare a statement
#[op2(fast)]
#[smi]
pub fn op_sqlite_prepare(#[smi] db_id: u32, #[string] sql: String) -> Result<u32, CoreError> {
    Ok(REGISTRY.prepare_statement(db_id, sql)?)
}

/// Execute a query and return all rows
#[op2]
#[serde]
pub fn op_sqlite_query_all(
    #[smi] stmt_id: u32,
    #[serde] params: Vec<serde_json::Value>,
) -> Result<Vec<serde_json::Value>, CoreError> {
    let stmt_info = REGISTRY.get_statement(stmt_id)?;
    let db = REGISTRY.get_database(stmt_info.db_id)?;

    let conn = db.conn.lock().unwrap();
    let mut stmt = conn.prepare(&stmt_info.sql).map_err(|e| {
        IoError::new(
            ErrorKind::Other,
            format!("Failed to prepare statement: {}", e),
        )
    })?;

    // Convert params to rusqlite values
    let rusqlite_params: Vec<rusqlite::types::Value> =
        params.iter().map(json_to_sqlite_value).collect();

    let column_count = stmt.column_count();
    let column_names: Vec<String> = (0..column_count)
        .map(|i| stmt.column_name(i).unwrap_or("").to_string())
        .collect();

    let rows = stmt
        .query_map(params_from_iter(rusqlite_params.iter()), |row| {
            let mut obj = serde_json::Map::new();
            for (i, name) in column_names.iter().enumerate() {
                let value = sqlite_value_to_json(row, i);
                obj.insert(name.clone(), value);
            }
            Ok(serde_json::Value::Object(obj))
        })
        .map_err(|e| IoError::new(ErrorKind::Other, format!("Query failed: {}", e)))?;

    let mut results = Vec::new();
    for row in rows {
        results.push(row.map_err(|e| IoError::new(ErrorKind::Other, format!("Row error: {}", e)))?);
    }

    Ok(results)
}

/// Execute a query and return first row
#[op2]
#[serde]
pub fn op_sqlite_query_get(
    #[smi] stmt_id: u32,
    #[serde] params: Vec<serde_json::Value>,
) -> Result<Option<serde_json::Value>, CoreError> {
    let stmt_info = REGISTRY.get_statement(stmt_id)?;
    let db = REGISTRY.get_database(stmt_info.db_id)?;

    let conn = db.conn.lock().unwrap();
    let mut stmt = conn.prepare(&stmt_info.sql).map_err(|e| {
        IoError::new(
            ErrorKind::Other,
            format!("Failed to prepare statement: {}", e),
        )
    })?;

    // Convert params to rusqlite values
    let rusqlite_params: Vec<rusqlite::types::Value> =
        params.iter().map(json_to_sqlite_value).collect();

    let column_count = stmt.column_count();
    let column_names: Vec<String> = (0..column_count)
        .map(|i| stmt.column_name(i).unwrap_or("").to_string())
        .collect();

    let result = stmt.query_row(params_from_iter(rusqlite_params.iter()), |row| {
        let mut obj = serde_json::Map::new();
        for (i, name) in column_names.iter().enumerate() {
            let value = sqlite_value_to_json(row, i);
            obj.insert(name.clone(), value);
        }
        Ok(serde_json::Value::Object(obj))
    });

    match result {
        Ok(row) => Ok(Some(row)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(IoError::new(ErrorKind::Other, format!("Query failed: {}", e)).into()),
    }
}

/// Execute a statement (INSERT/UPDATE/DELETE) and return metadata
#[op2]
#[serde]
pub fn op_sqlite_execute(
    #[smi] stmt_id: u32,
    #[serde] params: Vec<serde_json::Value>,
) -> Result<serde_json::Value, CoreError> {
    let stmt_info = REGISTRY.get_statement(stmt_id)?;
    let db = REGISTRY.get_database(stmt_info.db_id)?;

    let conn = db.conn.lock().unwrap();
    let mut stmt = conn.prepare(&stmt_info.sql).map_err(|e| {
        IoError::new(
            ErrorKind::Other,
            format!("Failed to prepare statement: {}", e),
        )
    })?;

    // Convert params to rusqlite values
    let rusqlite_params: Vec<rusqlite::types::Value> =
        params.iter().map(json_to_sqlite_value).collect();

    let changes = stmt
        .execute(params_from_iter(rusqlite_params.iter()))
        .map_err(|e| IoError::new(ErrorKind::Other, format!("Execute failed: {}", e)))?;

    let last_insert_rowid = conn.last_insert_rowid();

    Ok(serde_json::json!({
        "changes": changes,
        "lastInsertRowid": last_insert_rowid,
    }))
}

/// Convert JSON value to SQLite value
fn json_to_sqlite_value(value: &serde_json::Value) -> rusqlite::types::Value {
    match value {
        serde_json::Value::Null => rusqlite::types::Value::Null,
        serde_json::Value::Bool(b) => rusqlite::types::Value::Integer(if *b { 1 } else { 0 }),
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                rusqlite::types::Value::Integer(i)
            } else if let Some(f) = n.as_f64() {
                rusqlite::types::Value::Real(f)
            } else {
                rusqlite::types::Value::Null
            }
        }
        serde_json::Value::String(s) => rusqlite::types::Value::Text(s.clone()),
        _ => rusqlite::types::Value::Text(value.to_string()),
    }
}

/// Convert SQLite row value to JSON
fn sqlite_value_to_json(row: &rusqlite::Row, index: usize) -> serde_json::Value {
    use rusqlite::types::ValueRef;

    match row.get_ref(index).unwrap() {
        ValueRef::Null => serde_json::Value::Null,
        ValueRef::Integer(i) => serde_json::Value::Number(i.into()),
        ValueRef::Real(f) => serde_json::json!(f),
        ValueRef::Text(s) => {
            let text = std::str::from_utf8(s).unwrap_or("");
            serde_json::Value::String(text.to_string())
        }
        ValueRef::Blob(b) => {
            // Convert blob to base64
            serde_json::Value::String(base64_encode(b))
        }
    }
}

/// Simple base64 encoding for blobs
fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = Vec::new();

    for chunk in data.chunks(3) {
        let mut buf = [0u8; 3];
        for (i, &byte) in chunk.iter().enumerate() {
            buf[i] = byte;
        }

        result.push(CHARS[((buf[0] >> 2) & 0x3F) as usize]);
        result.push(CHARS[(((buf[0] << 4) | (buf[1] >> 4)) & 0x3F) as usize]);

        if chunk.len() > 1 {
            result.push(CHARS[(((buf[1] << 2) | (buf[2] >> 6)) & 0x3F) as usize]);
        } else {
            result.push(b'=');
        }

        if chunk.len() > 2 {
            result.push(CHARS[(buf[2] & 0x3F) as usize]);
        } else {
            result.push(b'=');
        }
    }

    String::from_utf8(result).unwrap()
}
