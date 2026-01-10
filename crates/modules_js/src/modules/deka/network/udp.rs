use std::collections::HashMap;
use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};
use std::sync::{
    Arc, Mutex, OnceLock,
    atomic::{AtomicU64, Ordering},
};

use deno_core::{JsBuffer, error::CoreError, op2};
use socket2::{Domain, Protocol, SockRef, Socket, Type};
use tokio::net::UdpSocket;
use tokio::sync::watch;

use super::UdpAddrResult;

#[derive(Clone, Copy, Debug, PartialEq)]
enum UdpFamily {
    V4,
    V6,
}

#[derive(Clone)]
struct UdpSocketEntry {
    socket: Arc<UdpSocket>,
    family: UdpFamily,
    reuse_addr: bool,
    reuse_port: bool,
    ipv6_only: bool,
    broadcast: bool,
    ttl: Option<u32>,
    multicast_ttl: Option<u32>,
    multicast_loop: Option<bool>,
    multicast_if_v4: Option<Ipv4Addr>,
    multicast_if_v6: Option<u32>,
    recv_buffer_size: Option<u32>,
    send_buffer_size: Option<u32>,
    close_tx: watch::Sender<bool>,
    close_rx: watch::Receiver<bool>,
}

static UDP_SOCKETS: OnceLock<Mutex<HashMap<u64, UdpSocketEntry>>> = OnceLock::new();
static UDP_IDS: AtomicU64 = AtomicU64::new(1);

#[derive(serde::Serialize)]
struct UdpRecvResult {
    data: Vec<u8>,
    address: String,
    port: u16,
}

fn udp_store() -> &'static Mutex<HashMap<u64, UdpSocketEntry>> {
    UDP_SOCKETS.get_or_init(|| Mutex::new(HashMap::new()))
}

fn udp_lookup(id: u64) -> Result<Arc<UdpSocket>, CoreError> {
    let store = udp_store();
    let guard = store.lock().map_err(|_| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "UDP store locked",
        ))
    })?;
    guard
        .get(&id)
        .map(|entry| entry.socket.clone())
        .ok_or_else(|| {
            CoreError::from(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "UDP socket not found",
            ))
        })
}

fn udp_entry(id: u64) -> Result<UdpSocketEntry, CoreError> {
    let store = udp_store();
    let guard = store.lock().map_err(|_| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "UDP store locked",
        ))
    })?;
    guard.get(&id).cloned().ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "UDP socket not found",
        ))
    })
}

fn udp_update<T>(
    id: u64,
    updater: impl FnOnce(&mut UdpSocketEntry) -> Result<T, CoreError>,
) -> Result<T, CoreError> {
    let store = udp_store();
    let mut guard = store.lock().map_err(|_| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "UDP store locked",
        ))
    })?;
    let entry = guard.get_mut(&id).ok_or_else(|| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "UDP socket not found",
        ))
    })?;
    updater(entry)
}

fn normalize_port(port: f64) -> Result<u16, CoreError> {
    if !port.is_finite() || port < 0.0 || port > u16::MAX as f64 {
        return Err(CoreError::from(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            "Invalid UDP port",
        )));
    }
    Ok(port as u16)
}

fn normalize_udp_len(max_len: f64) -> Result<usize, CoreError> {
    if !max_len.is_finite() || max_len < 1.0 {
        return Ok(65535);
    }
    Ok(max_len as usize)
}

