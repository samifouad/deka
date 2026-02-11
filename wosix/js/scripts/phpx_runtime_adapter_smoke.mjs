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
});

const runtime = createPhpRuntimeAdapter({ bridge });
const result = await runtime.run("/*__DEKA_PHPX__*/\necho 'hello\\n'\n", "phpx", {
  filename: "main.phpx",
  cwd: "/project",
});

assert(typeof result.ok === "boolean", "result.ok must be boolean");
assert(typeof result.stdout === "string", "result.stdout must be string");
assert(typeof result.stderr === "string", "result.stderr must be string");
assert(Array.isArray(result.diagnostics), "result.diagnostics must be array");
assert(typeof result.meta === "object" && result.meta !== null, "result.meta must be object");
assert(result.ok === false, "adapter should return not implemented for now");
assert(
  result.diagnostics.some((d) => String(d.message).includes("not wired yet")),
  "expected not wired diagnostic"
);

console.log("phpx runtime adapter smoke ok");
