use std::path::PathBuf;

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub plugins: Vec<Plugin>,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub struct Plugin {
    pub tag: String,
    #[serde(flatten)]
    pub plugin_type: PluginType,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum PluginType {
    Cache {
        size: Option<usize>,
        lazy_cache_ttl: Option<u64>,
        dump_file: Option<PathBuf>,
        dump_interval: Option<u64>,
    },
    Hosts {
        entries: Vec<String>,
        files: Vec<PathBuf>,
    },
    Forward {
        concurrent: Option<usize>,
        idle_timeout: Option<u64>,
        upstreams: Vec<ForwardUpstream>,
    },
    Sequence {
        matches: Vec<String>,
        exec: String,
    },
    UdpServer {
        entry: String,
        listen: String,
    },
    TcpServer {
        entry: String,
        listen: String,
        cert: Option<PathBuf>,
        key: Option<PathBuf>,
        idle_timeout: Option<u64>,
    },
    QuicServer {
        entry: String,
        listen: String,
        cert: PathBuf,
        key: PathBuf,
        idle_timeout: Option<u64>,
    },
    HttpServer {
        entries: Vec<HttpServerEntry>,
        src_ip_headers: Vec<String>, // X-Forwarded-For, X-Real-IP
        listen: String,
        cert: Option<PathBuf>,
        key: Option<PathBuf>,
        idle_timeout: Option<u64>,
    },
    DomainSet {
        exps: Vec<String>,
        files: Vec<PathBuf>,
    },
    IpSet {
        ips: Vec<String>,
        files: Vec<PathBuf>,
    },
}

impl PluginType {
    pub fn is_server(&self) -> bool {
        match self {
            PluginType::UdpServer { .. } => true,
            PluginType::TcpServer { .. } => true,
            PluginType::QuicServer { .. } => true,
            PluginType::HttpServer { .. } => true,
            _ => false,
        }
    }

    pub fn is_executable(&self) -> bool {
        match self {
            PluginType::Sequence { .. } => true,
            PluginType::Cache { .. } => true,
            PluginType::Hosts { .. } => true,
            PluginType::Forward { .. } => true,

            _ => false,
        }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ForwardUpstream {
    pub tag: String,
    pub addr: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct HttpServerEntry {
    pub path: String,
    pub exec: String,
}
