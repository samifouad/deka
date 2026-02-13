import * as wasm from "./vendor/wosix_wasm/wosix_wasm.js";
import {
  WebContainer,
  createDekaBrowserCommandRuntime,
  createDekaWasmCommandRuntime,
  createPhpHostBridge,
  createPhpRuntimeAdapter,
  createPhpRuntimeWasmExecutor,
} from "./vendor/wosix_js/index.js";

const logEl = document.getElementById("log");
const sourceEl = document.getElementById("source");
const runBtn = document.getElementById("runBtn");
let phpAdapterRef = null;
let containerRef = null;
let serveProc = null;

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

const readProcessOutput = async (proc) => {
  const stdoutReader = proc.stdout.getReader();
  const stderrReader = proc.stderr.getReader();

  const pump = async (reader, prefix = "") => {
    const decoder = new TextDecoder();
    for (;;) {
      const { value, done } = await reader.read();
      if (done) break;
      if (!value || value.length === 0) continue;
      const text = decoder.decode(value).trimEnd();
      if (text.length > 0) {
        log(prefix ? `${prefix}${text}` : text);
      }
    }
  };

  await Promise.all([pump(stdoutReader), pump(stderrReader, "[stderr] ")]);
};

const restartServe = async () => {
  if (!(sourceEl instanceof HTMLTextAreaElement)) {
    throw new Error("missing source editor");
  }
  if (!containerRef) {
    throw new Error("web container not initialized");
  }

  if (serveProc) {
    serveProc.kill();
    await serveProc.exit();
    serveProc = null;
  }

  await containerRef.fs.writeFile(
    "/app/home.phpx",
    new TextEncoder().encode(sourceEl.value),
    { create: true, truncate: true }
  );

  resetLog("Starting deka serve...");
  serveProc = await containerRef.spawn("deka", ["serve"], {
    cwd: "/",
    env: {
      PATH: "/bin",
    },
  });
  readProcessOutput(serveProc).catch((err) => {
    log(`[stderr] ${err instanceof Error ? err.message : String(err)}`);
  });
};

try {
  resetLog("Loading wasm...");
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

  containerRef = await WebContainer.boot(wasm, {
    init: wasm.default,
    commandRuntimes: {
      deka: createDekaBrowserCommandRuntime({
        cliRuntime: createDekaWasmCommandRuntime({
          wasmUrl: new URL("./vendor/wosix_js/deka_cli.wasm", import.meta.url).toString(),
        }),
        phpRuntime: phpAdapterRef,
        projectRoot: "/",
        defaultServeEntry: "/app/home.phpx",
      }),
    },
  });
  await containerRef.mount({
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
        file: [
          "---",
          "interface HelloProps {",
          "  $name: string",
          "}",
          "function Hello({ $name }: HelloProps): string {",
          "  return $name",
          "}",
          "",
          "---",
          "<!doctype html>",
          "<html>",
          "  <body class=\"m-0 bg-slate-950 text-slate-100\">",
          "    <main class=\"min-h-screen flex items-center justify-center p-8\">",
          "      <h1 class=\"text-5xl font-bold tracking-tight\">Hello <Hello name=\"wosix\" /></h1>",
          "    </main>",
          "  </body>",
          "</html>",
        ].join("\n"),
      },
    },
    bin: {
      deka: {
        file: "#!/usr/bin/deka\n",
        executable: true,
      },
    },
  });
  log("Demo: deka serve (browser runtime)");
  if (sourceEl instanceof HTMLTextAreaElement) {
    sourceEl.value = new TextDecoder().decode(await containerRef.fs.readFile("/app/home.phpx"));
  }

  if (runBtn instanceof HTMLButtonElement) {
    runBtn.addEventListener("click", async () => {
      runBtn.disabled = true;
      try {
        await restartServe();
      } catch (err) {
        resetLog("Run failed.");
        log(err instanceof Error ? err.message : String(err));
      } finally {
        runBtn.disabled = false;
      }
    });
  }

  await restartServe();
} catch (err) {
  log(`Error: ${err instanceof Error ? err.message : String(err)}`);
}
