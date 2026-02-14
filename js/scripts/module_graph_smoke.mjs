import { WebContainer } from "../dist/index.js";

class MemoryFs {
  #files = new Map();
  #dirs = new Set(["/"]);

  readFile(path) {
    const key = this.#norm(path);
    const data = this.#files.get(key);
    if (!data) throw new Error(`not found: ${key}`);
    return data;
  }

  writeFile(path, data) {
    const key = this.#norm(path);
    this.#ensureParentDirs(key);
    this.#files.set(key, data);
  }

  readdir(path) {
    const base = this.#norm(path);
    const out = new Set();
    for (const dir of this.#dirs) {
      if (dir !== base && this.#dirname(dir) === base) out.add(this.#basename(dir));
    }
    for (const file of this.#files.keys()) {
      if (this.#dirname(file) === base) out.add(this.#basename(file));
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
    this.#ensureParentDirs(dst);
    this.#files.delete(src);
    this.#files.set(dst, data);
  }

  stat(path) {
    const key = this.#norm(path);
    if (this.#files.has(key)) return { size: this.#files.get(key).length, fileType: "file" };
    if (this.#dirs.has(key)) return { size: 0, fileType: "dir" };
    throw new Error(`not found: ${key}`);
  }

  mount(tree, prefix = "/") {
    if (typeof tree === "string" || tree instanceof Uint8Array) {
      throw new Error("root mount must be an object");
    }
    for (const [name, node] of Object.entries(tree)) {
      const full = this.#norm(`${prefix}/${name}`);
      if (typeof node === "string") {
        this.writeFile(full, new TextEncoder().encode(node));
        continue;
      }
      if (node instanceof Uint8Array) {
        this.writeFile(full, node);
        continue;
      }
      if (node && typeof node === "object" && "file" in node) {
        const value = node.file;
        if (typeof value === "string") {
          this.writeFile(full, new TextEncoder().encode(value));
          continue;
        }
        if (value instanceof Uint8Array) {
          this.writeFile(full, value);
          continue;
        }
      }
      this.mkdir(full);
      this.mount(node, full);
    }
  }

  #ensureParentDirs(path) {
    let dir = this.#dirname(path);
    const all = [];
    while (dir !== "/" && !this.#dirs.has(dir)) {
      all.push(dir);
      dir = this.#dirname(dir);
    }
    all.reverse().forEach((d) => this.#dirs.add(d));
    this.#dirs.add("/");
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

class MemoryProcess {
  #pid;
  #code = 0;
  #stdout = [];
  #stderr = [];
  #stdin = [];
  #done = false;
  #outputCtrl;
  #stdoutCtrl;
  #stderrCtrl;

  constructor(pid) {
    this.#pid = pid;
    this._outputStream = new ReadableStream({
      start: (ctrl) => {
        this.#outputCtrl = ctrl;
      },
    });
    this._stdoutStream = new ReadableStream({
      start: (ctrl) => {
        this.#stdoutCtrl = ctrl;
      },
    });
    this._stderrStream = new ReadableStream({
      start: (ctrl) => {
        this.#stderrCtrl = ctrl;
      },
    });
    this._stdinStream = new WritableStream({
      write: (chunk) => {
        const bytes = chunk instanceof Uint8Array ? chunk : new TextEncoder().encode(String(chunk));
        this.#stdin.push(bytes);
      },
    });
  }

  pid() {
    return this.#pid;
  }

  wait() {
    return { code: this.#code };
  }

  async exit() {
    return { code: this.#code };
  }

  writeStdin(data) {
    const bytes = data instanceof Uint8Array ? data : new TextEncoder().encode(String(data));
    this.#stdin.push(bytes);
    return bytes.length;
  }

  readStdout(maxBytes) {
    return this.#shift(this.#stdout, maxBytes);
  }

  readStderr(maxBytes) {
    return this.#shift(this.#stderr, maxBytes);
  }

  readOutput(maxBytes) {
    return this.#shift(this.#stdout, maxBytes) ?? this.#shift(this.#stderr, maxBytes);
  }

  stdinStream() {
    return this._stdinStream;
  }

  stdoutStream() {
    return this._stdoutStream;
  }

  stderrStream() {
    return this._stderrStream;
  }

  outputStream() {
    return this._outputStream;
  }

  kill() {
    this.#done = true;
  }

  close() {
    this.#done = true;
  }

  _pushStdout(bytes) {
    this.#stdout.push(bytes);
    this.#stdoutCtrl?.enqueue(bytes);
    this.#outputCtrl?.enqueue(bytes);
  }

  _pushStderr(bytes) {
    this.#stderr.push(bytes);
    this.#stderrCtrl?.enqueue(bytes);
    this.#outputCtrl?.enqueue(bytes);
  }

  _finish(code) {
    this.#code = code;
    this.#done = true;
    this.#stdoutCtrl?.close();
    this.#stderrCtrl?.close();
    this.#outputCtrl?.close();
  }

  #shift(queue, maxBytes) {
    const chunk = queue[0];
    if (!chunk) return null;
    if (!maxBytes || chunk.length <= maxBytes) return queue.shift() ?? null;
    const head = chunk.slice(0, maxBytes);
    queue[0] = chunk.slice(maxBytes);
    return head;
  }
}

function createBindings(fs) {
  let nextPid = 1;
  return {
    WebContainer: {
      boot() {
        return {
          fs() {
            return fs;
          },
          spawn(_program, _args, _options) {
            return new MemoryProcess(nextPid++);
          },
          publishPort(port, options) {
            return {
              port,
              url: `http://${options?.host ?? "localhost"}:${port}`,
              protocol: options?.protocol ?? "http",
            };
          },
          unpublishPort() {},
          nextPortEvent() {
            return null;
          },
          onPortEvent() {
            return 1;
          },
          offPortEvent() {},
        };
      },
    },
  };
}

function assert(cond, msg) {
  if (!cond) throw new Error(msg);
}

async function readAll(stream) {
  const reader = stream.getReader();
  const chunks = [];
  while (true) {
    const { done, value } = await reader.read();
    if (done) break;
    chunks.push(value);
  }
  let total = 0;
  for (const c of chunks) total += c.length;
  const out = new Uint8Array(total);
  let off = 0;
  for (const c of chunks) {
    out.set(c, off);
    off += c.length;
  }
  return out;
}

const fs = new MemoryFs();
const container = await WebContainer.boot(createBindings(fs), { nodeRuntime: "shim" });
await container.mount({
  project: {
    "deka.lock": JSON.stringify({
      lockfileVersion: 1,
      php: {
        packages: {
          "cached-only": {
            descriptor: "cached-only@1.0.0",
            resolved: "local",
            metadata: { source: "virtual-fs" },
          },
        },
      },
    }),
    "main.js":
      "const util = require('@/lib/util'); const pkg = require('example'); const cached = require('cached-only'); const fs = require('fs'); const lock = JSON.parse(fs.readFileSync('/project/deka.lock', 'utf8')); fs.writeFileSync('/project/php_modules/.cache/runtime.json', JSON.stringify({ warmed: true })); fs.writeFileSync('/project/out.txt', util.msg + ':' + pkg.msg + ':' + cached.msg + ':' + lock.lockfileVersion);",
    lib: {
      "util.js": "module.exports = { msg: 'alias-ok' };",
    },
    php_modules: {
      example: {
        "index.js": "module.exports = { msg: 'module-ok' };",
      },
      ".cache": {
        "cached-only": {
          "index.js": "module.exports = { msg: 'cache-ok' };",
        },
      },
    },
  },
});

const proc = await container.spawn("node", ["/project/main.js"], { cwd: "/project" });
await proc.wait();
const out = await container.fs.readFile("/project/out.txt");
const text = new TextDecoder().decode(out);
assert(text === "alias-ok:module-ok:cache-ok:1", `unexpected output: ${text}`);

const runtimeCache = new TextDecoder().decode(
  await container.fs.readFile("/project/php_modules/.cache/runtime.json")
);
assert(runtimeCache.includes('"warmed":true'), `missing runtime cache marker: ${runtimeCache}`);

console.log("wosix module graph smoke ok");
