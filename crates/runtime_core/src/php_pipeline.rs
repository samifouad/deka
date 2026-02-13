pub fn build_run_handler_code(handler_path: &str) -> String {
    let php_file = serde_json::to_string(handler_path).unwrap_or_else(|_| "\"\"".to_string());
    format!(
        "const __dekaPhpFile = {php_file};\
(async () => {{\
  try {{\
    const __dekaPhpResult = await globalThis.__dekaPhp.runFile(__dekaPhpFile);\
    const __dekaPhpStdout = (__dekaPhpResult && __dekaPhpResult.stdout) ? __dekaPhpResult.stdout : \"\";\
    if (__dekaPhpStdout) Deno.core.print(String(__dekaPhpStdout), false);\
    let __dekaPhpStderr = (__dekaPhpResult && __dekaPhpResult.stderr) ? __dekaPhpResult.stderr : \"\";\
    if (!__dekaPhpStderr && __dekaPhpResult && __dekaPhpResult.error) {{\
      __dekaPhpStderr = String(__dekaPhpResult.error);\
    }}\
    if (__dekaPhpStderr) Deno.core.print(String(__dekaPhpStderr), true);\
    const __dekaPhpOk = __dekaPhpResult && __dekaPhpResult.ok !== false;\
    let __dekaPhpExit = (__dekaPhpResult && typeof __dekaPhpResult.exit_code === \"number\") ? __dekaPhpResult.exit_code : 0;\
    if (!__dekaPhpOk && __dekaPhpExit === 0) __dekaPhpExit = 1;\
    if (__dekaPhpExit) globalThis.__dekaExitCode = __dekaPhpExit;\
  }} catch (err) {{\
    const __dekaPhpMsg = err && (err.stack || err.message) ? (err.stack || err.message) : String(err);\
    if (__dekaPhpMsg) Deno.core.print(String(__dekaPhpMsg) + \"\\n\", true);\
    globalThis.__dekaExitCode = 1;\
  }}\
}})();",
    )
}

pub fn build_serve_handler_code(handler_path: &str) -> String {
    let php_file = serde_json::to_string(handler_path).unwrap_or_else(|_| "\"\"".to_string());
    format!(
        "const app = globalThis.__dekaPhp.servePhp({});\nglobalThis.app = app;",
        php_file
    )
}
