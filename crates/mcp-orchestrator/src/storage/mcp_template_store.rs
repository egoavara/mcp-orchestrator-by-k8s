use std::collections::BTreeMap;

use k8s_openapi::api::core::v1::ConfigMap;
use kube::{
    api::{DeleteParams, ListParams, ObjectMeta, PostParams},
    Api, Client,
};
use serde::{Deserialize, Serialize};

use super::label_query::{build_label_selector, LabelQuery};
use super::labels::{
    add_prefix_to_user_labels, LABEL_MANAGED_BY, LABEL_MANAGED_BY_VALUE, LABEL_TYPE_OF,
    TYPE_MCP_TEMPLATE,
};
use crate::error::AppError;

pub struct McpTemplateStore {
    client: Client,
    default_namespace: String,
}

impl McpTemplateStore {
    pub fn new(client: Client, default_namespace: impl Into<String>) -> Self {
        Self {
            client,
            default_namespace: default_namespace.into(),
        }
    }

    fn api(&self, namespace: &str) -> Api<ConfigMap> {
        Api::namespaced(self.client.clone(), namespace)
    }

    pub fn default_namespace(&self) -> &str {
        &self.default_namespace
    }

    fn resolve_namespace(&self, namespace: Option<&str>) -> String {
        namespace.unwrap_or(&self.default_namespace).to_string()
    }

    pub async fn create<T: Serialize>(
        &self,
        namespace: &str,
        id: &str,
        labels: BTreeMap<String, String>,
        data: &T,
    ) -> Result<ConfigMap, AppError> {
        let mut final_labels = add_prefix_to_user_labels(labels);
        final_labels.insert(
            LABEL_MANAGED_BY.to_string(),
            LABEL_MANAGED_BY_VALUE.to_string(),
        );
        final_labels.insert(LABEL_TYPE_OF.to_string(), TYPE_MCP_TEMPLATE.to_string());

        let json_data = serde_json::to_string(data).map_err(AppError::SerializationError)?;

        let mut data_map = BTreeMap::new();
        data_map.insert("data.json".to_string(), json_data);

        let configmap = ConfigMap {
            metadata: ObjectMeta {
                name: Some(format!("mcp-template-{}", id)),
                namespace: Some(namespace.to_string()),
                labels: Some(final_labels),
                ..Default::default()
            },
            data: Some(data_map),
            ..Default::default()
        };

        self.api(namespace)
            .create(&PostParams::default(), &configmap)
            .await
            .map_err(AppError::from)
    }

    pub async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        namespace: &str,
        id: &str,
    ) -> Result<Option<T>, AppError> {
        let name = format!("mcp-template-{}", id);
        match self.api(namespace).get(&name).await {
            Ok(cm) => {
                if let Some(labels) = &cm.metadata.labels {
                    if labels.get(LABEL_MANAGED_BY) != Some(&LABEL_MANAGED_BY_VALUE.to_string())
                        || labels.get(LABEL_TYPE_OF) != Some(&TYPE_MCP_TEMPLATE.to_string())
                    {
                        return Ok(None);
                    }
                } else {
                    return Ok(None);
                }

                let data = cm
                    .data
                    .and_then(|mut d| d.remove("data.json"))
                    .ok_or_else(|| {
                        AppError::NotFound(format!("ConfigMap {} has no data", name))
                    })?;

                let parsed: T = serde_json::from_str(&data).map_err(AppError::SerializationError)?;
                Ok(Some(parsed))
            }
            Err(kube::Error::Api(err)) if err.code == 404 => Ok(None),
            Err(e) => Err(AppError::from(e)),
        }
    }

    pub async fn list<T: for<'de> Deserialize<'de>>(
        &self,
        namespace: Option<&str>,
        queries: &[LabelQuery],
    ) -> Result<Vec<T>, AppError> {
        let mut all_queries = vec![
            LabelQuery::Equal {
                key: LABEL_MANAGED_BY.to_string(),
                value: LABEL_MANAGED_BY_VALUE.to_string(),
            },
            LabelQuery::Equal {
                key: LABEL_TYPE_OF.to_string(),
                value: TYPE_MCP_TEMPLATE.to_string(),
            },
        ];
        
        let prefixed_queries: Vec<LabelQuery> = queries
            .iter()
            .map(|q| q.clone().with_prefix())
            .collect();
        all_queries.extend(prefixed_queries);

        let selector = build_label_selector(&all_queries);
        let lp = ListParams::default().labels(&selector);

        let list = if let Some(ns) = namespace {
            self.api(ns).list(&lp).await.map_err(AppError::from)?
        } else {
            Api::<ConfigMap>::all(self.client.clone())
                .list(&lp)
                .await
                .map_err(AppError::from)?
        };

        let mut results = Vec::new();
        for cm in list.items {
            if let Some(data) = cm.data.and_then(|mut d| d.remove("data.json")) {
                let parsed: T = serde_json::from_str(&data).map_err(AppError::SerializationError)?;
                results.push(parsed);
            }
        }

        Ok(results)
    }

    pub async fn delete(&self, namespace: &str, id: &str) -> Result<(), AppError> {
        if self.get::<serde_json::Value>(namespace, id).await?.is_none() {
            return Err(AppError::NotFound(format!(
                "MCP Template {}/{} not found or not managed by mcp-orchestrator",
                namespace, id
            )));
        }

        let name = format!("mcp-template-{}", id);
        self.api(namespace)
            .delete(&name, &DeleteParams::default())
            .await
            .map_err(AppError::from)?;
        Ok(())
    }
}
