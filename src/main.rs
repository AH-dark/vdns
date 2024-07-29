use anyhow::Context;

use crate::types::rule;

mod utils;
mod observability;
pub mod types;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let observability_config = envy::from_env::<observability::config::ObservabilityConfig>()?;
    observability::init("vdns".into(), env!("CARGO_PKG_VERSION").into(), &observability_config)?;
    log::debug!("Observability initialized, configurion: {:?}", observability_config);

    let rules_config = config::Config::builder()
        .add_source(config::File::with_name("config/rules").required(false))
        .add_source(config::File::with_name("config").required(false))
        .add_source(config::Environment::with_prefix("RULES"))
        .build()
        .context("failed to build configuration")?;

    let rules = rules_config.try_deserialize::<rule::Config>().context("failed to deserialize configuration")?;

    run(&rules).await.context("failed to run")?;

    Ok(())
}

async fn run(config: &rule::Config) -> anyhow::Result<()> {
    Ok(())
}
