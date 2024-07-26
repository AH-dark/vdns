use std::path::PathBuf;

use clap::Parser;

#[derive(Debug, Parser)]
#[clap(name = "vdns")]
pub struct Opt {
    pub tls_cert: PathBuf,
    pub tls_key: PathBuf,
    pub dns_listen: Option<String>,
    pub tls_listen: Option<String>,
}
