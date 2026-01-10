use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{
    Arc, Mutex, OnceLock,
    atomic::{AtomicU64, Ordering},
};

use deno_core::{JsBuffer, error::CoreError, op2};
use serde::Serialize;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::tcp::{OwnedReadHalf, OwnedWriteHalf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::{Mutex as AsyncMutex, watch};

use super::UdpAddrResult;

struct TcpListenerEntry {
    listener: AsyncMutex<TcpListener>,
    close_tx: watch::Sender<bool>,
    close_rx: watch::Receiver<bool>,
}

static TCP_LISTENERS: OnceLock<Mutex<HashMap<u64, Arc<TcpListenerEntry>>>> = OnceLock::new();

struct TcpStreamEntry {
    read: AsyncMutex<OwnedReadHalf>,
    write: AsyncMutex<OwnedWriteHalf>,
    local: SocketAddr,
    peer: SocketAddr,
    close_tx: watch::Sender<bool>,
    close_rx: watch::Receiver<bool>,
}

static TCP_STREAMS: OnceLock<Mutex<HashMap<u64, Arc<TcpStreamEntry>>>> = OnceLock::new();
static TCP_IDS: AtomicU64 = AtomicU64::new(1);

#[derive(Serialize)]
struct TcpAcceptResult {
    id: u64,
    address: String,
    port: u16,
}

#[derive(Serialize)]
struct TcpListenResult {
    id: u64,
    address: String,
    port: u16,
}

#[derive(Serialize)]
struct TcpReadResult {
    data: Vec<u8>,
    eof: bool,
}

fn tcp_listener_store() -> &'static Mutex<HashMap<u64, Arc<TcpListenerEntry>>> {
    TCP_LISTENERS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn tcp_listener_lookup(id: u64) -> Result<Arc<TcpListenerEntry>, CoreError> {
    let store = tcp_listener_store();
    let guard = store.lock().map_err(|_| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "TCP listener store locked",
        ))
    })?;
    guard.get(&id).cloned().ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "TCP listener not found",
        ))
    })
}

fn tcp_stream_store() -> &'static Mutex<HashMap<u64, Arc<TcpStreamEntry>>> {
    TCP_STREAMS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn tcp_stream_lookup(id: u64) -> Result<Arc<TcpStreamEntry>, CoreError> {
    let store = tcp_stream_store();
    let guard = store.lock().map_err(|_| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "TCP store locked",
        ))
    })?;
    guard.get(&id).cloned().ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "TCP stream not found",
        ))
    })
}

fn normalize_port(port: f64) -> Result<u16, CoreError> {
    if !port.is_finite() || port < 0.0 || port > u16::MAX as f64 {
        return Err(CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid TCP port",
        )));
    }
    Ok(port as u16)
}

fn normalize_tcp_len(max_len: f64) -> Result<usize, CoreError> {
    if !max_len.is_finite() || max_len < 1.0 {
        return Ok(65535);
    }
    Ok(max_len as usize)
}

#[op2(async)]
#[serde]
pub(crate) async fn op_tcp_listen(
    #[string] address: String,
    #[number] port: u64,
) -> Result<TcpListenResult, CoreError> {
    let port = normalize_port(port as f64)?;
    let addr: SocketAddr = format!("{}:{}", address, port).parse().map_err(|err| {
        CoreError::from(std::io::Error::new(std::io::ErrorKind::InvalidInput, err))
    })?;
    let listener = TcpListener::bind(addr).await.map_err(CoreError::from)?;
    let local = listener.local_addr().map_err(CoreError::from)?;
    let (close_tx, close_rx) = watch::channel(false);
    let id = TCP_IDS.fetch_add(1, Ordering::Relaxed);
    let store = tcp_listener_store();
    let mut guard = store.lock().map_err(|_| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "TCP listener store locked",
        ))
    })?;
    guard.insert(
        id,
        Arc::new(TcpListenerEntry {
            listener: AsyncMutex::new(listener),
            close_tx,
            close_rx,
        }),
    );
    Ok(TcpListenResult {
        id,
        address: local.ip().to_string(),
        port: local.port(),
    })
}

#[op2(async)]
#[serde]
pub(crate) async fn op_tcp_accept(
    #[bigint] listener_id: u64,
) -> Result<TcpAcceptResult, CoreError> {
    let listener = tcp_listener_lookup(listener_id)?;
    let mut close_rx = listener.close_rx.clone();
    let (stream, addr) = tokio::select! {
        result = async {
            let guard = listener.listener.lock().await;
            guard.accept().await
        } => result.map_err(CoreError::from)?,
        _ = close_rx.changed() => {
            return Err(CoreError::from(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                "TCP listener closed",
            )));
        }
    };
    let local = stream.local_addr().map_err(CoreError::from)?;
    let (read, write) = stream.into_split();
    let (close_tx, close_rx) = watch::channel(false);
    let stream_id = TCP_IDS.fetch_add(1, Ordering::Relaxed);
    let store = tcp_stream_store();
    let mut guard = store.lock().map_err(|_| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "TCP stream store locked",
        ))
    })?;
    guard.insert(
        stream_id,
        Arc::new(TcpStreamEntry {
            read: AsyncMutex::new(read),
            write: AsyncMutex::new(write),
            local,
            peer: addr,
            close_tx,
            close_rx,
        }),
    );
    Ok(TcpAcceptResult {
        id: stream_id,
        address: addr.ip().to_string(),
        port: addr.port(),
    })
}

