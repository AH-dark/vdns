#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct ObservabilityConfig {
    #[serde(default)]
    pub trace_exporter_type: TraceExporterType,
    #[serde(default)]
    pub trace_exporter_endpoint: Option<String>,
    #[serde(default)]
    pub trace_exporter_sample_rate: Option<f64>,

    #[serde(default)]
    pub metrics_exporter_type: MetricsExporterType,
    #[serde(default)]
    pub metrics_prometheus_listen_addr: Option<String>,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TraceExporterType {
    OtelGrpc,
    OtelHttp,
    #[default]
    #[serde(other)]
    None,
}

#[derive(Debug, Clone, Default, serde::Serialize, serde::Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum MetricsExporterType {
    Prometheus,
    #[default]
    #[serde(other)]
    None,
}