fn create_udp_socket(
    addr: SocketAddr,
    family: UdpFamily,
    reuse_addr: bool,
    reuse_port: bool,
    ipv6_only: bool,
    broadcast: bool,
    recv_buffer_size: u64,
    send_buffer_size: u64,
) -> Result<UdpSocketEntry, CoreError> {
    let domain = match family {
        UdpFamily::V4 => Domain::IPV4,
        UdpFamily::V6 => Domain::IPV6,
    };
    let socket = Socket::new(domain, Type::DGRAM, Some(Protocol::UDP)).map_err(CoreError::from)?;
    socket.set_nonblocking(true).map_err(CoreError::from)?;
    socket
        .set_reuse_address(reuse_addr)
        .map_err(CoreError::from)?;
    #[cfg(unix)]
    {
        let _ = socket.set_reuse_port(reuse_port);
    }
    if family == UdpFamily::V6 {
        socket.set_only_v6(ipv6_only).map_err(CoreError::from)?;
    }
    if broadcast {
        socket.set_broadcast(true).map_err(CoreError::from)?;
    }
    if recv_buffer_size > 0 {
        let _ = socket.set_recv_buffer_size(recv_buffer_size as usize);
    }
    if send_buffer_size > 0 {
        let _ = socket.set_send_buffer_size(send_buffer_size as usize);
    }
    socket.bind(&addr.into()).map_err(CoreError::from)?;
    let std_socket: std::net::UdpSocket = socket.into();
    std_socket.set_nonblocking(true).map_err(CoreError::from)?;
    let socket = UdpSocket::from_std(std_socket).map_err(CoreError::from)?;
    let (close_tx, close_rx) = watch::channel(false);

    Ok(UdpSocketEntry {
        socket: Arc::new(socket),
        family,
        reuse_addr,
        reuse_port,
        ipv6_only,
        broadcast,
        ttl: None,
        multicast_ttl: None,
        multicast_loop: None,
        multicast_if_v4: None,
        multicast_if_v6: None,
        recv_buffer_size: if recv_buffer_size > 0 {
            Some(recv_buffer_size as u32)
        } else {
            None
        },
        send_buffer_size: if send_buffer_size > 0 {
            Some(send_buffer_size as u32)
        } else {
            None
        },
        close_tx,
        close_rx,
    })
}

fn parse_ipv4(addr: &str) -> Result<Ipv4Addr, CoreError> {
    addr.parse::<Ipv4Addr>()
        .map_err(|err| CoreError::from(std::io::Error::new(std::io::ErrorKind::InvalidInput, err)))
}

fn parse_ipv6(addr: &str) -> Result<Ipv6Addr, CoreError> {
    let cleaned = addr.split('%').next().unwrap_or(addr);
    cleaned
        .parse::<Ipv6Addr>()
        .map_err(|err| CoreError::from(std::io::Error::new(std::io::ErrorKind::InvalidInput, err)))
}

#[cfg(unix)]
fn interface_to_index(name: &str) -> Option<u32> {
    use std::ffi::CString;
    let cstr = CString::new(name).ok()?;
    let idx = unsafe { libc::if_nametoindex(cstr.as_ptr()) };
    if idx == 0 { None } else { Some(idx) }
}

fn parse_ipv6_interface(interface: Option<String>) -> Result<u32, CoreError> {
    let Some(interface) = interface else {
        return Ok(0);
    };
    let name = interface.split('%').last().unwrap_or(&interface);
    if let Ok(idx) = name.parse::<u32>() {
        return Ok(idx);
    }
    #[cfg(unix)]
    {
        if let Some(idx) = interface_to_index(name) {
            return Ok(idx);
        }
    }
    Err(CoreError::from(std::io::Error::new(
        std::io::ErrorKind::InvalidInput,
        "Invalid IPv6 interface",
    )))
}

