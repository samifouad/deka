import { createPhpHostBridge } from "../dist/phpx_host_bridge.js";

class MemoryFs {
  #files = new Map();
  #dirs = new Set(["/"]);
  readFile(path) { const k=this.#n(path); const v=this.#files.get(k); if(!v) throw new Error(`not found: ${k}`); return v; }
  writeFile(path,data){ const k=this.#n(path); this.#dirs.add(this.#d(k)); this.#files.set(k,data);} 
  readdir(path){ const b=this.#n(path); const out=new Set(); for(const d of this.#dirs){ if(d!==b&&this.#d(d)===b) out.add(this.#b(d)); } for(const f of this.#files.keys()){ if(this.#d(f)===b) out.add(this.#b(f)); } return [...out].sort(); }
  mkdir(path){ this.#dirs.add(this.#n(path)); }
  rm(path){ const k=this.#n(path); this.#files.delete(k); this.#dirs.delete(k); }
  rename(from,to){ const s=this.#n(from), d=this.#n(to); const v=this.#files.get(s); if(!v) throw new Error(`not found: ${s}`); this.#files.delete(s); this.#files.set(d,v);} 
  stat(path){ const k=this.#n(path); if(this.#files.has(k)) return {size:this.#files.get(k).length,fileType:'file'}; if(this.#dirs.has(k)) return {size:0,fileType:'dir'}; throw new Error(`not found: ${k}`);} 
  #n(path){ const parts=String(path).split('/').filter(Boolean); const stack=[]; for(const p of parts){ if(p==='.') continue; if(p==='..'){ stack.pop(); continue;} stack.push(p);} return `/${stack.join('/')}`||'/'; }
  #d(path){ const parts=path.split('/').filter(Boolean); parts.pop(); return `/${parts.join('/')}`||'/'; }
  #b(path){ const parts=path.split('/').filter(Boolean); return parts[parts.length-1]??''; }
}

const assert = (cond, msg) => { if (!cond) throw new Error(msg); };

const bridge = createPhpHostBridge({
  fs: new MemoryFs(),
  target: "adwa",
  projectRoot: "/project",
  cwd: "/project",
});

const opened = bridge.call({
  kind: "db",
  action: "open",
  payload: { driver: "sqlite", config: { database: "smoke" } },
});
assert(opened.ok, "db open should succeed on adwa");
const handle = opened.value.handle;

const create = bridge.call({
  kind: "db",
  action: "exec",
  payload: { handle, sql: "create table if not exists packages (id int, name text)" },
});
assert(create.ok, "create table should succeed");

const insert = bridge.call({
  kind: "db",
  action: "exec",
  payload: { handle, sql: "insert into packages (id, name) values (?, ?)", params: [1, "alpha"] },
});
assert(insert.ok, "insert should succeed");
assert(insert.value.affected_rows === 1, "insert should affect one row");

const query = bridge.call({
  kind: "db",
  action: "query",
  payload: { handle, sql: "select id, name from packages where id = ?", params: [1] },
});
assert(query.ok, "query should succeed");
assert(Array.isArray(query.value.rows), "query rows should be array");
assert(query.value.rows.length === 1, "query should return one row");
assert(String(query.value.rows[0].name) === "alpha", "row value mismatch");

const stats = bridge.call({ kind: "db", action: "stats", payload: {} });
assert(stats.ok, "stats should succeed");
assert(Number(stats.value.active_handles) >= 1, "stats active_handles expected");

console.log("phpx db bridge smoke ok");
