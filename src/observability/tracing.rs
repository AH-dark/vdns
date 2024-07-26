use opentelemetry::global;
use opentelemetry_otlp::{
    ExportConfig, HttpExporterBuilder, SpanExporterBuilder, TonicExporterBuilder, WithExportConfig,
};
use opentelemetry_sdk::propagation::TraceContextPropagator;
use opentelemetry_sdk::Resource;
use opentelemetry_sdk::runtime::Tokio;
use opentelemetry_sdk::trace::{Sampler, Tracer};
use tracing_opentelemetry::OpenTelemetryLayer;
use tracing_subscriber::Registry;

use super::config::{ObservabilityConfig, TraceExporterType};

pub fn tracer_layer(config: &ObservabilityConfig, resource: Resource) -> anyhow::Result<OpenTelemetryLayer<Registry, Tracer>> {
    let export_config = ExportConfig {
        endpoint: config.trace_exporter_endpoint.clone().unwrap(),
        ..Default::default()
    };

    let exporter = match &config.trace_exporter_type
    {
        TraceExporterType::OtelHttp => SpanExporterBuilder::Http(
            HttpExporterBuilder::default().with_export_config(export_config),
        ),
        TraceExporterType::OtelGrpc => SpanExporterBuilder::Tonic(
            TonicExporterBuilder::default().with_export_config(export_config),
        ),
        _ => {
            panic!("`OTEL_EXPORTER` not supported");
        }
    };

    global::set_text_map_propagator(TraceContextPropagator::new());

    let tracer = opentelemetry_otlp::new_pipeline()
        .tracing()
        .with_exporter(exporter)
        .with_trace_config(
            opentelemetry_sdk::trace::Config::default()
                .with_sampler(Sampler::TraceIdRatioBased(config.trace_exporter_sample_rate.unwrap_or(1.0)))
                .with_resource(resource),
        )
        .install_batch(Tokio)
        .expect("Failed to install `opentelemetry` tracer.");

    let telemetry = tracing_opentelemetry::layer().with_tracer(tracer);
    Ok(telemetry)
}
