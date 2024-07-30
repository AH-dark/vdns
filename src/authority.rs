use std::sync::Arc;

use async_trait::async_trait;
use hickory_proto::op::{Query, ResponseCode};
use hickory_proto::rr::{LowerName, Name, Record, RecordType};
use hickory_resolver::error::ResolveError;
use hickory_resolver::lookup::Lookup;
use hickory_server::authority::{Authority, LookupError, LookupOptions, MessageRequest, UpdateResult, ZoneType};
use hickory_server::server::RequestInfo;
use hickory_server::store::forwarder::ForwardLookup;

use crate::app::App;

#[derive(Clone, Debug)]
pub struct VDnsAuthority {
    app: Arc<App>,
    entry: String,
    origin: LowerName,
}

impl VDnsAuthority {
    pub fn new(app: Arc<App>, entry: String) -> Self {
        Self {
            origin: LowerName::from(Name::root()),
            entry,
            app,
        }
    }

    #[tracing::instrument]
    async fn query(&self, query: &Query) -> Vec<Record> {
        let mut next_plugin = Some(self.entry.clone());
        while let Some(name) = &next_plugin {
            let plugin = self.app.get_plugin(&name).unwrap();
            let result = plugin.exec(&self.app, &query).await.map_err(|e| ResolveError::from(e.to_string()));
            match result {
                Ok(result) => {
                    if !result.records.is_empty() {
                        return result.records;
                    }

                    next_plugin = result.next;
                }
                Err(err) => {
                    tracing::warn!(plugin = &plugin.tag(), "error: {}", err);
                    continue;
                }
            }
        }

        vec![]
    }
}

#[async_trait]
impl Authority for VDnsAuthority {
    type Lookup = ForwardLookup;

    fn zone_type(&self) -> ZoneType {
        ZoneType::Forward
    }

    fn is_axfr_allowed(&self) -> bool {
        false
    }

    async fn update(&self, _: &MessageRequest) -> UpdateResult<bool> {
        Err(ResponseCode::NotImp)
    }

    fn origin(&self) -> &LowerName {
        &self.origin
    }

    async fn lookup(&self, name: &LowerName, rtype: RecordType, _: LookupOptions) -> Result<Self::Lookup, LookupError> {
        let query = Query::query(name.into(), rtype);
        let records = self.query(&query).await;

        Ok(ForwardLookup(Lookup::new_with_max_ttl(query, Arc::from(records))))
    }

    async fn search(&self, request_info: RequestInfo<'_>, _: LookupOptions) -> Result<Self::Lookup, LookupError> {
        let query = request_info.query.original();
        let records = self.query(&query).await;

        Ok(ForwardLookup(Lookup::new_with_max_ttl(query.clone(), Arc::from(records))))
    }

    async fn get_nsec_records(&self, name: &LowerName, _: LookupOptions) -> Result<Self::Lookup, LookupError> {
        let query = Query::query(name.into(), RecordType::NSEC);
        let records = self.query(&query).await;

        Ok(ForwardLookup(Lookup::new_with_max_ttl(query, Arc::from(records))))
    }
}
