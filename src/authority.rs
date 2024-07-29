use std::sync::Arc;

use async_trait::async_trait;
use hickory_proto::op::{Query, ResponseCode};
use hickory_proto::rr::{LowerName, Name, RecordType};
use hickory_resolver::error::ResolveError;
use hickory_resolver::lookup::Lookup;
use hickory_server::authority::{Authority, AuthorityObject, LookupError, LookupOptions, MessageRequest, UpdateResult, ZoneType};
use hickory_server::server::RequestInfo;
use hickory_server::store::forwarder::ForwardLookup;

use crate::app::App;
use crate::plugin::Plugin;

#[derive(Clone, Debug)]
pub struct VDnsAuthority {
    app: Arc<App>,
    entry: String,
    entry_plugin: Arc<dyn Plugin>,
    origin: LowerName,
}

impl VDnsAuthority {
    pub fn new(app: Arc<App>, entry: String) -> Self {
        Self {
            entry_plugin: app.get_plugin(&entry).unwrap(),
            origin: LowerName::from(Name::root()),
            entry,
            app,
        }
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
        let records = self.entry_plugin.exec(&self.app, &query, self.entry.clone()).await.map_err(|e| ResolveError::from(e.to_string()))?;

        Ok(ForwardLookup(Lookup::new_with_max_ttl(query, Arc::from(records.into_boxed_slice()))))
    }

    async fn search(&self, request_info: RequestInfo<'_>, _: LookupOptions) -> Result<Self::Lookup, LookupError> {
        let query = request_info.query.original();
        let records = self.entry_plugin.exec(&self.app, query, self.entry.clone()).await.map_err(|e| ResolveError::from(e.to_string()))?;

        Ok(ForwardLookup(Lookup::new_with_max_ttl(query.clone(), Arc::from(records.into_boxed_slice()))))
    }

    async fn get_nsec_records(&self, name: &LowerName, _: LookupOptions) -> Result<Self::Lookup, LookupError> {
        let query = Query::query(name.into(), RecordType::NSEC);
        let records = self.entry_plugin.exec(&self.app, &query, self.entry.clone()).await.map_err(|e| ResolveError::from(e.to_string()))?;

        Ok(ForwardLookup(Lookup::new_with_max_ttl(query, Arc::from(records.into_boxed_slice()))))
    }
}
