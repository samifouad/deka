use axum::extract::ws::{CloseCode, CloseFrame, Message, WebSocket};
use futures_util::{SinkExt, StreamExt};
use serde_json::Value;
use std::collections::HashMap;
use std::sync::{
    Arc, Mutex, OnceLock,
    atomic::{AtomicU64, AtomicUsize, Ordering},
};
use tokio::sync::mpsc;

use engine::RuntimeState;
use pool::{ExecutionMode, RequestData};

static NEXT_WS_ID: AtomicU64 = AtomicU64::new(1);
struct WsEntry {
    sender: mpsc::UnboundedSender<Message>,
    pending: Arc<AtomicUsize>,
}

static WS_REGISTRY: OnceLock<Mutex<HashMap<u64, WsEntry>>> = OnceLock::new();
static HMR_REGISTRY: OnceLock<Mutex<HashMap<u64, mpsc::UnboundedSender<Message>>>> = OnceLock::new();

pub fn register_sender(sender: mpsc::UnboundedSender<Message>, pending: Arc<AtomicUsize>) -> u64 {
    let id = NEXT_WS_ID.fetch_add(1, Ordering::Relaxed);
    let registry = WS_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut guard) = registry.lock() {
        guard.insert(id, WsEntry { sender, pending });
    }
    id
}

pub fn unregister_sender(id: u64) {
    if let Some(registry) = WS_REGISTRY.get() {
        if let Ok(mut guard) = registry.lock() {
            guard.remove(&id);
        }
    }
}

pub async fn handle_hmr_websocket(socket: WebSocket) {
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
    let id = NEXT_WS_ID.fetch_add(1, Ordering::Relaxed);
    let registry = HMR_REGISTRY.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut guard) = registry.lock() {
        guard.insert(id, tx);
    }

    let (mut ws_sender, mut ws_receiver) = socket.split();
    let write_task = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if ws_sender.send(message).await.is_err() {
                break;
            }
        }
    });

    while let Some(message) = ws_receiver.next().await {
        match message {
            Ok(Message::Close(_)) => break,
            Ok(_) => {}
            Err(_) => break,
        }
    }

    if let Some(registry) = HMR_REGISTRY.get() {
        if let Ok(mut guard) = registry.lock() {
            guard.remove(&id);
        }
    }
    write_task.abort();
}

pub fn broadcast_hmr_changed(paths: &[String]) {
    let payload = serde_json::json!({
        "type": "changed",
        "paths": paths,
    })
    .to_string();

    let Some(registry) = HMR_REGISTRY.get() else {
        return;
    };
    let mut dead = Vec::new();
    if let Ok(guard) = registry.lock() {
        for (id, sender) in guard.iter() {
            if sender.send(Message::Text(payload.clone())).is_err() {
                dead.push(*id);
            }
        }
    }
    if dead.is_empty() {
        return;
    }
    if let Ok(mut guard) = registry.lock() {
        for id in dead {
            guard.remove(&id);
        }
    }
}

pub fn send_text(id: u64, message: String) -> Result<(), String> {
    if let Some(registry) = WS_REGISTRY.get() {
        if let Ok(guard) = registry.lock() {
            if let Some(entry) = guard.get(&id) {
                entry
                    .sender
                    .send(Message::Text(message))
                    .map_err(|err| err.to_string())?;
                entry.pending.fetch_add(1, Ordering::Relaxed);
                return Ok(());
            }
        }
    }
    Err(format!("WebSocket {} not found", id))
}

pub fn send_binary(id: u64, data: &[u8]) -> Result<(), String> {
    if let Some(registry) = WS_REGISTRY.get() {
        if let Ok(guard) = registry.lock() {
            if let Some(entry) = guard.get(&id) {
                entry
                    .sender
                    .send(Message::Binary(data.to_vec()))
                    .map_err(|err| err.to_string())?;
                entry.pending.fetch_add(1, Ordering::Relaxed);
                return Ok(());
            }
        }
    }
    Err(format!("WebSocket {} not found", id))
}

