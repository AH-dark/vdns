use std::net::SocketAddr;

use anyhow::Context;
use metrics_exporter_prometheus::{PrometheusBuilder, PrometheusRecorder};

use crate::observability::config::{MetricsExporterType, ObservabilityConfig};

pub fn metrics_layer(cfg: &ObservabilityConfig) -> anyhow::Result<PrometheusRecorder> {
    let socket = cfg.metrics_prometheus_listen_addr.clone().unwrap_or("0.0.0.0:9090".into())
        .parse::<SocketAddr>()?;
    let builder = PrometheusBuilder::new().with_http_listener(socket);

    match cfg.metrics_exporter_type {
        MetricsExporterType::Prometheus => {
            let (recorder, exporter_future) = builder.build().context("Failed to install recorder")?;
            tokio::spawn(exporter_future);
            Ok(recorder)
        }
        MetricsExporterType::None => unreachable!()
    }
}