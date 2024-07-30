use std::collections::BTreeMap;
use std::path::PathBuf;

use crate::utils::{Matcher, parse_domain};

pub struct Hosts {
    tag: String,
    hosts: BTreeMap<String, Matcher>,
}

impl Hosts {
    pub async fn new(tag: String, entries: Vec<String>, files: Vec<PathBuf>) -> Self {
        let mut hosts = BTreeMap::new();

        let mut entries = entries;
        for file in files {
            let content = tokio::fs::read_to_string(&file).await.unwrap();
            entries.extend(
                content
                    .lines()
                    .map(|s| s.to_string())
                    .collect::<Vec<_>>()
            );
        }

        for entry in entries {
            let parts = entry.split_whitespace().collect::<Vec<_>>();
            if parts.len() < 2 {
                tracing::warn!("invalid hosts entry: {}", entry, "tag" = &tag);
                continue;
            }

            let domain = parts[0].to_string();
            let ips = parts[1..].to_vec();
            let matcher = match parse_domain(&domain) {
                Ok(matcher) => matcher,
                Err(e) => {
                    tracing::warn!("invalid domain: {}", e, "tag" = &tag);
                    continue;
                }
            };

            for ip in ips {
                hosts.insert(ip.to_string(), matcher.clone());
            }
        }

        Self { tag, hosts }
    }
}
