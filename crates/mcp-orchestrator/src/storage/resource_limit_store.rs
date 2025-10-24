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
    TYPE_RESOURCE_LIMIT,
};
use crate::error::AppError;

pub struct ResourceLimitStore {
    client: Client,
}

impl ResourceLimitStore {
    pub fn new(client: Client) -> Self {
        Self { client }
    }

    pub async fn create<T: Serialize>(
        &self,
        name: &str,
        labels: BTreeMap<String, String>,
        data: &T,
    ) -> Result<ConfigMap, AppError> {
        let mut final_labels = add_prefix_to_user_labels(labels);
        final_labels.insert(
            LABEL_MANAGED_BY.to_string(),
            LABEL_MANAGED_BY_VALUE.to_string(),
        );
        final_labels.insert(
            LABEL_TYPE_OF.to_string(),
            TYPE_RESOURCE_LIMIT.to_string(),
        );

        let json_data = serde_json::to_string(data).map_err(AppError::SerializationError)?;

        let mut data_map = BTreeMap::new();
        data_map.insert("data.json".to_string(), json_data);

        let configmap = ConfigMap {
            metadata: ObjectMeta {
                name: Some(format!("resource-limit-{}", name)),
                labels: Some(final_labels),
                ..Default::default()
            },
            data: Some(data_map),
            ..Default::default()
        };

        Api::<ConfigMap>::all(self.client.clone())
            .create(&PostParams::default(), &configmap)
            .await
            .map_err(AppError::from)
    }

    pub async fn get<T: for<'de> Deserialize<'de>>(
        &self,
        name: &str,
    ) -> Result<Option<T>, AppError> {
        let cm_name = format!("resource-limit-{}", name);
        match Api::<ConfigMap>::all(self.client.clone()).get(&cm_name).await {
            Ok(cm) => {
                if let Some(labels) = &cm.metadata.labels {
                    if labels.get(LABEL_MANAGED_BY) != Some(&LABEL_MANAGED_BY_VALUE.to_string())
                        || labels.get(LABEL_TYPE_OF) != Some(&TYPE_RESOURCE_LIMIT.to_string())
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
                        AppError::NotFound(format!("ConfigMap {} has no data", cm_name))
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
        queries: &[LabelQuery],
    ) -> Result<Vec<T>, AppError> {
        let mut all_queries = vec![
            LabelQuery::Equal {
                key: LABEL_MANAGED_BY.to_string(),
                value: LABEL_MANAGED_BY_VALUE.to_string(),
            },
            LabelQuery::Equal {
                key: LABEL_TYPE_OF.to_string(),
                value: TYPE_RESOURCE_LIMIT.to_string(),
            },
        ];
        
        let prefixed_queries: Vec<LabelQuery> = queries
            .iter()
            .map(|q| q.clone().with_prefix())
            .collect();
        all_queries.extend(prefixed_queries);

        let selector = build_label_selector(&all_queries);
        let lp = ListParams::default().labels(&selector);

        let list = Api::<ConfigMap>::all(self.client.clone())
            .list(&lp)
            .await
            .map_err(AppError::from)?;

        let mut results = Vec::new();
        for cm in list.items {
            if let Some(data) = cm.data.and_then(|mut d| d.remove("data.json")) {
                let parsed: T = serde_json::from_str(&data).map_err(AppError::SerializationError)?;
                results.push(parsed);
            }
        }

        Ok(results)
    }

    pub async fn delete(&self, name: &str) -> Result<(), AppError> {
        if self.get::<serde_json::Value>(name).await?.is_none() {
            return Err(AppError::NotFound(format!(
                "ResourceLimit {} not found or not managed by mcp-orchestrator",
                name
            )));
        }

        let cm_name = format!("resource-limit-{}", name);
        Api::<ConfigMap>::all(self.client.clone())
            .delete(&cm_name, &DeleteParams::default())
            .await
            .map_err(AppError::from)?;
        Ok(())
    }
}