#[op2(async)]
#[bigint]
pub(crate) async fn op_tcp_connect(
    #[string] address: String,
    #[number] port: u64,
) -> Result<u64, CoreError> {
    let port = normalize_port(port as f64)?;
    let addr: SocketAddr = format!("{}:{}", address, port).parse().map_err(|err| {
        CoreError::from(std::io::Error::new(std::io::ErrorKind::InvalidInput, err))
    })?;
    let stream = TcpStream::connect(addr).await.map_err(CoreError::from)?;
    let local = stream.local_addr().map_err(CoreError::from)?;
    let peer = stream.peer_addr().map_err(CoreError::from)?;
    let (read, write) = stream.into_split();
    let (close_tx, close_rx) = watch::channel(false);
    let id = TCP_IDS.fetch_add(1, Ordering::Relaxed);
    let store = tcp_stream_store();
    let mut guard = store.lock().map_err(|_| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "TCP stream store locked",
        ))
    })?;
    guard.insert(
        id,
        Arc::new(TcpStreamEntry {
            read: AsyncMutex::new(read),
            write: AsyncMutex::new(write),
            local,
            peer,
            close_tx,
            close_rx,
        }),
    );
    Ok(id)
}

#[op2(async)]
#[serde]
pub(crate) async fn op_tcp_read(
    #[bigint] id: u64,
    #[number] max_len: u64,
) -> Result<TcpReadResult, CoreError> {
    let stream = tcp_stream_lookup(id)?;
    let max_len = normalize_tcp_len(max_len as f64)?;
    let mut buf = vec![0u8; max_len];
    let mut close_rx = stream.close_rx.clone();
    let n = tokio::select! {
        result = async {
            let mut guard = stream.read.lock().await;
            guard.read(&mut buf).await
        } => result.map_err(CoreError::from)?,
        _ = close_rx.changed() => {
            return Ok(TcpReadResult { data: Vec::new(), eof: true });
        }
    };
    if n == 0 {
        return Ok(TcpReadResult {
            data: Vec::new(),
            eof: true,
        });
    }
    buf.truncate(n);
    Ok(TcpReadResult {
        data: buf,
        eof: false,
    })
}

#[op2(async)]
#[number]
pub(crate) async fn op_tcp_write(
    #[bigint] id: u64,
    #[buffer] data: JsBuffer,
) -> Result<u64, CoreError> {
    let stream = tcp_stream_lookup(id)?;
    let mut guard = stream.write.lock().await;
    let n = guard.write(data.as_ref()).await.map_err(CoreError::from)?;
    Ok(n as u64)
}

#[op2(fast)]
pub(crate) fn op_tcp_close(#[bigint] id: u64) -> Result<(), CoreError> {
    let store = tcp_stream_store();
    let mut guard = store.lock().map_err(|_| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "TCP stream store locked",
        ))
    })?;
    if let Some(entry) = guard.remove(&id) {
        let _ = entry.close_tx.send(true);
    }
    Ok(())
}

#[op2(async)]
pub(crate) async fn op_tcp_shutdown(#[bigint] id: u64) -> Result<(), CoreError> {
    let stream = tcp_stream_lookup(id)?;
    let mut guard = stream.write.lock().await;
    guard.shutdown().await.map_err(CoreError::from)?;
    Ok(())
}

#[op2(async)]
#[serde]
pub(crate) async fn op_tcp_local_addr(#[bigint] id: u64) -> Result<UdpAddrResult, CoreError> {
    let stream = tcp_stream_lookup(id)?;
    let addr = stream.local;
    Ok(UdpAddrResult {
        address: addr.ip().to_string(),
        port: addr.port(),
    })
}

#[op2(async)]
#[serde]
pub(crate) async fn op_tcp_peer_addr(#[bigint] id: u64) -> Result<UdpAddrResult, CoreError> {
    let stream = tcp_stream_lookup(id)?;
    let addr = stream.peer;
    Ok(UdpAddrResult {
        address: addr.ip().to_string(),
        port: addr.port(),
    })
}

#[op2(async)]
#[serde]
pub(crate) async fn op_tcp_listener_addr(#[bigint] id: u64) -> Result<UdpAddrResult, CoreError> {
    let listener = tcp_listener_lookup(id)?;
    let guard = listener.listener.lock().await;
    let addr = guard.local_addr().map_err(CoreError::from)?;
    Ok(UdpAddrResult {
        address: addr.ip().to_string(),
        port: addr.port(),
    })
}

#[op2(fast)]
pub(crate) fn op_tcp_listener_close(#[bigint] id: u64) -> Result<(), CoreError> {
    let store = tcp_listener_store();
    let mut guard = store.lock().map_err(|_| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "TCP listener store locked",
        ))
    })?;
    if let Some(entry) = guard.remove(&id) {
        let _ = entry.close_tx.send(true);
    }
    Ok(())
}
