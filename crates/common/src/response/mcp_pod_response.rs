use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use super::PodPhase;
use crate::request::{EnvVar, ResourceRequirements};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpPodResponse {
    pub id: String,
    pub name: String,
    pub namespace: String,
    pub status: PodPhase,
    pub image: String,
    pub created_at: DateTime<Utc>,
    pub labels: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_vars: Option<Vec<EnvVar>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resources: Option<ResourceRequirements>,
}
