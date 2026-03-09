use std::net::{IpAddr, Ipv4Addr, SocketAddr};

pub fn default_client_bind_addr() -> SocketAddr {
    SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), 0)
}
