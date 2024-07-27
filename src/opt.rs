use std::path::PathBuf;

#[derive(Debug, Clone, serde::Deserialize)]
pub struct DnsOptions {
    pub tls_cert: PathBuf,
    pub tls_key: PathBuf,
    #[serde(default)]
    pub dns_listen: Option<String>,
    #[serde(default)]
    pub tls_listen: Option<String>,
}
