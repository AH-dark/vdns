use std::collections::HashMap;
use std::fmt::Debug;
use std::net::IpAddr;
use std::path::PathBuf;

use async_trait::async_trait;
use hickory_proto::op::Query;
use hickory_proto::rr::{RData, Record};
use hickory_proto::rr::rdata::{A, AAAA};

use crate::app::App;
use crate::plugin::Plugin;
use crate::utils::{Matcher, parse_domain};

#[derive(Clone)]
pub struct Hosts {
    tag: String,
    hosts: HashMap<String, (Matcher, Vec<IpAddr>)>,
}

impl Hosts {
    #[tracing::instrument]
    pub async fn new(tag: String, entries: Vec<String>, files: Vec<PathBuf>) -> Self {
        let mut hosts: HashMap<String, (Matcher, Vec<IpAddr>)> = HashMap::new();

        let mut entries = entries;
        for file in files {
            let content = tokio::fs::read_to_string(&file).await.unwrap();
            entries.extend(content.lines().map(String::from));
        }

        for entry in entries {
            let parts = entry.split_whitespace().collect::<Vec<_>>();
            if parts.len() < 2 {
                tracing::warn!(tag = &tag, "invalid hosts entry: {}", entry);
                continue;
            }

            let rule = parts[0].to_string();
            let ips = parts[1..].to_vec();
            let matcher = match parse_domain(&rule) {
                Ok(matcher) => matcher,
                Err(e) => {
                    tracing::warn!(tag = &tag, "invalid domain: {}", e);
                    continue;
                }
            };

            let ip_list = ips.iter().filter_map(|ip| ip.parse::<IpAddr>().ok()).collect::<Vec<_>>();
            hosts.entry(rule).or_insert_with(|| (matcher, Vec::new())).1.extend(ip_list);
        }

        Self { tag, hosts }
    }
}

impl Debug for Hosts {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Hosts")
            .field("tag", &self.tag)
            .field("rules", &self.hosts.keys().collect::<Vec<_>>())
            .finish()
    }
}

#[async_trait]
impl Plugin for Hosts {
    fn tag(&self) -> String {
        self.tag.clone()
    }

    #[tracing::instrument(err, skip(self))]
    async fn exec(&self, _app: &App, query: &Query, _parent: String) -> Result<Vec<Record>, Box<dyn std::error::Error>> {
        let mut records = vec![];
        if let Some((matcher, ips)) = self.hosts.get(&query.name().to_string()) {
            if matcher(query.name().to_string()) {
                for ip in ips {
                    let record = match ip {
                        IpAddr::V4(ip) => Record::from_rdata(query.name().clone(), 0, RData::A(A::from(*ip))),
                        IpAddr::V6(ip) => Record::from_rdata(query.name().clone(), 0, RData::AAAA(AAAA::from(*ip))),
                    };

                    records.push(record);
                }
            }
        }

        Ok(records)
    }
}