use std::collections::HashMap;
use std::sync::{
    Arc, Mutex, OnceLock,
    atomic::{AtomicU64, Ordering},
};

use deno_core::{JsBuffer, error::CoreError, op2};
use reqwest::{Client, Method};
use tokio::sync::{Mutex as AsyncMutex, mpsc};

struct BlobEntry {
    data: Vec<u8>,
    mime: String,
}

static BLOB_STORE: OnceLock<Mutex<HashMap<u64, BlobEntry>>> = OnceLock::new();
static BLOB_IDS: AtomicU64 = AtomicU64::new(1);

enum StreamMessage {
    Chunk(Vec<u8>),
    Close,
}

struct StreamEntry {
    sender: mpsc::UnboundedSender<StreamMessage>,
    receiver: AsyncMutex<mpsc::UnboundedReceiver<StreamMessage>>,
}

static STREAMS: OnceLock<Mutex<HashMap<u64, Arc<StreamEntry>>>> = OnceLock::new();
static STREAM_IDS: AtomicU64 = AtomicU64::new(1);
static HTTP_CLIENT: OnceLock<Client> = OnceLock::new();

#[derive(serde::Serialize)]
struct HttpFetchResponse {
    status: u16,
    headers: HashMap<String, String>,
    body_id: u64,
}

#[op2(fast)]
#[bigint]
pub(crate) fn op_blob_create(
    #[buffer] data: &[u8],
    #[string] mime: String,
) -> Result<u64, CoreError> {
    let store = BLOB_STORE.get_or_init(|| Mutex::new(HashMap::new()));
    let id = BLOB_IDS.fetch_add(1, Ordering::Relaxed);
    let mut guard = store.lock().map_err(|_| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Blob store locked",
        ))
    })?;
    guard.insert(
        id,
        BlobEntry {
            data: data.to_vec(),
            mime,
        },
    );
    Ok(id)
}

#[op2]
#[buffer]
pub(crate) fn op_blob_get(#[bigint] id: u64) -> Result<Vec<u8>, CoreError> {
    let store = BLOB_STORE.get_or_init(|| Mutex::new(HashMap::new()));
    let guard = store.lock().map_err(|_| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Blob store locked",
        ))
    })?;
    guard
        .get(&id)
        .map(|entry| entry.data.clone())
        .ok_or_else(|| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Blob not found",
            ))
        })
}

#[op2(fast)]
#[bigint]
pub(crate) fn op_blob_size(#[bigint] id: u64) -> Result<u64, CoreError> {
    let store = BLOB_STORE.get_or_init(|| Mutex::new(HashMap::new()));
    let guard = store.lock().map_err(|_| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Blob store locked",
        ))
    })?;
    guard
        .get(&id)
        .map(|entry| entry.data.len() as u64)
        .ok_or_else(|| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Blob not found",
            ))
        })
}

#[op2]
#[string]
pub(crate) fn op_blob_type(#[bigint] id: u64) -> Result<String, CoreError> {
    let store = BLOB_STORE.get_or_init(|| Mutex::new(HashMap::new()));
    let guard = store.lock().map_err(|_| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Blob store locked",
        ))
    })?;
    guard
        .get(&id)
        .map(|entry| entry.mime.clone())
        .ok_or_else(|| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Blob not found",
            ))
        })
}

#[op2(fast)]
#[bigint]
pub(crate) fn op_blob_slice(
    #[bigint] id: u64,
    #[bigint] start: u64,
    #[bigint] end: u64,
    #[string] mime: String,
) -> Result<u64, CoreError> {
    let store = BLOB_STORE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = store.lock().map_err(|_| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Blob store locked",
        ))
    })?;
    let entry = guard.get(&id).ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "Blob not found",
        ))
    })?;
    let start = start.min(entry.data.len() as u64) as usize;
    let end = end.min(entry.data.len() as u64) as usize;
    let slice = if end > start {
        entry.data[start..end].to_vec()
    } else {
        Vec::new()
    };
    let new_id = BLOB_IDS.fetch_add(1, Ordering::Relaxed);
    guard.insert(new_id, BlobEntry { data: slice, mime });
    Ok(new_id)
}

#[op2(fast)]
pub(crate) fn op_blob_drop(#[bigint] id: u64) -> Result<(), CoreError> {
    let store = BLOB_STORE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Ok(mut guard) = store.lock() {
        guard.remove(&id);
    }
    Ok(())
}

#[derive(serde::Serialize)]
struct StreamRead {
    done: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    chunk: Option<Vec<u8>>,
}

