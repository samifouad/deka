import { createPhpHostBridge, createPhpRuntimeAdapter } from "../dist/index.js";

class MemoryFs {
  readFile() {
    return new Uint8Array(0);
  }
  writeFile() {}
  readdir() {
    return [];
  }
  mkdir() {}
  rm() {}
  rename() {}
  stat() {
    return { size: 0, fileType: "dir" };
  }
}

function assert(cond, message) {
  if (!cond) throw new Error(message);
}

const bridge = createPhpHostBridge({
  fs: new MemoryFs(),
  target: "wosix",
  projectRoot: "/project",
  cwd: "/project",
  capabilities: {
    fs: false,
  },
});

const runtime = createPhpRuntimeAdapter({ bridge });
const result = await runtime.run("echo 'hello'\n", "phpx", {
  filename: "main.phpx",
  cwd: "/project",
});

assert(result.ok === false, "expected capability denied run to fail");
assert(Array.isArray(result.diagnostics), "diagnostics must be array");
assert(result.diagnostics.length > 0, "expected at least one diagnostic");
assert(
  String(result.diagnostics[0].message).includes("Capability 'fs'"),
  "expected actionable capability message"
);
assert(
  String(result.meta.hostTarget) === "wosix",
  "expected host target in metadata"
);
assert(
  result.meta.hostCapabilities && result.meta.hostCapabilities.fs === false,
  "expected host capability map in metadata"
);
assert(
  typeof result.meta.error === "object" && result.meta.error !== null,
  "expected structured error metadata"
);
assert(
  String(result.meta.error.kind) === "capability_error",
  "expected capability_error kind"
);

console.log("phpx runtime capability-denied smoke ok");
