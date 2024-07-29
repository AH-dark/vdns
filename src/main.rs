use clap::Parser;

use types::opt::DnsOptions;

use crate::dns::{new_catalog, run_dns_server};

mod tls;
mod dns;
mod authority;
mod observability;
pub mod types;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let observability_config = envy::from_env::<observability::config::ObservabilityConfig>()?;
    observability::init("vdns".into(), env!("CARGO_PKG_VERSION").into(), &observability_config)?;
    log::debug!("Observability initialized, configurion: {:?}", observability_config);

    let options = envy::prefixed("VDNS_").from_env::<DnsOptions>()?;

    let catalog = new_catalog(&options).await?;
    run_dns_server(&options, catalog).await?;

    Ok(())
}
