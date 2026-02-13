import * as wasm from "./vendor/wosix_wasm/wosix_wasm.js";
import {
  WebContainer,
  createPhpHostBridge,
  createPhpRuntimeAdapter,
  createPhpRuntimeWasmExecutor,
} from "./vendor/wosix_js/index.js";

const logEl = document.getElementById("log");
const sourceEl = document.getElementById("source");
const runBtn = document.getElementById("runBtn");
let phpAdapterRef = null;

const log = (message) => {
  logEl.textContent += `\n${message}`;
};

const resetLog = (message) => {
  logEl.textContent = message;
};

const createNoopFs = () => ({
  readFile() {
    return new Uint8Array(0);
  },
  writeFile() {},
  readdir() {
    return [];
  },
  mkdir() {},
  rm() {},
  rename() {},
  stat() {
    return { size: 0, fileType: "dir" };
  },
});

const runPhpxScript = async () => {
  if (!(sourceEl instanceof HTMLTextAreaElement)) {
    throw new Error("missing source editor");
  }
  if (!phpAdapterRef) {
    throw new Error("php runtime adapter is not ready");
  }

  const result = await phpAdapterRef.run(sourceEl.value, "phpx", {
    filename: "main.phpx",
    cwd: "/",
  });

  resetLog("PHPX run complete.");
  if (result.stdout) {
    log(result.stdout.trimEnd());
  }
  if (result.stderr) {
    log(`[stderr]\n${result.stderr.trimEnd()}`);
  }
  if (result.diagnostics?.length) {
    for (const diag of result.diagnostics) {
      log(`[${diag.severity}] ${diag.message}`);
    }
  }
  if (!result.ok && !result.diagnostics?.length) {
    log("[error] runtime returned not-ok without diagnostics");
  }
};

try {
  resetLog("Loading wasm...");
  await WebContainer.boot(wasm, {
    init: wasm.default,
  });
  log("Demo: phpx");
  if (sourceEl instanceof HTMLTextAreaElement) {
    sourceEl.value = [
      "/*__DEKA_PHPX__*/",
      "$name = 'wosix'",
      "echo 'hello from phpx in ' . $name . \"\\n\"",
    ].join("\n");
  }

  const bridge = createPhpHostBridge({
    fs: createNoopFs(),
    target: "wosix",
    projectRoot: "/",
    cwd: "/",
  });
  const executor = createPhpRuntimeWasmExecutor({
    moduleUrl: new URL("./vendor/wosix_js/php_runtime.js", import.meta.url).toString(),
    bridge,
  });
  phpAdapterRef = createPhpRuntimeAdapter({ bridge, executor });

  if (runBtn instanceof HTMLButtonElement) {
    runBtn.addEventListener("click", async () => {
      runBtn.disabled = true;
      try {
        await runPhpxScript();
      } catch (err) {
        resetLog("Run failed.");
        log(err instanceof Error ? err.message : String(err));
      } finally {
        runBtn.disabled = false;
      }
    });
  }

  await runPhpxScript();
} catch (err) {
  log(`Error: ${err instanceof Error ? err.message : String(err)}`);
}
