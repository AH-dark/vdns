use std::time::Duration;

use opentelemetry::KeyValue;
use opentelemetry_sdk::resource::{
    EnvResourceDetector, SdkProvidedResourceDetector, TelemetryResourceDetector,
};
use opentelemetry_sdk::Resource;

/// Initialize the open-telemetry resource.
pub fn init_resource(service_name: String, service_version: String) -> Resource {
    let detector_resources = Resource::from_detectors(
        Duration::from_secs(10),
        vec![
            Box::new(EnvResourceDetector::new()),
            Box::new(TelemetryResourceDetector),
            Box::new(SdkProvidedResourceDetector),
        ],
    );

    let kvs = vec![
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_NAME,
            service_name,
        ),
        KeyValue::new(
            opentelemetry_semantic_conventions::resource::SERVICE_VERSION,
            service_version,
        ),
    ];

    detector_resources.merge(&Resource::new(kvs))
}
