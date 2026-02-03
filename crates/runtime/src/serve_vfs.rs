use std::sync::Arc;
use std::time::Duration;

use crate::env::init_env;
use crate::extensions::extensions_for_mode;
use crate::vfs_loader::VfsProvider;
use core::Context;
use modules_js::modules::deka::mount_vfs;
use engine::{RuntimeEngine, RuntimeState, config as runtime_config, set_engine};
use pool::{HandlerKey, PoolConfig};
use stdio as stdio_log;
use transport::{HttpOptions, UnixOptions, TcpOptions, UdpOptions, DnsOptions, WsOptions, RedisOptions};

pub fn serve_vfs(context: &Context, mut vfs: VfsProvider) {
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("failed to start tokio runtime");

    if let Err(err) = rt.block_on(serve_vfs_async(context, &mut vfs)) {
        stdio_log::error("serve", &err);
    }
}

async fn serve_vfs_async(_context: &Context, vfs: &mut VfsProvider) -> Result<(), String> {
    init_env();

    stdio_log::log("vfs", "Running in VFS mode (compiled binary)");

    // Mount VFS globally so modules_js can read from it
    let vfs_root = std::env::current_dir().unwrap_or_else(|_| std::path::PathBuf::from("/"));
    let vfs_files = vfs.get_all_files();

    stdio_log::log("vfs", &format!("Mounting VFS with {} files at {}", vfs_files.len(), vfs_root.display()));
    mount_vfs(vfs_root.clone(), vfs_files);

    // Get entry point from VFS
    let entry_point = vfs.entry_point().to_string();

    // Create absolute path for module resolution
    let handler_path = vfs_root.join(&entry_point);
    let handler_path_str = handler_path.to_string_lossy().to_string();

    stdio_log::log("handler", &format!("loaded {} (from VFS)", entry_point));

    // Set handler path env var (use absolute path)
    if std::env::var("HANDLER_PATH").is_err() {
        unsafe {
            std::env::set_var("HANDLER_PATH", &handler_path_str);
        }
    }

    // Load handler source from VFS (this pre-caches it)
    let _handler_source = vfs.read_file(&entry_point)
        .map_err(|e| format!("Failed to load handler from VFS: {}", e))?;

    // Determine serve mode from file extension
    let lowered = entry_point.to_lowercase();
    let serve_mode = if lowered.ends_with(".php") || lowered.ends_with(".phpx") {
        runtime_config::ServeMode::Php
    } else {
        runtime_config::ServeMode::Js
    };
    if matches!(serve_mode, runtime_config::ServeMode::Php) {
        ensure_phpx_module_root(&vfs_root);
    }

    // Use default serve options for now (TODO: extract from handler if needed)
    let serve_options = pool::validation::ServeOptions::default();

    // Configure pools (simplified for VFS mode)
    let runtime_cfg = runtime_config::RuntimeConfig::load();
    let mut server_pool_config = PoolConfig::from_env();
    let user_pool_config = server_pool_config.clone();

    if let Some(enabled) = runtime_cfg.code_cache_enabled() {
        server_pool_config.enable_code_cache = enabled;
    }

    let server_pool_workers = server_pool_config.num_workers;

    let serve_mode_clone = serve_mode.clone();
    let extensions_provider = Arc::new(move || extensions_for_mode(&serve_mode_clone));

    let engine = Arc::new(RuntimeEngine::new(
        server_pool_config,
        user_pool_config,
        &runtime_cfg,
        extensions_provider,
    ));
    let _ = set_engine(Arc::clone(&engine));

    let handler_key = HandlerKey::new(&entry_point);

    // Build handler code (use absolute path for module resolution)
    let handler_code = match serve_mode {
        runtime_config::ServeMode::Php => {
            // For PHP in VFS mode, we need to write the file to a temp location
            // or pass the content directly (TODO: enhance PHP module)
            stdio_log::warn("vfs", "PHP VFS mode not fully implemented yet");
            format!(
                "const app = globalThis.__dekaPhp.servePhp({});\\nglobalThis.app = app;",
                serde_json::to_string(&handler_path_str).unwrap_or_else(|_| "\"\"".to_string())
            )
        }
        _ => {
            // For JS/TS, pass absolute path so module resolver can find it in VFS
            format!(
                "const app = globalThis.__dekaLoadModule({});",
                serde_json::to_string(&handler_path_str).unwrap_or_else(|_| "\"\"".to_string())
            )
        }
    };

    let perf_request_value = serde_json::json!({
        "url": "http://localhost/",
        "method": "GET",
        "headers": {},
        "body": null,
    });

    let perf_mode = std::env::var("DEKA_PERF_MODE")
        .map(|value| value != "false" && value != "0")
        .unwrap_or(false);

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

fn ensure_phpx_module_root(root: &std::path::Path) {
    if std::env::var("PHPX_MODULE_ROOT").is_ok() {
        return;
    }
    let candidate = root.join("deka.lock");
    if candidate.exists() {
        unsafe {
            std::env::set_var("PHPX_MODULE_ROOT", root);
        }
    }
}

async fn serve_listeners(
    state: Arc<RuntimeState>,
    serve_options: &pool::validation::ServeOptions,
    perf_mode: bool,
    server_pool_workers: usize,
) -> Result<(), String> {
    // Check for Unix socket
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

    // Check for TCP
    if let Some(addr) = serve_options
        .tcp
        .clone()
        .or_else(|| std::env::var("DEKA_TCP").ok())
    {
        stdio_log::log("listen", &format!("tcp://{}", addr));
        return transport::serve(state, transport::ListenConfig::Tcp(TcpOptions { addr })).await;
    }

    // Check for UDP
    if let Some(addr) = serve_options
        .udp
        .clone()
        .or_else(|| std::env::var("DEKA_UDP").ok())
    {
        stdio_log::log("listen", &format!("udp://{}", addr));
        return transport::serve(state, transport::ListenConfig::Udp(UdpOptions { addr })).await;
    }

    // Check for DNS
    if let Some(addr) = serve_options
        .dns
        .clone()
        .or_else(|| std::env::var("DEKA_DNS").ok())
    {
        stdio_log::log("listen", &format!("dns://{}", addr));
        return transport::serve(state, transport::ListenConfig::Dns(DnsOptions { addr })).await;
    }

    // Check for WebSocket
    if let Some(port) = serve_options.ws.or_else(|| {
        std::env::var("DEKA_WS")
            .ok()
            .and_then(|value| value.parse().ok())
    }) {
        stdio_log::log("listen", &format!("ws://localhost:{}", port));
        return transport::serve(state, transport::ListenConfig::Ws(WsOptions { port })).await;
    }

    // Check for Redis
    if let Some(addr) = serve_options
        .redis
        .clone()
        .or_else(|| std::env::var("DEKA_REDIS").ok())
    {
        stdio_log::log("listen", &format!("redis://{}", addr));
        return transport::serve(state, transport::ListenConfig::Redis(RedisOptions { addr }))
            .await;
    }

    // Default to HTTP
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
