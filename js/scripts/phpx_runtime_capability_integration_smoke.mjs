import { createPhpHostBridge, createPhpRuntimeAdapter } from "../dist/index.js";

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
    this.#dirs.add(this.#dirname(key));
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

function assert(cond, message) {
  if (!cond) throw new Error(message);
}

const fs = new MemoryFs();
fs.mkdir("/project");

const bridge = createPhpHostBridge({
  fs,
  target: "wosix",
  projectRoot: "/project",
  cwd: "/project",
  capabilities: {
    fs: false,
  },
});

const adapter = createPhpRuntimeAdapter({ bridge });
const result = await adapter.run("echo 'hi'", "phpx", {
  filename: "main.phpx",
  cwd: "/project",
});

assert(result.ok === false, "expected adapter preflight to fail");
assert(
  result.diagnostics.some((d) => String(d.message).includes("CapabilityError")),
  "expected capability error diagnostic"
);
assert(
  String(result.meta?.bridgeError?.code || "") === "CAPABILITY_DENIED",
  "expected structured bridge error code"
);
assert(
  String(result.meta?.bridgeError?.info?.capability || "") === "fs",
  "expected fs capability in structured info"
);
assert(
  typeof result.meta?.host === "object" && result.meta.host?.capabilities?.fs === false,
  "expected host capability report in adapter meta"
);

console.log("phpx runtime capability integration smoke ok");

