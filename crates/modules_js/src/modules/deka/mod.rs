use crate::redis_client;
use bundler;
use deno_core::v8;
use deno_core::{JsBuffer, error::CoreError, op2};
use engine;
use http::websocket as ws_transport;
use pool::validation::error_formatter;
use pool::{ExecutionMode, HandlerKey, RequestData, SortBy};
use swc_common::{
    DUMMY_SP, FileName, GLOBALS, Globals, Mark, SourceMap, Spanned, SyntaxContext, sync::Lrc,
};
use swc_ecma_ast::*;
use swc_ecma_codegen::{Emitter, text_writer::JsWriter};
use swc_ecma_parser::{
    EsSyntax, Parser, StringInput, Syntax, TsSyntax, error::SyntaxError, lexer::Lexer,
};
use swc_ecma_transforms_base::resolver;
use swc_ecma_transforms_react::{Options as JsxOptions, Runtime as JsxRuntime, react};
use swc_ecma_transforms_typescript::strip;
use swc_ecma_visit::{Fold, FoldWith, Visit, VisitWith};
use url::Url;
mod cache;
mod crypto;
mod fs;
mod network;
mod process;
mod stdin;
mod web;
use flate2::Compression;
use flate2::write::GzEncoder;
use serde::Deserialize;
use std::io::{ErrorKind, Write};

use cache::{op_read_handler_source, op_read_module_source};
use crypto::{
    op_crypto_aes_gcm_decrypt, op_crypto_aes_gcm_encrypt, op_crypto_digest,
    op_crypto_ecdh_compute_secret, op_crypto_ecdh_convert, op_crypto_ecdh_generate,
    op_crypto_ecdh_get_private, op_crypto_ecdh_get_public, op_crypto_ecdh_new,
    op_crypto_ecdh_set_private, op_crypto_generate_keypair, op_crypto_get_curves, op_crypto_hmac,
    op_crypto_key_equals, op_crypto_key_export_der, op_crypto_key_export_jwk,
    op_crypto_key_export_pem, op_crypto_key_from_der, op_crypto_key_from_jwk,
    op_crypto_key_from_pem, op_crypto_key_from_secret, op_crypto_key_info, op_crypto_key_public,
    op_crypto_pbkdf2, op_crypto_random, op_crypto_sign, op_crypto_verify,
};
use fs::{
    op_fs_append, op_fs_append_bytes, op_fs_close, op_fs_copy_file, op_fs_exists, op_fs_mkdir,
    op_fs_open, op_fs_read, op_fs_read_dir, op_fs_remove_file, op_fs_stat, op_fs_write,
    op_read_file, op_write_file, op_write_file_base64, resolve_path,
};
use network::{
    op_dns_lookup, op_dns_reverse, op_tcp_accept, op_tcp_close, op_tcp_connect, op_tcp_listen,
    op_tcp_listener_addr, op_tcp_listener_close, op_tcp_local_addr, op_tcp_peer_addr, op_tcp_read,
    op_tcp_shutdown, op_tcp_write, op_udp_bind, op_udp_close, op_udp_connect, op_udp_disconnect,
    op_udp_get_recv_buffer_size, op_udp_get_send_buffer_size, op_udp_join_multicast,
    op_udp_leave_multicast, op_udp_local_addr, op_udp_peer_addr, op_udp_recv, op_udp_send,
    op_udp_set_broadcast, op_udp_set_multicast_if, op_udp_set_multicast_loop,
    op_udp_set_multicast_ttl, op_udp_set_recv_buffer_size, op_udp_set_send_buffer_size,
    op_udp_set_ttl,
};
use process::{
    op_process_close_stdin, op_process_exit, op_process_kill, op_process_read_stderr,
    op_process_read_stdout, op_process_spawn, op_process_spawn_immediate, op_process_spawn_sync, op_process_wait,
    op_process_write_stdin, op_sleep,
};
use stdin::{op_stdin_read, op_stdin_set_raw_mode};
use web::{
    op_blob_create, op_blob_drop, op_blob_get, op_blob_size, op_blob_slice, op_blob_type,
    op_http_fetch, op_stream_close, op_stream_create, op_stream_drop, op_stream_enqueue,
    op_stream_read,
};

deno_core::extension!(
    deka_core,
    ops = [
        op_read_handler_source,
        op_read_module_source,
        op_read_env,
        op_stdout_write,
        op_stderr_write,
        op_read_file,
        op_fs_exists,
        op_fs_stat,
        op_fs_read_dir,
        op_fs_mkdir,
        op_fs_remove_file,
        op_fs_append,
        op_fs_append_bytes,
        op_fs_open,
        op_fs_close,
        op_fs_read,
        op_fs_write,
        op_fs_copy_file,
        op_zlib_gzip,
        op_execute_isolate,
        op_transform_module,
        op_introspect_stats,
        op_introspect_top,
        op_introspect_workers,
        op_introspect_isolate,
        op_introspect_kill_isolate,
        op_introspect_requests,
        op_introspect_evict,
        op_set_introspect_profiling,
        op_bundle_browser,
        op_bundle_browser_assets,
        op_bundle_css,
        op_transform_css,
        op_tailwind_process,
        op_write_file,
        op_write_file_base64,
        op_ws_send,
        op_ws_send_binary,
        op_ws_close,
        op_blob_create,
        op_blob_get,
        op_blob_size,
        op_blob_type,
        op_blob_slice,
        op_blob_drop,
        op_stream_create,
        op_stream_enqueue,
        op_stream_close,
        op_stream_read,
        op_stream_drop,
        op_crypto_random,
        op_crypto_digest,
        op_crypto_hmac,
        op_crypto_pbkdf2,
        op_crypto_aes_gcm_encrypt,
        op_crypto_aes_gcm_decrypt,
        op_crypto_key_info,
        op_crypto_key_from_secret,
        op_crypto_key_from_pem,
        op_crypto_key_from_der,
        op_crypto_key_from_jwk,
        op_crypto_key_export_pem,
        op_crypto_key_export_der,
        op_crypto_key_export_jwk,
        op_crypto_key_public,
        op_crypto_key_equals,
        op_crypto_sign,
        op_crypto_verify,
        op_crypto_generate_keypair,
        op_crypto_get_curves,
        op_crypto_ecdh_new,
        op_crypto_ecdh_generate,
        op_crypto_ecdh_get_public,
        op_crypto_ecdh_get_private,
        op_crypto_ecdh_set_private,
        op_crypto_ecdh_compute_secret,
        op_crypto_ecdh_convert,
        op_url_parse,
        op_http_fetch,
        op_process_exit,
        op_process_spawn,
        op_process_spawn_immediate,
        op_process_spawn_sync,
        op_process_read_stdout,
        op_process_read_stderr,
        op_process_write_stdin,
        op_process_close_stdin,
        op_process_wait,
        op_process_kill,
        op_sleep,
        op_stdin_read,
        op_stdin_set_raw_mode,
        op_redis_connect,
        op_redis_close,
        op_redis_call,
        op_redis_get_buffer,
        op_async_context_get,
        op_async_context_set,
        op_udp_bind,
        op_udp_send,
        op_udp_recv,
        op_udp_close,
        op_udp_local_addr,
        op_udp_peer_addr,
        op_udp_connect,
        op_udp_disconnect,
        op_udp_set_broadcast,
        op_udp_set_ttl,
        op_udp_set_multicast_ttl,
        op_udp_set_multicast_loop,
        op_udp_set_multicast_if,
        op_udp_join_multicast,
        op_udp_leave_multicast,
        op_udp_set_recv_buffer_size,
        op_udp_set_send_buffer_size,
        op_udp_get_recv_buffer_size,
        op_udp_get_send_buffer_size,
        op_dns_lookup,
        op_dns_reverse,
        op_tcp_listen,
        op_tcp_accept,
        op_tcp_connect,
        op_tcp_read,
        op_tcp_write,
        op_tcp_close,
        op_tcp_shutdown,
        op_tcp_local_addr,
        op_tcp_peer_addr,
        op_tcp_listener_addr,
        op_tcp_listener_close,
    ],
    esm_entry_point = "ext:deka_core/deka.js",
    esm = [dir "src/modules/deka", "deka.js"],
);

