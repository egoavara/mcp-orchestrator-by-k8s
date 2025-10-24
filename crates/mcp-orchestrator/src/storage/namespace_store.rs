use std::collections::BTreeMap;

use k8s_openapi::api::core::v1::Namespace;
use kube::{
    api::{DeleteParams, ListParams, ObjectMeta, Patch, PatchParams, PostParams},
    Api, Client,
};
use serde_json;

use super::label_query::{build_label_selector, LabelQuery};
use super::labels::{add_prefix_to_user_labels, LABEL_MANAGED_BY, LABEL_MANAGED_BY_VALUE};
use crate::error::AppError;

const FINALIZER_NAME: &str = "mcp-orchestrator.egoavara.net/dependency-check";

pub struct NamespaceStore {
    api: Api<Namespace>,
    client: Client,
}

impl NamespaceStore {
    pub fn new(client: Client) -> Self {
        Self {
            api: Api::all(client.clone()),
            client,
        }
    }

    pub async fn create(
        &self,
        name: &str,
        labels: BTreeMap<String, String>,
        annotations: BTreeMap<String, String>,
    ) -> Result<Namespace, AppError> {
        match self.api.get(name).await {
            Ok(_) => {
                return Err(AppError::Internal(format!(
                    "Namespace {} already exists",
                    name
                )));
            }
            Err(kube::Error::Api(err)) if err.code == 404 => {}
            Err(e) => return Err(e.into()),
        }

        let mut final_labels = add_prefix_to_user_labels(labels);
        final_labels.insert(
            LABEL_MANAGED_BY.to_string(),
            LABEL_MANAGED_BY_VALUE.to_string(),
        );

        let namespace = Namespace {
            metadata: ObjectMeta {
                name: Some(name.to_string()),
                labels: Some(final_labels),
                annotations: if annotations.is_empty() {
                    None
                } else {
                    Some(annotations)
                },
                ..Default::default()
            },
            ..Default::default()
        };

        self.api
            .create(&PostParams::default(), &namespace)
            .await
            .map_err(AppError::from)
    }

    pub async fn get(&self, name: &str) -> Result<Option<Namespace>, AppError> {
        match self.api.get(name).await {
            Ok(ns) => {
                if let Some(labels) = &ns.metadata.labels {
                    if labels.get(LABEL_MANAGED_BY) == Some(&LABEL_MANAGED_BY_VALUE.to_string()) {
                        return Ok(Some(ns));
                    }
                }
                Ok(None)
            }
            Err(kube::Error::Api(err)) if err.code == 404 => Ok(None),
            Err(e) => Err(AppError::from(e)),
        }
    }

    pub async fn get_raw(&self, name: &str) -> Result<Option<Namespace>, AppError> {
        match self.api.get(name).await {
            Ok(ns) => Ok(Some(ns)),
            Err(kube::Error::Api(err)) if err.code == 404 => Ok(None),
            Err(e) => Err(AppError::from(e)),
        }
    }

    pub fn is_managed(ns: &Namespace) -> bool {
        ns.metadata
            .labels
            .as_ref()
            .and_then(|labels| labels.get(LABEL_MANAGED_BY))
            .map(|v| v == LABEL_MANAGED_BY_VALUE)
            .unwrap_or(false)
    }

    pub async fn list(&self, queries: &[LabelQuery]) -> Result<Vec<Namespace>, AppError> {
        let mut all_queries = vec![LabelQuery::Equal {
            key: LABEL_MANAGED_BY.to_string(),
            value: LABEL_MANAGED_BY_VALUE.to_string(),
        }];

        let prefixed_queries: Vec<LabelQuery> =
            queries.iter().map(|q| q.clone().with_prefix()).collect();
        all_queries.extend(prefixed_queries);

        let selector = build_label_selector(&all_queries);
        let lp = ListParams::default().labels(&selector);
        let list = self.api.list(&lp).await.map_err(AppError::from)?;
        Ok(list.items)
    }

    pub async fn delete(&self, name: &str) -> Result<(), AppError> {
        match self.api.get(name).await {
            Ok(ns) => {
                let labels = ns.metadata.labels.as_ref().ok_or_else(|| {
                    AppError::NotFound(format!(
                        "Namespace {} has no labels, cannot verify ownership",
                        name
                    ))
                })?;

                if labels.get(LABEL_MANAGED_BY) != Some(&LABEL_MANAGED_BY_VALUE.to_string()) {
                    return Err(AppError::NotFound(format!(
                        "Namespace {} is not managed by mcp-orchestrator ({}={} not found)",
                        name, LABEL_MANAGED_BY, LABEL_MANAGED_BY_VALUE
                    )));
                }
            }
            Err(kube::Error::Api(err)) if err.code == 404 => {
                return Err(AppError::NotFound(format!("Namespace {} not found", name)));
            }
            Err(e) => return Err(AppError::from(e)),
        }

        self.api
            .delete(name, &DeleteParams::default())
            .await
            .map_err(AppError::from)?;
        Ok(())
    }

    pub async fn check_dependencies(&self, name: &str) -> Result<Vec<String>, AppError> {
        use k8s_openapi::api::core::v1::{Pod, Secret};

        let mut dependencies = Vec::new();

        let pod_api: Api<Pod> = Api::namespaced(self.client.clone(), name);
        let pod_lp = ListParams::default()
            .labels(&format!("{}={}", LABEL_MANAGED_BY, LABEL_MANAGED_BY_VALUE));
        if let Ok(pods) = pod_api.list(&pod_lp).await {
            if !pods.items.is_empty() {
                dependencies.push(format!("{} MCP server(s)", pods.items.len()));
            }
        }

        let secret_api: Api<Secret> = Api::namespaced(self.client.clone(), name);
        let secret_lp = ListParams::default()
            .labels(&format!("{}={}", LABEL_MANAGED_BY, LABEL_MANAGED_BY_VALUE));
        if let Ok(secrets) = secret_api.list(&secret_lp).await {
            if !secrets.items.is_empty() {
                dependencies.push(format!("{} secret(s)", secrets.items.len()));
            }
        }

        Ok(dependencies)
    }

