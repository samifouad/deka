import { WebContainer, createDekaBrowserCommandRuntime } from "../dist/index.js";

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

  watch() {
    return {
      nextEvent() {
        return null;
      },
      close() {},
    };
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

class EmptyProcess {
  constructor(pid = 1) {
    this._pid = pid;
    this._out = new ReadableStream();
    this._err = new ReadableStream();
    this._all = new ReadableStream();
    this._in = new WritableStream();
  }
  pid() {
    return this._pid;
  }
  wait() {
    return { code: 0 };
  }
  async exit() {
    return { code: 0 };
  }
  writeStdin() {
    return 0;
  }
  readStdout() {
    return null;
  }
  readStderr() {
    return null;
  }
  readOutput() {
    return null;
  }
  stdinStream() {
    return this._in;
  }
  stdoutStream() {
    return this._out;
  }
  stderrStream() {
    return this._err;
  }
  outputStream() {
    return this._all;
  }
  kill() {}
  close() {}
}

function createBindings(fs, events) {
  return {
    WebContainer: {
      boot() {
        return {
          fs() {
            return fs;
          },
          spawn() {
            return new EmptyProcess(9999);
          },
          publishPort(port, options) {
            const evt = {
              kind: "server-ready",
              port,
              url: `http://localhost:${port}`,
              protocol: options?.protocol ?? "http",
            };
            events.push(evt);
            return {
              port,
              url: evt.url,
              protocol: evt.protocol,
            };
          },
          unpublishPort(port) {
            events.push({ kind: "port-closed", port });
          },
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

const fs = new MemoryFs();
const events = [];
const noopCli = async () => ({ code: 0, stdout: "noop cli" });
const phpRuntime = {
  async run(_source, _mode, _context) {
    return { ok: true, stdout: "", stderr: "", diagnostics: [] };
  },
};

const container = await WebContainer.boot(createBindings(fs, events), {
  commandRuntimes: {
    deka: createDekaBrowserCommandRuntime({
      cliRuntime: noopCli,
      phpRuntime,
      projectRoot: "/",
    }),
  },
});

await container.mount({
  "deka.json": {
    file: JSON.stringify({
      serve: {
        entry: "app/home.phpx",
        mode: "phpx",
        port: 8530,
      },
    }),
  },
  app: {
    "home.phpx": {
      file: "---\n$greeting = 'hello'\n---\n<!doctype html><html><body>{$greeting}</body></html>",
    },
  },
  bin: {
    deka: {
      file: "#!/usr/bin/deka\n",
      executable: true,
    },
  },
});

const proc = await container.spawn("deka", ["serve"], {
  cwd: "/",
  env: { PATH: "/bin" },
});

const reader = proc.stdout.getReader();
const first = await reader.read();
const text = first.value ? new TextDecoder().decode(first.value) : "";
if (!text.includes("[listen] http://localhost:8530")) {
  throw new Error(`unexpected serve output: ${text}`);
}

proc.kill();
const status = await proc.exit();
if (status.code !== 128) {
  throw new Error(`expected killed status 128, got ${status.code}`);
}

const published = events.find((event) => event.kind === "server-ready" && event.port === 8530);
const closed = events.find((event) => event.kind === "port-closed" && event.port === 8530);
if (!published || !closed) {
  throw new Error(`expected publish+close events, got: ${JSON.stringify(events)}`);
}

console.log("deka browser serve smoke ok");
