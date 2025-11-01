use clap::Parser;
use figment::{
    Figment,
    providers::{Env, Format, Serialized, Yaml},
};
use oidc_auth::OpenIdConfig;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, time::Duration};

#[derive(Debug, Clone, Parser)]
#[command(name = "mcp-orchestrator")]
#[command(about = "MCP Orchestrator - Kubernetes-based MCP server management", long_about = None)]
pub struct Cli {
    #[arg(
        short,
        long,
        value_name = "FILE",
        default_value = "./config.yaml",
        help = "Path to configuration file"
    )]
    pub config: PathBuf,

    #[arg(long, env = "URL", help = "Server bind URL")]
    pub url: Option<String>,

    #[arg(
        long,
        env = "PORT",
        help = "Server bind port [env: PORT or MCP_SERVER_PORT]"
    )]
    pub port: Option<u16>,

    #[arg(
        short,
        long,
        env = "LOG_LEVEL",
        help = "Log level: trace, debug, info, warn, error [env: LOG_LEVEL or MCP_SERVER_LOG_LEVEL]"
    )]
    pub log_level: Option<String>,

    #[arg(
        short = 'n',
        long,
        env = "KUBE_NAMESPACE",
        help = "Default Kubernetes namespace for MCP servers [env: KUBE_NAMESPACE or MCP_KUBERNETES_NAMESPACE]"
    )]
    pub kube_namespace: Option<String>,

    #[arg(
        short,
        long,
        env = "KUBE_CONTEXT",
        help = "Kubernetes context to use [env: KUBE_CONTEXT or MCP_KUBERNETES_CONTEXT]"
    )]
    pub kube_context: Option<String>,

    #[arg(
        long,
        env = "POD_NAME",
        help = "Name of the pod running this instance [env: POD_NAME or MCP_POD_NAME]"
    )]
    pub pod_name: Option<String>,

    #[arg(
        long,
        env = "OIDC_DISCOVERY",
        help = "OIDC Discovery URL [env: OIDC_DISCOVERY]"
    )]
    pub oidc_discovery: Option<String>,
}

fn remove_nulls(value: serde_json::Value) -> serde_json::Value {
    use serde_json::{Map, Value};

    match value {
        Value::Object(map) => {
            let filtered: Map<String, Value> = map
                .into_iter()
                .filter_map(|(k, v)| {
                    let cleaned = remove_nulls(v);
                    if cleaned.is_null() {
                        None
                    } else if let Value::Object(ref obj) = cleaned {
                        if obj.is_empty() {
                            None
                        } else {
                            Some((k, cleaned))
                        }
                    } else {
                        Some((k, cleaned))
                    }
                })
                .collect();
            Value::Object(filtered)
        }
        other => other,
    }
}

impl Cli {
    fn to_figment_map(&self) -> serde_json::Value {
        use serde_json::json;

        let value = json!({
            "server": {
                "port": self.port,
                "log_level": self.log_level,
                "url": self.url,
            },
            "kubernetes": {
                "namespace": self.kube_namespace,
                "context": self.kube_context,
                "pod": {
                    "name": self.pod_name,
                }
            },
            "auth": {
                "openid": {
                    "discovery": {
                        "discovery_url": self.oidc_discovery,
                    }
                }
            }
        });

        remove_nulls(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_url")]
    pub url: String,

    #[serde(default = "default_ip_addr")]
    pub ip_addr: String,

    #[serde(default = "default_port")]
    pub port: u16,

    #[serde(default = "default_log_level")]
    pub log_level: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KubernetesConfig {
    #[serde(default = "default_kube_namespace")]
    pub namespace: String,

    #[serde(default)]
    pub context: Option<String>,

    #[serde(default)]
    pub pod: Option<PodConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PodConfig {
    pub name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpConfig {
    #[serde(with = "humantime_serde", default = "default_keep_alive")]
    pub keep_alive: Option<Duration>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthConfig {
    #[serde(default = "default_audience")]
    pub audience: String,

    #[serde(default = "default_allow_expireless_token")]
    pub allow_expireless_token: bool,

    #[serde(default)]
    pub openid: Option<OpenIdConfig>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AppConfig {
    #[serde(default)]
    pub server: ServerConfig,

    #[serde(default)]
    pub kubernetes: KubernetesConfig,

    #[serde(default)]
    pub mcp: McpConfig,

    #[serde(default)]
    pub auth: AuthConfig,
}

fn default_keep_alive() -> Option<Duration> {
    Some(std::time::Duration::from_secs(30))
}

fn default_url() -> String {
    "http://localhost:3000".to_string()
}

fn default_ip_addr() -> String {
    "0.0.0.0".to_string()
}

fn default_port() -> u16 {
    3000
}

fn default_log_level() -> String {
    "info".to_string()
}

fn default_kube_namespace() -> String {
    "mcp-servers".to_string()
}

fn default_audience() -> String {
    "mcp-orchestrator".to_string()
}

fn default_allow_expireless_token() -> bool {
    false
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            url: default_url(),
            ip_addr: default_ip_addr(),
            port: default_port(),
            log_level: default_log_level(),
        }
    }
}

impl Default for KubernetesConfig {
    fn default() -> Self {
        Self {
            namespace: default_kube_namespace(),
            context: None,
            pod: None,
        }
    }
}

impl Default for McpConfig {
    fn default() -> Self {
        Self {
            keep_alive: default_keep_alive(),
        }
    }
}

impl Default for AuthConfig {
    fn default() -> Self {
        Self {
            audience: default_audience(),
            allow_expireless_token: default_allow_expireless_token(),
            openid: None,
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, figment::Error> {
        if let Some(err) = dotenvy::dotenv().err() {
            tracing::warn!("Failed to load .env file, ignore .env: {}", err);
        }

        let cli = Cli::parse();

        let mut figment = Figment::new().merge(Serialized::defaults(AppConfig::default()));
        if cli.config.exists() {
            figment = figment.merge(Yaml::file(&cli.config));
        }

        figment = figment
            .merge(Env::prefixed("MCP_").split("_"))
            .merge(Serialized::defaults(cli.to_figment_map()));

        figment.extract()
    }
}
