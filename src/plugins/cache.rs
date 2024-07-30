use async_trait::async_trait;
use hickory_proto::op::Query;
use hickory_proto::rr::Record;

use crate::app::App;
use crate::plugin::{Plugin, PluginQueryResult};

#[derive(Clone, Debug)]
pub struct Cache {
    tag: String,
    lazy_cache_ttl: Option<u64>,

    /// `Query` is the key, `Vec<Record>` is the value, and `chrono::NaiveDateTime` is the store timestamp
    cache: moka::future::Cache<Query, (Vec<Record>, chrono::NaiveDateTime)>,
}

impl Cache {
    #[tracing::instrument]
    pub async fn new(
        tag: String,
        size: Option<usize>,
        lazy_cache_ttl: Option<u64>,
    ) -> Self {
        let cache = moka::future::Cache::new(size.unwrap_or(1024) as u64);

        Self {
            tag,
            lazy_cache_ttl,

            cache,
        }
    }
}

#[async_trait]
impl Plugin for Cache {
    fn tag(&self) -> String {
        self.tag.clone()
    }

    #[tracing::instrument(err, skip(self))]
    async fn exec(&self, _: &App, query: &Query) -> Result<PluginQueryResult, Box<dyn std::error::Error>> {
        let cache = self.cache.clone();
        let lazy_cache_ttl = self.lazy_cache_ttl;

        if let Some((records, timestamp)) = cache.get(query).await {
            if let Some(lazy_cache_ttl) = lazy_cache_ttl {
                if chrono::Utc::now().naive_utc() - timestamp <= chrono::Duration::seconds(lazy_cache_ttl as i64) {
                    return Ok(PluginQueryResult::return_records(records));
                }
            } else {
                return Ok(PluginQueryResult::return_records(records));
            }
        }

        Ok(PluginQueryResult::empty())
    }
}
