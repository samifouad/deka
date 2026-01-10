mod dns;
mod tcp;
mod udp;

pub(super) use dns::{op_dns_lookup, op_dns_reverse};
pub(super) use tcp::{
    op_tcp_accept, op_tcp_close, op_tcp_connect, op_tcp_listen, op_tcp_listener_addr,
    op_tcp_listener_close, op_tcp_local_addr, op_tcp_peer_addr, op_tcp_read, op_tcp_shutdown,
    op_tcp_write,
};
pub(super) use udp::{
    op_udp_bind, op_udp_close, op_udp_connect, op_udp_disconnect, op_udp_get_recv_buffer_size,
    op_udp_get_send_buffer_size, op_udp_join_multicast, op_udp_leave_multicast, op_udp_local_addr,
    op_udp_peer_addr, op_udp_recv, op_udp_send, op_udp_set_broadcast, op_udp_set_multicast_if,
    op_udp_set_multicast_loop, op_udp_set_multicast_ttl, op_udp_set_recv_buffer_size,
    op_udp_set_send_buffer_size, op_udp_set_ttl,
};

#[derive(serde::Serialize)]
pub(super) struct UdpAddrResult {
    address: String,
    port: u16,
}
