const STORAGE_KEY = "__adwa_sqlite_store_v1";
const handles = new Map();
let nextHandle = 1;
const metrics = {
    opens: 0,
    queries: 0,
    queryMsTotal: 0,
    execs: 0,
    execMsTotal: 0,
};
export function handleDbBridge(kind, action, payload) {
    const op = action.trim().toLowerCase();
    if (op === "stats") {
        const byDriver = {};
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
    if (op === "list") {
        return { ok: true, databases: Object.keys(loadStore()).sort() };
    }
    if (op === "reset") {
        for (const handle of handles.values()) {
            try {
                handle.db.close();
            }
            catch { }
        }
        handles.clear();
        persistStore({});
        return { ok: true };
    }
    if (op === "open") {
        const config = asRecord(payload.config) ?? {};
        const driver = normalizeDriver(kind, payload.driver);
        const dbName = dbNameFromConfig(config, driver);
        const sql = requireSqlJsModule();
        const store = loadStore();
        const encoded = store[dbName];
        const db = encoded ? new sql.Database(base64ToBytes(encoded)) : new sql.Database();
        if (!encoded) {
            store[dbName] = bytesToBase64(db.export());
            persistStore(store);
        }
        const id = `h_${nextHandle++}`;
        handles.set(id, { id, driver, dbName, db, dirty: false });
        metrics.opens += 1;
        return { ok: true, handle: id, driver, db: dbName };
    }
    const handleId = requireString(payload.handle, "handle");
    const handle = handles.get(handleId);
    if (!handle) {
        throw new Error(`unknown db handle '${handleId}'`);
    }
    if (op === "close") {
        flushHandle(handle);
        handle.db.close();
        handles.delete(handleId);
        return { ok: true };
    }
    if (op === "begin") {
        handle.db.run("begin");
        return { ok: true };
    }
    if (op === "rollback") {
        handle.db.run("rollback");
        handle.dirty = true;
        flushHandle(handle);
        return { ok: true };
    }
    if (op === "commit") {
        handle.db.run("commit");
        handle.dirty = true;
        flushHandle(handle);
        return { ok: true };
    }
    const sqlText = requireString(payload.sql, "sql");
    const params = toParams(payload.params);
    if (op === "query") {
        const t0 = Date.now();
        const rows = runQuery(handle.db, sqlText, params);
        metrics.queries += 1;
        metrics.queryMsTotal += Date.now() - t0;
        return { ok: true, rows };
    }
    if (op === "exec") {
        const t0 = Date.now();
        handle.db.run(sqlText, params);
        handle.dirty = true;
        const affectedRows = readChanges(handle.db);
        metrics.execs += 1;
        metrics.execMsTotal += Date.now() - t0;
        flushHandle(handle);
        return { ok: true, affected_rows: affectedRows };
    }
    throw new Error(`unknown db action '${action}'`);
}
function requireSqlJsModule() {
    if (globalThis.__adwaSqlJsModule)
        return globalThis.__adwaSqlJsModule;
    const init = globalThis.initSqlJs;
    if (typeof init !== "function") {
        throw new Error("sql.js loader missing. Ensure sql-wasm.js is loaded.");
    }
    if (!globalThis.__adwaSqlJsPromise) {
        globalThis.__adwaSqlJsPromise = init({
            locateFile: (file) => new URL(`./${file}`, import.meta.url).toString(),
        }).then((mod) => {
            globalThis.__adwaSqlJsModule = mod;
            return mod;
        });
        throw new Error("sqlite engine initializing, retry in a moment.");
    }
    throw new Error("sqlite engine still initializing, retry in a moment.");
}
function readChanges(db) {
    try {
        const out = db.exec("select changes() as changes");
        if (!Array.isArray(out) || out.length === 0)
            return 0;
        const first = out[0];
        if (!first || !Array.isArray(first.values) || first.values.length === 0)
            return 0;
        return Number(first.values[0]?.[0] ?? 0) || 0;
    }
    catch {
        return 0;
    }
}
function runQuery(db, sql, params) {
    const stmt = db.prepare(sql);
    try {
        stmt.bind(params);
        const rows = [];
        while (stmt.step()) {
            rows.push(stmt.getAsObject());
        }
        return rows;
    }
    finally {
        stmt.free();
    }
}
function flushHandle(handle) {
    if (!handle.dirty)
        return;
    const store = loadStore();
    store[handle.dbName] = bytesToBase64(handle.db.export());
    persistStore(store);
    handle.dirty = false;
}
function dbNameFromConfig(config, driver) {
    const direct = config.database ?? config.name ?? config.db;
    if (typeof direct === "string" && direct.trim())
        return direct.trim();
    if (driver === "sqlite")
        return "adwa_demo";
    return `${driver}_default`;
}
function normalizeDriver(kind, driverValue) {
    const driver = String(driverValue || kind || "sqlite").toLowerCase();
    if (driver === "mysql")
        return "mysql";
    if (driver === "postgres" || driver === "postgresql" || driver === "pg")
        return "postgres";
    return "sqlite";
}
function requireString(value, field) {
    if (typeof value !== "string")
        throw new Error(`db payload field '${field}' must be a string`);
    const trimmed = value.trim();
    if (!trimmed)
        throw new Error(`db payload field '${field}' cannot be empty`);
    return trimmed;
}
function toParams(value) {
    if (value == null)
        return [];
    if (!Array.isArray(value))
        throw new Error("db payload field 'params' must be an array");
    return value;
}
function asRecord(value) {
    if (!value || typeof value !== "object" || Array.isArray(value))
        return null;
    return value;
}
function loadStore() {
    try {
        const raw = localStorage.getItem(STORAGE_KEY);
        if (!raw)
            return {};
        const parsed = JSON.parse(raw);
        if (!parsed || typeof parsed !== "object" || Array.isArray(parsed))
            return {};
        const out = {};
        for (const [k, v] of Object.entries(parsed)) {
            if (typeof v === "string" && k)
                out[k] = v;
        }
        return out;
    }
    catch {
        return {};
    }
}
function persistStore(store) {
    try {
        localStorage.setItem(STORAGE_KEY, JSON.stringify(store));
    }
    catch (err) {
        throw new Error(`failed to persist sqlite store: ${err instanceof Error ? err.message : String(err)}`);
    }
}
function bytesToBase64(bytes) {
    let bin = "";
    for (let i = 0; i < bytes.length; i += 1) {
        bin += String.fromCharCode(bytes[i]);
    }
    return btoa(bin);
}
function base64ToBytes(encoded) {
    const bin = atob(encoded);
    const out = new Uint8Array(bin.length);
    for (let i = 0; i < bin.length; i += 1) {
        out[i] = bin.charCodeAt(i);
    }
    return out;
}
