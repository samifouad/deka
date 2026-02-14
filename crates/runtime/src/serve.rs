use std::path::Path as FsPath;
use std::sync::Arc;
use std::time::Duration;
use std::{sync::{Mutex, OnceLock}};

use crate::env::init_env;
use crate::extensions::extensions_for_mode;
use core::Context;
use engine::{RuntimeEngine, RuntimeState, config as runtime_config, set_engine};
use modules_php::validation::{format_validation_error, modules::validate_module_resolution};
use notify::Watcher;
use platform::Platform;
use platform_server::ServerPlatform;
use pool::validation::{PoolWorkers, extract_pool_options};
use pool::{HandlerKey, PoolConfig};
use runtime_core::env::{flag_or_env_truthy_with, set_dev_flag_with, set_handler_path_with};
use runtime_core::modules::ensure_phpx_module_root_env_with;
use runtime_core::php_pipeline::build_serve_handler_code;
use runtime_core::validation::validate_phpx_handler_with;
use stdio as stdio_log;
use transport::{
    DnsOptions, HttpOptions, RedisOptions, TcpOptions, UdpOptions, UnixOptions, WsOptions,
};

static WATCHER_GUARDS: OnceLock<Mutex<Vec<notify::RecommendedWatcher>>> = OnceLock::new();

pub fn serve(context: &Context) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to start tokio runtime");

    if let Err(err) = rt.block_on(serve_async(context)) {
        stdio_log::error("serve", &err);
    }
}

async fn serve_async(context: &Context) -> Result<(), String> {
    init_env();
    let platform = ServerPlatform::default();
    let env_get = |key: &str| platform.env().get(key);

    let dev_mode = dev_enabled(context);
    let watch_enabled = watch_enabled(context) || dev_mode;
    let mut env_set = |key: &str, value: &str| {
        let _ = platform.env().set(key, value);
    };
    set_dev_flag_with(dev_mode, &env_get, &mut env_set);
    let resolved = runtime_config::resolve_handler_path(&context.handler.input)
        .map_err(|err| format!("Failed to resolve handler path: {}", err))?;

    let handler_path = resolved.path.to_string_lossy().to_string();
    if !matches!(resolved.mode, runtime_config::ServeMode::Php) {
        return Err(format!(
            "Serve mode in reboot MVP only supports .php and .phpx handlers: {}",
            handler_path
        ));
    }
    let mut env_set = |key: &str, value: &str| {
        let _ = platform.env().set(key, value);
    };
    ensure_phpx_module_root_env_with(
        &handler_path,
        &|path| platform.fs().exists(path),
        &|| platform.fs().current_exe().ok(),
        &env_get,
        &mut env_set,
    );
    validate_phpx_modules(&handler_path)?;
    let mut env_set = |key: &str, value: &str| {
        let _ = platform.env().set(key, value);
    };
    set_handler_path_with(&handler_path, &env_get, &mut env_set);

    let handler_source = load_handler_source(&handler_path, &resolved.mode)?;

    stdio_log::log("handler", &format!("loaded {}", handler_path));
    if dev_mode {
        stdio_log::log("dev", "enabled");
    }

    let mut serve_options = pool::validation::ServeOptions::default();
    apply_cli_serve_overrides(context, &mut serve_options);

    let (server_pool_config, user_pool_config) = configure_pools(
        &handler_source,
        &handler_path,
        &serve_options,
        watch_enabled,
    );
    let server_pool_workers = server_pool_config.num_workers;

    let serve_mode = resolved.mode.clone();
    let extensions_provider = Arc::new(move || extensions_for_mode(&serve_mode));

    let runtime_cfg = runtime_config::RuntimeConfig::load();
    let engine = Arc::new(RuntimeEngine::new(
        server_pool_config,
        user_pool_config,
        &runtime_cfg,
        extensions_provider,
    ));
    let _ = set_engine(Arc::clone(&engine));

    if watch_enabled {
        if let Err(err) = start_watch(&handler_path, Arc::clone(&engine), dev_mode) {
            tracing::warn!("watch mode failed: {}", err);
        }
    }

    let handler_key = HandlerKey::new(
        FsPath::new(&handler_path)
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or(&handler_path),
    );

    let handler_code = build_handler_code(
        &handler_path,
        &resolved,
    );

    let perf_request_value = serde_json::json!({
        "url": "http://localhost/",
        "method": "GET",
        "headers": {},
        "body": null,
    });
    let perf_mode = perf_mode_enabled();

    let state = Arc::new(RuntimeState {
        engine: Arc::clone(&engine),
        handler_code,
        handler_key,
        perf_mode,
        perf_request_value,
    });

    spawn_archive_task(&state, engine.archive());

    serve_listeners(state, &serve_options, perf_mode, server_pool_workers).await
}

fn apply_cli_serve_overrides(context: &Context, serve_options: &mut pool::validation::ServeOptions) {
    if let Some(port) = context.args.params.get("--port") {
        if let Ok(value) = port.parse::<u16>() {
            serve_options.port = Some(value);
        }
    }
}

