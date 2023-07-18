use std::net::{IpAddr, Ipv4Addr, SocketAddr};

/**
 * Network address
 */

// This module only supports Ipv4
pub struct NetAddress {
    pub port: u16,
    pub addr_str: String,
    pub sock_addr: SocketAddr,
}

impl NetAddress {
    pub fn new(first: u8, second: u8, third: u8, fourth: u8, port: u16) -> Self {
        Self {
            port,
            addr_str: format!("{}.{}.{}.{}:{}", first, second, third, fourth, port),
            sock_addr: SocketAddr::new(
                IpAddr::V4(Ipv4Addr::new(first, second, third, fourth)),
                port,
            ),
        }
    }
}
