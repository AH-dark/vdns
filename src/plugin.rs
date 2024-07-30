use std::fmt::Debug;

use async_trait::async_trait;
use hickory_proto::op::Query;
use hickory_proto::rr::Record;

use crate::app::App;

pub struct PluginQueryResult {
    pub records: Vec<Record>,
    pub next: Option<String>,
}

impl PluginQueryResult {
    pub fn new(records: Vec<Record>, next: Option<String>) -> Self {
        Self { records, next }
    }

    pub fn return_records(records: Vec<Record>) -> Self {
        Self { records, next: None }
    }

    pub fn empty() -> Self {
        Self { records: vec![], next: None }
    }

    pub fn with_next(mut self, next: String) -> Self {
        self.next = Some(next);
        self
    }
}

#[async_trait]
pub trait Plugin: Debug + Send + Sync {
    fn tag(&self) -> String;
    fn children(&self) -> Vec<String> {
        vec![]
    }

    async fn exec(&self, app: &App, query: &Query) -> Result<PluginQueryResult, Box<dyn std::error::Error>>;
}
