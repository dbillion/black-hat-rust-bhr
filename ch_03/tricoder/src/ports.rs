use crate::{
    common_ports::MOST_COMMON_PORTS_100,
    model::{Port, Subdomain},
};
use futures::StreamExt;
use std::sync::Arc;
use std::net::{SocketAddr, ToSocketAddrs};
use std::time::Duration;
use tokio::net::TcpStream;
use tokio::sync::mpsc;

pub async fn scan_ports(concurrency: usize, subdomain: Subdomain) -> Subdomain {
    let mut ret = subdomain.clone();
    let socket_addresses: Vec<SocketAddr> = format!("{}:1024", subdomain.domain)
        .to_socket_addrs()
        .expect("port scanner: Creating socket address")
        .collect();

    if socket_addresses.is_empty() {
        return subdomain;
    }

    // Chapter 2 fixed this in the blocking scanner and Chapter 3 inherited the
    // bug unchanged: scanning only socket_addresses[0] probes a single address
    // family. On a host that resolves IPv6 first (localhost, and many real
    // domains) that silently misses every service bound to IPv4, and the output
    // looks complete. Probe every resolved address instead; ports are numbered
    // identically across addresses so a port open on any of them is open.
    let socket_addresses = Arc::new(socket_addresses);

    // Concurrent stream method 3: using channels
    let (input_tx, input_rx) = mpsc::channel::<(SocketAddr, u16)>(concurrency);
    let (output_tx, output_rx) = mpsc::channel(concurrency);

    {
        let addresses = Arc::clone(&socket_addresses);
        tokio::spawn(async move {
            for socket_address in addresses.iter() {
                for port in MOST_COMMON_PORTS_100 {
                    let _ = input_tx.send((*socket_address, *port)).await;
                }
            }
        });
    }

    let input_rx_stream = tokio_stream::wrappers::ReceiverStream::new(input_rx);
    input_rx_stream
        .for_each_concurrent(concurrency, |(socket_address, port)| {
            let output_tx = output_tx.clone();
            async move {
                let port = scan_port(socket_address, port).await;
                if port.is_open {
                    let _ = output_tx.send(port).await;
                }
            }
        })
        .await;
    // close channel
    drop(output_tx);

    let output_rx_stream = tokio_stream::wrappers::ReceiverStream::new(output_rx);
    let mut open_ports: Vec<Port> = output_rx_stream.collect().await;

    // Probing every resolved address means a port that is open on more than one
    // of them is reported once per address. Deduplicate by port number and keep
    // the order stable, so the output reads the same as the single-address
    // version did.
    let mut seen = std::collections::HashSet::new();
    open_ports.retain(|p| seen.insert(p.port));
    ret.open_ports = open_ports;

    ret
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
    }
}
