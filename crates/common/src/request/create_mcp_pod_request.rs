use super::{EnvVar, ResourceRequirements};
use serde::{Deserialize, Serialize};
use serde_with::formats::PreferOne;
use serde_with::{serde_as, OneOrMany};

#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateMcpPodRequest {
    pub image: String,

    #[serde_as(as = "Option<OneOrMany<_, PreferOne>>")]
    pub command: Option<Vec<String>>,

    pub args: Option<Vec<String>>,

    #[serde(default)]
    pub env_vars: Vec<EnvVar>,

    #[serde(default)]
    pub resources: Option<ResourceRequirements>,
}
