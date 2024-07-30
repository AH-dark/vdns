use std::collections::BTreeMap;
use std::sync::Arc;
use std::time::Duration;

use anyhow::Context;
use hickory_proto::rr::Name;
use hickory_server::authority::Catalog;
use tokio::net::{TcpListener, UdpSocket};

use crate::authority::VDnsAuthority;
use crate::types::rule;
use crate::types::rule::PluginType;

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
        let plugin_executor = match &plugin.plugin_type {
            PluginType::Cache { size, lazy_cache_ttl, .. } => {
                let plugin = plugins::cache::Cache::new(
                    plugin.tag.clone(),
                    *size,
                    *lazy_cache_ttl,
                ).await;

                Arc::new(plugin) as Arc<dyn plugin::Plugin>
            }
            PluginType::Hosts { entries, files } => {
                let plugin = plugins::hosts::Hosts::new(
                    plugin.tag.clone(),
                    entries.clone(),
                    files.clone(),
                ).await;

                Arc::new(plugin) as Arc<dyn plugin::Plugin>
            }
            _ => todo!("plugin: {:?}", plugin),
        };

        plugins.insert(plugin.tag.clone(), plugin_executor);
    }

    let app = app::App::new(plugins);

    let servers = config.plugins
        .iter()
        .filter(|p| p.plugin_type.is_server())
        .cloned()
        .collect::<Vec<_>>();

    for server in servers {
        let mut catalog = Catalog::new();
        let authority = VDnsAuthority::new(app.clone(), {
            match server.plugin_type {
                PluginType::TcpServer { entry, .. } => entry,
                PluginType::UdpServer { entry, .. } => entry,
                PluginType::QuicServer { entry, .. } => entry,
                PluginType::HttpServer { .. } => todo!("http server: {:?}", server),
                _ => unreachable!("plugin: {:?}", server),
            }
        });
        catalog.upsert(Name::root().into(), Box::new(Arc::new(authority)));

        let mut svr = hickory_server::ServerFuture::new(catalog);

        match server.plugin_type {
            PluginType::TcpServer { listen, cert, key, idle_timeout, .. } => {
                match (cert, key) {
                    (Some(cert), Some(key)) => {
                        let (cert, key) = utils::new_tls_key_pair(&cert, &key).await?;
                        svr.register_tls_listener(
                            TcpListener::bind(listen).await?,
                            Duration::from_secs(idle_timeout.unwrap_or(30)),
                            (cert, key),
                        )?
                    }
                    _ => svr.register_listener(TcpListener::bind(listen).await?, Duration::from_secs(idle_timeout.unwrap_or(30))),
                }
            }
            PluginType::UdpServer { listen, .. } => svr.register_socket(UdpSocket::bind(listen).await?),
            PluginType::QuicServer { listen, cert, key, idle_timeout, .. } => {
                let (cert, key) = utils::new_tls_key_pair(&cert, &key).await?;
                svr.register_quic_listener(
                    UdpSocket::bind(listen).await?,
                    Duration::from_secs(idle_timeout.unwrap_or(30)),
                    (cert, key),
                    None,
                )?
            }
            PluginType::HttpServer { .. } => {
                todo!("http server: {:?}", server)
            }
            _ => unreachable!("plugin: {:?}", server),
        }
    }

    Ok(())
}