pub fn close_socket(id: u64, code: u16, reason: String) -> Result<(), String> {
    if let Some(registry) = WS_REGISTRY.get() {
        if let Ok(guard) = registry.lock() {
            if let Some(entry) = guard.get(&id) {
                let frame = CloseFrame {
                    code: CloseCode::from(code),
                    reason: reason.into(),
                };
                entry
                    .sender
                    .send(Message::Close(Some(frame)))
                    .map_err(|err| err.to_string())?;
                return Ok(());
            }
        }
    }
    Err(format!("WebSocket {} not found", id))
}

pub async fn handle_websocket(socket: WebSocket, state: Arc<RuntimeState>, upgrade: Option<Value>) {
    let (tx, mut rx) = mpsc::unbounded_channel::<Message>();
    let pending = Arc::new(AtomicUsize::new(0));
    let id = register_sender(tx, Arc::clone(&pending));

    let upgrade_data = upgrade
        .and_then(|value| value.get("data").cloned())
        .unwrap_or(Value::Null);

    let engine = Arc::clone(&state.engine);
    let handler_key = state.handler_key.clone();
    let handler_code = state.handler_code.clone();

    emit_event(
        Arc::clone(&engine),
        handler_key.clone(),
        handler_code.clone(),
        serde_json::json!({
            "__dekaWsEvent": "open",
            "__dekaWsId": id,
            "__dekaWsData": upgrade_data,
        }),
    )
    .await;

    let (mut ws_sender, mut ws_receiver) = socket.split();
    let drain_engine = Arc::clone(&engine);
    let drain_handler_key = handler_key.clone();
    let drain_handler_code = handler_code.clone();
    let drain_pending = Arc::clone(&pending);
    let write_task = tokio::spawn(async move {
        while let Some(message) = rx.recv().await {
            if ws_sender.send(message).await.is_err() {
                break;
            }
            if drain_pending.fetch_sub(1, Ordering::Relaxed) == 1 {
                let _ = emit_event(
                    Arc::clone(&drain_engine),
                    drain_handler_key.clone(),
                    drain_handler_code.clone(),
                    serde_json::json!({
                        "__dekaWsEvent": "drain",
                        "__dekaWsId": id,
                    }),
                )
                .await;
            }
        }
    });

    while let Some(message) = ws_receiver.next().await {
        match message {
            Ok(Message::Text(text)) => {
                emit_event(
                    Arc::clone(&engine),
                    handler_key.clone(),
                    handler_code.clone(),
                    serde_json::json!({
                        "__dekaWsEvent": "message",
                        "__dekaWsId": id,
                        "__dekaWsMessage": text,
                    }),
                )
                .await;
            }
            Ok(Message::Binary(bytes)) => {
                let data = bytes.into_iter().collect::<Vec<u8>>();
                emit_event(
                    Arc::clone(&engine),
                    handler_key.clone(),
                    handler_code.clone(),
                    serde_json::json!({
                        "__dekaWsEvent": "message",
                        "__dekaWsId": id,
                        "__dekaWsBinary": true,
                        "__dekaWsMessage": data,
                    }),
                )
                .await;
            }
            Ok(Message::Close(frame)) => {
                emit_event(
                    Arc::clone(&engine),
                    handler_key.clone(),
                    handler_code.clone(),
                    serde_json::json!({
                        "__dekaWsEvent": "close",
                        "__dekaWsId": id,
                        "__dekaWsCode": frame.as_ref().map(|f| u16::from(f.code)),
                        "__dekaWsReason": frame.as_ref().map(|f| f.reason.as_ref()),
                    }),
                )
                .await;
                break;
            }
            Err(_) => break,
            _ => {}
        }
    }

    unregister_sender(id);
    write_task.abort();
}

async fn emit_event(
    engine: Arc<engine::RuntimeEngine>,
    handler_key: pool::HandlerKey,
    handler_code: String,
    payload: Value,
) {
    let _ = engine
        .execute(
            handler_key,
            RequestData {
                handler_code,
                request_value: payload,
                request_parts: None,
                mode: ExecutionMode::Request,
            },
        )
        .await;
}