pub fn init() -> deno_core::Extension {
    deka_core::init_ops_and_esm()
}

pub use cache::{module_cache_stats, mount_vfs, is_vfs_mounted};

#[op2]
fn op_async_context_get<'a>(scope: &mut v8::HandleScope<'a>) -> v8::Local<'a, v8::Value> {
    scope.get_continuation_preserved_embedder_data()
}

#[op2(fast)]
fn op_async_context_set(scope: &mut v8::HandleScope, value: v8::Local<v8::Value>) {
    scope.set_continuation_preserved_embedder_data(value);
}

#[op2]
#[string]
fn op_bundle_browser(#[string] entry: String) -> Result<String, CoreError> {
    bundler::bundle_browser(&entry)
        .map_err(|err| CoreError::from(std::io::Error::new(std::io::ErrorKind::Other, err)))
}

#[derive(serde::Serialize)]
struct UrlParts {
    href: String,
    protocol: String,
    hostname: String,
    port: String,
    pathname: String,
    search: String,
    hash: String,
}

#[op2]
#[serde]
fn op_url_parse(
    #[string] input: String,
    #[string] base: Option<String>,
) -> Result<UrlParts, CoreError> {
    let url = if let Some(base) = base {
        let base_url = Url::parse(&base).map_err(|err| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                err.to_string(),
            ))
        })?;
        base_url.join(&input).map_err(|err| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                err.to_string(),
            ))
        })?
    } else {
        Url::parse(&input).map_err(|err| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                err.to_string(),
            ))
        })?
    };

    let protocol = format!("{}:", url.scheme());
    let hostname = url.host_str().unwrap_or("").to_string();
    let port = url.port().map(|p| p.to_string()).unwrap_or_default();
    let pathname = url.path().to_string();
    let search = url.query().map(|q| format!("?{}", q)).unwrap_or_default();
    let hash = url
        .fragment()
        .map(|h| format!("#{}", h))
        .unwrap_or_default();

    Ok(UrlParts {
        href: url.to_string(),
        protocol,
        hostname,
        port,
        pathname,
        search,
        hash,
    })
}

#[derive(serde::Serialize)]
struct JsBundleResult {
    code: String,
    css: Option<String>,
    assets: Vec<CssBundleAsset>,
}

#[op2]
#[serde]
fn op_bundle_browser_assets(#[string] entry: String) -> Result<JsBundleResult, CoreError> {
    let bundle = bundler::bundle_browser_assets(&entry)
        .map_err(|err| CoreError::from(std::io::Error::new(std::io::ErrorKind::Other, err)))?;
    Ok(JsBundleResult {
        code: bundle.code,
        css: bundle.css,
        assets: bundle
            .assets
            .into_iter()
            .map(|asset| CssBundleAsset {
                placeholder: asset.placeholder,
                file_name: asset.file_name,
                content_type: asset.content_type,
                body_base64: asset.body_base64,
            })
            .collect(),
    })
}

#[derive(serde::Deserialize)]
struct CssBundleOptions {
    css_modules: Option<bool>,
    minify: Option<bool>,
}

#[derive(serde::Serialize)]
struct CssBundleAsset {
    placeholder: String,
    file_name: String,
    content_type: String,
    body_base64: String,
}

#[derive(serde::Serialize)]
struct CssBundleResult {
    code: String,
    exports: Option<std::collections::HashMap<String, String>>,
    assets: Vec<CssBundleAsset>,
}

#[op2]
#[serde]
fn op_bundle_css(
    #[string] path: String,
    #[serde] options: Option<CssBundleOptions>,
) -> Result<CssBundleResult, CoreError> {
    let css_modules = options
        .as_ref()
        .and_then(|opts| opts.css_modules)
        .unwrap_or(false);
    let minify = options
        .as_ref()
        .and_then(|opts| opts.minify)
        .unwrap_or(true);
    let bundle = bundler::bundle_css(&path, css_modules, minify)
        .map_err(|err| CoreError::from(std::io::Error::new(std::io::ErrorKind::Other, err)))?;
    Ok(CssBundleResult {
        code: bundle.code,
        exports: bundle.exports,
        assets: bundle
            .assets
            .into_iter()
            .map(|asset| CssBundleAsset {
                placeholder: asset.placeholder,
                file_name: asset.file_name,
                content_type: asset.content_type,
                body_base64: asset.body_base64,
            })
            .collect(),
    })
}

#[op2]
#[serde]
fn op_transform_css(
    #[string] source: String,
    #[string] filename: String,
    #[serde] options: Option<CssBundleOptions>,
) -> Result<CssBundleResult, CoreError> {
    let css_modules = options
        .as_ref()
        .and_then(|opts| opts.css_modules)
        .unwrap_or(false);
    let minify = options
        .as_ref()
        .and_then(|opts| opts.minify)
        .unwrap_or(true);
    let bundle = bundler::transform_css(&source, &filename, css_modules, minify)
        .map_err(|err| CoreError::from(std::io::Error::new(std::io::ErrorKind::Other, err)))?;
    Ok(CssBundleResult {
        code: bundle.code,
        exports: bundle.exports,
        assets: bundle
            .assets
            .into_iter()
            .map(|asset| CssBundleAsset {
                placeholder: asset.placeholder,
                file_name: asset.file_name,
                content_type: asset.content_type,
                body_base64: asset.body_base64,
            })
            .collect(),
    })
}

#[op2(async)]
#[string]
async fn op_tailwind_process(
    #[string] css: String,
    #[serde] content: Vec<String>,
    #[string] base_dir: String,
) -> Result<String, CoreError> {
    tokio::task::spawn_blocking(move || {
        process_tailwind(css, content, base_dir)
            .map_err(|err| CoreError::from(std::io::Error::new(std::io::ErrorKind::Other, err)))
    })
    .await
    .map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            format!("Tailwind processing failed: {}", err),
        ))
    })?
}

#[derive(Debug, Deserialize)]
struct ZlibCompressOptions {
    level: Option<i32>,
}

