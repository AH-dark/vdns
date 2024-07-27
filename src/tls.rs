use std::path::PathBuf;

use anyhow::Context;
use rustls::{Certificate, PrivateKey};
use tokio::fs;

pub async fn new_tls_key_pair(cert_path: &PathBuf, key_path: &PathBuf) -> anyhow::Result<(Vec<Certificate>, PrivateKey)> {
    let key_file = fs::read(key_path).await.context("failed to read private key")?;

    // Try to parse the key as an RSA key, then as a PKCS8 key, then as an ECC key
    let keys = {
        if let Ok(rsa_private_keys) = rustls_pemfile::rsa_private_keys(&mut &*key_file) {
            Some(rsa_private_keys)
        } else if let Ok(pkcs8_private_keys) = rustls_pemfile::pkcs8_private_keys(&mut &*key_file) {
            Some(pkcs8_private_keys)
        } else if let Ok(ecc_private_keys) = rustls_pemfile::ec_private_keys(&mut &*key_file) {
            Some(ecc_private_keys)
        } else {
            return Err(anyhow::anyhow!("failed to parse private key"));
        }
    };
    
    let key = PrivateKey(keys.unwrap().remove(0));

    let cert_file = fs::read(cert_path).await.context("failed to read certificate chain")?;
    let cert_chain = rustls_pemfile::certs(&mut &*cert_file)?
        .into_iter()
        .map(|data| Certificate(data))
        .collect::<Vec<_>>();

    Ok((cert_chain, key))
}
