import { createPhpHostBridge } from "../dist/phpx_host_bridge.js";

class MemoryFs {
  #files = new Map();
  #dirs = new Set(["/"]);

  readFile(path) {
    const key = this.#norm(path);
    const data = this.#files.get(key);
    if (!data) {
      throw new Error(`not found: ${key}`);
    }
    return data;
  }

  writeFile(path, data) {
    const key = this.#norm(path);
    this.#dirs.add(this.#dirname(key));
    this.#files.set(key, data);
  }

  readdir(path) {
    const base = this.#norm(path);
    const out = new Set();
    for (const dir of this.#dirs) {
      if (dir !== base && this.#dirname(dir) === base) {
        out.add(this.#basename(dir));
      }
    }
    for (const file of this.#files.keys()) {
      if (this.#dirname(file) === base) {
        out.add(this.#basename(file));
      }
    }
    return [...out].sort();
  }

  mkdir(path) {
    this.#dirs.add(this.#norm(path));
  }

  rm(path) {
    const key = this.#norm(path);
    this.#files.delete(key);
    this.#dirs.delete(key);
  }

  rename(from, to) {
    const src = this.#norm(from);
    const dst = this.#norm(to);
    const data = this.#files.get(src);
    if (!data) throw new Error(`not found: ${src}`);
    this.#files.delete(src);
    this.#files.set(dst, data);
  }

  stat(path) {
    const key = this.#norm(path);
    if (this.#files.has(key)) {
      return { size: this.#files.get(key).length, fileType: "file" };
    }
    if (this.#dirs.has(key)) {
      return { size: 0, fileType: "dir" };
    }
    throw new Error(`not found: ${key}`);
  }

  #norm(path) {
    const parts = path.split("/").filter(Boolean);
    const stack = [];
    for (const part of parts) {
      if (part === ".") continue;
      if (part === "..") {
        stack.pop();
        continue;
      }
      stack.push(part);
    }
    return `/${stack.join("/")}`;
  }

  #dirname(path) {
    const parts = path.split("/").filter(Boolean);
    parts.pop();
    return `/${parts.join("/")}` || "/";
  }

  #basename(path) {
    const parts = path.split("/").filter(Boolean);
    return parts[parts.length - 1] ?? "";
  }
}

function assert(cond, msg) {
  if (!cond) throw new Error(msg);
}

const fs = new MemoryFs();
const bridge = createPhpHostBridge({
  fs,
  target: "wosix",
  projectRoot: "/project",
  cwd: "/project",
  env: { APP_ENV: "dev" },
});

const denied = bridge.call({ kind: "db", action: "stats", payload: {} });
assert(!denied.ok, "db should be denied in wosix");
assert(String(denied.error || "").includes("CapabilityError"), "capability error expected");

const netDenied = bridge.call({
  kind: "net",
  action: "fetch",
  payload: { url: "https://example.com" },
});
assert(!netDenied.ok, "net should be denied in wosix");
assert(String(netDenied.error || "").includes("CapabilityError"), "net capability error expected");

const write = bridge.call({
  kind: "fs",
  action: "writeFile",
  payload: { path: "tmp/hello.txt", data: "hello" },
});
assert(write.ok, "fs write should succeed");

const read = bridge.call({
  kind: "fs",
  action: "readFile",
  payload: { path: "tmp/hello.txt" },
});
assert(read.ok, "fs read should succeed");
assert(new TextDecoder().decode(read.value.data) === "hello", "fs read content mismatch");

const envGet = bridge.call({
  kind: "process",
  action: "envGet",
  payload: { key: "APP_ENV" },
});
assert(!envGet.ok, "process/env should be denied in wosix");

const stdoutChunks = [];
const stderrChunks = [];
const stdinChunk = new TextEncoder().encode("input-bytes");
const ioBridge = createPhpHostBridge({
  fs,
  target: "server",
  projectRoot: "/project",
  cwd: "/project",
  env: { APP_ENV: "dev" },
  stdio: {
    writeStdout: (chunk) => stdoutChunks.push(chunk),
    writeStderr: (chunk) => stderrChunks.push(chunk),
    readStdin: () => stdinChunk,
  },
});

const outWrite = ioBridge.call({
  kind: "process",
  action: "writeStdout",
  payload: { data: "hello-stdout" },
});
assert(outWrite.ok, "stdout write should succeed");
assert(stdoutChunks.length === 1, "stdout channel expected 1 chunk");

const errWrite = ioBridge.call({
  kind: "process",
  action: "writeStderr",
  payload: { data: "hello-stderr" },
});
assert(errWrite.ok, "stderr write should succeed");
assert(stderrChunks.length === 1, "stderr channel expected 1 chunk");

const stdinRead = ioBridge.call({
  kind: "process",
  action: "readStdin",
  payload: { maxBytes: 64 },
});
assert(stdinRead.ok, "stdin read should succeed");
assert(new TextDecoder().decode(stdinRead.value.data) === "input-bytes", "stdin read mismatch");

console.log("phpx host bridge smoke ok");
