import * as wasm from "./vendor/wosix_wasm/wosix_wasm.js";
import {
  DekaBrowserRuntime,
  WebContainer,
  createPhpHostBridge,
  createPhpRuntimeAdapter,
  createPhpRuntimeWasmExecutor,
} from "./vendor/wosix_js/index.js";

const logEl = document.getElementById("log");
const sourceEl = document.getElementById("source");
const runBtn = document.getElementById("runBtn");
const playgroundEl = document.getElementById("playground");
const decoder = new TextDecoder();
const params = new URLSearchParams(window.location.search);
const nodeRuntime = params.get("node") === "wasm" ? "wasm" : "shim";
const demo = params.get("demo") ?? "node";
let containerRef = null;
let phpAdapterRef = null;

const log = (message) => {
  logEl.textContent += `\n${message}`;
};

const resetLog = (message) => {
  logEl.textContent = message;
};

const runNodeScript = async () => {
  if (!containerRef) {
    throw new Error("container is not ready");
  }
  if (!(sourceEl instanceof HTMLTextAreaElement)) {
    throw new Error("missing source editor");
  }

  const code = sourceEl.value;
  await containerRef.mount({
    "index.js": {
      file: code,
    },
  });

  const proc = await containerRef.spawn("node", ["index.js"]);
  const status = await proc.exit();
  const output = await proc.readOutput();

  resetLog("Run complete.");
  if (output && output.length > 0) {
    log(decoder.decode(output));
  } else {
    log("(no output)");
  }
  log(`Exit code: ${status.code}`);
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
  const container = await WebContainer.boot(wasm, {
    init: wasm.default,
    nodeRuntime,
    nodeWasm: nodeRuntime === "wasm" ? { url: "./node.wasm" } : undefined,
  });
  containerRef = container;

  if (demo === "deka") {
    if (playgroundEl instanceof HTMLElement) {
      playgroundEl.style.display = "none";
    }
    log("Demo: deka");
    await container.mount({
      "handler.js": {
        file: "module.exports.fetch = async () => new Response('hello from deka browser runtime');",
      },
    });
    log("Mounted /handler.js");
    const runtime = new DekaBrowserRuntime({
      readFile: (path) => container.fs.readFile(path),
    });
    const server = await runtime.serve("handler.js", { port: 8787 });
    const response = await server.fetch(new Request("http://localhost/"));
    const text = await response.text();
    log(`Deka response: ${response.status} ${text}`);
  } else if (demo === "phpx") {
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
  } else {
    log(`Node runtime: ${nodeRuntime}`);
    const greeting = nodeRuntime === "wasm"
      ? "hello from wosix node wasm"
      : "hello from wosix node shim";
    if (sourceEl instanceof HTMLTextAreaElement) {
      sourceEl.value = `console.log('${greeting}')`;
    }

    if (runBtn instanceof HTMLButtonElement) {
      runBtn.addEventListener("click", async () => {
        runBtn.disabled = true;
        try {
          await runNodeScript();
        } catch (err) {
          resetLog("Run failed.");
          log(err instanceof Error ? err.message : String(err));
        } finally {
          runBtn.disabled = false;
        }
      });
    }

  }
} catch (err) {
  log(`Error: ${err instanceof Error ? err.message : String(err)}`);
}
