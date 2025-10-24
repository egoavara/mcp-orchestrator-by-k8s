use std::collections::BTreeMap;

use k8s_openapi::api::core::v1::Secret;
use kube::{
    api::{DeleteParams, ListParams, ObjectMeta, Patch, PatchParams, PostParams},
    Api, Client,
};

use super::label_query::{build_label_selector, LabelQuery};
use super::labels::{add_prefix_to_user_labels, LABEL_MANAGED_BY, LABEL_MANAGED_BY_VALUE};
use super::namespace_store::DeleteResult;
use crate::error::AppError;

const FINALIZER_NAME: &str = "mcp-orchestrator.egoavara.net/dependency-check";

pub struct SecretStore {
    client: Client,
    default_namespace: String,
}

impl SecretStore {
    pub fn new(client: Client, default_namespace: impl Into<String>) -> Self {
        Self {
            client,
            default_namespace: default_namespace.into(),
        }
    }

    fn api(&self, namespace: &str) -> Api<Secret> {
        Api::namespaced(self.client.clone(), namespace)
    }

    pub fn default_namespace(&self) -> &str {
        &self.default_namespace
    }

    fn resolve_namespace(&self, namespace: Option<&str>) -> String {
        namespace.unwrap_or(&self.default_namespace).to_string()
    }

    pub async fn create(
        &self,
        namespace: &str,
        name: &str,
        labels: BTreeMap<String, String>,
        data: BTreeMap<String, Vec<u8>>,
        secret_type: Option<String>,
    ) -> Result<Secret, AppError> {
        let mut final_labels = add_prefix_to_user_labels(labels);
        final_labels.insert(
            LABEL_MANAGED_BY.to_string(),
            LABEL_MANAGED_BY_VALUE.to_string(),
        );

        let string_data: BTreeMap<String, String> = data
            .into_iter()
            .map(|(k, v)| (k, String::from_utf8(v).unwrap_or_default()))
            .collect();

        let secret = Secret {
            metadata: ObjectMeta {
                name: Some(name.to_string()),
                namespace: Some(namespace.to_string()),
                labels: Some(final_labels),
                ..Default::default()
            },
            string_data: Some(string_data),
            type_: secret_type,
            ..Default::default()
        };

        self.api(namespace)
            .create(&PostParams::default(), &secret)
            .await
            .map_err(AppError::from)
    }

    pub async fn get(&self, namespace: &str, name: &str) -> Result<Option<Secret>, AppError> {
        match self.api(namespace).get(name).await {
            Ok(secret) => {
                if let Some(labels) = &secret.metadata.labels {
                    if labels.get(LABEL_MANAGED_BY) == Some(&LABEL_MANAGED_BY_VALUE.to_string()) {
                        return Ok(Some(secret));
                    }
                }
                Ok(None)
            }
            Err(kube::Error::Api(err)) if err.code == 404 => Ok(None),
            Err(e) => Err(AppError::from(e)),
        }
    }

    pub async fn list(
        &self,
        namespace: Option<&str>,
        queries: &[LabelQuery],
    ) -> Result<Vec<Secret>, AppError> {
        let mut all_queries = vec![LabelQuery::Equal {
            key: LABEL_MANAGED_BY.to_string(),
            value: LABEL_MANAGED_BY_VALUE.to_string(),
        }];
        
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
            Api::<Secret>::all(self.client.clone())
                .list(&lp)
                .await
                .map_err(AppError::from)?
        };

