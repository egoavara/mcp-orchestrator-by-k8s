use proto_web::{CreateMcpTemplateRequest, McpTemplateResponse};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Template {
    pub namespace: String,
    pub name: String,
    pub labels: HashMap<String, String>,
    pub image: String,
    pub command: Vec<String>,
    pub args: Vec<String>,
    pub envs: HashMap<String, String>,
    pub secret_envs: Vec<String>,
    pub resource_limit_name: Option<String>,
    pub created_at: String,
    pub deleted_at: Option<String>,
}

impl From<McpTemplateResponse> for Template {
    fn from(response: McpTemplateResponse) -> Self {
        Self {
            namespace: response.namespace,
            name: response.name,
            labels: response.labels,
            image: response.image,
            command: response.command,
            args: response.args,
            envs: response.envs,
            secret_envs: response.secret_envs,
            resource_limit_name: if response.resource_limit_name.is_empty() {
                None
            } else {
                Some(response.resource_limit_name)
            },
            created_at: response.created_at,
            deleted_at: response.deleted_at,
        }
    }
}

#[derive(Clone, Debug, Default, PartialEq)]
pub struct TemplateFormData {
    pub namespace: String,
    pub name: String,
    pub image: String,
    pub command: Vec<String>,
    pub args: Vec<String>,
    pub envs: HashMap<String, String>,
    pub secret_envs: Vec<String>,
    pub resource_limit_name: Option<String>,
    pub labels: HashMap<String, String>,
}

impl TemplateFormData {
    pub fn to_create_request(self) -> CreateMcpTemplateRequest {
        let filtered_secret_envs: Vec<String> = self
            .secret_envs
            .into_iter()
            .filter(|s| !s.is_empty())
            .collect();

        web_sys::console::log_1(
            &format!("Secret envs after filter: {:?}", filtered_secret_envs).into(),
        );

        CreateMcpTemplateRequest {
            namespace: Some(self.namespace),
            name: self.name,
            labels: self.labels,
            image: self.image,
            command: self.command,
            args: self.args,
            envs: self.envs,
            secret_envs: filtered_secret_envs,
            resource_limit_name: self.resource_limit_name.unwrap_or_default(),
            volume_mounts: Vec::new(),
            secret_mounts: Vec::new(),
        }
    }
}