fn validate_phpx_modules(handler_path: &str) -> Result<(), String> {
    validate_phpx_handler_with(
        handler_path,
        &|path| {
            std::fs::read_to_string(path)
                .map_err(|err| format!("Failed to read PHPX handler {}: {}", path, err))
        },
        &|source, path| validate_module_resolution(source, path),
        &|source, path, error| format_validation_error(source, path, error),
    )
}

fn watch_enabled(context: &Context) -> bool {
    flag_or_env_truthy_with(
        &context.args.flags,
        "--watch",
        Some("-W"),
        "DEKA_WATCH",
        &|key| std::env::var(key).ok(),
    )
}

fn dev_enabled(context: &Context) -> bool {
    flag_or_env_truthy_with(
        &context.args.flags,
        "--dev",
        None,
        "DEKA_DEV",
        &|key| std::env::var(key).ok(),
    )
}

fn perf_mode_enabled() -> bool {
    std::env::var("DEKA_PERF_MODE")
        .map(|value| value != "false" && value != "0")
        .unwrap_or(false)
}

fn load_handler_source(
    _handler_path: &str,
    mode: &runtime_config::ServeMode,
) -> Result<String, String> {
    if matches!(mode, runtime_config::ServeMode::Php) {
        return Ok(String::new());
    }

    Err("Only PHP/PHPX serve mode is supported in reboot MVP".to_string())
}

fn configure_pools(
    handler_source: &str,
    handler_path: &str,
    serve_options: &pool::validation::ServeOptions,
    watch_enabled: bool,
) -> (PoolConfig, PoolConfig) {
    let runtime_cfg = runtime_config::RuntimeConfig::load();
    let mut server_pool_config = PoolConfig::from_env();
    let mut user_pool_config = server_pool_config.clone();

    if !handler_source.is_empty() {
        let pool_options = extract_pool_options(handler_source, handler_path);
        if let Some(workers) = pool_options.workers {
            user_pool_config.num_workers = match workers {
                PoolWorkers::Fixed(value) => {
                    if value < 1 {
                        1
                    } else {
                        value
                    }
                }
                PoolWorkers::Max => num_cpus::get(),
            };
        }
        if let Some(max) = pool_options.isolates_per_worker {
            user_pool_config.max_isolates_per_worker = max;
        }
    }

    if let Some(workers) = serve_options.workers.clone() {
        server_pool_config.num_workers = match workers {
            PoolWorkers::Fixed(value) => {
                if value < 1 {
                    1
                } else {
                    value
                }
            }
            PoolWorkers::Max => num_cpus::get(),
        };
    }
    if let Some(max) = serve_options.isolates_per_worker {
        server_pool_config.max_isolates_per_worker = max;
    }

    if let Some(enabled) = runtime_cfg.code_cache_enabled() {
        server_pool_config.enable_code_cache = enabled;
        user_pool_config.enable_code_cache = enabled;
    }

    if watch_enabled {
        server_pool_config.enable_code_cache = false;
        user_pool_config.enable_code_cache = false;
        server_pool_config.introspect_profiling = true;
        user_pool_config.introspect_profiling = true;
    }

    server_pool_config.introspect_profiling = runtime_cfg.introspect_profiling_enabled();
    user_pool_config.introspect_profiling = runtime_cfg.introspect_profiling_enabled();

    if perf_mode_enabled() {
        server_pool_config.enable_metrics = false;
        user_pool_config.enable_metrics = false;
        server_pool_config.introspect_profiling = false;
        user_pool_config.introspect_profiling = false;
    }

    (server_pool_config, user_pool_config)
}

fn build_handler_code(
    handler_path: &str,
    _resolved: &runtime_config::ResolvedHandler,
) -> String {
    build_serve_handler_code(handler_path)
}

