use serde::{Deserialize, Serialize};

pub mod create_mcp_pod_request;

pub use create_mcp_pod_request::CreateMcpPodRequest;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnvVar {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceRequirements {
    pub cpu: String,
    pub memory: String,
}

impl Default for ResourceRequirements {
    fn default() -> Self {
        Self {
            cpu: "500m".to_string(),
            memory: "256Mi".to_string(),
        }
    }
}