        Ok(list.items)
    }

    pub async fn update(
        &self,
        namespace: &str,
        name: &str,
        data: BTreeMap<String, Vec<u8>>,
        strategy: UpdateStrategy,
    ) -> Result<Secret, AppError> {
        let existing = self.get(namespace, name).await?;
        if existing.is_none() {
            return Err(AppError::NotFound(format!(
                "Secret {}/{} not found or not managed by mcp-orchestrator",
                namespace, name
            )));
        }

        let string_data: BTreeMap<String, String> = data
            .into_iter()
            .map(|(k, v)| (k, String::from_utf8(v).unwrap_or_default()))
            .collect();

        let final_data = match strategy {
            UpdateStrategy::Replace => string_data,
            UpdateStrategy::Merge | UpdateStrategy::Patch => {
                if let Some(secret) = existing {
                    let mut merged = secret
                        .data
                        .unwrap_or_default()
                        .into_iter()
                        .map(|(k, v)| (k, String::from_utf8(v.0).unwrap_or_default()))
                        .collect::<BTreeMap<_, _>>();

                    if matches!(strategy, UpdateStrategy::Patch) {
                        for (k, v) in string_data {
                            merged.insert(k, v);
                        }
                    } else {
                        merged.extend(string_data);
                    }
                    merged
                } else {
                    return Err(AppError::NotFound(format!(
                        "Secret {}/{} not found",
                        namespace, name
                    )));
                }
            }
        };

        let secret = Secret {
            metadata: ObjectMeta {
                name: Some(name.to_string()),
                namespace: Some(namespace.to_string()),
                ..Default::default()
            },
            string_data: Some(final_data),
            ..Default::default()
        };

        let patch_params = PatchParams::apply("mcp-orchestrator").force();
        self.api(namespace)
            .patch(name, &patch_params, &Patch::Apply(&secret))
            .await
            .map_err(AppError::from)
    }

    pub async fn delete(&self, namespace: &str, name: &str) -> Result<(), AppError> {
        if self.get(namespace, name).await?.is_none() {
            return Err(AppError::NotFound(format!(
                "Secret {}/{} not found or not managed by mcp-orchestrator",
                namespace, name
            )));
        }

        self.api(namespace)
            .delete(name, &DeleteParams::default())
            .await
            .map_err(AppError::from)?;
        Ok(())
    }

    pub async fn check_dependencies(&self, _namespace: &str, _name: &str) -> Result<Vec<String>, AppError> {
        Ok(vec![])
    }

    pub async fn add_finalizer(&self, namespace: &str, name: &str) -> Result<(), AppError> {
        let secret = self.api(namespace).get(name).await.map_err(AppError::from)?;
        
        let mut finalizers = secret.metadata.finalizers.unwrap_or_default();
        if finalizers.contains(&FINALIZER_NAME.to_string()) {
            return Ok(());
        }
        
        finalizers.push(FINALIZER_NAME.to_string());
        
        let patch = serde_json::json!({
            "metadata": {
                "finalizers": finalizers
            }
        });
        
        self.api(namespace)
            .patch(name, &PatchParams::default(), &Patch::Merge(&patch))
            .await
            .map_err(AppError::from)?;
        
        Ok(())
    }

    pub async fn remove_finalizer(&self, namespace: &str, name: &str) -> Result<(), AppError> {
        let secret = self.api(namespace).get(name).await.map_err(AppError::from)?;
        
        let mut finalizers = secret.metadata.finalizers.unwrap_or_default();
        finalizers.retain(|f| f != FINALIZER_NAME);
        
        let patch = serde_json::json!({
            "metadata": {
                "finalizers": finalizers
            }
        });
        
        self.api(namespace)
            .patch(name, &PatchParams::default(), &Patch::Merge(&patch))
            .await
            .map_err(AppError::from)?;
        
        Ok(())
    }

    pub fn has_finalizer(secret: &Secret) -> bool {
        secret
            .metadata
            .finalizers
            .as_ref()
            .map(|f| f.contains(&FINALIZER_NAME.to_string()))
            .unwrap_or(false)
    }

    pub fn is_being_deleted(secret: &Secret) -> bool {
        secret.metadata.deletion_timestamp.is_some()
    }

    pub async fn delete_with_lease(
        &self,
        namespace: &str,
        name: &str,
        pod_name: &str,
        lease_namespace: &str,
    ) -> Result<DeleteResult, AppError> {
        use super::lease_manager::{LeaseManager, resource_lock_name};

        let lock_name = resource_lock_name("secret", namespace, name);
        let lease_mgr = LeaseManager::new(
            self.client.clone(),
            lease_namespace,
            &lock_name,
            pod_name,
        );

        if !lease_mgr.try_acquire().await? {
            return Ok(DeleteResult::Locked);
        }

        let secret = match self.api(namespace).get(name).await {
            Ok(s) => s,
            Err(kube::Error::Api(err)) if err.code == 404 => {
                lease_mgr.release().await?;
                return Err(AppError::NotFound(format!(
                    "Secret {}/{} not found",
                    namespace, name
                )));
            }
            Err(e) => {
                lease_mgr.release().await?;
                return Err(AppError::from(e));
            }
        };

        if let Some(labels) = &secret.metadata.labels {
            if labels.get(LABEL_MANAGED_BY) != Some(&LABEL_MANAGED_BY_VALUE.to_string()) {
                lease_mgr.release().await?;
                return Err(AppError::NotFound(format!(
                    "Secret {}/{} is not managed by mcp-orchestrator",
                    namespace, name
                )));
            }
        } else {
            lease_mgr.release().await?;
            return Err(AppError::NotFound(format!(
                "Secret {}/{} has no labels",
                namespace, name
            )));
        }

        if Self::is_being_deleted(&secret) {
            if !Self::has_finalizer(&secret) {
                lease_mgr.release().await?;
                return Ok(DeleteResult::Deleting);
            }

            let deps = self.check_dependencies(namespace, name).await?;
            if !deps.is_empty() {
                lease_mgr.release().await?;
                return Ok(DeleteResult::HasDependencies(deps));
            }

            self.remove_finalizer(namespace, name).await?;
            lease_mgr.release().await?;
            return Ok(DeleteResult::FinalizerRemoved);
        }

        let deps = self.check_dependencies(namespace, name).await?;
        if !deps.is_empty() {
            self.add_finalizer(namespace, name).await?;
        }

        self.api(namespace)
            .delete(name, &DeleteParams::default())
            .await
            .map_err(AppError::from)?;

        lease_mgr.release().await?;

        if deps.is_empty() {
            Ok(DeleteResult::Deleted)
        } else {
            Ok(DeleteResult::DeletionStarted(deps))
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum UpdateStrategy {
    Replace,
    Merge,
    Patch,
}