#[op2(async)]
#[bigint]
pub(crate) async fn op_udp_bind(
    #[string] address: String,
    #[number] port: u64,
    reuse_addr: bool,
    reuse_port: bool,
    ipv6_only: bool,
    broadcast: bool,
    #[number] recv_buffer_size: u64,
    #[number] send_buffer_size: u64,
) -> Result<u64, CoreError> {
    let port = normalize_port(port as f64)?;
    let addr: SocketAddr = format!("{}:{}", address, port).parse().map_err(|err| {
        CoreError::from(std::io::Error::new(std::io::ErrorKind::InvalidInput, err))
    })?;
    let family = match addr {
        SocketAddr::V4(_) => UdpFamily::V4,
        SocketAddr::V6(_) => UdpFamily::V6,
    };
    let socket = create_udp_socket(
        addr,
        family,
        reuse_addr,
        reuse_port,
        ipv6_only,
        broadcast,
        recv_buffer_size,
        send_buffer_size,
    )?;
    let id = UDP_IDS.fetch_add(1, Ordering::Relaxed);
    let store = udp_store();
    let mut guard = store.lock().map_err(|_| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "UDP store locked",
        ))
    })?;
    guard.insert(id, socket);
    Ok(id)
}

#[op2(async)]
#[number]
pub(crate) async fn op_udp_send(
    #[bigint] id: u64,
    #[buffer] data: JsBuffer,
    #[number] port: u64,
    #[string] address: String,
) -> Result<u64, CoreError> {
    let port = normalize_port(port as f64)?;
    let socket = udp_lookup(id)?;
    let addr: SocketAddr = format!("{}:{}", address, port).parse().map_err(|err| {
        CoreError::from(std::io::Error::new(std::io::ErrorKind::InvalidInput, err))
    })?;
    let sent = socket
        .send_to(data.as_ref(), addr)
        .await
        .map_err(CoreError::from)?;
    Ok(sent as u64)
}

#[op2(async)]
#[serde]
pub(crate) async fn op_udp_recv(
    #[bigint] id: u64,
    #[number] max_len: u64,
) -> Result<UdpRecvResult, CoreError> {
    let entry = udp_entry(id)?;
    let socket = entry.socket.clone();
    let mut close_rx = entry.close_rx.clone();
    let max_len = normalize_udp_len(max_len as f64)?;
    let mut buf = vec![0u8; max_len];
    let (size, addr) = tokio::select! {
        result = socket.recv_from(&mut buf) => result.map_err(CoreError::from)?,
        _ = close_rx.changed() => {
            return Err(CoreError::from(std::io::Error::new(
                std::io::ErrorKind::Interrupted,
                "UDP socket closed",
            )));
        }
    };
    buf.truncate(size);
    Ok(UdpRecvResult {
        data: buf,
        address: addr.ip().to_string(),
        port: addr.port(),
    })
}

#[op2(fast)]
pub(crate) fn op_udp_close(#[bigint] id: u64) -> Result<(), CoreError> {
    let store = udp_store();
    let mut guard = store.lock().map_err(|_| {
        CoreError::from(std::io::Error::new(
            std::io::ErrorKind::Other,
            "UDP store locked",
        ))
    })?;
    if let Some(entry) = guard.remove(&id) {
        let _ = entry.close_tx.send(true);
    }
    Ok(())
}

#[op2]
#[serde]
pub(crate) fn op_udp_local_addr(#[bigint] id: u64) -> Result<UdpAddrResult, CoreError> {
    let socket = udp_lookup(id)?;
    let addr = socket.local_addr().map_err(CoreError::from)?;
    Ok(UdpAddrResult {
        address: addr.ip().to_string(),
        port: addr.port(),
    })
}

#[op2]
#[serde]
pub(crate) fn op_udp_peer_addr(#[bigint] id: u64) -> Result<UdpAddrResult, CoreError> {
    let socket = udp_lookup(id)?;
    let addr = socket.peer_addr().map_err(CoreError::from)?;
    Ok(UdpAddrResult {
        address: addr.ip().to_string(),
        port: addr.port(),
    })
}

#[op2(async)]
pub(crate) async fn op_udp_connect(
    #[bigint] id: u64,
    #[string] address: String,
    #[number] port: u64,
) -> Result<(), CoreError> {
    let port = normalize_port(port as f64)?;
    let addr: SocketAddr = format!("{}:{}", address, port).parse().map_err(|err| {
        CoreError::from(std::io::Error::new(std::io::ErrorKind::InvalidInput, err))
    })?;
    let socket = udp_lookup(id)?;
    socket.connect(addr).await.map_err(CoreError::from)?;
    Ok(())
}

