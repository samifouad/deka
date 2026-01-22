use std::path::Path as FsPath;
use std::sync::Arc;

use core::Context;
use engine::{RuntimeEngine, config as runtime_config, set_engine};
use pool::{ExecutionMode, HandlerKey, PoolConfig, RequestData};
use crate::env::init_env;
use crate::extensions::extensions_for_mode;
fn parse_exit_code(message: &str) -> Option<i32> {
    let marker = "DekaExit:";
    let idx = message.find(marker)?;
    let tail = &message[idx + marker.len()..];
    let digits: String = tail
        .chars()
        .take_while(|ch| ch.is_ascii_digit() || *ch == '-')
        .collect();
    if digits.is_empty() {
        return None;
    }
    digits.parse::<i32>().ok()
}

pub fn run(context: &Context) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to start tokio runtime");

    if let Err(err) = rt.block_on(run_async(context)) {
        eprintln!("{}", err);
        std::process::exit(1);
    }
}

async fn run_async(context: &Context) -> Result<(), String> {
    init_env();
    if std::env::var("LOG_LEVEL").is_err() {
        unsafe {
            std::env::set_var("LOG_LEVEL", "error");
        }
    }

    let (handler_path, extra_args) = handler_input(context);
    set_runtime_args(&extra_args);

    if std::env::var("HANDLER_PATH").is_err() {
        unsafe {
            std::env::set_var("HANDLER_PATH", &handler_path);
        }
    }

    let normalized = normalize_handler_path(&handler_path);
    if normalized.to_ascii_lowercase().ends_with(".html") {
        return Err(format!(
            "Run mode does not support HTML entrypoints: {}",
            normalized
        ));
    }

    // Check for --deka and --watch flags
    let force_deka = context.args.flags.contains_key("--deka");
    let watch_mode = context.args.flags.contains_key("--watch")
        || context.args.flags.contains_key("-W")
        || std::env::var("DEKA_WATCH").ok().as_deref() == Some("1");

    // Check if this is Next.js - delegate to node even with --deka because Next.js requires IPC
    let is_nextjs = normalized.contains("next/dist/bin/next")
        || normalized.contains("next/dist/cli/");

    if is_nextjs {
        eprintln!("[deka] Detected Next.js - delegating to Node.js (Next.js requires IPC)");
        return run_with_shebang("node", &normalized, &extra_args);
    }

    // If --deka is not set, check shebang and potentially delegate to node/bun
    if !force_deka {
        if let Some(shebang_cmd) = parse_shebang(&normalized) {
            if shebang_cmd == "node" || shebang_cmd == "bun" {
                return run_with_shebang(&shebang_cmd, &normalized, &extra_args);
            }
        }
    }

    let is_php = normalized.to_ascii_lowercase().ends_with(".php");
    let serve_mode = if is_php {
        runtime_config::ServeMode::Php
    } else {
        runtime_config::ServeMode::Js
    };

    let _ = std::fs::read_to_string(&normalized)
        .map_err(|err| format!("Failed to read handler from {}: {}", normalized, err))?;

    let handler_code = if matches!(serve_mode, runtime_config::ServeMode::Php) {
        let php_file = serde_json::to_string(&normalized).unwrap_or_else(|_| "\"\"".to_string());
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
    } else {
        format!(
            "globalThis.__dekaLoadModuleAsync({}).then(async () => {{\
if (globalThis.__dekaRuntimeHold) {{ await globalThis.__dekaRuntimeHold; }}\
}}).catch((err) => {{\
const msg = err && (err.stack || err.message) ? (err.stack || err.message) : String(err);\
if (String(msg).includes('DekaExit:')) {{ return; }}\
console.error(err);\
throw err;\
}});",
            serde_json::to_string(&normalized).unwrap_or_else(|_| "\"\"".to_string())
        )
    };

    let runtime_cfg = runtime_config::RuntimeConfig::load();
    let mut pool_config = PoolConfig::from_env();
    // Run mode should allow long-lived servers without timing out.
    pool_config.request_timeout_ms = 0;
    if let Some(enabled) = runtime_cfg.code_cache_enabled() {
        pool_config.enable_code_cache = enabled;
    }

    let serve_mode_for_extensions = serve_mode.clone();
    let extensions_provider = Arc::new(move || extensions_for_mode(&serve_mode_for_extensions));

    let engine = Arc::new(RuntimeEngine::new(
        pool_config.clone(),
        pool_config,
        &runtime_cfg,
        extensions_provider,
    ));
    let _ = set_engine(Arc::clone(&engine));

    let handler_key = HandlerKey::new(
        FsPath::new(&normalized)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(&normalized),
    );

    let request_value = serde_json::json!({
        "url": "http://localhost/run",
        "method": "GET",
        "headers": {},
        "body": "",
    });
    let execution_mode = ExecutionMode::Module;

    let response = engine
        .execute(
            handler_key,
            RequestData {
            handler_code,
            request_value,
            request_parts: None,
            mode: execution_mode,
        },
    )
    .await
    .map_err(|err| format!("Run failed: {}", err))?;

    if !response.success {
        if let Some(error) = response.error {
            if let Some(code) = parse_exit_code(&error) {
                std::process::exit(code);
            }
            return Err(format!("Run failed: {}", error));
        }
        return Err("Run failed: unknown error".to_string());
    }

    if let Some(result) = response.result.as_ref() {
        if let Some(code) = result.get("exit_code").and_then(|value| value.as_i64()) {
            std::process::exit(code as i32);
        }
    }

    // The JavaScript code already handles keep-alive by awaiting __dekaRuntimeHold
    // If a server is running, the promise won't resolve and execution won't complete
    // This matches Node.js/Bun/Deno behavior automatically
    Ok(())
}

