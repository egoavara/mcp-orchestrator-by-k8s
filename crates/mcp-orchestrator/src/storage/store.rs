use kube::Client;

use super::{
    McpServerStore, McpTemplateStore, NamespaceStore, ResourceLimitStore, SecretStore,
};
use crate::error::AppError;

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
        NamespaceStore::new(self.client.clone())
    }

    pub fn secrets(&self, namespace: Option<String>) -> SecretStore {
        let ns = namespace.unwrap_or_else(|| self.default_namespace.clone());
        SecretStore::new(self.client.clone(), ns)
    }

    pub fn mcp_templates(&self, namespace: Option<String>) -> McpTemplateStore {
        let ns = namespace.unwrap_or_else(|| self.default_namespace.clone());
        McpTemplateStore::new(self.client.clone(), ns)
    }

    pub fn resource_limits(&self) -> ResourceLimitStore {
        ResourceLimitStore::new(self.client.clone())
    }

    pub fn mcp_servers(&self, namespace: Option<String>) -> McpServerStore {
        let ns = namespace.unwrap_or_else(|| self.default_namespace.clone());
        McpServerStore::new(self.client.clone(), ns)
    }

    pub async fn ensure_default_namespace(&self) -> Result<(), AppError> {
        self.namespaces()
            .ensure_default_namespace(&self.default_namespace)
            .await?;
        Ok(())
    }
}