#[op2(async)]
pub(crate) async fn op_udp_disconnect(#[bigint] id: u64) -> Result<(), CoreError> {
    let entry = udp_entry(id)?;
    let local = entry.socket.local_addr().map_err(CoreError::from)?;
    let new_entry = create_udp_socket(
        local,
        entry.family,
        entry.reuse_addr,
        entry.reuse_port,
        entry.ipv6_only,
        entry.broadcast,
        entry.recv_buffer_size.unwrap_or(0) as u64,
        entry.send_buffer_size.unwrap_or(0) as u64,
    )?;
    udp_update(id, |slot| {
        *slot = new_entry;
        Ok(())
    })
}

#[op2(fast)]
pub(crate) fn op_udp_set_broadcast(#[bigint] id: u64, enabled: bool) -> Result<(), CoreError> {
    udp_update(id, |entry| {
        let sock = SockRef::from(entry.socket.as_ref());
        sock.set_broadcast(enabled).map_err(CoreError::from)?;
        entry.broadcast = enabled;
        Ok(())
    })
}

#[op2(fast)]
pub(crate) fn op_udp_set_ttl(#[bigint] id: u64, #[number] ttl: u64) -> Result<(), CoreError> {
    let ttl = normalize_port(ttl as f64)? as u32;
    udp_update(id, |entry| {
        let sock = SockRef::from(entry.socket.as_ref());
        sock.set_ttl(ttl).map_err(CoreError::from)?;
        entry.ttl = Some(ttl);
        Ok(())
    })
}

#[op2(fast)]
pub(crate) fn op_udp_set_multicast_ttl(
    #[bigint] id: u64,
    #[number] ttl: u64,
) -> Result<(), CoreError> {
    let ttl = normalize_port(ttl as f64)? as u32;
    udp_update(id, |entry| {
        let sock = SockRef::from(entry.socket.as_ref());
        match entry.family {
            UdpFamily::V4 => sock.set_multicast_ttl_v4(ttl).map_err(CoreError::from)?,
            UdpFamily::V6 => sock.set_multicast_hops_v6(ttl).map_err(CoreError::from)?,
        }
        entry.multicast_ttl = Some(ttl);
        Ok(())
    })
}

#[op2(fast)]
pub(crate) fn op_udp_set_multicast_loop(#[bigint] id: u64, enabled: bool) -> Result<(), CoreError> {
    udp_update(id, |entry| {
        let sock = SockRef::from(entry.socket.as_ref());
        match entry.family {
            UdpFamily::V4 => sock
                .set_multicast_loop_v4(enabled)
                .map_err(CoreError::from)?,
            UdpFamily::V6 => sock
                .set_multicast_loop_v6(enabled)
                .map_err(CoreError::from)?,
        }
        entry.multicast_loop = Some(enabled);
        Ok(())
    })
}

#[op2]
pub(crate) fn op_udp_set_multicast_if(
    #[bigint] id: u64,
    #[string] iface: Option<String>,
) -> Result<(), CoreError> {
    udp_update(id, |entry| {
        let sock = SockRef::from(entry.socket.as_ref());
        match entry.family {
            UdpFamily::V4 => {
                let addr = iface
                    .as_deref()
                    .map(parse_ipv4)
                    .unwrap_or_else(|| Ok(Ipv4Addr::UNSPECIFIED))?;
                sock.set_multicast_if_v4(&addr).map_err(CoreError::from)?;
                entry.multicast_if_v4 = Some(addr);
            }
            UdpFamily::V6 => {
                let index = parse_ipv6_interface(iface)?;
                sock.set_multicast_if_v6(index).map_err(CoreError::from)?;
                entry.multicast_if_v6 = Some(index);
            }
        }
        Ok(())
    })
}