fn compression_from_level(level: Option<i32>) -> Result<Compression, CoreError> {
    match level {
        None => Ok(Compression::default()),
        Some(value) => {
            if value == -1 {
                Ok(Compression::default())
            } else if (0..=9).contains(&value) {
                Ok(Compression::new(value as u32))
            } else {
                Err(CoreError::from(std::io::Error::new(
                    ErrorKind::InvalidInput,
                    format!("Invalid compression level: {}", value),
                )))
            }
        }
    }
}

#[op2]
#[buffer]
fn op_zlib_gzip(
    #[buffer] data: JsBuffer,
    #[serde] options: Option<ZlibCompressOptions>,
) -> Result<Vec<u8>, CoreError> {
    let compression = compression_from_level(options.and_then(|opts| opts.level))?;
    let mut encoder = GzEncoder::new(Vec::new(), compression);
    encoder.write_all(data.as_ref()).map_err(CoreError::from)?;
    encoder.finish().map_err(CoreError::from)
}

#[op2]
#[serde]
fn op_read_env() -> Result<std::collections::HashMap<String, String>, CoreError> {
    if std::env::var("DEKA_STDIO_DEBUG").is_ok() {
        eprintln!("[deka-stdio] op_read_env");
    }
    Ok(std::env::vars().collect())
}

#[op2]
fn op_stdout_write(#[buffer] data: JsBuffer) -> Result<(), CoreError> {
    if std::env::var("DEKA_STDIO_DEBUG").is_ok() {
        eprintln!("[deka-stdio] stdout bytes={}", data.len());
    }
    let mut stdout = std::io::stdout();
    stdout.write_all(&data).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            err.to_string(),
        ))
    })?;
    stdout.flush().map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            err.to_string(),
        ))
    })?;
    Ok(())
}

#[op2]
fn op_stderr_write(#[buffer] data: JsBuffer) -> Result<(), CoreError> {
    if std::env::var("DEKA_STDIO_DEBUG").is_ok() {
        eprintln!("[deka-stdio] stderr bytes={}", data.len());
    }
    let mut stderr = std::io::stderr();
    stderr.write_all(&data).map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            err.to_string(),
        ))
    })?;
    stderr.flush().map_err(|err| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            err.to_string(),
        ))
    })?;
    Ok(())
}

#[op2]
#[serde]
fn op_introspect_stats() -> Result<serde_json::Value, CoreError> {
    let engine = engine::engine().ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Runtime engine not initialized",
        ))
    })?;
    let mut stats = engine.pool().stats();
    if let Some(obj) = stats.as_object_mut() {
        obj.insert(
            "modules".to_string(),
            serde_json::json!({
                "cache": module_cache_stats(),
            }),
        );
    }
    Ok(stats)
}

#[op2(async)]
#[serde]
async fn op_introspect_top(
    #[string] sort: String,
    #[smi] limit: u32,
) -> Result<Vec<pool::IsolateMetrics>, CoreError> {
    let engine = engine::engine().ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Runtime engine not initialized",
        ))
    })?;
    let sort = match sort.to_ascii_lowercase().as_str() {
        "memory" => SortBy::Memory,
        "requests" => SortBy::Requests,
        _ => SortBy::Cpu,
    };
    let limit = if limit == 0 { 10 } else { limit as usize };
    Ok(engine.pool().get_top_isolates(sort, limit).await)
}

#[op2(async)]
#[serde]
async fn op_introspect_workers() -> Result<Vec<pool::WorkerStats>, CoreError> {
    let engine = engine::engine().ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Runtime engine not initialized",
        ))
    })?;
    Ok(engine.pool().get_worker_stats().await)
}

#[op2(async)]
#[serde]
async fn op_introspect_isolate(
    #[string] handler: String,
) -> Result<Option<pool::IsolateMetrics>, CoreError> {
    let engine = engine::engine().ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Runtime engine not initialized",
        ))
    })?;
    Ok(engine.pool().get_isolate_metrics(handler).await)
}

#[op2(async)]
#[serde]
async fn op_introspect_kill_isolate(
    #[string] handler: String,
) -> Result<serde_json::Value, CoreError> {
    let engine = engine::engine().ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Runtime engine not initialized",
        ))
    })?;
    match engine.pool().kill_isolate(handler).await {
        Ok(()) => Ok(serde_json::json!({ "status": "ok" })),
        Err(error) => Ok(serde_json::json!({ "status": "error", "error": error })),
    }
}

#[op2(async)]
#[serde]
async fn op_introspect_requests(
    #[smi] limit: u32,
    #[smi] archive: u8,
) -> Result<Vec<pool::RequestTrace>, CoreError> {
    let engine = engine::engine().ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Runtime engine not initialized",
        ))
    })?;
    let limit = if limit == 0 { 200 } else { limit as usize };
    let archive = archive != 0;
    if archive {
        if let Some(archive) = engine.archive() {
            let cutoff_ms = now_millis().saturating_sub(60_000);
            let result =
                tokio::task::spawn_blocking(move || archive.fetch_traces_before(limit, cutoff_ms))
                    .await;

            return Ok(match result {
                Ok(Ok(traces)) => traces,
                Ok(Err(err)) => {
                    tracing::warn!("introspect archive read failed: {}", err);
                    Vec::new()
                }
                Err(err) => {
                    tracing::warn!("introspect archive task failed: {}", err);
                    Vec::new()
                }
            });
        }
    }

    Ok(engine.pool().get_recent_requests(limit).await)
}

#[op2(async)]
#[serde]
async fn op_introspect_evict() -> Result<serde_json::Value, CoreError> {
    let engine = engine::engine().ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Runtime engine not initialized",
        ))
    })?;
    let evicted = engine.pool().evict_all().await;
    Ok(serde_json::json!({
        "success": true,
        "evicted": evicted,
        "message": format!("Evicted {} isolates from cache", evicted)
    }))
}

#[op2(async)]
#[serde]
async fn op_set_introspect_profiling(#[smi] enabled: u8) -> Result<serde_json::Value, CoreError> {
    let engine = engine::engine().ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Runtime engine not initialized",
        ))
    })?;
    let enabled = enabled != 0;
    let evicted = engine.pool().set_introspect_profiling(enabled).await;
    Ok(serde_json::json!({
        "enabled": enabled,
        "evicted": evicted
    }))
}

#[op2(async)]
#[serde]
async fn op_execute_isolate(
    #[string] handler_path: String,
    #[serde] request: serde_json::Value,
    #[string] handler_key: Option<String>,
) -> Result<serde_json::Value, CoreError> {
    let engine = engine::engine().ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Runtime engine not initialized",
        ))
    })?;

    let path = resolve_path(&handler_path)?;
    let handler_key = handler_key
        .map(HandlerKey::new)
        .unwrap_or_else(|| HandlerKey::new(path.to_string_lossy()));

    let handler_code = format!(
        "const app = globalThis.__dekaLoadModule({});",
        serde_json::to_string(&path.display().to_string()).unwrap_or_else(|_| "\"\"".to_string())
    );
    let request_data = RequestData {
        handler_code,
        request_value: request,
        request_parts: None,
        mode: ExecutionMode::Request,
    };

    let pool_response = engine
        .execute_user(handler_key, request_data)
        .await
        .map_err(|err| CoreError::from(std::io::Error::new(std::io::ErrorKind::Other, err)))?;

    if !pool_response.success {
        return Err(CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            pool_response
                .error
                .unwrap_or_else(|| "Unknown error".to_string()),
        )));
    }

    let result = pool_response.result.ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Handler returned no result",
        ))
    })?;

    Ok(result)
}