#[op2(fast)]
#[bigint]
pub(crate) fn op_stream_create() -> Result<u64, CoreError> {
    let store = STREAMS.get_or_init(|| Mutex::new(HashMap::new()));
    let id = STREAM_IDS.fetch_add(1, Ordering::Relaxed);
    let (sender, receiver) = mpsc::unbounded_channel();
    let entry = Arc::new(StreamEntry {
        sender,
        receiver: AsyncMutex::new(receiver),
    });
    let mut guard = store.lock().map_err(|_| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Stream store locked",
        ))
    })?;
    guard.insert(id, entry);
    Ok(id)
}

#[op2(fast)]
pub(crate) fn op_stream_enqueue(
    #[bigint] id: u64,
    #[buffer] chunk: &[u8],
) -> Result<(), CoreError> {
    let store = STREAMS.get_or_init(|| Mutex::new(HashMap::new()));
    let entry = {
        let guard = store.lock().map_err(|_| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Stream store locked",
            ))
        })?;
        guard.get(&id).cloned().ok_or_else(|| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Stream not found",
            ))
        })?
    };
    let _ = entry.sender.send(StreamMessage::Chunk(chunk.to_vec()));
    Ok(())
}

#[op2(fast)]
pub(crate) fn op_stream_close(#[bigint] id: u64) -> Result<(), CoreError> {
    let store = STREAMS.get_or_init(|| Mutex::new(HashMap::new()));
    let entry = {
        let guard = store.lock().map_err(|_| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Stream store locked",
            ))
        })?;
        guard.get(&id).cloned().ok_or_else(|| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Stream not found",
            ))
        })?
    };
    let _ = entry.sender.send(StreamMessage::Close);
    Ok(())
}

#[op2(async)]
#[serde]
pub(crate) async fn op_stream_read(#[bigint] id: u64) -> Result<StreamRead, CoreError> {
    let store = STREAMS.get_or_init(|| Mutex::new(HashMap::new()));
    let entry = {
        let guard = store.lock().map_err(|_| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Stream store locked",
            ))
        })?;
        guard.get(&id).cloned().ok_or_else(|| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Stream not found",
            ))
        })?
    };
    let mut receiver = entry.receiver.lock().await;
    match receiver.recv().await {
        Some(StreamMessage::Chunk(chunk)) => Ok(StreamRead {
            done: false,
            chunk: Some(chunk),
        }),
        Some(StreamMessage::Close) | None => Ok(StreamRead {
            done: true,
            chunk: None,
        }),
    }
}

#[op2(fast)]
pub(crate) fn op_stream_drop(#[bigint] id: u64) -> Result<(), CoreError> {
    let store = STREAMS.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = store.lock().map_err(|_| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Stream store locked",
        ))
    })?;
    guard.remove(&id);
    Ok(())
}

#[op2(async)]
#[serde]
pub(crate) async fn op_http_fetch(
    #[string] url: String,
    #[string] method: String,
    #[serde] headers: HashMap<String, String>,
    #[buffer] body: JsBuffer,
) -> Result<HttpFetchResponse, CoreError> {
    let client = HTTP_CLIENT.get_or_init(|| Client::new());
    let method = Method::from_bytes(method.as_bytes()).map_err(|_| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Invalid method",
        ))
    })?;
    let mut request = client.request(method, url);

    for (key, value) in headers {
        request = request.header(key, value);
    }

    if !body.is_empty() {
        request = request.body(body.to_vec());
    }

    let response = request
        .send()
        .await
        .map_err(|err| CoreError::from(std::io::Error::new(std::io::ErrorKind::Other, err)))?;
    let status = response.status().as_u16();
    let mut header_map = HashMap::new();
    for (name, value) in response.headers() {
        if let Ok(value) = value.to_str() {
            header_map.insert(name.as_str().to_string(), value.to_string());
        }
    }
    let bytes = response
        .bytes()
        .await
        .map_err(|err| CoreError::from(std::io::Error::new(std::io::ErrorKind::Other, err)))?;

    let store = BLOB_STORE.get_or_init(|| Mutex::new(HashMap::new()));
    let body_id = BLOB_IDS.fetch_add(1, Ordering::Relaxed);
    let mut guard = store.lock().map_err(|_| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Blob store locked",
        ))
    })?;
    guard.insert(
        body_id,
        BlobEntry {
            data: bytes.to_vec(),
            mime: header_map
                .get("content-type")
                .cloned()
                .unwrap_or_else(|| "application/octet-stream".to_string()),
        },
    );

    Ok(HttpFetchResponse {
        status,
        headers: header_map,
        body_id,
    })
}