#[op2]
pub(crate) fn op_udp_join_multicast(
    #[bigint] id: u64,
    #[string] multicast_address: String,
    #[string] interface: Option<String>,
) -> Result<(), CoreError> {
    udp_update(id, |entry| {
        let sock = SockRef::from(entry.socket.as_ref());
        match entry.family {
            UdpFamily::V4 => {
                let multi = parse_ipv4(&multicast_address)?;
                if !multi.is_multicast() {
                    return Err(CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Invalid multicast address",
                    )));
                }
                let iface = interface
                    .as_deref()
                    .map(parse_ipv4)
                    .unwrap_or_else(|| Ok(Ipv4Addr::UNSPECIFIED))?;
                sock.join_multicast_v4(&multi, &iface)
                    .map_err(CoreError::from)?;
            }
            UdpFamily::V6 => {
                let multi = parse_ipv6(&multicast_address)?;
                if !multi.is_multicast() {
                    return Err(CoreError::from(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        "Invalid multicast address",
                    )));
                }
                let index = parse_ipv6_interface(interface)?;
                sock.join_multicast_v6(&multi, index)
                    .map_err(CoreError::from)?;
            }
        }
        Ok(())
    })
}

#[op2]
pub(crate) fn op_udp_leave_multicast(
    #[bigint] id: u64,
    #[string] multicast_address: String,
    #[string] interface: Option<String>,
) -> Result<(), CoreError> {
    udp_update(id, |entry| {
        let sock = SockRef::from(entry.socket.as_ref());
        match entry.family {
            UdpFamily::V4 => {
                let multi = parse_ipv4(&multicast_address)?;
                let iface = interface
                    .as_deref()
                    .map(parse_ipv4)
                    .unwrap_or_else(|| Ok(Ipv4Addr::UNSPECIFIED))?;
                sock.leave_multicast_v4(&multi, &iface)
                    .map_err(CoreError::from)?;
            }
            UdpFamily::V6 => {
                let multi = parse_ipv6(&multicast_address)?;
                let index = parse_ipv6_interface(interface)?;
                sock.leave_multicast_v6(&multi, index)
                    .map_err(CoreError::from)?;
            }
        }
        Ok(())
    })
}

#[op2(fast)]
pub(crate) fn op_udp_set_recv_buffer_size(
    #[bigint] id: u64,
    #[number] size: u64,
) -> Result<(), CoreError> {
    udp_update(id, |entry| {
        let sock = SockRef::from(entry.socket.as_ref());
        sock.set_recv_buffer_size(size as usize)
            .map_err(CoreError::from)?;
        entry.recv_buffer_size = Some(size as u32);
        Ok(())
    })
}

#[op2(fast)]
pub(crate) fn op_udp_set_send_buffer_size(
    #[bigint] id: u64,
    #[number] size: u64,
) -> Result<(), CoreError> {
    udp_update(id, |entry| {
        let sock = SockRef::from(entry.socket.as_ref());
        sock.set_send_buffer_size(size as usize)
            .map_err(CoreError::from)?;
        entry.send_buffer_size = Some(size as u32);
        Ok(())
    })
}

#[op2(fast)]
#[number]
pub(crate) fn op_udp_get_recv_buffer_size(#[bigint] id: u64) -> Result<u64, CoreError> {
    let entry = udp_entry(id)?;
    let sock = SockRef::from(entry.socket.as_ref());
    let size = sock.recv_buffer_size().map_err(CoreError::from)?;
    Ok(size as u64)
}

#[op2(fast)]
#[number]
pub(crate) fn op_udp_get_send_buffer_size(#[bigint] id: u64) -> Result<u64, CoreError> {
    let entry = udp_entry(id)?;
    let sock = SockRef::from(entry.socket.as_ref());
    let size = sock.send_buffer_size().map_err(CoreError::from)?;
    Ok(size as u64)
}
