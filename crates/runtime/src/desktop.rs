use std::sync::Arc;
use std::path::PathBuf;

use crate::env::init_env;
use crate::extensions::extensions_for_mode;
use crate::vfs_loader::VfsProvider;
use core::Context;
use modules_js::modules::deka::mount_vfs;
use engine::{RuntimeEngine, RuntimeState, config as runtime_config, set_engine};
use pool::{HandlerKey, PoolConfig, RequestData, ExecutionMode};
use stdio as stdio_log;

use tao::{
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoop},
    window::WindowBuilder,
    dpi::LogicalSize,
};
use wry::{
    WebViewBuilder,
    http::{Response as HttpResponse, header::CONTENT_TYPE},
};
use std::borrow::Cow;

pub fn serve_desktop(_context: &Context, mut vfs: VfsProvider) {
    init_env();

    stdio_log::log("desktop", "Running in desktop mode");

    // Mount VFS globally
    let vfs_root = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("/"));
    let vfs_files = vfs.get_all_files();

    stdio_log::log("desktop", &format!("Mounting VFS with {} files", vfs_files.len()));
    mount_vfs(vfs_root.clone(), vfs_files.clone());

    let entry_point = vfs.entry_point().to_string();
    let handler_path = vfs_root.join(&entry_point);
    let handler_path_str = handler_path.to_string_lossy().to_string();

    stdio_log::log("desktop", &format!("loaded {} (from VFS)", entry_point));

    // Set handler path env var
    if std::env::var("HANDLER_PATH").is_err() {
        unsafe {
            std::env::set_var("HANDLER_PATH", &handler_path_str);
        }
    }

    // Determine serve mode
    let serve_mode = if entry_point.to_lowercase().ends_with(".php") {
        runtime_config::ServeMode::Php
    } else {
        runtime_config::ServeMode::Js
    };

    // Configure runtime
    let runtime_cfg = runtime_config::RuntimeConfig::load();
    let mut server_pool_config = PoolConfig::from_env();
    let user_pool_config = server_pool_config.clone();

    if let Some(enabled) = runtime_cfg.code_cache_enabled() {
        server_pool_config.enable_code_cache = enabled;
    }

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

    // Build handler code
    let handler_code = match serve_mode {
        runtime_config::ServeMode::Php => {
            stdio_log::warn("desktop", "PHP desktop mode not fully implemented yet");
            format!(
                "const app = globalThis.__dekaPhp.servePhp({});\\nglobalThis.app = app;",
                serde_json::to_string(&handler_path_str).unwrap_or_else(|_| "\"\"".to_string())
            )
        }
        _ => {
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

    let state = Arc::new(RuntimeState {
        engine: Arc::clone(&engine),
        handler_code,
        handler_key,
        perf_mode: false,
        perf_request_value,
    });

    // Create window and webview
    let event_loop = EventLoop::new();
    let window = WindowBuilder::new()
        .with_title("Deka App")
        .with_inner_size(LogicalSize::new(1200, 800))
        .build(&event_loop)
        .expect("Failed to create window");

    // Clone for custom protocol handler
    let vfs_files_clone = vfs_files.clone();
    let state_clone = Arc::clone(&state);

    let _webview = WebViewBuilder::new()
        .with_custom_protocol("deka".into(), move |_webview, request| {
            handle_deka_request(request, &vfs_files_clone, &state_clone)
        })
        .with_url("deka://app/")
        .build(&window)
        .expect("Failed to build webview");

    stdio_log::log("desktop", "Window opened");

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match event {
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                stdio_log::log("desktop", "Window closed");
                *control_flow = ControlFlow::Exit;
            }
            _ => (),
        }
    });
}

