/**
 * deka/sqlite - SQLite database (Bun-compatible API)
 *
 * File-based and in-memory SQLite databases with prepared statements.
 *
 * Example:
 *   import { Database } from 'deka/sqlite'
 *
 *   // File-based database
 *   const db = new Database('mydb.sqlite')
 *
 *   // In-memory database
 *   const memdb = new Database(':memory:')
 *   const memdb2 = new Database() // also in-memory
 *
 *   // Prepare and execute queries
 *   const query = db.query('SELECT * FROM users WHERE name = ?')
 *   const users = query.all('Alice')
 *   const user = query.get('Bob')
 *
 *   // Insert/Update/Delete
 *   const insert = db.query('INSERT INTO users (name, email) VALUES (?, ?)')
 *   const result = insert.run('Charlie', 'charlie@example.com')
 *   console.log(result.lastInsertRowid, result.changes)
 */

// Statement class - represents a prepared SQL statement
class Statement {
  constructor(db, stmtId, sql) {
    this.db = db
    this.stmtId = stmtId
    this.sql = sql
  }

  // Execute query and return all rows
  all(...params) {
    return Deno.core.ops.op_sqlite_query_all(this.stmtId, params)
  }

  // Execute query and return first row (or undefined)
  get(...params) {
    return Deno.core.ops.op_sqlite_query_get(this.stmtId, params)
  }

  // Execute statement and return metadata {changes, lastInsertRowid}
  run(...params) {
    return Deno.core.ops.op_sqlite_execute(this.stmtId, params)
  }

  // Return expanded SQL with bound parameters (for debugging)
  toString() {
    return this.sql
  }
}

// Database class - main interface for SQLite operations
class Database {
  constructor(filename = ':memory:', options = {}) {
    // Default options
    const opts = {
      readonly: options.readonly || false,
      create: options.create !== undefined ? options.create : true,
      readwrite: options.readwrite || false,
    }

    // Create database synchronously (Bun-compatible)
    this.id = Deno.core.ops.op_sqlite_open(filename, opts)

    // Cache for prepared statements (query() caches, prepare() doesn't)
    this._queryCache = new Map()
  }

  // Prepare and cache a statement (Bun behavior: caches statements)
  query(sql) {
    // Check cache first
    if (this._queryCache.has(sql)) {
      return this._queryCache.get(sql)
    }

    // Prepare statement
    const stmtId = Deno.core.ops.op_sqlite_prepare(this.id, sql)
    const stmt = new Statement(this, stmtId, sql)

    // Cache it
    this._queryCache.set(sql, stmt)

    return stmt
  }

  // Prepare statement without caching (for one-off queries)
  prepare(sql) {
    const stmtId = Deno.core.ops.op_sqlite_prepare(this.id, sql)
    return new Statement(this, stmtId, sql)
  }

  // Execute SQL directly without returning results
  run(sql) {
    const stmt = this.prepare(sql)
    return stmt.run()
  }

  // Bun-compatible alias for run()
  exec(sql) {
    return this.run(sql)
  }

  // Close database connection
  close() {
    if (this.id !== null) {
      Deno.core.ops.op_sqlite_close(this.id)
      this.id = null
      this._queryCache.clear()
    }
  }
}

export { Database, Statement }

// Also expose as global for handler code
globalThis.__dekaSqlite = { Database, Statement }
