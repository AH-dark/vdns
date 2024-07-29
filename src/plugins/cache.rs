use async_trait::async_trait;
use hickory_proto::op::Query;
use hickory_proto::rr::Record;

use crate::app::App;
use crate::plugin::Plugin;

#[derive(Clone, Debug)]
pub struct Cache {
    tag: String,
    lazy_cache_ttl: Option<u64>,

    /// `Query` is the key, `Vec<Record>` is the value, and `chrono::NaiveDateTime` is the store timestamp
    cache: moka::future::Cache<Query, (Vec<Record>, chrono::NaiveDateTime)>,
}

impl Cache {
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

    async fn exec(&self, app: &App, query: &Query, parent: String) -> Result<Vec<Record>, Box<dyn std::error::Error>> {
        let cache = self.cache.clone();
        let lazy_cache_ttl = self.lazy_cache_ttl;

        if let Some((records, timestamp)) = cache.get(query).await {
            if let Some(lazy_cache_ttl) = lazy_cache_ttl {
                if chrono::Utc::now().naive_utc() - timestamp <= chrono::Duration::seconds(lazy_cache_ttl as i64) {
                    return Ok(records);
                }
            } else {
                return Ok(records);
            }
        }

        let parent_plugin = app.get_plugin(&parent).unwrap();
        let children = parent_plugin.children();
        let next = {
            let mut iter = children.iter();
            let mut target = None;
            while let Some(value) = iter.next() {
                if value == &self.tag {
                    target = iter.next();
                    break;
                }
            }
            target
        };

        if let Some(next) = next {
            let plugin = app.get_plugin(next).unwrap();
            let records = plugin.exec(app, query, self.tag.clone()).await?;
            cache.insert(query.clone(), (records.clone(), chrono::Utc::now().naive_utc())).await;
            return Ok(records);
        }

        Ok(vec![])
    }
}
