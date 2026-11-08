use crate::{
    common_ports::MOST_COMMON_PORTS_100,
    model::{Port, Subdomain},
};
use rayon::prelude::*;
use std::collections::HashSet;
use std::net::{SocketAddr, ToSocketAddrs};
use std::{net::TcpStream, time::Duration};

pub fn scan_ports(mut subdomain: Subdomain) -> Subdomain {
    let socket_addresses: Vec<SocketAddr> = format!("{}:1024", subdomain.domain)
        .to_socket_addrs()
        .expect("port scanner: Creating socket address")
        .collect();

    if socket_addresses.is_empty() {
        return subdomain;
    }

    // A hostname usually resolves to several addresses, and on modern Linux
    // localhost and many real domains resolve to IPv6 first. Scanning only
    // socket_addresses[0] probes a single address family and silently reports
    // false negatives for every service bound to the other one: the output
    // looks complete while missing open ports, which is worse than an error.
    //
    // Ports are numbered identically across addresses, so probe every resolved
    // address and report a port as open when it answers on any of them.
    //
    // The pairs are materialised first and then scanned in parallel: nesting
    // into_par_iter inside flat_map over a parallel iterator does not execute,
    // so a flat work list is required for rayon to actually run the probes.
    let mut probes: Vec<(SocketAddr, u16)> = Vec::with_capacity(
        socket_addresses.len() * MOST_COMMON_PORTS_100.len(),
    );
    for socket_address in &socket_addresses {
        for port in MOST_COMMON_PORTS_100.iter() {
            probes.push((*socket_address, *port));
        }
    }

    let open_ports: HashSet<u16> = probes
        .into_par_iter()
        .filter(|(socket_address, port)| socket_is_open(*socket_address, *port))
        .map(|(_, port)| port)
        .collect();

    let mut open_ports: Vec<u16> = open_ports.into_iter().collect();
    open_ports.sort_unstable();

    subdomain.open_ports = open_ports
        .into_iter()
        .map(|port| Port { port, is_open: true })
        .collect();
    subdomain
}

fn socket_is_open(mut socket_address: SocketAddr, port: u16) -> bool {
    socket_address.set_port(port);
    TcpStream::connect_timeout(&socket_address, Duration::from_secs(3)).is_ok()
}
