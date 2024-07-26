use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use clap::Parser;
use hickory_resolver::name_server::TokioConnectionProvider;
use hickory_server::authority::Catalog;
use hickory_server::proto::rr::Name;
use hickory_server::server::Protocol::Tcp;
use hickory_server::ServerFuture;
use hickory_server::store::forwarder::ForwardAuthority;
use rustls::{Certificate, PrivateKey};
use tokio::fs;
use tokio::net::{TcpListener, UdpSocket};

use crate::authority::new_catalog;
use crate::tls::new_tls_key_pair;

mod authority;
mod tls;

#[derive(Debug, Parser)]
#[clap(name = "vdns")]
struct Opt {
    tls_cert: PathBuf,
    tls_key: PathBuf,
    dns_listen: Option<String>,
    tls_listen: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let options = Opt::parse();

    let catalog = new_catalog().await?;
    let mut server_future = ServerFuture::new(catalog);

    let tls_listen = options.tls_listen.clone().unwrap_or("0.0.0.0:853".into());
    log::info!("TLS Authority Listening on {}", tls_listen);

    let key_pair = new_tls_key_pair(&options.tls_cert, &options.tls_key).await?;

    // Register TLS listener
    server_future.register_tls_listener(
        TcpListener::bind(&tls_listen).await?,
        Duration::from_secs(60),
        key_pair.clone(),
    )?;

    // Register QUIC listener
    server_future.register_quic_listener(
        UdpSocket::bind(&tls_listen).await?,
        std::time::Duration::from_secs(60),
        key_pair,
        None,
    )?;

    // Register regular DNS listener
    server_future.register_socket(
        UdpSocket::bind({
            let listen_addr = options.dns_listen.unwrap_or("0.0.0.0:53".into());
            log::info!("DNS Listening on {}", listen_addr);
            listen_addr
        }).await?
    );

    log::info!("Server started");
    server_future.block_until_done().await?;

    Ok(())
}
