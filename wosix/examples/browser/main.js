import * as wasm from "../../crates/wosix-wasm/pkg/wosix_wasm.js";
import { DekaBrowserRuntime, WebContainer } from "../../js/dist/index.js";

const logEl = document.getElementById("log");
const decoder = new TextDecoder();
const params = new URLSearchParams(window.location.search);
const nodeRuntime = params.get("node") === "wasm" ? "wasm" : "shim";
const demo = params.get("demo") ?? "node";

const log = (message) => {
  logEl.textContent += `\n${message}`;
};

try {
  logEl.textContent = "Loading wasm...";
  const container = await WebContainer.boot(wasm, {
    init: wasm.default,
    nodeRuntime,
    nodeWasm: nodeRuntime === "wasm" ? { url: "./node.wasm" } : undefined,
  });
  if (demo === "deka") {
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
  } else {
    log(`Node runtime: ${nodeRuntime}`);
    const greeting =
      nodeRuntime === "wasm" ? "hello from wosix node wasm" : "hello from wosix node shim";
    await container.mount({
      "index.js": {
        file: `console.log('${greeting}');`,
      },
    });
    log("Mounted /index.js");

    try {
      const proc = await container.spawn("node", ["index.js"]);
      const status = await proc.exit();
      const output = await proc.readOutput();
      if (output) {
        log(decoder.decode(output));
      } else {
        log("No output from process.");
      }
      log(`Exit code: ${status.code}`);
    } catch (err) {
      log(`Spawn failed: ${err instanceof Error ? err.message : String(err)}`);
    }
  }
} catch (err) {
  log(`Error: ${err instanceof Error ? err.message : String(err)}`);
}