#[op2(fast)]
fn op_ws_send(#[bigint] id: u64, #[string] message: String) -> Result<(), CoreError> {
    ws_transport::send_text(id, message)
        .map_err(|err| CoreError::from(std::io::Error::new(std::io::ErrorKind::Other, err)))
}

#[op2(fast)]
fn op_ws_send_binary(#[bigint] id: u64, #[buffer] data: &[u8]) -> Result<(), CoreError> {
    ws_transport::send_binary(id, data)
        .map_err(|err| CoreError::from(std::io::Error::new(std::io::ErrorKind::Other, err)))
}

#[op2(fast)]
fn op_ws_close(
    #[bigint] id: u64,
    #[smi] code: i32,
    #[string] reason: String,
) -> Result<(), CoreError> {
    let code = u16::try_from(code).unwrap_or(1000);
    ws_transport::close_socket(id, code, reason)
        .map_err(|err| CoreError::from(std::io::Error::new(std::io::ErrorKind::Other, err)))
}

#[op2(async)]
#[bigint]
async fn op_redis_connect(#[string] url: Option<String>) -> Result<u64, CoreError> {
    redis_client::connect(url)
        .await
        .map_err(|err| CoreError::from(std::io::Error::new(std::io::ErrorKind::Other, err)))
}

#[op2(async)]
async fn op_redis_close(#[bigint] id: u64) -> Result<(), CoreError> {
    redis_client::close(id)
        .await
        .map_err(|err| CoreError::from(std::io::Error::new(std::io::ErrorKind::Other, err)))
}

#[op2(async)]
#[serde]
async fn op_redis_call(
    #[bigint] id: u64,
    #[string] command: String,
    #[serde] args: Vec<serde_json::Value>,
) -> Result<serde_json::Value, CoreError> {
    redis_client::execute(id, &command, args)
        .await
        .map_err(|err| CoreError::from(std::io::Error::new(std::io::ErrorKind::Other, err)))
}

#[op2(async)]
#[buffer]
async fn op_redis_get_buffer(
    #[bigint] id: u64,
    #[string] key: String,
) -> Result<Vec<u8>, CoreError> {
    redis_client::get_buffer(id, key)
        .await
        .map_err(|err| CoreError::from(std::io::Error::new(std::io::ErrorKind::Other, err)))
}

fn process_tailwind(css: String, content: Vec<String>, base_dir: String) -> Result<String, String> {
    let tailwind = which::which("tailwindcss").map_err(|_| {
        "tailwindcss not found in PATH. Install it to enable Tailwind bundling.".to_string()
    })?;

    let base_dir = std::path::PathBuf::from(base_dir);
    let config = find_tailwind_config(&base_dir);
    let input_path = std::env::temp_dir().join(format!("deka-tailwind-{}.css", nanoid::nanoid!()));
    let output_path =
        std::env::temp_dir().join(format!("deka-tailwind-out-{}.css", nanoid::nanoid!()));

    std::fs::write(&input_path, &css)
        .map_err(|err| format!("Failed to write tailwind input: {}", err))?;

    let mut cmd = std::process::Command::new(tailwind);
    cmd.arg("-i").arg(&input_path).arg("-o").arg(&output_path);

    for entry in content {
        cmd.arg("--content").arg(entry);
    }

    if let Some(config_path) = config {
        cmd.arg("-c").arg(config_path);
    }

    cmd.current_dir(&base_dir);
    let output = cmd
        .output()
        .map_err(|err| format!("Failed to run tailwindcss: {}", err))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        return Err(format!(
            "tailwindcss failed (status {})\n{}\n{}",
            output.status, stderr, stdout
        ));
    }

    let result = std::fs::read_to_string(&output_path)
        .map_err(|err| format!("Failed to read tailwind output: {}", err))?;

    let _ = std::fs::remove_file(&input_path);
    let _ = std::fs::remove_file(&output_path);

    Ok(result)
}

fn find_tailwind_config(base_dir: &std::path::Path) -> Option<std::path::PathBuf> {
    let names = [
        "tailwind.config.js",
        "tailwind.config.cjs",
        "tailwind.config.mjs",
        "tailwind.config.ts",
    ];
    for dir in base_dir.ancestors() {
        for name in names {
            let candidate = dir.join(name);
            if candidate.exists() {
                return Some(candidate);
            }
        }
    }
    None
}

#[derive(serde::Serialize)]
struct TransformedModule {
    code: String,
    deps: Vec<String>,
    top_level_await: bool,
}

#[op2]
#[serde]
fn op_transform_module(
    #[string] path: String,
    #[string] source: String,
) -> Result<TransformedModule, CoreError> {
    let transformed = transform_module(&path, &source)
        .map_err(|err| CoreError::from(std::io::Error::new(std::io::ErrorKind::Other, err)))?;
    Ok(transformed)
}

fn transform_module(path: &str, source: &str) -> Result<TransformedModule, String> {
    let cm: Lrc<SourceMap> = Default::default();
    let fm = cm.new_source_file(
        FileName::Custom(path.to_string()).into(),
        source.to_string(),
    );

    let is_ts = is_typescript(path);
    let is_jsx = is_jsx(path);
    let syntax = if is_ts {
        Syntax::Typescript(TsSyntax {
            tsx: is_jsx,
            decorators: false,
            dts: false,
            no_early_errors: true,
            disallow_ambiguous_jsx_like: true,
        })
    } else {
        Syntax::Es(EsSyntax {
            jsx: is_jsx,
            decorators: false,
            ..Default::default()
        })
    };

    let lexer = Lexer::new(syntax, EsVersion::Es2022, StringInput::from(&*fm), None);
    let mut parser = Parser::new_from(lexer);
    let module = parser
        .parse_module()
        .map_err(|err| format_parse_error(path, source, &cm, err))?;

    let mut deps = collect_deps(&module);
    let jsx_import_source = std::env::var("DEKA_JSX_IMPORT_SOURCE")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "deka".to_string());
    let jsx_runtime = format!("{}/jsx-runtime", jsx_import_source);
    if is_jsx && !deps.iter().any(|dep| dep == &jsx_runtime) {
        deps.push(jsx_runtime);
    }
    let top_level_await = has_top_level_await(&module);

    let transformed = GLOBALS.set(&Globals::new(), || {
        let unresolved_mark = Mark::new();
        let top_level_mark = Mark::new();
        let mut program = Program::Module(module);
        let mut pass = resolver(unresolved_mark, top_level_mark, false);
        pass.process(&mut program);

        if is_ts {
            let mut pass = strip(unresolved_mark, top_level_mark);
            pass.process(&mut program);
        }

        if is_jsx {
            let mut options = JsxOptions::default();
            options.runtime = Some(JsxRuntime::Automatic);
            options.import_source = Some(jsx_import_source.into());
            let mut pass = react(
                cm.clone(),
                None::<swc_common::comments::SingleThreadedComments>,
                options,
                top_level_mark,
                unresolved_mark,
            );
            pass.process(&mut program);
        }

        let module = match program {
            Program::Module(module) => module,
            Program::Script(_) => return Err("Expected module after transform passes".to_string()),
        };

        let mut import_meta = ImportMetaTransformer;
        let module = module.fold_with(&mut import_meta);
        let module = module.fold_with(&mut ModuleTransformer::new());
        let mut buf = Vec::new();
        let mut cfg = swc_ecma_codegen::Config::default();
        cfg.minify = false;
        {
            let mut emitter = Emitter {
                cfg,
                cm: cm.clone(),
                comments: None,
                wr: JsWriter::new(cm, "\n", &mut buf, None),
            };
            emitter
                .emit_module(&module)
                .map_err(|err| err.to_string())?;
        }
        let code = String::from_utf8(buf).map_err(|err| err.to_string())?;
        Ok::<String, String>(code)
    })?;

    Ok(TransformedModule {
        code: transformed,
        deps,
        top_level_await,
    })
}

