export type DbBridgePayload = Record<string, unknown>;

type DbDriver = "sqlite" | "postgres" | "mysql";
type DbRow = Record<string, unknown>;
type DbTable = {
  columns: string[];
  rows: DbRow[];
  autoInc: number;
};
type DbDatabase = {
  tables: Record<string, DbTable>;
};
type DbHandle = {
  id: string;
  driver: DbDriver;
  dbName: string;
  txSnapshot: DbDatabase | null;
};

const STORAGE_KEY = "__adwa_phpx_db_v1";
const handles = new Map<string, DbHandle>();
let nextHandle = 1;
let store: Record<string, DbDatabase> = loadStore();
const metrics = {
  opens: 0,
  queries: 0,
  queryMsTotal: 0,
  execs: 0,
  execMsTotal: 0,
};

export function handleDbBridge(kind: string, action: string, payload: DbBridgePayload): unknown {
  const op = action.trim().toLowerCase();

  if (op === "open") {
    const config = asRecord(payload.config) ?? {};
    const driver = normalizeDriver(kind, payload.driver);
    const dbName = dbNameFromConfig(config, driver);
    ensureDatabase(dbName);
    const id = `h_${nextHandle++}`;
    handles.set(id, { id, driver, dbName, txSnapshot: null });
    metrics.opens += 1;
    return { ok: true, handle: id, driver, db: dbName };
  }

  if (op === "stats") {
    const byDriver: Record<string, number> = {};
    for (const handle of handles.values()) {
      byDriver[handle.driver] = (byDriver[handle.driver] || 0) + 1;
    }
    return {
      ok: true,
      active_handles: handles.size,
      handles_by_driver: byDriver,
      query_calls: metrics.queries,
      query_avg_ms: metrics.queries > 0 ? Math.round(metrics.queryMsTotal / metrics.queries) : 0,
      exec_calls: metrics.execs,
      exec_avg_ms: metrics.execs > 0 ? Math.round(metrics.execMsTotal / metrics.execs) : 0,
    };
  }

  const handleId = requireString(payload.handle, "handle");
  const handle = handles.get(handleId);
  if (!handle) {
    throw new Error(`unknown db handle '${handleId}'`);
  }

  if (op === "close") {
    handles.delete(handleId);
    return { ok: true };
  }

  if (op === "begin") {
    if (!handle.txSnapshot) {
      handle.txSnapshot = cloneDb(ensureDatabase(handle.dbName));
    }
    return { ok: true };
  }

  if (op === "rollback") {
    if (handle.txSnapshot) {
      store[handle.dbName] = cloneDb(handle.txSnapshot);
      handle.txSnapshot = null;
      persistStore();
    }
    return { ok: true };
  }

  if (op === "commit") {
    handle.txSnapshot = null;
    persistStore();
    return { ok: true };
  }

  const sql = requireString(payload.sql, "sql");
  const params = toParams(payload.params);
  const db = ensureDatabase(handle.dbName);

  if (op === "query") {
    const t0 = Date.now();
    const rows = sqlQuery(db, sql, params);
    metrics.queries += 1;
    metrics.queryMsTotal += Date.now() - t0;
    return { ok: true, rows };
  }

  if (op === "exec") {
    const t0 = Date.now();
    const affected = sqlExec(db, sql, params);
    metrics.execs += 1;
    metrics.execMsTotal += Date.now() - t0;
    persistStore();
    return { ok: true, affected_rows: affected };
  }

  throw new Error(`unknown db action '${action}'`);
}

