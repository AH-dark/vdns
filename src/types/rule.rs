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
}

#[derive(Debug, Clone, Default, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ForwardUpstream {
    pub tag: Option<String>,
    pub addr: Option<String>,
    pub dial_addr: Option<String>,
    pub bootstrap: Option<String>,
    pub bootstrap_version: Option<i8>,
    pub socks5: Option<String>,
    pub idle_timeout: Option<u64>,
    pub enable_pipeline: bool,
    pub enable_http3: bool,
    pub max_conns: Option<u32>,
    pub insecure_skip_verify: bool,
    pub so_mark: u32,
    pub bind_to_device: String,
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct HttpServerEntry {
    pub path: String,
    pub exec: String,
}