fn is_typescript(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".ts") || lower.ends_with(".tsx")
}

fn is_jsx(path: &str) -> bool {
    let lower = path.to_ascii_lowercase();
    lower.ends_with(".jsx") || lower.ends_with(".tsx")
}

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_else(|_| std::time::Duration::from_secs(0))
        .as_millis() as u64
}

fn format_parse_error(
    path: &str,
    source: &str,
    cm: &SourceMap,
    err: swc_ecma_parser::error::Error,
) -> String {
    let span = err.span();
    let loc_start = cm.lookup_char_pos(span.lo);
    let loc_end = cm.lookup_char_pos(span.hi);
    let line = loc_start.line;
    let col = loc_start.col.0 + 1;
    let underline_length = if loc_start.line == loc_end.line {
        (loc_end.col.0.saturating_sub(loc_start.col.0)).max(1)
    } else {
        1
    };

    let (message, hint) = match err.kind() {
        SyntaxError::Eof => (
            "Unexpected end of file".to_string(),
            "Check for an unclosed block, string, or parenthesis.".to_string(),
        ),
        SyntaxError::UnterminatedStrLit => (
            "Unterminated string literal".to_string(),
            "Add the missing closing quote.".to_string(),
        ),
        SyntaxError::UnterminatedTpl => (
            "Unterminated template literal".to_string(),
            "Add the missing closing backtick or ${} bracket.".to_string(),
        ),
        SyntaxError::UnterminatedRegExp => (
            "Unterminated regular expression".to_string(),
            "Add the missing closing /.".to_string(),
        ),
        SyntaxError::InvalidStrEscape => (
            "Invalid string escape".to_string(),
            "Check backslash escapes like \\n, \\t, or \\\".".to_string(),
        ),
        SyntaxError::InvalidUnicodeEscape => (
            "Invalid unicode escape".to_string(),
            "Use \\uXXXX or \\u{...} for unicode escapes.".to_string(),
        ),
        SyntaxError::ExpectedUnicodeEscape => (
            "Expected unicode escape".to_string(),
            "After \\u, provide four hex digits or \\u{...}.".to_string(),
        ),
        SyntaxError::BadCharacterEscapeSequence { expected } => (
            format!("Invalid escape sequence, expected {}", expected),
            "Check the escape sequence in the string literal.".to_string(),
        ),
        SyntaxError::InvalidIdentChar => (
            "Invalid identifier character".to_string(),
            "Remove the invalid character or quote the property name.".to_string(),
        ),
        SyntaxError::LegacyOctal => (
            "Legacy octal literal not allowed".to_string(),
            "Use 0o... for octal, or use decimal.".to_string(),
        ),
        SyntaxError::LineBreakInThrow => (
            "Line break after throw".to_string(),
            "Keep the throw expression on the same line.".to_string(),
        ),
        SyntaxError::LineBreakBeforeArrow => (
            "Line break before =>".to_string(),
            "Move the arrow to the same line as the parameters.".to_string(),
        ),
        SyntaxError::TopLevelAwaitInScript => (
            "Top-level await is not allowed here".to_string(),
            "Wrap in an async function or use a module context.".to_string(),
        ),
        SyntaxError::Unexpected { got, expected } => (
            format!("Unexpected token {}, expected {}", got, expected),
            "Check for missing punctuation or a stray character.".to_string(),
        ),
        _ => (
            format!("{:?}", err.kind()),
            "Check the syntax near the highlighted location.".to_string(),
        ),
    };

    error_formatter::format_validation_error(
        source,
        path,
        "Parse Error",
        line,
        col,
        &message,
        &hint,
        underline_length,
    )
}

fn collect_deps(module: &Module) -> Vec<String> {
    struct Collector {
        deps: Vec<String>,
    }

    impl Visit for Collector {
        fn visit_import_decl(&mut self, n: &ImportDecl) {
            if n.type_only {
                return;
            }
            self.deps.push(n.src.value.to_string_lossy().into_owned());
        }

        fn visit_export_all(&mut self, n: &ExportAll) {
            if n.type_only {
                return;
            }
            self.deps.push(n.src.value.to_string_lossy().into_owned());
        }

        fn visit_named_export(&mut self, n: &NamedExport) {
            if n.type_only {
                return;
            }
            if let Some(src) = &n.src {
                self.deps.push(src.value.to_string_lossy().into_owned());
            }
        }
    }

    let mut collector = Collector { deps: Vec::new() };
    collector.visit_module(module);
    collector.deps
}

fn has_top_level_await(module: &Module) -> bool {
    struct Detector {
        in_function: usize,
        found: bool,
    }

    impl Detector {
        fn in_function_scope<F: FnOnce(&mut Detector)>(&mut self, f: F) {
            self.in_function += 1;
            f(self);
            self.in_function = self.in_function.saturating_sub(1);
        }
    }

    impl Visit for Detector {
        fn visit_function(&mut self, n: &Function) {
            self.in_function_scope(|this| n.visit_children_with(this));
        }

        fn visit_arrow_expr(&mut self, n: &ArrowExpr) {
            self.in_function_scope(|this| n.visit_children_with(this));
        }

        fn visit_getter_prop(&mut self, n: &GetterProp) {
            self.in_function_scope(|this| n.visit_children_with(this));
        }

        fn visit_setter_prop(&mut self, n: &SetterProp) {
            self.in_function_scope(|this| n.visit_children_with(this));
        }

        fn visit_await_expr(&mut self, _: &AwaitExpr) {
            if self.in_function == 0 {
                self.found = true;
            }
        }
    }

    let mut detector = Detector {
        in_function: 0,
        found: false,
    };
    detector.visit_module(module);
    detector.found
}

struct ImportMetaTransformer;

impl Fold for ImportMetaTransformer {
    fn fold_expr(&mut self, expr: Expr) -> Expr {
        match expr {
            Expr::MetaProp(meta) => {
                if matches!(meta.kind, MetaPropKind::ImportMeta) {
                    Expr::Ident(ident_from("__dekaImportMeta"))
                } else {
                    Expr::MetaProp(meta)
                }
            }
            _ => expr.fold_children_with(self),
        }
    }
}

struct ModuleTransformer {
    temp_index: usize,
}