function sqlExec(db: DbDatabase, rawSql: string, params: unknown[]): number {
  const sql = normalizeSql(rawSql);
  if (sql.length === 0) return 0;

  const create = sql.match(/^create\s+table\s+(?:if\s+not\s+exists\s+)?([a-zA-Z_][a-zA-Z0-9_]*)\s*\((.*)\)$/i);
  if (create) {
    const tableName = create[1];
    const colParts = splitCsv(create[2]);
    const columns = colParts
      .map((part) => part.trim().split(/\s+/)[0])
      .filter((col) => /^[a-zA-Z_][a-zA-Z0-9_]*$/.test(col));
    if (!db.tables[tableName]) {
      db.tables[tableName] = { columns: [...new Set(columns)], rows: [], autoInc: 1 };
      return 1;
    }
    return 0;
  }

  const insert = sql.match(/^insert\s+into\s+([a-zA-Z_][a-zA-Z0-9_]*)\s*(?:\(([^)]*)\))?\s*values\s*\((.*)\)$/i);
  if (insert) {
    const tableName = insert[1];
    const table = db.tables[tableName] ?? (db.tables[tableName] = { columns: [], rows: [], autoInc: 1 });
    const columns = insert[2] ? splitCsv(insert[2]).map((s) => s.trim()) : [...table.columns];
    const values = splitCsv(insert[3]);
    const row: DbRow = {};
    const cursor = { i: 0 };
    for (let i = 0; i < values.length; i += 1) {
      const col = columns[i] || `col_${i + 1}`;
      row[col] = parseValueToken(values[i], params, cursor);
      if (!table.columns.includes(col)) table.columns.push(col);
    }
    if (table.columns.includes("id") && (row.id === undefined || row.id === null || row.id === "")) {
      row.id = table.autoInc++;
    }
    table.rows.push(row);
    return 1;
  }

  const update = sql.match(/^update\s+([a-zA-Z_][a-zA-Z0-9_]*)\s+set\s+(.+?)(?:\s+where\s+(.+))?$/i);
  if (update) {
    const table = db.tables[update[1]];
    if (!table) return 0;
    const assignments = splitCsv(update[2]);
    const where = update[3] ? parseWhere(update[3], params) : null;
    const cursor = { i: 0 };
    let affected = 0;
    for (const row of table.rows) {
      if (where && !matchWhere(row, where)) continue;
      for (const assignment of assignments) {
        const m = assignment.match(/^([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*(.+)$/);
        if (!m) continue;
        const col = m[1];
        row[col] = parseValueToken(m[2], params, cursor);
        if (!table.columns.includes(col)) table.columns.push(col);
      }
      affected += 1;
    }
    return affected;
  }

  const del = sql.match(/^delete\s+from\s+([a-zA-Z_][a-zA-Z0-9_]*)(?:\s+where\s+(.+))?$/i);
  if (del) {
    const table = db.tables[del[1]];
    if (!table) return 0;
    const where = del[2] ? parseWhere(del[2], params) : null;
    if (!where) {
      const count = table.rows.length;
      table.rows = [];
      return count;
    }
    const keep: DbRow[] = [];
    let removed = 0;
    for (const row of table.rows) {
      if (matchWhere(row, where)) removed += 1;
      else keep.push(row);
    }
    table.rows = keep;
    return removed;
  }

  throw new Error(`unsupported exec SQL in adwa db bridge: ${rawSql}`);
}

function sqlQuery(db: DbDatabase, rawSql: string, params: unknown[]): DbRow[] {
  const sql = normalizeSql(rawSql);
  const from = sql.match(/^select\s+(.+?)\s+from\s+([a-zA-Z_][a-zA-Z0-9_]*)(.*)$/i);
  if (from) {
    const fieldsRaw = from[1].trim();
    const tableName = from[2];
    const tail = from[3] || "";
    const table = db.tables[tableName];
    if (!table) return [];

    const whereMatch = tail.match(/\bwhere\s+(.+?)(?=\border\s+by\b|\blimit\b|$)/i);
    const orderMatch = tail.match(/\border\s+by\s+([a-zA-Z_][a-zA-Z0-9_]*)(?:\s+(asc|desc))?/i);
    const limitMatch = tail.match(/\blimit\s+(\d+)/i);

    const where = whereMatch ? parseWhere(whereMatch[1], params) : null;
    let rows = table.rows.map((row) => ({ ...row }));
    if (where) rows = rows.filter((row) => matchWhere(row, where));

    if (orderMatch) {
      const col = orderMatch[1];
      const dir = String(orderMatch[2] || "asc").toLowerCase();
      rows.sort((a, b) => compareValues(a[col], b[col]) * (dir === "desc" ? -1 : 1));
    }

    if (limitMatch) {
      rows = rows.slice(0, Number(limitMatch[1]));
    }

    if (fieldsRaw === "*") return rows;
    const fields = splitCsv(fieldsRaw).map((field) => field.trim());
    return rows.map((row) => projectRow(row, fields));
  }

  const literal = sql.match(/^select\s+(.+)$/i);
  if (literal) {
    const fields = splitCsv(literal[1]).map((field) => field.trim());
    const row: DbRow = {};
    const cursor = { i: 0 };
    for (const field of fields) {
      const m = field.match(/^(.+?)\s+as\s+([a-zA-Z_][a-zA-Z0-9_]*)$/i);
      if (m) row[m[2]] = parseValueToken(m[1], params, cursor);
      else row[field] = parseValueToken(field, params, cursor);
    }
    return [row];
  }

  throw new Error(`unsupported query SQL in adwa db bridge: ${rawSql}`);
}

function projectRow(row: DbRow, fields: string[]): DbRow {
  const out: DbRow = {};
  for (const field of fields) {
    const m = field.match(/^([a-zA-Z_][a-zA-Z0-9_]*)(?:\s+as\s+([a-zA-Z_][a-zA-Z0-9_]*))?$/i);
    if (!m) continue;
    const col = m[1];
    const alias = m[2] || col;
    out[alias] = row[col];
  }
  return out;
}

function parseWhere(expr: string, params: unknown[]): { key: string; value: unknown } {
  const m = expr.trim().match(/^([a-zA-Z_][a-zA-Z0-9_]*)\s*=\s*(.+)$/);
  if (!m) throw new Error(`unsupported where expression: ${expr}`);
  const cursor = { i: 0 };
  return { key: m[1], value: parseValueToken(m[2], params, cursor) };
}

function matchWhere(row: DbRow, where: { key: string; value: unknown }): boolean {
  return normalizeCompare(row[where.key]) === normalizeCompare(where.value);
}

function parseValueToken(tokenRaw: string, params: unknown[], cursor: { i: number }): unknown {
  const token = tokenRaw.trim();
  if (token === "?") return params[cursor.i++] ?? null;
  const numbered = token.match(/^\$(\d+)$/);
  if (numbered) return params[Math.max(0, Number(numbered[1]) - 1)] ?? null;
  if (/^'.*'$/.test(token) || /^".*"$/.test(token)) return token.slice(1, -1);
  if (/^null$/i.test(token)) return null;
  if (/^true$/i.test(token)) return true;
  if (/^false$/i.test(token)) return false;
  if (/^-?\d+(?:\.\d+)?$/.test(token)) return Number(token);
  return token;
}

function normalizeSql(sql: string): string {
  return String(sql || "").trim().replace(/;\s*$/, "");
}

function splitCsv(input: string): string[] {
  const out: string[] = [];
  let buf = "";
  let quote: string | null = null;
  for (let i = 0; i < input.length; i += 1) {
    const ch = input[i];
    if (ch === "'" || ch === '"') {
      if (!quote) quote = ch;
      else if (quote === ch) quote = null;
      buf += ch;
      continue;
    }
    if (ch === "," && !quote) {
      out.push(buf.trim());
      buf = "";
      continue;
    }
    buf += ch;
  }
  if (buf.trim().length > 0) out.push(buf.trim());
  return out;
}

function compareValues(a: unknown, b: unknown): number {
  const av = normalizeCompare(a);
  const bv = normalizeCompare(b);
  if (av < bv) return -1;
  if (av > bv) return 1;
  return 0;
}

function normalizeCompare(value: unknown): string | number {
  if (typeof value === "number") return value;
  if (typeof value === "boolean") return value ? 1 : 0;
  if (value === null || value === undefined) return "";
  if (typeof value === "string" && /^-?\d+(?:\.\d+)?$/.test(value)) return Number(value);
  return String(value);
}

function normalizeDriver(kind: string, payloadDriver: unknown): DbDriver {
  const raw = String(payloadDriver ?? kind ?? "sqlite").toLowerCase();
  if (raw.includes("postgres")) return "postgres";
  if (raw.includes("mysql")) return "mysql";
  return "sqlite";
}

function dbNameFromConfig(config: Record<string, unknown>, driver: DbDriver): string {
  const fromConfig = [config.database, config.dbname, config.name, config.path, config.file].find(
    (value) => typeof value === "string" && String(value).trim().length > 0
  );
  if (fromConfig) return String(fromConfig).trim();
  return `${driver}:default`;
}

function ensureDatabase(name: string): DbDatabase {
  const key = String(name || "default");
  if (!store[key]) {
    store[key] = { tables: {} };
    persistStore();
  }
  return store[key];
}

function asRecord(value: unknown): Record<string, unknown> | undefined {
  if (!value || typeof value !== "object" || Array.isArray(value)) return undefined;
  return value as Record<string, unknown>;
}

function toParams(value: unknown): unknown[] {
  if (Array.isArray(value)) return value;
  if (value === null || value === undefined) return [];
  return [value];
}

function requireString(value: unknown, name: string): string {
  if (typeof value !== "string" || value.length === 0) {
    throw new Error(`${name} must be a non-empty string`);
  }
  return value;
}

function cloneDb<T>(value: T): T {
  if (typeof structuredClone === "function") return structuredClone(value);
  return JSON.parse(JSON.stringify(value));
}

function loadStore(): Record<string, DbDatabase> {
  if (typeof localStorage === "undefined") return {};
  try {
    const raw = localStorage.getItem(STORAGE_KEY);
    if (!raw) return {};
    const parsed = JSON.parse(raw);
    if (!parsed || typeof parsed !== "object") return {};
    return parsed as Record<string, DbDatabase>;
  } catch {
    return {};
  }
}

function persistStore(): void {
  if (typeof localStorage === "undefined") return;
  try {
    localStorage.setItem(STORAGE_KEY, JSON.stringify(store));
  } catch {
    // Ignore quota/storage errors in browser demo mode.
  }
}
