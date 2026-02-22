const http = require("http");
const { spawn } = require("child_process");

const wasmPath =
  process.env.PHP_WASM_PATH || "target/wasm32-wasip1/debug/php-wasm.wasm";
const phpScript = "tests/php/test-001/hello.php";

const server = http.createServer((req, res) => {
  const child = spawn("wasmtime", [wasmPath, "--", phpScript], {
    env: process.env,
  });

  let output = "";
  child.stdout.on("data", (chunk) => {
    output += chunk;
  });

  child.stderr.on("data", (chunk) => {
    console.error(chunk.toString());
  });

  child.on("close", (code) => {
    res.statusCode = code === 0 ? 200 : 500;
    res.end(output || `php-wasm exited with code ${code}`);
  });
});

server.listen(8540, () => {
  console.log(`PHP HTTP bridge listening on http://localhost:8540`);
});