impl ModuleTransformer {
    fn new() -> Self {
        Self { temp_index: 0 }
    }

    fn next_temp(&mut self, prefix: &str) -> Ident {
        let ident = format!("{}_{}", prefix, self.temp_index);
        self.temp_index += 1;
        ident_from(&ident)
    }
}

impl Fold for ModuleTransformer {
    fn fold_module(&mut self, module: Module) -> Module {
        let mut body = Vec::new();
        for item in module.body {
            match item {
                ModuleItem::Stmt(stmt) => body.push(ModuleItem::Stmt(stmt)),
                ModuleItem::ModuleDecl(decl) => {
                    self.transform_decl(decl, &mut body);
                }
            }
        }
        Module { body, ..module }
    }
}

impl ModuleTransformer {
    fn transform_decl(&mut self, decl: ModuleDecl, out: &mut Vec<ModuleItem>) {
        match decl {
            ModuleDecl::Import(import) => {
                for stmt in transform_import(import) {
                    out.push(ModuleItem::Stmt(stmt));
                }
            }
            ModuleDecl::ExportDefaultExpr(expr) => {
                out.push(ModuleItem::Stmt(export_default_expr(*expr.expr)));
            }
            ModuleDecl::ExportDefaultDecl(decl) => {
                self.handle_export_default_decl(decl.decl, out);
            }
            ModuleDecl::ExportDecl(export_decl) => {
                self.handle_export_decl(export_decl.decl, out);
            }
            ModuleDecl::ExportNamed(named) => {
                if named.type_only {
                    return;
                }
                self.handle_export_named(named, out);
            }
            ModuleDecl::ExportAll(export_all) => {
                self.handle_export_all(export_all, out);
            }
            ModuleDecl::TsImportEquals(_)
            | ModuleDecl::TsExportAssignment(_)
            | ModuleDecl::TsNamespaceExport(_) => {}
        }
    }

    fn handle_export_default_decl(&mut self, decl: DefaultDecl, out: &mut Vec<ModuleItem>) {
        match decl {
            DefaultDecl::Fn(func) => {
                if let Some(ident) = func.ident.clone() {
                    out.push(ModuleItem::Stmt(Stmt::Decl(Decl::Fn(FnDecl {
                        ident: ident.clone(),
                        declare: false,
                        function: func.function,
                    }))));
                    out.push(ModuleItem::Stmt(export_default_ident(&ident)));
                } else {
                    let expr = Expr::Fn(FnExpr {
                        ident: None,
                        function: func.function,
                    });
                    out.push(ModuleItem::Stmt(export_default_expr(expr)));
                }
            }
            DefaultDecl::Class(class_decl) => {
                if let Some(ident) = class_decl.ident.clone() {
                    out.push(ModuleItem::Stmt(Stmt::Decl(Decl::Class(ClassDecl {
                        ident: ident.clone(),
                        declare: false,
                        class: class_decl.class,
                    }))));
                    out.push(ModuleItem::Stmt(export_default_ident(&ident)));
                } else {
                    let expr = Expr::Class(ClassExpr {
                        ident: None,
                        class: class_decl.class,
                    });
                    out.push(ModuleItem::Stmt(export_default_expr(expr)));
                }
            }
            DefaultDecl::TsInterfaceDecl(_) => {}
        }
    }

    fn handle_export_decl(&mut self, decl: Decl, out: &mut Vec<ModuleItem>) {
        out.push(ModuleItem::Stmt(Stmt::Decl(decl.clone())));
        let idents = extract_decl_idents(&decl);
        for ident in idents {
            out.push(ModuleItem::Stmt(export_named_ident(
                &ident,
                &ident.sym.to_string(),
            )));
        }
    }

    fn handle_export_named(&mut self, named: NamedExport, out: &mut Vec<ModuleItem>) {
        if let Some(src) = named.src {
            let mod_ident = self.next_temp("__dekaMod");
            out.push(ModuleItem::Stmt(import_to_ident(
                &mod_ident,
                src.value.to_string_lossy().into_owned(),
            )));
            for spec in named.specifiers {
                match spec {
                    ExportSpecifier::Named(named_spec) => {
                        let local = export_name_to_string(&named_spec.orig);
                        let exported = named_spec
                            .exported
                            .as_ref()
                            .map(export_name_to_string)
                            .unwrap_or_else(|| local.clone());
                        out.push(ModuleItem::Stmt(export_from_module(
                            &mod_ident, &local, &exported,
                        )));
                    }
                    ExportSpecifier::Default(default_spec) => {
                        let exported = default_spec.exported.sym.to_string();
                        out.push(ModuleItem::Stmt(export_from_module(
                            &mod_ident, "default", &exported,
                        )));
                    }
                    ExportSpecifier::Namespace(ns_spec) => {
                        let exported = export_name_to_string(&ns_spec.name);
                        out.push(ModuleItem::Stmt(export_namespace_from_module(
                            &mod_ident, &exported,
                        )));
                    }
                }
            }
        } else {
            for spec in named.specifiers {
                if let ExportSpecifier::Named(named_spec) = spec {
                    let local = export_name_to_string(&named_spec.orig);
                    let exported = named_spec
                        .exported
                        .as_ref()
                        .map(export_name_to_string)
                        .unwrap_or_else(|| local.clone());
                    let local_ident = ident_from(&local);
                    out.push(ModuleItem::Stmt(export_named_ident(
                        &local_ident,
                        &exported,
                    )));
                }
            }
        }
    }

    fn handle_export_all(&mut self, export_all: ExportAll, out: &mut Vec<ModuleItem>) {
        if export_all.type_only {
            return;
        }
        let mod_ident = self.next_temp("__dekaMod");
        let key_ident = self.next_temp("__dekaKey");
        out.push(ModuleItem::Stmt(import_to_ident(
            &mod_ident,
            export_all.src.value.to_string_lossy().into_owned(),
        )));
        out.push(ModuleItem::Stmt(export_all_loop(mod_ident, key_ident)));
    }
}

fn ident_from(name: &str) -> Ident {
    Ident::new(name.into(), DUMMY_SP, SyntaxContext::empty())
}

fn ident_name_from(name: &str) -> IdentName {
    IdentName::new(name.into(), DUMMY_SP)
}

fn export_name_to_string(name: &ModuleExportName) -> String {
    match name {
        ModuleExportName::Ident(ident) => ident.sym.to_string(),
        ModuleExportName::Str(str) => str.value.to_string_lossy().into_owned(),
    }
}

#[cfg(test)]
mod tests {
    use super::transform_module;

    fn fixture(name: &str) -> String {
        let root = env!("CARGO_MANIFEST_DIR");
        let path = format!("{}/src/modules/deka/fixtures/{}", root, name);
        std::fs::read_to_string(path).expect("fixture should load")
    }

