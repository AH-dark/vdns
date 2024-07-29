use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use hickory_proto::rr::Name;
use hickory_server::authority::Catalog;
use tokio::net::{TcpListener, UdpSocket};

use crate::authority::VDnsAuthority;
use crate::types::rule;
use crate::types::rule::{Listener, PluginType, Server};

mod utils;
mod observability;
pub mod types;
pub mod app;
pub mod plugin;
pub mod plugins;
mod authority;

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
    let mut plugins = BTreeMap::new();
    for plugin in &config.plugins {
        let plugin_executor = match plugin.plugin_type {
            PluginType::Cache { size, lazy_cache_ttl, .. } => {
                let plugin = plugins::cache::Cache::new(
                    plugin.tag.clone(),
                    size,
                    lazy_cache_ttl,
                ).await;

                Arc::new(plugin) as Arc<dyn plugin::Plugin>
            }
            _ => todo!("plugin: {:?}", plugin),
        };

        plugins.insert(plugin.tag.clone(), plugin_executor);
    }

    let app = app::App::new(plugins);

    let servers = config.servers.clone();
    for server in servers {
        let mut catalog = Catalog::new();
        let authority = VDnsAuthority::new(app.clone(), server.exec.clone());
        catalog.upsert(Name::root().into(), Box::new(Arc::new(authority)));

        let mut svr = hickory_server::ServerFuture::new(catalog);

        for listener in server.listeners {
            match listener {
                Listener::Tcp { addr } => svr.register_listener(TcpListener::bind(addr).await?, Duration::from_secs(5)),
                Listener::Udp { addr } => svr.register_socket(UdpSocket::bind(addr).await?),
                Listener::Quic { addr, cert, key } => {
                    let (cert, key) = utils::new_tls_key_pair(&cert, &key).await?;
                    svr.register_quic_listener(
                        UdpSocket::bind(addr).await?,
                        Duration::from_secs(5),
                        (cert, key),
                        None,
                    )?
                }
                Listener::Tls { addr, cert, key } => {
                    let (cert, key) = utils::new_tls_key_pair(&cert, &key).await?;
                    svr.register_tls_listener(
                        TcpListener::bind(addr).await?,
                        Duration::from_secs(5),
                        (cert, key),
                    )?
                }
                _ => todo!("listener: {:?}", listener),
            }
        }
    }

    Ok(())
}
