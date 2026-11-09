use crate::{
    common_ports::MOST_COMMON_PORTS,
    modules::{Port, Subdomain},
};
use futures::{stream, StreamExt};
use std::collections::HashSet;
use std::net::{SocketAddr, ToSocketAddrs};
use std::time::Duration;
use tokio::net::TcpStream;

pub async fn scan_ports(concurrency: usize, mut subdomain: Subdomain) -> Subdomain {
    let socket_addresses: Vec<SocketAddr> = format!("{}:1024", subdomain.domain)
        .to_socket_addrs()
        .expect("port scanner: Creating socket address")
        .collect();

    if socket_addresses.is_empty() {
        return subdomain;
    }

    // The same defect fixed in Chapters 2 and 3 reappears here: using
    // socket_addresses[0] probes only the first resolved address. On a
    // dual-stack host IPv6 resolves first, so every service bound to IPv4 is
    // invisible and the output looks complete. Materialise a flat
    // (address, port) work list; nesting a parallel scan inside buffer_unordered
    // over a stream would execute nothing.
    let mut probes: Vec<(SocketAddr, u16)> = Vec::with_capacity(
        socket_addresses.len() * MOST_COMMON_PORTS.len(),
    );
    for socket_address in &socket_addresses {
        for port in MOST_COMMON_PORTS.iter() {
            probes.push((*socket_address, *port));
        }
    }

    subdomain.open_ports = stream::iter(probes.into_iter())
        .map(|(socket_address, port)| async move {
            let port = scan_port(socket_address, port).await;
            if port.is_open {
                return Some(port);
            }
            None
        })
        .buffer_unordered(concurrency)
        .filter_map(|port| async { port })
        .collect()
        .await;

    // A port open on several resolved addresses is one open port; deduplicate
    // it so the output matches the single-address version's shape.
    let mut seen = HashSet::new();
    subdomain.open_ports.retain(|p| seen.insert(p.port));

    subdomain
}

async fn scan_port(mut socket_address: SocketAddr, port: u16) -> Port {
    let timeout = Duration::from_secs(3);
    socket_address.set_port(port);

    let is_open = matches!(
        tokio::time::timeout(timeout, TcpStream::connect(&socket_address)).await,
        Ok(Ok(_)),
    );

    Port {
        port: port,
        is_open,
        findings: Vec::new(),
    }
}
