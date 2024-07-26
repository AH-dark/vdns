use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Context;
use clap::Parser;
use hickory_resolver::name_server::TokioConnectionProvider;
use hickory_server::authority::Catalog;
use hickory_server::proto::rr::Name;
use hickory_server::ServerFuture;
use hickory_server::store::forwarder::ForwardAuthority;
use rustls::{Certificate, PrivateKey};
use tokio::fs;
use tokio::net::UdpSocket;

#[derive(Debug, Parser)]
#[clap(name = "vdns")]
struct Opt {
    tls_cert: PathBuf,
    tls_key: PathBuf,
    dns_listen: Option<String>,
    quic_listen: Option<String>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();

    let options = Opt::parse();

    let connection_provider = TokioConnectionProvider::default();
    let forwarder = Arc::new(
        ForwardAuthority::new(connection_provider)
            .map_err(|e| {
                anyhow::anyhow!("Failed to create forwarder: {:?}", e)
            })?
    );

    let mut catalog = Catalog::new();
    catalog.upsert(Name::root().into(), Box::new(forwarder));

    let (certs, key) = {
        let key_path = &options.tls_key;
        let cert_path = &options.tls_cert;

        let key_file = fs::read(key_path).await.context("failed to read private key")?;
        let mut key = rustls_pemfile::pkcs8_private_keys(&mut &*key_file)
            .context("malformed PKCS #1 private key")?;
        let key = PrivateKey(key.remove(0));

        let cert_file = fs::read(cert_path).await.context("failed to read certificate chain")?;
        let cert_chain = rustls_pemfile::certs(&mut &*cert_file)?
            .into_iter()
            .map(|data| Certificate(data))
            .collect::<Vec<_>>();

        (cert_chain, key)
    };

    let mut server_future = ServerFuture::new(catalog);

    // Register QUIC listener
    server_future.register_quic_listener(
        UdpSocket::bind({
            let listen_addr = options.quic_listen.unwrap_or("0.0.0.0:4433".to_string());
            log::info!("QUIC DNS Listening on {}", listen_addr);
            listen_addr
        }).await?,
        std::time::Duration::from_secs(60),
        (certs, key),
        None,
    )?;

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
