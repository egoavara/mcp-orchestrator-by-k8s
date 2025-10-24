use clap::Parser;
use figment::{
    providers::{Env, Format, Serialized, Yaml},
    Figment,
};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Parser)]
#[command(name = "mcp-orchestrator")]
#[command(about = "MCP Orchestrator - Kubernetes-based MCP server management", long_about = None)]
pub struct Cli {
    #[arg(short, long, value_name = "FILE", default_value = "./config.yaml", help = "Path to configuration file")]
    pub config: PathBuf,

    #[arg(long, env = "HOST", help = "Server bind address [env: HOST or MCP_SERVER_HOST]")]
    pub host: Option<String>,

    #[arg(long, env = "PORT", help = "Server bind port [env: PORT or MCP_SERVER_PORT]")]
    pub port: Option<u16>,

    #[arg(long, env = "LOG_LEVEL", help = "Log level: trace, debug, info, warn, error [env: LOG_LEVEL or MCP_SERVER_LOG_LEVEL]")]
    pub log_level: Option<String>,

    #[arg(long, env = "KUBE_NAMESPACE", help = "Default Kubernetes namespace for MCP servers [env: KUBE_NAMESPACE or MCP_KUBERNETES_NAMESPACE]")]
    pub kube_namespace: Option<String>,

    #[arg(long, env = "KUBE_CONTEXT", help = "Kubernetes context to use [env: KUBE_CONTEXT or MCP_KUBERNETES_CONTEXT]")]
    pub kube_context: Option<String>,
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
                "host": self.host,
                "port": self.port,
                "log_level": self.log_level,
            },
            "kubernetes": {
                "namespace": self.kube_namespace,
                "context": self.kube_context,
            }
        });
        
        remove_nulls(value)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,

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
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    #[serde(default)]
    pub server: ServerConfig,

    #[serde(default)]
    pub kubernetes: KubernetesConfig,
}

fn default_host() -> String {
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

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: default_host(),
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
        }
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            server: ServerConfig::default(),
            kubernetes: KubernetesConfig::default(),
        }
    }
}

impl AppConfig {
    pub fn load() -> Result<Self, figment::Error> {
        let cli = Cli::parse();
        
        let mut figment = Figment::new()
            .merge(Serialized::defaults(AppConfig::default()));

        if cli.config.exists() {
            figment = figment.merge(Yaml::file(&cli.config));
        }

        figment = figment
            .merge(Env::prefixed("MCP_").split("_"))
            .merge(Serialized::defaults(cli.to_figment_map()));

        figment.extract()
    }
}
