import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

import { createDekaWasmCommandRuntime } from "../dist/index.js";

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const wasmPath = path.resolve(__dirname, "../../../target/wasm32-unknown-unknown/release/cli.wasm");

if (!fs.existsSync(wasmPath)) {
  throw new Error(`missing cli wasm at ${wasmPath}; run: cargo build --release -p cli --target wasm32-unknown-unknown --no-default-features`);
}

const wasmBytes = fs.readFileSync(wasmPath);
const runtime = createDekaWasmCommandRuntime({
  wasmUrl: "file://unused",
  wasmBytes,
});

const fsStub = {
  readFile() { return new Uint8Array(0); },
  writeFile() {},
  readdir() { return []; },
  mkdir() {},
  rm() {},
  rename() {},
  stat() { return { size: 0, fileType: "dir" }; },
  mount() {},
  watch() { return { nextEvent() { return null; }, close() {} }; },
};

const versionResult = await runtime(["--version"], undefined, { fs: fsStub });
if (versionResult.code !== 0 || !String(versionResult.stdout || "").includes("deka [version")) {
  throw new Error(`unexpected --version output: ${JSON.stringify(versionResult)}`);
}

const unsupportedResult = await runtime(["run", "main.phpx"], undefined, { fs: fsStub });
if (unsupportedResult.code === 0) {
  throw new Error(`expected non-zero code for unsupported command: ${JSON.stringify(unsupportedResult)}`);
}

console.log("deka wasm runtime smoke ok");
