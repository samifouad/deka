import { dirname, normalizePath } from "./path_utils.js";
import { toBytes } from "./bytes.js";

export class VirtualFs {
  constructor(initialFiles = {}) {
    this.files = new Map();
    this.dirs = new Set(["/"]);
    for (const [path, content] of Object.entries(initialFiles)) {
      this.writeFile(path, toBytes(content));
    }
  }

  readFile(path) {
    const normalized = normalizePath(path);
    const value = this.files.get(normalized);
    if (!value) throw new Error(`ENOENT: ${normalized}`);
    return new Uint8Array(value);
  }

  writeFile(path, data) {
    const normalized = normalizePath(path);
    this.ensureDir(dirname(normalized));
    this.files.set(normalized, new Uint8Array(data));
  }

  readdir(path) {
    const dir = normalizePath(path);
    if (!this.dirs.has(dir)) throw new Error(`ENOTDIR: ${dir}`);
    const names = new Set();
    const prefix = dir === "/" ? "/" : `${dir}/`;

    for (const d of this.dirs) {
      if (d === dir || !d.startsWith(prefix)) continue;
      const first = d.slice(prefix.length).split("/")[0];
      if (first) names.add(first);
    }
    for (const f of this.files.keys()) {
      if (!f.startsWith(prefix)) continue;
      const first = f.slice(prefix.length).split("/")[0];
      if (first) names.add(first);
    }
    return Array.from(names).sort((a, b) => a.localeCompare(b));
  }

  mkdir(path) {
    this.ensureDir(normalizePath(path));
  }

  rm(path, options = {}) {
    const normalized = normalizePath(path);
    const recursive = Boolean(options?.recursive);
    if (this.files.has(normalized)) {
      this.files.delete(normalized);
      return;
    }
    if (!this.dirs.has(normalized)) {
      throw new Error(`ENOENT: ${normalized}`);
    }
    if (normalized === "/") {
      throw new Error("EPERM: cannot remove root");
    }
    const prefix = `${normalized}/`;
    const childDirs = Array.from(this.dirs).filter((d) => d.startsWith(prefix));
    const childFiles = Array.from(this.files.keys()).filter((f) => f.startsWith(prefix));
    if (!recursive && (childDirs.length > 0 || childFiles.length > 0)) {
      throw new Error(`ENOTEMPTY: ${normalized}`);
    }
    for (const file of childFiles) this.files.delete(file);
    for (const dir of childDirs) this.dirs.delete(dir);
    this.dirs.delete(normalized);
  }

  rename(from, to) {
    const fromN = normalizePath(from);
    const toN = normalizePath(to);
    if (this.files.has(fromN)) {
      const data = this.readFile(fromN);
      this.writeFile(toN, data);
      this.files.delete(fromN);
      return;
    }
    if (!this.dirs.has(fromN)) {
      throw new Error(`ENOENT: ${fromN}`);
    }
    this.ensureDir(dirname(toN));
    const fromPrefix = `${fromN}/`;
    const toPrefix = `${toN}/`;
    const movedFiles = [];
    for (const file of Array.from(this.files.keys())) {
      if (file === fromN || file.startsWith(fromPrefix)) {
        const rest = file === fromN ? "" : file.slice(fromPrefix.length);
        const next = rest ? `${toPrefix}${rest}` : toN;
        movedFiles.push([file, normalizePath(next)]);
      }
    }
    const movedDirs = [];
    for (const dir of Array.from(this.dirs)) {
      if (dir === fromN || dir.startsWith(fromPrefix)) {
        const rest = dir === fromN ? "" : dir.slice(fromPrefix.length);
        const next = rest ? `${toPrefix}${rest}` : toN;
        movedDirs.push([dir, normalizePath(next)]);
      }
    }
    for (const [oldPath, newPath] of movedFiles) {
      const bytes = this.files.get(oldPath);
      if (bytes) this.files.set(newPath, bytes);
    }
    for (const [oldPath] of movedFiles) this.files.delete(oldPath);
    for (const [oldDir] of movedDirs) this.dirs.delete(oldDir);
    for (const [, newDir] of movedDirs) this.dirs.add(newDir);
  }

  stat(path) {
    const normalized = normalizePath(path);
    if (this.files.has(normalized)) {
      return { size: this.files.get(normalized).byteLength, fileType: "file" };
    }
    if (this.dirs.has(normalized)) {
      return { size: 0, fileType: "dir" };
    }
    throw new Error(`ENOENT: ${normalized}`);
  }

  listFiles() {
    return Array.from(this.files.keys()).sort((a, b) => a.localeCompare(b));
  }

  listDirs() {
    return Array.from(this.dirs).sort((a, b) => a.localeCompare(b));
  }

  ensureDir(path) {
    const normalized = normalizePath(path);
    let current = "";
    for (const part of normalized.split("/").filter(Boolean)) {
      current = `${current}/${part}`;
      this.dirs.add(current || "/");
    }
    this.dirs.add(normalized);
  }
}
