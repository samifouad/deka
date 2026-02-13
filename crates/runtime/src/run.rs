use std::path::Path as FsPath;
use std::sync::Arc;

use core::Context;
use engine::{RuntimeEngine, config as runtime_config, set_engine};
use modules_php::validation::{format_validation_error, modules::validate_module_resolution};
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

    let lowered = normalized.to_ascii_lowercase();
    let is_php = lowered.ends_with(".php") || lowered.ends_with(".phpx");
    if is_php {
        ensure_phpx_module_root(&normalized);
        validate_phpx_modules(&normalized)?;
    }
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
            "if (globalThis.process?.env?.DEKA_IPC_DEBUG === '1') {{ console.error('[HANDLER-LOAD] Loading module:', {}); }}\
globalThis.__dekaLoadModuleAsync({}).then(async () => {{\
if (globalThis.process?.env?.DEKA_IPC_DEBUG === '1') {{ console.error('[HANDLER-LOAD] Module loaded, checking runtime hold'); }}\
if (globalThis.__dekaRuntimeHold) {{ \
if (globalThis.process?.env?.DEKA_IPC_DEBUG === '1') {{ console.error('[HANDLER-LOAD] Waiting for runtime hold'); }}\
await globalThis.__dekaRuntimeHold; \
}}\
if (globalThis.process?.env?.DEKA_IPC_DEBUG === '1') {{ console.error('[HANDLER-LOAD] Handler execution complete'); }}\
}}).catch((err) => {{\
const msg = err && (err.stack || err.message) ? (err.stack || err.message) : String(err);\
if (String(msg).includes('DekaExit:')) {{ return; }}\
console.error(err);\
throw err;\
}});",
            serde_json::to_string(&normalized).unwrap_or_else(|_| "\"\"".to_string()),
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

fn validate_phpx_modules(handler_path: &str) -> Result<(), String> {
    if !handler_path.to_ascii_lowercase().ends_with(".phpx") {
        return Ok(());
    }
    let source = std::fs::read_to_string(handler_path)
        .map_err(|err| format!("Failed to read PHPX handler {}: {}", handler_path, err))?;
    let errors = validate_module_resolution(&source, handler_path);
    if errors.is_empty() {
        return Ok(());
    }
    let mut out = String::new();
    for error in errors.iter().take(3) {
        out.push_str(&format_validation_error(&source, handler_path, error));
    }
    if errors.len() > 3 {
        out.push_str(&format!(
            "\n... plus {} additional module validation error(s)\n",
            errors.len() - 3
        ));
    }
    Err(format!(
        "PHPX module graph validation failed for {}:\n{}",
        handler_path, out
    ))
}

fn ensure_phpx_module_root(handler_path: &str) {
    if std::env::var("PHPX_MODULE_ROOT").is_ok() {
        return;
    }
    let path = FsPath::new(handler_path);
    let start = if path.is_dir() {
        path
    } else {
        match path.parent() {
            Some(parent) => parent,
            None => return,
        }
    };
    let mut current = start.to_path_buf();
    loop {
        let candidate = current.join("deka.lock");
        if candidate.exists() {
            unsafe {
                std::env::set_var("PHPX_MODULE_ROOT", &current);
            }
            return;
        }
        if !current.pop() {
            break;
        }
    }
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
    (handler, extra_args)
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