fn handler_input(context: &Context) -> (String, Vec<String>) {
    let mut positionals = context.args.positionals.clone();
    let handler = positionals
        .get(0)
        .cloned()
        .or_else(|| std::env::var("HANDLER_PATH").ok())
        .unwrap_or_else(|| ".".to_string());
    let extra_args = if positionals.len() > 1 {
        positionals.split_off(1)
    } else {
        Vec::new()
    };

    // Check if handler is a bare name (no path separators) and might be a package.json script
    if !handler.contains('/') && !handler.contains('\\') && !handler.starts_with('.') {
        if let Some((script_cmd, script_args)) = resolve_package_script(&handler) {
            // Combine script args with extra args
            let mut combined_args = script_args;
            combined_args.extend(extra_args);
            return (script_cmd, combined_args);
        }
    }

    (handler, extra_args)
}

fn resolve_package_script(script_name: &str) -> Option<(String, Vec<String>)> {
    use std::fs;

    let cwd = std::env::current_dir().ok()?;
    let package_json_path = cwd.join("package.json");

    if !package_json_path.exists() {
        return None;
    }

    let content = fs::read_to_string(&package_json_path).ok()?;
    let package: serde_json::Value = serde_json::from_str(&content).ok()?;

    let scripts = package.get("scripts")?.as_object()?;
    let script = scripts.get(script_name)?.as_str()?;

    // Parse the script command to extract the actual entry point and args
    // Split on whitespace
    let parts: Vec<&str> = script.split_whitespace().collect();
    if parts.is_empty() {
        return None;
    }

    let entry = parts[0];
    let script_args: Vec<String> = parts[1..].iter().map(|s| s.to_string()).collect();

    // Try to resolve the entry to an actual file
    // First check if it's in node_modules/.bin
    let bin_path = cwd.join("node_modules").join(".bin").join(entry);
    if bin_path.exists() {
        // Follow symlink if it exists
        let resolved = fs::canonicalize(&bin_path).unwrap_or(bin_path);
        return Some((resolved.to_string_lossy().to_string(), script_args));
    }

    // Check if it's a relative/absolute path
    let path = std::path::Path::new(entry);
    if path.exists() {
        let resolved = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());
        return Some((resolved.to_string_lossy().to_string(), script_args));
    }

    // If not found, return None so caller can handle the error
    None
}

fn set_runtime_args(extra_args: &[String]) {
    if !extra_args.is_empty() {
        if let Ok(encoded) = serde_json::to_string(extra_args) {
            unsafe {
                std::env::set_var("DEKA_ARGS", encoded);
            }
        }
    }
    if let Some(bin) = std::env::args().next() {
        unsafe {
            std::env::set_var("DEKA_BIN", bin);
        }
    }
}

fn normalize_handler_path(path: &str) -> String {
    let path = std::path::Path::new(path);
    if path.is_absolute() {
        return path.to_string_lossy().to_string();
    }
    let cwd = match std::env::current_dir() {
        Ok(dir) => dir,
        Err(_) => return path.to_string_lossy().to_string(),
    };
    let joined = cwd.join(path);
    match joined.canonicalize() {
        Ok(canon) => canon.to_string_lossy().to_string(),
        Err(_) => joined.to_string_lossy().to_string(),
    }
}

fn parse_shebang(file_path: &str) -> Option<String> {
    use std::fs;
    use std::io::{BufRead, BufReader};

    let file = fs::File::open(file_path).ok()?;
    let mut reader = BufReader::new(file);
    let mut first_line = String::new();
    reader.read_line(&mut first_line).ok()?;

    if !first_line.starts_with("#!") {
        return None;
    }

    // Parse shebang: #!/usr/bin/env node OR #!/usr/bin/node
    let shebang = first_line.trim_start_matches("#!").trim();
    let parts: Vec<&str> = shebang.split_whitespace().collect();

    if parts.is_empty() {
        return None;
    }

    // Handle "#!/usr/bin/env node" format
    if parts[0].ends_with("/env") && parts.len() > 1 {
        return Some(parts[1].to_string());
    }

    // Handle "#!/usr/bin/node" format
    if let Some(cmd) = parts[0].split('/').last() {
        return Some(cmd.to_string());
    }

    None
}

fn run_with_shebang(cmd: &str, file_path: &str, args: &[String]) -> Result<(), String> {
    use std::process::Command;

    let mut command = Command::new(cmd);
    command.arg(file_path);
    command.args(args);

    // Inherit stdio so output goes to terminal
    command.stdin(std::process::Stdio::inherit());
    command.stdout(std::process::Stdio::inherit());
    command.stderr(std::process::Stdio::inherit());

    let status = command
        .status()
        .map_err(|err| format!("Failed to execute {}: {}", cmd, err))?;

    if !status.success() {
        let code = status.code().unwrap_or(1);
        std::process::exit(code);
    }

    Ok(())
}