async fn serve_listeners(
    state: Arc<RuntimeState>,
    serve_options: &pool::validation::ServeOptions,
    perf_mode: bool,
    server_pool_workers: usize,
) -> Result<(), String> {
    if let Some(unix) = serve_options
        .unix
        .clone()
        .or_else(|| std::env::var("DEKA_UNIX").ok())
    {
        let label = if unix.starts_with('\0') {
            format!("unix:@{}", unix.trim_start_matches('\0'))
        } else {
            format!("unix:{}", unix)
        };
        stdio_log::log("listen", &label);
        return transport::serve(
            state,
            transport::ListenConfig::Unix(UnixOptions { path: unix }),
        )
        .await;
    }

    if let Some(addr) = serve_options
        .tcp
        .clone()
        .or_else(|| std::env::var("DEKA_TCP").ok())
    {
        stdio_log::log("listen", &format!("tcp://{}", addr));
        return transport::serve(state, transport::ListenConfig::Tcp(TcpOptions { addr })).await;
    }

    if let Some(addr) = serve_options
        .udp
        .clone()
        .or_else(|| std::env::var("DEKA_UDP").ok())
    {
        stdio_log::log("listen", &format!("udp://{}", addr));
        return transport::serve(state, transport::ListenConfig::Udp(UdpOptions { addr })).await;
    }

    if let Some(addr) = serve_options
        .dns
        .clone()
        .or_else(|| std::env::var("DEKA_DNS").ok())
    {
        stdio_log::log("listen", &format!("dns://{}", addr));
        return transport::serve(state, transport::ListenConfig::Dns(DnsOptions { addr })).await;
    }

    if let Some(port) = serve_options.ws.or_else(|| {
        std::env::var("DEKA_WS")
            .ok()
            .and_then(|value| value.parse().ok())
    }) {
        stdio_log::log("listen", &format!("ws://localhost:{}", port));
        return transport::serve(state, transport::ListenConfig::Ws(WsOptions { port })).await;
    }

    if let Some(addr) = serve_options
        .redis
        .clone()
        .or_else(|| std::env::var("DEKA_REDIS").ok())
    {
        stdio_log::log("listen", &format!("redis://{}", addr));
        return transport::serve(state, transport::ListenConfig::Redis(RedisOptions { addr }))
            .await;
    }

    let port = serve_options
        .port
        .or_else(|| std::env::var("PORT").ok().and_then(|p| p.parse().ok()))
        .unwrap_or(8530);
    let listeners = server_pool_workers.max(1);

    stdio_log::log("listen", &format!("http://localhost:{}", port));
    transport::serve(
        state,
        transport::ListenConfig::Http(HttpOptions {
            port,
            listeners,
            perf_mode,
        }),
    )
    .await?;
    Ok(())
}

fn spawn_archive_task(state: &Arc<RuntimeState>, archive: Option<engine::IntrospectArchive>) {
    let Some(archive) = archive else {
        return;
    };
    let state = Arc::clone(state);
    tokio::spawn(async move {
        let mut interval = tokio::time::interval(Duration::from_secs(60));
        loop {
            interval.tick().await;
            let cutoff_ms = now_millis().saturating_sub(60_000);
            let traces = state.engine.drain_request_history_before(cutoff_ms).await;
            if traces.is_empty() {
                continue;
            }
            let archive = archive.clone();
            let _ = tokio::task::spawn_blocking(move || {
                let _ = archive.record_traces(&traces);
            })
            .await;
        }
    });
}

fn now_millis() -> u64 {
    use std::time::{SystemTime, UNIX_EPOCH};
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_millis() as u64
}

fn start_watch(handler_path: &str, engine: Arc<RuntimeEngine>, dev_mode: bool) -> Result<(), String> {
    let path = FsPath::new(handler_path);
    let watch_root = path.parent().unwrap_or_else(|| FsPath::new("."));
    let (tx, mut rx) = tokio::sync::mpsc::unbounded_channel::<notify::Result<notify::Event>>();

    let mut watcher = notify::recommended_watcher(move |res| {
        let _ = tx.send(res);
    })
    .map_err(|err| err.to_string())?;

    watcher
        .watch(watch_root, notify::RecursiveMode::Recursive)
        .map_err(|err| err.to_string())?;

    // Keep watcher alive for process lifetime; dropping it stops event delivery.
    if let Ok(mut guards) = WATCHER_GUARDS.get_or_init(|| Mutex::new(Vec::new())).lock() {
        guards.push(watcher);
    }

    tokio::spawn(async move {
        while let Some(event) = rx.recv().await {
            match event {
                Ok(event) => {
                    if dev_mode {
                        let mut changed = Vec::new();
                        for path in &event.paths {
                            changed.push(path.to_string_lossy().to_string());
                        }
                        if !changed.is_empty() {
                            stdio_log::log("hmr", &format!("changed {}", changed.join(", ")));
                            transport::notify_hmr_changed(&changed);
                        }
                    }
                    tokio::time::sleep(Duration::from_millis(5)).await;
                    let evicted = engine.pool().evict_all().await;
                    if evicted > 0 {
                        stdio_log::log("watch", &format!("evicted {}", evicted));
                    }
                }
                Err(err) => {
                    tracing::warn!("watch error: {}", err);
                }
            }
        }
    });

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::flag_or_env_truthy_with;
    use runtime_core::env::is_truthy;
    use std::collections::HashMap;

    #[test]
    fn truthy_parser_matches_expected_values() {
        assert!(is_truthy("1"));
        assert!(is_truthy("true"));
        assert!(is_truthy("yes"));
        assert!(is_truthy("on"));
        assert!(!is_truthy("0"));
        assert!(!is_truthy("false"));
        assert!(!is_truthy("off"));
    }

    #[test]
    fn flag_overrides_env_for_watch_or_dev() {
        let mut flags = HashMap::new();
        flags.insert("--dev".to_string(), true);
        assert!(flag_or_env_truthy_with(
            &flags,
            "--dev",
            None,
            "DEKA_DEV",
            &|_| None,
        ));

        let mut watch_flags = HashMap::new();
        watch_flags.insert("-W".to_string(), true);
        assert!(flag_or_env_truthy_with(
            &watch_flags,
            "--watch",
            Some("-W"),
            "DEKA_WATCH",
            &|_| None,
        ));
    }
}