    #[test]
    fn transforms_imports_and_exports() {
        let source = r#"
import foo from "./foo"
import { bar as baz } from "./bar"
import * as ns from "./ns"

export const value = 1
export default function main() { return value }
export { value as renamed }
export * from "./reexp"
export * as star from "./reexp2"
"#;

        let result = transform_module("app.ts", source).expect("transform should succeed");
        let code = result.code;

        assert!(code.contains("__dekaImport"));
        assert!(code.contains("exports.default"));
        assert!(code.contains("exports.renamed"));
        assert!(code.contains("exports.star"));
        assert!(code.contains("exports["));

        let mut deps = result.deps;
        deps.sort();
        assert_eq!(
            deps,
            vec![
                "./bar".to_string(),
                "./foo".to_string(),
                "./ns".to_string(),
                "./reexp".to_string(),
                "./reexp2".to_string(),
            ]
        );
    }

    #[test]
    fn strips_typescript_types() {
        let source = r#"
export interface User { id: string }
export type Status = "ok" | "fail"
export const value = 1
"#;
        let result = transform_module("types.ts", source).expect("transform should succeed");
        assert!(!result.code.contains("interface"));
        assert!(!result.code.contains("type Status"));
        assert!(result.code.contains("exports.value"));
    }

    #[test]
    fn formats_parse_errors_with_code_frame() {
        let source = "export default function(";
        let err = match transform_module("broken.ts", source) {
            Ok(_) => panic!("transform should fail"),
            Err(err) => err,
        };
        assert!(err.contains("Validation Error"));
        assert!(err.contains("❌ Parse Error"));
        assert!(err.contains("broken.ts"));
    }

    #[test]
    fn transforms_default_export_fixture() {
        let source = fixture("default_export.ts");
        let result =
            transform_module("default_export.ts", &source).expect("transform should succeed");
        assert!(result.code.contains("exports.default"));
        assert!(result.deps.is_empty());
    }

    #[test]
    fn transforms_mixed_imports_fixture() {
        let source = fixture("mixed_imports.ts");
        let result =
            transform_module("mixed_imports.ts", &source).expect("transform should succeed");
        assert!(result.code.contains("__dekaImport"));
        assert!(result.code.contains("exports.default"));
        let mut deps = result.deps;
        deps.sort();
        assert_eq!(deps, vec!["./mod".to_string(), "./ns".to_string()]);
    }

    #[test]
    fn transforms_reexports_fixture() {
        let source = fixture("reexports.ts");
        let result = transform_module("reexports.ts", &source).expect("transform should succeed");
        assert!(result.code.contains("exports.ns"));
        let mut deps = result.deps;
        deps.sort();
        assert_eq!(
            deps,
            vec![
                "./bar".to_string(),
                "./baz".to_string(),
                "./foo".to_string(),
                "./ns".to_string(),
            ]
        );
    }

    #[test]
    fn transforms_ts_types_fixture() {
        let source = fixture("types.ts");
        let result = transform_module("types.ts", &source).expect("transform should succeed");
        assert!(!result.code.contains("interface"));
        assert!(!result.code.contains("type Status"));
        assert!(result.code.contains("exports.value"));
    }

    #[test]
    fn transforms_tsx_fixture() {
        let source = fixture("tsx.tsx");
        let result = transform_module("tsx.tsx", &source).expect("transform should succeed");
        assert!(result.code.contains("exports.default"));
        let mut deps = result.deps;
        deps.sort();
        assert_eq!(
            deps,
            vec!["deka/jsx-runtime".to_string(), "deka/router".to_string()]
        );
    }

    #[test]
    fn ignores_type_only_imports() {
        let source = fixture("type_only.ts");
        let result = transform_module("type_only.ts", &source).expect("transform should succeed");
        assert!(!result.code.contains("__dekaImport"));
        assert_eq!(result.deps, Vec::<String>::new());
    }

    #[test]
    fn detects_top_level_await() {
        let source = fixture("top_level_await.ts");
        let result =
            transform_module("top_level_await.ts", &source).expect("transform should succeed");
        assert!(result.top_level_await);
    }
}

fn transform_import(import: ImportDecl) -> Vec<Stmt> {
    if import.type_only {
        return Vec::new();
    }
    let mut default_ident: Option<Ident> = None;
    let mut namespace_ident: Option<Ident> = None;
    let mut named_props: Vec<ObjectPatProp> = Vec::new();

    for spec in import.specifiers {
        match spec {
            ImportSpecifier::Default(default) => {
                default_ident = Some(default.local);
            }
            ImportSpecifier::Namespace(ns) => {
                namespace_ident = Some(ns.local);
            }
            ImportSpecifier::Named(named) => {
                let local = named.local;
                let key = match named.imported {
                    Some(ModuleExportName::Ident(ident)) => PropName::Ident(ident.into()),
                    Some(ModuleExportName::Str(str)) => PropName::Str(str),
                    None => PropName::Ident(local.clone().into()),
                };
                let value = Pat::Ident(BindingIdent {
                    id: local,
                    type_ann: None,
                });
                named_props.push(ObjectPatProp::KeyValue(KeyValuePatProp {
                    key,
                    value: Box::new(value),
                }));
            }
        }
    }

    let mut statements = Vec::new();

    if namespace_ident.is_some() {
        let ident = namespace_ident.unwrap();
        statements.push(import_to_ident(
            &ident,
            import.src.value.to_string_lossy().into_owned(),
        ));
    }

    if default_ident.is_some() || !named_props.is_empty() {
        if let Some(default) = default_ident {
            named_props.insert(
                0,
                ObjectPatProp::KeyValue(KeyValuePatProp {
                    key: PropName::Ident(ident_name_from("default")),
                    value: Box::new(Pat::Ident(BindingIdent {
                        id: default,
                        type_ann: None,
                    })),
                }),
            );
        }
        let pat = Pat::Object(ObjectPat {
            span: DUMMY_SP,
            props: named_props,
            optional: false,
            type_ann: None,
        });
        let init = import_call(import.src.value.to_string_lossy().into_owned());
        statements.push(Stmt::Decl(Decl::Var(Box::new(VarDecl {
            span: DUMMY_SP,
            kind: VarDeclKind::Const,
            declare: false,
            ctxt: SyntaxContext::empty(),
            decls: vec![VarDeclarator {
                span: DUMMY_SP,
                name: pat,
                init: Some(Box::new(init)),
                definite: false,
            }],
        }))));
    }

    if statements.is_empty() {
        statements.push(Stmt::Expr(ExprStmt {
            span: DUMMY_SP,
            expr: Box::new(import_call(import.src.value.to_string_lossy().into_owned())),
        }));
    }

    statements
}

fn import_call(specifier: String) -> Expr {
    Expr::Call(CallExpr {
        span: DUMMY_SP,
        ctxt: SyntaxContext::empty(),
        callee: Callee::Expr(Box::new(Expr::Ident(ident_from("__dekaImport")))),
        args: vec![ExprOrSpread {
            spread: None,
            expr: Box::new(Expr::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: specifier.into(),
                raw: None,
            }))),
        }],
        type_args: None,
    })
}

fn import_to_ident(ident: &Ident, specifier: String) -> Stmt {
    Stmt::Decl(Decl::Var(Box::new(VarDecl {
        span: DUMMY_SP,
        kind: VarDeclKind::Const,
        declare: false,
        ctxt: SyntaxContext::empty(),
        decls: vec![VarDeclarator {
            span: DUMMY_SP,
            name: Pat::Ident(BindingIdent {
                id: ident.clone(),
                type_ann: None,
            }),
            init: Some(Box::new(import_call(specifier))),
            definite: false,
        }],
    })))
}

