use std::path::PathBuf;

use anyhow::Context;
use rustls::{Certificate, PrivateKey};
use tokio::fs;

pub async fn new_tls_key_pair(cert_path: &PathBuf, key_path: &PathBuf) -> anyhow::Result<(Vec<Certificate>, PrivateKey)> {
    let key_file = fs::read(key_path).await.context("failed to read private key")?;
    let mut key = rustls_pemfile::pkcs8_private_keys(&mut &*key_file)
        .context("malformed PKCS #1 private key")?;
    let key = PrivateKey(key.remove(0));

    let cert_file = fs::read(cert_path).await.context("failed to read certificate chain")?;
    let cert_chain = rustls_pemfile::certs(&mut &*cert_file)?
        .into_iter()
        .map(|data| Certificate(data))
        .collect::<Vec<_>>();

    Ok((cert_chain, key))
}
