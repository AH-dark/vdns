use clap::Parser;

use crate::dns::{new_catalog, run_dns_server};
use crate::opt::Opt;

mod tls;
mod dns;
pub mod opt;
mod authority;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt::init();
    let options = Opt::parse();

    let catalog = new_catalog().await?;
    run_dns_server(&options, catalog).await?;

    Ok(())
}
