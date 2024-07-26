// Copyright 2015-2021 Benjamin Fry <benjaminfry@me.com>
//
// Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
// https://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
// https://opensource.org/licenses/MIT>, at your option. This file may not be
// copied, modified, or distributed except according to those terms.

use std::io;

use hickory_resolver::name_server::TokioConnectionProvider;
use hickory_server::{
    authority::{
        Authority, LookupError, LookupObject, LookupOptions, MessageRequest, UpdateResult, ZoneType,
    },
    proto::{
        op::ResponseCode,
        rr::{LowerName, Name, Record, RecordType},
    },
    resolver::{config::ResolverConfig, lookup::Lookup as ResolverLookup, TokioAsyncResolver},
    server::RequestInfo,
    store::forwarder::ForwardConfig,
};
use tracing::{debug, info};

/// An authority that will forward resolutions to upstream resolvers.
///
/// This uses the hickory-resolver for resolving requests.
#[derive(Clone, Debug)]
pub struct ForwardAuthority {
    origin: LowerName,
    resolver: TokioAsyncResolver,
}

impl ForwardAuthority {
    /// TODO: change this name to create or something
    #[allow(clippy::new_without_default)]
    #[doc(hidden)]
    pub fn new(resolver: TokioAsyncResolver) -> Result<Self, String> {
        Ok(Self {
            origin: Name::root().into(),
            resolver,
        })
    }
}

#[async_trait::async_trait]
impl Authority for ForwardAuthority {
    type Lookup = ForwardLookup;

    /// Always Forward
    #[tracing::instrument]
    fn zone_type(&self) -> ZoneType {
        ZoneType::Forward
    }

    /// Always false for Forward zones
    #[tracing::instrument]
    fn is_axfr_allowed(&self) -> bool {
        false
    }

    #[tracing::instrument(err)]
    async fn update(&self, _update: &MessageRequest) -> UpdateResult<bool> {
        Err(ResponseCode::NotImp)
    }

    /// Get the origin of this zone, i.e. example.com is the origin for www.example.com
    ///
    /// In the context of a forwarder, this is either a zone which this forwarder is associated,
    ///   or `.`, the root zone for all zones. If this is not the root zone, then it will only forward
    ///   for lookups which match the given zone name.
    #[tracing::instrument]
    fn origin(&self) -> &LowerName {
        &self.origin
    }

    /// Forwards a lookup given the resolver configuration for this Forwarded zone
    #[tracing::instrument(err)]
    async fn lookup(
        &self,
        name: &LowerName,
        rtype: RecordType,
        _lookup_options: LookupOptions,
    ) -> Result<Self::Lookup, LookupError> {
        // TODO: make this an error?
        debug_assert!(self.origin.zone_of(name));

        debug!("forwarding lookup: {} {}", name, rtype);
        let name: LowerName = name.clone();
        let resolve = self.resolver.lookup(name, rtype).await;

        resolve.map(ForwardLookup).map_err(LookupError::from)
    }

    #[tracing::instrument(err, skip(request_info))]
    async fn search(
        &self,
        request_info: RequestInfo<'_>,
        lookup_options: LookupOptions,
    ) -> Result<Self::Lookup, LookupError> {
        self.lookup(
            request_info.query.name(),
            request_info.query.query_type(),
            lookup_options,
        )
            .await
    }

    #[tracing::instrument(err)]
    async fn get_nsec_records(
        &self,
        _name: &LowerName,
        _lookup_options: LookupOptions,
    ) -> Result<Self::Lookup, LookupError> {
        Err(LookupError::from(io::Error::new(
            io::ErrorKind::Other,
            "Getting NSEC records is unimplemented for the forwarder",
        )))
    }
}

/// A structure that holds the results of a forwarding lookup.
///
/// This exposes an iterator interface for consumption downstream.
#[derive(Clone, Debug)]
pub struct ForwardLookup(pub ResolverLookup);

impl LookupObject for ForwardLookup {
    fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    fn iter<'a>(&'a self) -> Box<dyn Iterator<Item=&'a Record> + Send + 'a> {
        Box::new(self.0.record_iter())
    }

    fn take_additionals(&mut self) -> Option<Box<dyn LookupObject>> {
        None
    }
}
