//! SQLite Database Management
//!
//! Manages database connections and prepared statements

use rusqlite::{Connection, OpenFlags};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::io::{Error as IoError, ErrorKind};
use std::sync::{Arc, Mutex};

/// Database open options
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseOptions {
    #[serde(default)]
    pub readonly: bool,
    #[serde(default = "default_true")]
    pub create: bool,
    #[serde(default)]
    pub readwrite: bool,
}

fn default_true() -> bool {
    true
}

impl Default for DatabaseOptions {
    fn default() -> Self {
        Self {
            readonly: false,
            create: true,
            readwrite: false,
        }
    }
}

/// Managed database connection
pub struct SqliteDatabase {
    pub conn: Mutex<Connection>,
}

impl SqliteDatabase {
    pub fn open(path: &str, options: DatabaseOptions) -> Result<Self, IoError> {
        // Build open flags based on options
        let flags = if options.readonly {
            OpenFlags::SQLITE_OPEN_READ_ONLY
        } else if options.readwrite {
            OpenFlags::SQLITE_OPEN_READ_WRITE
        } else if options.create {
            OpenFlags::SQLITE_OPEN_READ_WRITE | OpenFlags::SQLITE_OPEN_CREATE
        } else {
            OpenFlags::SQLITE_OPEN_READ_WRITE
        };

        let conn = Connection::open_with_flags(path, flags).map_err(|e| {
            IoError::new(ErrorKind::Other, format!("Failed to open database: {}", e))
        })?;

        Ok(Self {
            conn: Mutex::new(conn),
        })
    }
}

/// Managed prepared statement
pub struct SqliteStatement {
    pub db_id: u32,
    pub sql: String,
}

/// Global database registry
pub struct DatabaseRegistry {
    databases: Mutex<HashMap<u32, Arc<SqliteDatabase>>>,
    statements: Mutex<HashMap<u32, Arc<SqliteStatement>>>,
    next_db_id: Mutex<u32>,
    next_stmt_id: Mutex<u32>,
}

impl DatabaseRegistry {
    pub fn new() -> Self {
        Self {
            databases: Mutex::new(HashMap::new()),
            statements: Mutex::new(HashMap::new()),
            next_db_id: Mutex::new(1),
            next_stmt_id: Mutex::new(1),
        }
    }

    /// Open a database and return its ID
    pub fn open_database(&self, path: &str, options: DatabaseOptions) -> Result<u32, IoError> {
        let db = Arc::new(SqliteDatabase::open(path, options)?);

        let mut next_id = self.next_db_id.lock().unwrap();
        let db_id = *next_id;
        *next_id += 1;

        let mut databases = self.databases.lock().unwrap();
        databases.insert(db_id, db);

        Ok(db_id)
    }

    /// Get a database by ID
    pub fn get_database(&self, db_id: u32) -> Result<Arc<SqliteDatabase>, IoError> {
        let databases = self.databases.lock().unwrap();
        databases.get(&db_id).cloned().ok_or_else(|| {
            IoError::new(
                ErrorKind::NotFound,
                format!("Invalid database ID: {}", db_id),
            )
        })
    }

    /// Close and remove a database
    pub fn close_database(&self, db_id: u32) -> Result<(), IoError> {
        let mut databases = self.databases.lock().unwrap();
        databases.remove(&db_id).ok_or_else(|| {
            IoError::new(
                ErrorKind::NotFound,
                format!("Invalid database ID: {}", db_id),
            )
        })?;
        Ok(())
    }

    /// Prepare a statement and return its ID
    pub fn prepare_statement(&self, db_id: u32, sql: String) -> Result<u32, IoError> {
        // Verify database exists
        let _ = self.get_database(db_id)?;

        let stmt = Arc::new(SqliteStatement { db_id, sql });

        let mut next_id = self.next_stmt_id.lock().unwrap();
        let stmt_id = *next_id;
        *next_id += 1;

        let mut statements = self.statements.lock().unwrap();
        statements.insert(stmt_id, stmt);

        Ok(stmt_id)
    }

    /// Get a statement by ID
    pub fn get_statement(&self, stmt_id: u32) -> Result<Arc<SqliteStatement>, IoError> {
        let statements = self.statements.lock().unwrap();
        statements.get(&stmt_id).cloned().ok_or_else(|| {
            IoError::new(
                ErrorKind::NotFound,
                format!("Invalid statement ID: {}", stmt_id),
            )
        })
    }
}

// Global registry instance
lazy_static::lazy_static! {
    pub static ref REGISTRY: DatabaseRegistry = DatabaseRegistry::new();
}
