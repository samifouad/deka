use std::net::SocketAddr;
use std::sync::Arc;

use engine::RuntimeState;

use crate::fast::serve_http_fast;
use crate::listener::bind_reuseport;
use crate::router::app_router;

pub async fn serve_http(state: Arc<RuntimeState>, port: u16, listeners: usize, perf_mode: bool) {
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("🚀 Deka Runtime listening on {}", addr);
    tracing::info!("📦 Loaded modules: deka, postgres, docker, router, t4, sqlite");

    let listener_count = listeners.max(1);
    if listener_count == 1 {
        let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
        if perf_mode {
            serve_http_fast(listener, state).await;
        } else {
            let app = app_router(Arc::clone(&state));
            axum::serve(listener, app).await.unwrap();
        }
        return;
    }

    let mut handles = Vec::with_capacity(listener_count);
    for _ in 0..listener_count {
        let listener = bind_reuseport(addr)
            .unwrap_or_else(|err| panic!("Failed to bind reuseport listener: {}", err));
        let state = Arc::clone(&state);
        if perf_mode {
            handles.push(tokio::spawn(async move {
                serve_http_fast(listener, state).await;
            }));
        } else {
            let app = app_router(Arc::clone(&state));
            handles.push(tokio::spawn(async move {
                if let Err(err) = axum::serve(listener, app).await {
                    tracing::error!("HTTP listener exited: {}", err);
                }
            }));
        }
    }

    for handle in handles {
        let _ = handle.await;
    }
}
