
use kube::Client;

use crate::{
    error::AppError,
    storage::{McpTemplateStore, NamespaceStore, ResourceLimitStore, SecretStore},
};

#[derive(Clone)]
pub struct KubeStore {
    client: Client,
    default_namespace: String,
}

impl KubeStore {
    pub fn new(client: Client, default_namespace: impl Into<String>) -> Self {
        Self {
            client,
            default_namespace: default_namespace.into(),
        }
    }

    pub fn client(&self) -> &Client {
        &self.client
    }

    pub fn default_namespace(&self) -> &str {
        &self.default_namespace
    }

    pub fn namespaces(&self) -> NamespaceStore {
        NamespaceStore::new(self.client.clone(), self.default_namespace.clone())
    }

    pub fn secrets(&self, namespace: Option<String>) -> SecretStore {
        let ns = namespace.unwrap_or_else(|| self.default_namespace.clone());
        SecretStore::new(self.client.clone(), ns)
    }

    pub fn mcp_templates(&self, namespace: Option<String>) -> McpTemplateStore {
        let ns = namespace.unwrap_or_else(|| self.default_namespace.clone());
        McpTemplateStore::new(self.client.clone(), ns)
    }

    pub fn resource_limits(&self, namespace: Option<String>) -> ResourceLimitStore {
        let ns = namespace.unwrap_or_else(|| self.default_namespace.clone());
        ResourceLimitStore::new(self.client.clone(), ns)
    }

    // pub fn mcp_servers(&self, namespace: Option<String>) -> McpServerStore {
    //     let ns = namespace.unwrap_or_else(|| self.default_namespace.clone());
    //     McpServerStore::new(self.client.clone(), ns)
    // }

    pub async fn ensure_default_namespace(&self) -> Result<(), AppError> {
        self.namespaces().ensure_default_namespace().await?;
        Ok(())
    }
}
