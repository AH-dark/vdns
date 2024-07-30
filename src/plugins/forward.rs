use std::time::Duration;

use async_trait::async_trait;
use hickory_proto::op::Query;
use hickory_resolver::config::{NameServerConfig, Protocol, ResolverConfig, ResolverOpts};
use hickory_resolver::name_server::TokioConnectionProvider;
use hickory_resolver::TokioAsyncResolver;

use crate::app::App;
use crate::plugin::{Plugin, PluginQueryResult};
use crate::types::rule::ForwardUpstream;

#[derive(Debug, Clone)]
pub struct Forward {
    pub tag: String,
    pub resolver: TokioAsyncResolver,
}

impl Forward {
    #[tracing::instrument(err)]
    fn upstream_to_nameserver(upstream: &ForwardUpstream) -> anyhow::Result<NameServerConfig> {
        let (protocol, addr) = upstream.addr.split_once("://").unwrap_or(("udp", &upstream.addr));
        let protocol = match protocol {
            "udp" => Protocol::Udp,
            "tcp" => Protocol::Tcp,
            "tls" => Protocol::Tls,
            "https" => Protocol::Https,
            "quic" => Protocol::Quic,
            "h3" => Protocol::H3,
            _ => return Err(anyhow::anyhow!("unsupported protocol: {}", protocol)),
        };

        Ok(NameServerConfig::new(addr.parse()?, protocol))
    }

    #[tracing::instrument]
    pub fn new(tag: String, concurrent: Option<usize>, timeout: Option<u64>, upstreams: Vec<ForwardUpstream>) -> Self {
        let mut config = ResolverConfig::new();
        for upstream in upstreams {
            match Self::upstream_to_nameserver(&upstream) {
                Ok(ns) => {
                    config.add_name_server(ns);
                }
                Err(e) => {
                    tracing::warn!(tag = &tag, upstream_tag = &upstream.tag, "invalid upstream: {}", e);
                }
            }
        }

        let mut opts = ResolverOpts::default();
        opts.timeout = Duration::from_secs(timeout.unwrap_or(5));
        opts.num_concurrent_reqs = concurrent.unwrap_or(1).min(3).max(1);

        let provider = TokioConnectionProvider::default();
        let resolver = TokioAsyncResolver::new(
            config,
            opts,
            provider,
        );

        Self {
            tag,
            resolver,
        }
    }
}

#[async_trait]
impl Plugin for Forward {
    fn tag(&self) -> String {
        self.tag.clone()
    }

    #[tracing::instrument(err, skip(self))]
    async fn exec(&self, _: &App, query: &Query) -> Result<PluginQueryResult, Box<dyn std::error::Error>> {
        let res = self.resolver.lookup(query.name().clone(), query.query_type()).await?;
        Ok(PluginQueryResult::return_records(res.records().into()))
    }
}

