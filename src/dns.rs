use std::sync::Arc;
use std::time::Duration;

use hickory_resolver::Name;
use hickory_resolver::name_server::TokioConnectionProvider;
use hickory_server::authority::Catalog;
use hickory_server::ServerFuture;
use hickory_server::store::forwarder::ForwardAuthority;
use tokio::net::{TcpListener, UdpSocket};

use crate::opt::Opt;
use crate::tls::new_tls_key_pair;

pub async fn new_catalog() -> anyhow::Result<Catalog> {
    let connection_provider = TokioConnectionProvider::default();
    let forwarder = Arc::new(
        ForwardAuthority::new(connection_provider)
            .map_err(|e| {
                anyhow::anyhow!("Failed to create forwarder: {:?}", e)
            })?
    );

    let mut catalog = Catalog::new();
    catalog.upsert(Name::root().into(), Box::new(forwarder));

    Ok(catalog)
}

pub async fn run_dns_server(options: &Opt, catalog: Catalog) -> anyhow::Result<()> {
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
        Duration::from_secs(60),
        key_pair,
        None,
    )?;

    // Register regular DNS listener
    server_future.register_socket(
        UdpSocket::bind({
            let listen_addr = options.dns_listen.clone().unwrap_or("0.0.0.0:53".into());
            log::info!("DNS Listening on {}", listen_addr);
            listen_addr
        }).await?
    );

    log::info!("DNS Server started");
    server_future.block_until_done().await?;

    Ok(())
}