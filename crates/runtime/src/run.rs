use std::path::Path as FsPath;
use std::sync::Arc;

use core::Context;
use engine::{RuntimeEngine, config as runtime_config, set_engine};
use modules_php::validation::{format_validation_error, modules::validate_module_resolution};
use pool::{ExecutionMode, HandlerKey, PoolConfig, RequestData};
use runtime_core::env::{set_default_log_level_with, set_handler_path_with, set_runtime_args_with};
use runtime_core::handler::{handler_input_with, is_html_entry, is_php_entry, normalize_handler_path};
use runtime_core::modules::ensure_phpx_module_root_env_with;
use runtime_core::process::parse_exit_code;
use crate::env::init_env;
use crate::extensions::extensions_for_mode;

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
    let env_get = |key: &str| std::env::var(key).ok();
    let mut env_set = |key: &str, value: &str| unsafe { std::env::set_var(key, value) };
    set_default_log_level_with(&env_get, &mut env_set);

    let (handler_path, extra_args) = handler_input(context);
    let mut env_set = |key: &str, value: &str| unsafe { std::env::set_var(key, value) };
    set_runtime_args_with(&extra_args, &mut env_set, &|| std::env::args().next());

    let mut env_set = |key: &str, value: &str| unsafe { std::env::set_var(key, value) };
    set_handler_path_with(&handler_path, &env_get, &mut env_set);

    let normalized = normalize_handler_path(&handler_path);
    if is_html_entry(&normalized) {
        return Err(format!(
            "Run mode does not support HTML entrypoints: {}",
            normalized
        ));
    }

    if !is_php_entry(&normalized) {
        return Err(format!(
            "Run mode in reboot MVP only supports .php and .phpx entrypoints: {}",
            normalized
        ));
    }
    let mut env_set = |key: &str, value: &str| unsafe { std::env::set_var(key, value) };
    ensure_phpx_module_root_env_with(
        &normalized,
        &|path| path.exists(),
        &|| std::env::current_exe().ok(),
        &env_get,
        &mut env_set,
    );
    validate_phpx_modules(&normalized)?;
    let serve_mode = runtime_config::ServeMode::Php;

    let _ = std::fs::read_to_string(&normalized)
        .map_err(|err| format!("Failed to read handler from {}: {}", normalized, err))?;

    let php_file = serde_json::to_string(&normalized).unwrap_or_else(|_| "\"\"".to_string());
    let handler_code = format!(
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
    );

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

    // The runtime hold promise keeps long-lived run-mode handlers alive.
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

fn handler_input(context: &Context) -> (String, Vec<String>) {
    handler_input_with(&context.args.positionals, &|key| std::env::var(key).ok())
}
