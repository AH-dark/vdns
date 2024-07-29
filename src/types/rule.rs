use std::path::PathBuf;

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct Config {
    pub data_providers: Vec<DataProvider>,
    pub plugins: Vec<Plugin>,
    pub servers: Vec<Server>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub struct DataProvider {
    tag: String,
    file: PathBuf,
    auto_reload: bool,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub struct Plugin {
    tag: String,
    #[serde(flatten)]
    plugin_type: PluginType,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub enum PluginType {
    Cache {
        size: Option<usize>,
        lazy_cache_ttl: Option<u64>,
        dump_file: Option<PathBuf>,
        dump_interval: Option<u64>,
    },
    Hosts {
        entries: Vec<(String, String)>,
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
        cert: Option<PathBuf>,
        key: Option<PathBuf>,
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

#[derive(Debug, Clone, Default, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct ForwardUpstream {
    tag: Option<String>,
    addr: Option<String>,
    dial_addr: Option<String>,
    bootstrap: Option<String>,
    bootstrap_version: Option<i8>,
    socks5: Option<String>,
    idle_timeout: Option<u64>,
    enable_pipeline: bool,
    enable_http3: bool,
    max_conns: Option<u32>,
    insecure_skip_verify: bool,
    so_mark: u32,
    bind_to_device: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub struct HttpServerEntry {
    path: String,
    exec: String,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "type")]
pub struct Server {
    exec: String,
    timeout: Option<u64>,
    listeners: Vec<Listener>,
}

#[derive(Debug, Clone, serde::Deserialize)]
#[serde(rename_all = "snake_case", tag = "protocol")]
pub enum Listener {
    Udp {
        addr: String,
    },
    Tcp {
        addr: String,
    },
    Http {
        addr: String,
        url_path: String,
        get_user_ip_from_header: Option<String>,
    },
    Tls {
        addr: String,
        cert: PathBuf,
        key: PathBuf,
    },
}
