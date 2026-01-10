use std::sync::Arc;

use core::Context;
use engine::{RuntimeEngine, config as runtime_config, set_engine};
use pool::{ExecutionMode, HandlerKey, PoolConfig, RequestData};
use stdio as stdio_log;

use crate::env::init_env;
use crate::extensions::extensions_for_build;
pub fn build(context: &Context) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to start tokio runtime");

    if let Err(err) = rt.block_on(build_async(context)) {
        stdio_log::error("build", &err);
    }
}

async fn build_async(context: &Context) -> Result<(), String> {
    init_env();

    let entry = context
        .args
        .positionals
        .get(0)
        .cloned()
        .or_else(|| std::env::var("DEKA_BUILD_ENTRY").ok())
        .ok_or_else(|| "Missing build entry".to_string())?;

    let outdir = context
        .args
        .params
        .get("--outdir")
        .or_else(|| context.args.params.get("-o"))
        .cloned()
        .or_else(|| std::env::var("DEKA_BUILD_OUTDIR").ok())
        .unwrap_or_else(|| "./dist".to_string());

    let entry_json = serde_json::to_string(&entry).unwrap_or_else(|_| "\"\"".to_string());
    let outdir_json = serde_json::to_string(&outdir).unwrap_or_else(|_| "\"dist\"".to_string());
    let handler_code = format!(
        "globalThis.app = {{ fetch: async () => {{ await globalThis.__deka.build({{ entrypoints: [{}], outdir: {} }}); return {{ status: 200, headers: {{}}, body: \"build complete\" }}; }} }};",
        entry_json, outdir_json
    );

    let runtime_cfg = runtime_config::RuntimeConfig::load();
    let mut pool_config = PoolConfig::from_env();
    if let Some(enabled) = runtime_cfg.code_cache_enabled() {
        pool_config.enable_code_cache = enabled;
    }

    let extensions_provider = Arc::new(extensions_for_build);

    let engine = Arc::new(RuntimeEngine::new(
        pool_config.clone(),
        pool_config,
        &runtime_cfg,
        extensions_provider,
    ));
    let _ = set_engine(Arc::clone(&engine));

    let handler_key = HandlerKey::new("build");
    let request_value = serde_json::json!({
        "url": "http://localhost/build",
        "method": "GET",
        "headers": {},
        "body": "",
    });

    let response = engine
        .execute(
            handler_key,
            RequestData {
                handler_code,
                request_value,
                mode: ExecutionMode::Request,
            },
        )
        .await
        .map_err(|err| format!("Build failed: {}", err))?;

    if !response.success {
        return Err(format!(
            "Build failed: {}",
            response
                .error
                .unwrap_or_else(|| "unknown error".to_string())
        ));
    }

    stdio_log::log("build", "complete");
    Ok(())
}