fn export_default_expr(expr: Expr) -> Stmt {
    Stmt::Expr(ExprStmt {
        span: DUMMY_SP,
        expr: Box::new(Expr::Assign(AssignExpr {
            span: DUMMY_SP,
            op: AssignOp::Assign,
            left: AssignTarget::Simple(SimpleAssignTarget::Member(MemberExpr {
                span: DUMMY_SP,
                obj: Box::new(Expr::Ident(ident_from("exports"))),
                prop: MemberProp::Ident(ident_name_from("default")),
            })),
            right: Box::new(expr),
        })),
    })
}

fn export_default_ident(ident: &Ident) -> Stmt {
    export_default_expr(Expr::Ident(ident.clone()))
}

fn export_named_ident(local: &Ident, exported: &str) -> Stmt {
    Stmt::Expr(ExprStmt {
        span: DUMMY_SP,
        expr: Box::new(Expr::Assign(AssignExpr {
            span: DUMMY_SP,
            op: AssignOp::Assign,
            left: AssignTarget::Simple(SimpleAssignTarget::Member(MemberExpr {
                span: DUMMY_SP,
                obj: Box::new(Expr::Ident(ident_from("exports"))),
                prop: export_member_prop(exported),
            })),
            right: Box::new(Expr::Ident(local.clone())),
        })),
    })
}

fn export_from_module(module_ident: &Ident, local: &str, exported: &str) -> Stmt {
    Stmt::Expr(ExprStmt {
        span: DUMMY_SP,
        expr: Box::new(Expr::Assign(AssignExpr {
            span: DUMMY_SP,
            op: AssignOp::Assign,
            left: AssignTarget::Simple(SimpleAssignTarget::Member(MemberExpr {
                span: DUMMY_SP,
                obj: Box::new(Expr::Ident(ident_from("exports"))),
                prop: export_member_prop(exported),
            })),
            right: Box::new(Expr::Member(MemberExpr {
                span: DUMMY_SP,
                obj: Box::new(Expr::Ident(module_ident.clone())),
                prop: export_member_prop(local),
            })),
        })),
    })
}

fn export_namespace_from_module(module_ident: &Ident, exported: &str) -> Stmt {
    Stmt::Expr(ExprStmt {
        span: DUMMY_SP,
        expr: Box::new(Expr::Assign(AssignExpr {
            span: DUMMY_SP,
            op: AssignOp::Assign,
            left: AssignTarget::Simple(SimpleAssignTarget::Member(MemberExpr {
                span: DUMMY_SP,
                obj: Box::new(Expr::Ident(ident_from("exports"))),
                prop: export_member_prop(exported),
            })),
            right: Box::new(Expr::Ident(module_ident.clone())),
        })),
    })
}

fn export_member_prop(name: &str) -> MemberProp {
    if is_valid_ident(name) {
        MemberProp::Ident(ident_name_from(name))
    } else {
        MemberProp::Computed(ComputedPropName {
            span: DUMMY_SP,
            expr: Box::new(Expr::Lit(Lit::Str(Str {
                span: DUMMY_SP,
                value: name.into(),
                raw: None,
            }))),
        })
    }
}

fn is_valid_ident(name: &str) -> bool {
    let mut chars = name.chars();
    let first = match chars.next() {
        Some(ch) => ch,
        None => return false,
    };
    if !(first == '_' || first == '$' || first.is_ascii_alphabetic()) {
        return false;
    }
    for ch in chars {
        if !(ch == '_' || ch == '$' || ch.is_ascii_alphanumeric()) {
            return false;
        }
    }
    true
}

fn export_all_loop(module_ident: Ident, key_ident: Ident) -> Stmt {
    let test = Expr::Bin(BinExpr {
        span: DUMMY_SP,
        op: BinaryOp::NotEqEq,
        left: Box::new(Expr::Ident(key_ident.clone())),
        right: Box::new(Expr::Lit(Lit::Str(Str {
            span: DUMMY_SP,
            value: "default".into(),
            raw: None,
        }))),
    });

    let assign = Expr::Assign(AssignExpr {
        span: DUMMY_SP,
        op: AssignOp::Assign,
        left: AssignTarget::Simple(SimpleAssignTarget::Member(MemberExpr {
            span: DUMMY_SP,
            obj: Box::new(Expr::Ident(ident_from("exports"))),
            prop: MemberProp::Computed(ComputedPropName {
                span: DUMMY_SP,
                expr: Box::new(Expr::Ident(key_ident.clone())),
            }),
        })),
        right: Box::new(Expr::Member(MemberExpr {
            span: DUMMY_SP,
            obj: Box::new(Expr::Ident(module_ident.clone())),
            prop: MemberProp::Computed(ComputedPropName {
                span: DUMMY_SP,
                expr: Box::new(Expr::Ident(key_ident.clone())),
            }),
        })),
    });

    let body = Stmt::If(IfStmt {
        span: DUMMY_SP,
        test: Box::new(test),
        cons: Box::new(Stmt::Expr(ExprStmt {
            span: DUMMY_SP,
            expr: Box::new(assign),
        })),
        alt: None,
    });

    Stmt::ForIn(ForInStmt {
        span: DUMMY_SP,
        left: ForHead::VarDecl(Box::new(VarDecl {
            span: DUMMY_SP,
            kind: VarDeclKind::Const,
            declare: false,
            ctxt: SyntaxContext::empty(),
            decls: vec![VarDeclarator {
                span: DUMMY_SP,
                name: Pat::Ident(BindingIdent {
                    id: key_ident,
                    type_ann: None,
                }),
                init: None,
                definite: false,
            }],
        })),
        right: Box::new(Expr::Ident(module_ident)),
        body: Box::new(body),
    })
}

fn extract_decl_idents(decl: &Decl) -> Vec<Ident> {
    let mut idents = Vec::new();
    match decl {
        Decl::Var(var) => {
            for decl in &var.decls {
                collect_pat_idents(&decl.name, &mut idents);
            }
        }
        Decl::Fn(func) => {
            idents.push(func.ident.clone());
        }
        Decl::Class(class) => {
            idents.push(class.ident.clone());
        }
        _ => {}
    }
    idents
}

fn collect_pat_idents(pat: &Pat, idents: &mut Vec<Ident>) {
    match pat {
        Pat::Ident(ident) => idents.push(ident.id.clone()),
        Pat::Array(array) => {
            for elem in &array.elems {
                if let Some(pat) = elem {
                    collect_pat_idents(pat, idents);
                }
            }
        }
        Pat::Object(obj) => {
            for prop in &obj.props {
                match prop {
                    ObjectPatProp::Assign(assign) => idents.push(assign.key.clone().into()),
                    ObjectPatProp::KeyValue(kv) => collect_pat_idents(&kv.value, idents),
                    ObjectPatProp::Rest(rest) => collect_pat_idents(&rest.arg, idents),
                }
            }
        }
        Pat::Assign(assign) => collect_pat_idents(&assign.left, idents),
        Pat::Rest(rest) => collect_pat_idents(&rest.arg, idents),
        _ => {}
    }
}
