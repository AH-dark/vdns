use std::sync::Arc;

use hickory_resolver::Name;
use hickory_resolver::name_server::TokioConnectionProvider;
use hickory_server::authority::Catalog;
use hickory_server::store::forwarder::ForwardAuthority;

pub async fn new_catalog() -> anyhow::Result<Catalog> {
    let connection_provider = TokioConnectionProvider::default();
    let forwarder = Arc::new(
        ForwardAuthority::new(connection_provider)
            .map_err(|e| {
                anyhow::anyhow!("Failed to create forwarder: {:?}", e)
            })?
    );

    let mut catalog = Catalog::new();
    catalog.upsert(Name::root().into(), Box::new(forwarder));

    Ok(catalog)
}
