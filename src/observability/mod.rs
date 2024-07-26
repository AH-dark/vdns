use anyhow::Context;
use metrics_tracing_context::{MetricsLayer, TracingContextLayer};
use metrics_util::layers::Layer;
use tracing_subscriber::{EnvFilter, Registry};
use tracing_subscriber::layer::SubscriberExt;

use crate::observability::config::{MetricsExporterType, TraceExporterType};
use crate::observability::metrics::metrics_layer;

mod resource;
mod tracing;
pub mod config;
mod metrics;

pub fn init(
    service_name: String,
    service_version: String,
    config: &config::ObservabilityConfig,
) -> anyhow::Result<()> {
    let resource = resource::init_resource(service_name, service_version);

    if config.trace_exporter_type != TraceExporterType::None {
        let subscriber = Registry::default()
            .with(tracing::tracer_layer(config, resource)?)
            .with(EnvFilter::try_from_default_env().unwrap_or(EnvFilter::new("INFO")))
            .with(MetricsLayer::default());
        ::tracing::subscriber::set_global_default(subscriber).context("Failed to set global default subscriber")?;
    } else {
        tracing_subscriber::fmt::init();
    }

    if config.metrics_exporter_type != MetricsExporterType::None {
        let recorder = TracingContextLayer::all().layer(metrics_layer(&config)?);
        ::metrics::set_global_recorder(recorder).context("Failed to set global recorder")?;
    }

    Ok(())
}