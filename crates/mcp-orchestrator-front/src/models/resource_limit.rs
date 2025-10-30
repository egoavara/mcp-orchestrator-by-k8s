use proto_web::{ResourceLimit as ProtoResourceLimit, ResourceLimitResponse};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct ResourceLimit {
    pub name: String,
    pub description: String,
    pub limits: ResourceLimitSpec,
    pub labels: HashMap<String, String>,
    pub created_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ResourceLimitSpec {
    pub cpu: String,
    pub memory: String,
    pub cpu_limit: Option<String>,
    pub memory_limit: Option<String>,
    pub ephemeral_storage: Option<String>,
    pub node_selector: Option<String>,
    pub node_affinity: Option<String>,
}

impl From<ResourceLimitResponse> for ResourceLimit {
    fn from(response: ResourceLimitResponse) -> Self {
        let limits = response.limits.unwrap_or_else(|| ProtoResourceLimit {
            cpu: String::new(),
            memory: String::new(),
            cpu_limit: None,
            memory_limit: None,
            ephemeral_storage: None,
            volumes: HashMap::new(),
            node_selector: HashMap::new(),
            node_affinity: None,
        });

        let node_selector_yaml = if !limits.node_selector.is_empty() {
            serde_yaml::to_string(&limits.node_selector).ok()
        } else {
            None
        };

        let node_affinity_yaml = limits.node_affinity.as_ref().and_then(|json_str| {
            serde_json::from_str::<serde_json::Value>(json_str)
                .ok()
                .and_then(|v| serde_yaml::to_string(&v).ok())
        });

        Self {
            name: response.name,
            description: response.description,
            limits: ResourceLimitSpec {
                cpu: limits.cpu,
                memory: limits.memory,
                cpu_limit: limits.cpu_limit,
                memory_limit: limits.memory_limit,
                ephemeral_storage: limits.ephemeral_storage,
                node_selector: node_selector_yaml,
                node_affinity: node_affinity_yaml,
            },
            labels: response.labels,
            created_at: response.created_at,
            deleted_at: response.deleted_at,
        }
    }
}