    pub async fn add_finalizer(&self, name: &str) -> Result<(), AppError> {
        let ns = self.api.get(name).await.map_err(AppError::from)?;

        let mut finalizers = ns.metadata.finalizers.unwrap_or_default();
        if finalizers.contains(&FINALIZER_NAME.to_string()) {
            return Ok(());
        }

        finalizers.push(FINALIZER_NAME.to_string());

        let patch = serde_json::json!({
            "metadata": {
                "finalizers": finalizers
            }
        });

        self.api
            .patch(name, &PatchParams::default(), &Patch::Merge(&patch))
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    pub async fn remove_finalizer(&self, name: &str) -> Result<(), AppError> {
        let ns = self.api.get(name).await.map_err(AppError::from)?;

        let mut finalizers = ns.metadata.finalizers.unwrap_or_default();
        finalizers.retain(|f| f != FINALIZER_NAME);

        let patch = serde_json::json!({
            "metadata": {
                "finalizers": finalizers
            }
        });

        self.api
            .patch(name, &PatchParams::default(), &Patch::Merge(&patch))
            .await
            .map_err(AppError::from)?;

        Ok(())
    }

    pub fn has_finalizer(ns: &Namespace) -> bool {
        ns.metadata
            .finalizers
            .as_ref()
            .map(|f| f.contains(&FINALIZER_NAME.to_string()))
            .unwrap_or(false)
    }

    pub fn is_being_deleted(ns: &Namespace) -> bool {
        ns.metadata.deletion_timestamp.is_some()
    }

    pub async fn delete_with_lease(
        &self,
        name: &str,
        pod_name: &str,
        lease_namespace: &str,
    ) -> Result<DeleteResult, AppError> {
        use super::lease_manager::{resource_lock_name, LeaseManager};

        let lock_name = resource_lock_name("namespace", "", name);
        let lease_mgr =
            LeaseManager::new(self.client.clone(), lease_namespace, &lock_name, pod_name);

        if !lease_mgr.try_acquire().await? {
            return Ok(DeleteResult::Locked);
        }

        let ns = match self.api.get(name).await {
            Ok(ns) => ns,
            Err(kube::Error::Api(err)) if err.code == 404 => {
                lease_mgr.release().await?;
                return Err(AppError::NotFound(format!("Namespace {} not found", name)));
            }
            Err(e) => {
                lease_mgr.release().await?;
                return Err(AppError::from(e));
            }
        };

        if let Some(labels) = &ns.metadata.labels {
            if labels.get(LABEL_MANAGED_BY) != Some(&LABEL_MANAGED_BY_VALUE.to_string()) {
                lease_mgr.release().await?;
                return Err(AppError::NotFound(format!(
                    "Namespace {} is not managed by mcp-orchestrator",
                    name
                )));
            }
        } else {
            lease_mgr.release().await?;
            return Err(AppError::NotFound(format!(
                "Namespace {} has no labels",
                name
            )));
        }

        if Self::is_being_deleted(&ns) {
            if !Self::has_finalizer(&ns) {
                lease_mgr.release().await?;
                return Ok(DeleteResult::Deleting);
            }

            let deps = self.check_dependencies(name).await?;
            if !deps.is_empty() {
                lease_mgr.release().await?;
                return Ok(DeleteResult::HasDependencies(deps));
            }

            self.remove_finalizer(name).await?;
            lease_mgr.release().await?;
            return Ok(DeleteResult::FinalizerRemoved);
        }

        let deps = self.check_dependencies(name).await?;
        if !deps.is_empty() {
            self.add_finalizer(name).await?;
        }

        self.api
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

    pub async fn ensure_default_namespace(&self, name: &str) -> Result<Namespace, AppError> {
        if let Some(ns) = self.get_raw(name).await? {
            if let Some(labels) = &ns.metadata.labels {
                if let Some(existing_value) = labels.get(LABEL_MANAGED_BY) {
                    if existing_value != LABEL_MANAGED_BY_VALUE {
                        return Err(AppError::Internal(format!(
                            "Namespace {} already has {}={}, cannot take ownership",
                            name, LABEL_MANAGED_BY, existing_value
                        )));
                    }
                    return Ok(ns);
                }
            }
        }

        let mut labels = BTreeMap::new();
        labels.insert(
            LABEL_MANAGED_BY.to_string(),
            LABEL_MANAGED_BY_VALUE.to_string(),
        );

        let namespace = Namespace {
            metadata: ObjectMeta {
                name: Some(name.to_string()),
                labels: Some(labels),
                ..Default::default()
            },
            ..Default::default()
        };

        let patch_params = PatchParams::apply("mcp-orchestrator").force();
        self.api
            .patch(name, &patch_params, &Patch::Apply(&namespace))
            .await
            .map_err(AppError::from)
    }
}

#[derive(Debug, Clone)]
pub enum DeleteResult {
    Deleted,
    Deleting,
    DeletionStarted(Vec<String>),
    FinalizerRemoved,
    HasDependencies(Vec<String>),
    Locked,
}
