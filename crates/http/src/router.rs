use std::sync::Arc;

use axum::extract::ws::WebSocketUpgrade;
use axum::http::header::CONTENT_LENGTH;
use axum::{
    Router,
    extract::{Request, State},
    response::{IntoResponse, Response},
};
use base64::Engine;

use crate::websocket::handle_websocket;
use engine::{RequestEnvelope, RuntimeState, execute_request};

use crate::debug::http_debug_enabled;

pub fn app_router(state: Arc<RuntimeState>) -> Router {
    Router::new().fallback(handle_request).with_state(state)
}

async fn handle_request(
    State(state): State<Arc<RuntimeState>>,
    ws: Option<WebSocketUpgrade>,
    request: Request,
) -> impl IntoResponse {
    let method = request.method().as_str().to_string();
    let uri = request.uri().to_string();
    if http_debug_enabled() {
        tracing::info!("[http] request {} {}", method, uri);
    }
    let (headers, body) = if state.perf_mode {
        (std::collections::HashMap::new(), None)
    } else {
        let mut headers = std::collections::HashMap::with_capacity(request.headers().len());
        for (key, value) in request.headers().iter() {
            headers.insert(
                key.as_str().to_string(),
                value.to_str().unwrap_or("").to_string(),
            );
        }

        let content_len = request
            .headers()
            .get(CONTENT_LENGTH)
            .and_then(|value| value.to_str().ok())
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(usize::MAX);

        let body = if content_len == 0 {
            None
        } else {
            match axum::body::to_bytes(request.into_body(), usize::MAX).await {
                Ok(bytes) => {
                    if bytes.is_empty() {
                        None
                    } else {
                        Some(String::from_utf8_lossy(&bytes).to_string())
                    }
                }
                Err(_) => None,
            }
        };
        (headers, body)
    };

    let request_envelope = RequestEnvelope {
        url: format!("http://localhost{}", uri),
        method,
        headers,
        body,
    };

    match execute_request(Arc::clone(&state), request_envelope).await {
        Ok(response_envelope) => {
            if http_debug_enabled() {
                tracing::info!("[http] response {} {}", response_envelope.status, uri);
            }
            if let Some(upgrade) = response_envelope.upgrade {
                if let Some(ws) = ws {
                    return ws
                        .on_upgrade(move |socket| handle_websocket(socket, state, Some(upgrade)))
                        .into_response();
                }
                return Response::builder()
                    .status(426)
                    .body(axum::body::Body::from("WebSocket upgrade required"))
                    .unwrap();
            }

            let mut response = Response::builder().status(response_envelope.status);

            for (key, value) in response_envelope.headers {
                if key.eq_ignore_ascii_case("set-cookie") && value.contains('\n') {
                    for part in value.split('\n').filter(|part| !part.is_empty()) {
                        response = response.header(&key, part);
                    }
                    continue;
                }
                response = response.header(&key, value);
            }

            if let Some(body_base64) = response_envelope.body_base64 {
                let bytes: Vec<u8> = match base64::engine::general_purpose::STANDARD
                    .decode(body_base64.as_bytes())
                {
                    Ok(bytes) => bytes,
                    Err(err) => {
                        return Response::builder()
                            .status(500)
                            .body(axum::body::Body::from(format!(
                                "Failed to decode body: {}",
                                err
                            )))
                            .unwrap();
                    }
                };
                response.body(axum::body::Body::from(bytes)).unwrap()
            } else {
                response
                    .body(axum::body::Body::from(response_envelope.body))
                    .unwrap()
            }
        }
        Err(err) => {
            tracing::error!("Handler execution failed: {}", err);
            Response::builder()
                .status(500)
                .body(axum::body::Body::from(format!(
                    "Handler execution failed: {}",
                    err
                )))
                .unwrap()
        }
    }
}