fn handle_deka_request(
    request: wry::http::Request<Vec<u8>>,
    vfs_files: &std::collections::HashMap<String, String>,
    state: &Arc<RuntimeState>,
) -> HttpResponse<Cow<'static, [u8]>> {
    let path = request.uri().path();

    // Normalize path (remove leading slash, default to index.html)
    let normalized_path = if path == "/" || path.is_empty() {
        "index.html"
    } else {
        path.trim_start_matches('/')
    };

    stdio_log::log("desktop", &format!("Request: {} -> {}", path, normalized_path));

    // Try to serve static file from VFS first
    // Try exact match first, then with .html extension
    let with_html_extension = format!("{}.html", normalized_path);
    let file_path = if vfs_files.contains_key(normalized_path) {
        normalized_path.to_string()
    } else if !normalized_path.ends_with(".html") && vfs_files.contains_key(&with_html_extension) {
        with_html_extension.clone()
    } else {
        normalized_path.to_string()
    };

    if let Some(content) = vfs_files.get(&file_path) {
        let mime_type = get_mime_type(&file_path);
        stdio_log::log("desktop", &format!("Serving static file: {} ({})", file_path, mime_type));

        return HttpResponse::builder()
            .header(CONTENT_TYPE, mime_type)
            .body(Cow::Owned(content.as_bytes().to_vec()))
            .unwrap_or_else(|e| {
                stdio_log::error("desktop", &format!("Failed to build response: {}", e));
                HttpResponse::builder()
                    .status(500)
                    .body(Cow::Borrowed(&b"Internal error"[..]))
                    .unwrap()
            });
    }

    // If not found in VFS, run through handler
    stdio_log::log("desktop", &format!("Running handler for: {}", path));

    let rt = match tokio::runtime::Runtime::new() {
        Ok(rt) => rt,
        Err(e) => {
            stdio_log::error("desktop", &format!("Failed to create runtime: {}", e));
            return HttpResponse::builder()
                .status(500)
                .body(Cow::Borrowed(&b"Failed to create runtime"[..]))
                .unwrap();
        }
    };

    let response: Result<pool::IsolateResponse, String> = rt.block_on(async {
        // Build request for handler
        let method = request.method().as_str();
        let headers: Vec<(String, String)> = request
            .headers()
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
            .collect();

        let request_value = serde_json::json!({
            "url": format!("http://localhost{}", path),
            "method": method,
            "headers": headers.into_iter().collect::<std::collections::HashMap<_, _>>(),
            "body": null,
        });

        // Execute handler
        let request_data = RequestData {
            handler_code: state.handler_code.clone(),
            request_value,
            request_parts: None,
            mode: ExecutionMode::Request,
        };

        state.engine.execute_user(state.handler_key.clone(), request_data).await
    });

    match response {
        Ok(result) => {
            if !result.success {
                let error_msg = result.error.unwrap_or_else(|| "Unknown error".to_string());
                stdio_log::error("desktop", &format!("Handler error: {}", error_msg));
                return HttpResponse::builder()
                    .status(500)
                    .body(Cow::Owned(format!("Error: {}", error_msg).into_bytes()))
                    .unwrap_or_else(|_| {
                        HttpResponse::builder()
                            .status(500)
                            .body(Cow::Borrowed(&b"Internal error"[..]))
                            .unwrap()
                    });
            }

            // Parse response from result.result
            let output = result.result.unwrap_or(serde_json::json!({}));

            let body = output.get("body")
                .and_then(|v: &serde_json::Value| v.as_str())
                .unwrap_or("")
                .as_bytes()
                .to_vec();

            let content_type = output.get("headers")
                .and_then(|h: &serde_json::Value| h.get("content-type"))
                .and_then(|v: &serde_json::Value| v.as_str())
                .unwrap_or("text/html");

            HttpResponse::builder()
                .header(CONTENT_TYPE, content_type)
                .body(Cow::Owned(body))
                .unwrap_or_else(|_| {
                    HttpResponse::builder()
                        .status(500)
                        .body(Cow::Borrowed(&b"Internal error"[..]))
                        .unwrap()
                })
        }
        Err(e) => {
            stdio_log::error("desktop", &format!("Handler error: {}", e));
            HttpResponse::builder()
                .status(500)
                .body(Cow::Owned(format!("Error: {}", e).into_bytes()))
                .unwrap_or_else(|_| {
                    HttpResponse::builder()
                        .status(500)
                        .body(Cow::Borrowed(&b"Internal error"[..]))
                        .unwrap()
                })
        }
    }
}

fn get_mime_type(path: &str) -> &'static str {
    if path.ends_with(".html") {
        "text/html"
    } else if path.ends_with(".css") {
        "text/css"
    } else if path.ends_with(".js") {
        "application/javascript"
    } else if path.ends_with(".json") {
        "application/json"
    } else if path.ends_with(".png") {
        "image/png"
    } else if path.ends_with(".jpg") || path.ends_with(".jpeg") {
        "image/jpeg"
    } else if path.ends_with(".svg") {
        "image/svg+xml"
    } else if path.ends_with(".woff") {
        "font/woff"
    } else if path.ends_with(".woff2") {
        "font/woff2"
    } else {
        "application/octet-stream"
    }
}
