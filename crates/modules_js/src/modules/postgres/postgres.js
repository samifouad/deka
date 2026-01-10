/**
 * deka/postgres - PostgreSQL operations
 */

function query(sql, params = []) {
  return Deno.core.ops.op_postgres_query(sql, params)
}

function execute(sql, params = []) {
  return Deno.core.ops.op_postgres_execute(sql, params)
}

globalThis.__dekaPostgres = { query, execute }

export { query, execute }
