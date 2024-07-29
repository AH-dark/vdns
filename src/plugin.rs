use std::fmt::Debug;

use async_trait::async_trait;
use hickory_proto::op::Query;
use hickory_proto::rr::Record;

use crate::app::App;

#[async_trait]
pub trait Plugin: Debug + Send + Sync {
    fn tag(&self) -> String;
    fn children(&self) -> Vec<String> {
        vec![]
    }

    async fn exec(&self, app: &App, query: &Query, parent: String) -> Result<Vec<Record>, Box<dyn std::error::Error>>;
}
