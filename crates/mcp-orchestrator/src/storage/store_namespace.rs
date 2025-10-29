use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use k8s_openapi::{
    List,
    api::core::v1::{Namespace, Pod, Secret, ServiceAccount},
};
use kube::{
    Api, Client, Resource, ResourceExt,
    api::{DeleteParams, ListParams, ObjectMeta, PostParams},
};
use proto::mcp::orchestrator::v1::AuthorizationType;

use super::label_query::LabelQuery;
use super::labels::{LABEL_MANAGED_BY, LABEL_MANAGED_BY_VALUE, setup_labels};
use crate::{
    error::AppError,
    storage::{
        label_query::build_label_query,
        labels::{LABEL_MANAGED_BY_QUERY, LABEL_TYPE_OF, is_managed_label},
        resource_type::RESOURCE_TYPE_NAMESPACE,
        store_authorization::AuthorizationStore,
        util_delete::{DeleteOption, DeleteResult},
        util_list::ListOption,
        utils::{add_safe_finalizer, del_safe_finalizer, interval_timeout},
    },
};
const FINALIZER_NAME: &str = "mcp-orchestrator.egoavara.net/namespace";

pub struct NamespaceData {
    pub raw: Namespace,
    pub name: String,
    pub labels: HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl NamespaceData {
    pub fn from_namespace(ns: Namespace) -> Self {
        Self {
            name: ns.name_any(),
            labels: ns
                .labels()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            created_at: ns
                .creation_timestamp()
                .map(|x| x.0)
                .unwrap_or_else(Utc::now),
            deleted_at: ns.meta().deletion_timestamp.clone().map(|x| x.0),
            raw: ns,
        }
    }
    pub fn from_opt_namespace(ns: Option<Namespace>) -> Option<Self> {
        ns.map(Self::from_namespace)
    }
}

pub struct NamespaceStore {
    pub api: Api<Namespace>,
    pub default_namespace: String,
    client: Client,
}

impl NamespaceStore {
    pub fn new(client: Client, default_namespace: String) -> Self {
        Self {
            api: Api::all(client.clone()),
            client,
            default_namespace,
        }
    }

    pub async fn ensure_default_namespace(&self) -> Result<Namespace, AppError> {
        let api = Api::<Namespace>::all(self.client.clone());
        let mut default_ns = match api.get(&self.default_namespace).await {
            Ok(ns) => ns,
            Err(kube::Error::Api(resp)) if resp.code == 404 => {
                // Namespace does not exist, create it
                let namespace = Namespace {
                    metadata: ObjectMeta {
                        name: Some(self.default_namespace.clone()),
                        labels: Some(
                            setup_labels(RESOURCE_TYPE_NAMESPACE, std::iter::empty()).collect(),
                        ),
                        ..Default::default()
                    },
                    ..Default::default()
                };
                tracing::info!("Default namespace '{}' created", self.default_namespace);
                api.create(&PostParams::default(), &namespace)
                    .await
                    .map_err(AppError::from)?
            }
            Err(err) => return Err(AppError::from(err)),
        };
        {
            let label_mut = default_ns.labels_mut();
            label_mut.insert(
                LABEL_MANAGED_BY.to_string(),
                LABEL_MANAGED_BY_VALUE.to_string(),
            );
            label_mut.insert(
                LABEL_TYPE_OF.to_string(),
                RESOURCE_TYPE_NAMESPACE.to_string(),
            );
        }
        default_ns = api
            .replace(&self.default_namespace, &PostParams::default(), &default_ns)
            .await
            .map_err(AppError::from)?;

        let mut has_more = true;
        let mut cursor = None;
        while has_more {
            let (data, next_cursor, next_has_more) = self
                .list(
                    &[],
                    ListOption {
                        after: cursor.clone(),
                        first: Some(100),
                    },
                )
                .await?;
            cursor = next_cursor;
            has_more = next_has_more;

            for ns in data {
                // Default Authorizer
                let store_at = AuthorizationStore::new(self.client.clone(), ns.name.clone());
                let (default, _, _) = store_at
                    .list(
                        Some(AuthorizationType::Anonymous),
                        &[],
                        ListOption::default(),
                    )
                    .await?;
                if default.is_empty() {
                    tracing::info!(
                        "Default anonymous authorizer created, in namespace '{}'",
                        default_ns.name_any()
                    );
                    store_at
                        .create(
                            "anonymous",
                            vec![].into_iter(),
                            AuthorizationType::Anonymous,
                            &serde_json::json!({}),
                        )
                        .await?;
                }
            }
        }
        Ok(default_ns)
    }

    pub async fn create<L: Iterator<Item = (String, String)>>(
        &self,
        name: &str,
        labels: L,
    ) -> Result<NamespaceData, AppError> {
        let store_at = AuthorizationStore::new(self.client.clone(), name.to_string());
        let namespace = Namespace {
            metadata: ObjectMeta {
                name: Some(name.to_string()),
                labels: Some(setup_labels(RESOURCE_TYPE_NAMESPACE, labels).collect()),
                ..Default::default()
            },
            ..Default::default()
        };

        let ns = self
            .api
            .create(&PostParams::default(), &namespace)
            .await
            .map(NamespaceData::from_namespace)
            .map_err(AppError::from)?;

        store_at
            .create(
                "anonymous",
                vec![].into_iter(),
                AuthorizationType::Anonymous,
                &serde_json::json!({}),
            )
            .await?;
        Ok(ns)
    }

    pub async fn get(&self, name: &str) -> Result<Option<NamespaceData>, AppError> {
        self.api
            .get(name)
            .await
            .map(|x| {
                if is_managed_label(RESOURCE_TYPE_NAMESPACE, x.labels()) {
                    Some(x)
                } else {
                    None
                }
            })
            .map(NamespaceData::from_opt_namespace)
            .map_err(AppError::from)
    }

    pub async fn list(
        &self,
        queries: &[LabelQuery],
        option: ListOption,
    ) -> Result<(Vec<NamespaceData>, Option<String>, bool), AppError> {
        let label_query = build_label_query(RESOURCE_TYPE_NAMESPACE, queries)?;
        let lp = option.to_list_param(label_query);
        let list = self.api.list(&lp).await.map_err(AppError::from)?;
        Ok((
            list.items
                .into_iter()
                .take(option.get_limit())
                .map(NamespaceData::from_namespace)
                .collect::<Vec<_>>(),
            list.metadata.continue_.clone(),
            option.has_more(&list.metadata),
        ))
    }

    pub async fn delete(
        &self,
        name: &str,
        option: Option<DeleteOption>,
    ) -> Result<DeleteResult, AppError> {
        if name == self.default_namespace {
            return Err(AppError::ProtectedNamespace(self.default_namespace.clone()));
        }
        let option = option.unwrap_or_default();

        if option.remove_finalizer.unwrap_or_default() {
            del_safe_finalizer(self.api.clone(), name, FINALIZER_NAME, 5).await?;
        } else {
            add_safe_finalizer(self.api.clone(), name, FINALIZER_NAME, 5).await?;
        }

        let mut result = self
            .api
            .delete(name, &DeleteParams::default())
            .await
            .map(|ok| {
                ok.map_left(|_x| DeleteResult::Deleting)
                    .map_right(|_x| DeleteResult::Deleted)
                    .into_inner()
            })
            .or_else(|err| match err {
                kube::Error::Api(ae)
                    if ae.code == 404 && option.remove_finalizer.unwrap_or_default() =>
                {
                    Ok(DeleteResult::Deleted)
                }
                err => Err(AppError::from(err)),
            })?;
        if let Some(wait) = option.timeout {
            result = interval_timeout(Duration::milliseconds(300), wait, || async {
                self.api.get(name).await.map(|_| None).unwrap_or_else(|e| {
                    if let kube::Error::Api(ae) = e {
                        if ae.code == 404 {
                            return Some(true);
                        }
                    } else {
                        tracing::error!("Error while waiting for secret deletion: {}", e);
                    }
                    None
                })
            })
            .await
            .map(|_| DeleteResult::Deleted)
            .unwrap_or(DeleteResult::Deleting);
        }

        Ok(result)
    }

    pub async fn is_deletable(&self, name: &str) -> Result<bool, AppError> {
        if name == self.default_namespace {
            return Ok(false);
        }
        let api_namespace = Api::<Namespace>::all(self.client.clone());
        let Some(_namespace) = api_namespace.get_opt(name).await? else {
            return Ok(false);
        };

        let pod = Api::<Pod>::namespaced(self.client.clone(), name);
        let secret = Api::<Secret>::namespaced(self.client.clone(), name);
        let configmap =
            Api::<k8s_openapi::api::core::v1::ConfigMap>::namespaced(self.client.clone(), name);
        let service =
            Api::<k8s_openapi::api::core::v1::Service>::namespaced(self.client.clone(), name);
        let pvc = Api::<k8s_openapi::api::core::v1::PersistentVolumeClaim>::namespaced(
            self.client.clone(),
            name,
        );
        let ingress =
            Api::<k8s_openapi::api::networking::v1::Ingress>::namespaced(self.client.clone(), name);

        let pod_exists = !pod
            .list_metadata(
                &ListParams::default()
                    .labels(LABEL_MANAGED_BY_QUERY)
                    .limit(1),
            )
            .await
            .map_err(AppError::from)?
            .items
            .is_empty();
        let secret_exists = !secret
            .list_metadata(
                &ListParams::default()
                    .labels(LABEL_MANAGED_BY_QUERY)
                    .limit(1),
            )
            .await
            .map_err(AppError::from)?
            .items
            .is_empty();
        let configmap_exists = !configmap
            .list_metadata(
                &ListParams::default()
                    .labels(LABEL_MANAGED_BY_QUERY)
                    .limit(1),
            )
            .await
            .map_err(AppError::from)?
            .items
            .is_empty();
        let service_exists = !service
            .list_metadata(
                &ListParams::default()
                    .labels(LABEL_MANAGED_BY_QUERY)
                    .limit(1),
            )
            .await
            .map_err(AppError::from)?
            .items
            .is_empty();
        let pvc_exists = !pvc
            .list_metadata(
                &ListParams::default()
                    .labels(LABEL_MANAGED_BY_QUERY)
                    .limit(1),
            )
            .await
            .map_err(AppError::from)?
            .items
            .is_empty();
        let ingress_exists = !ingress
            .list_metadata(
                &ListParams::default()
                    .labels(LABEL_MANAGED_BY_QUERY)
                    .limit(1),
            )
            .await
            .map_err(AppError::from)?
            .items
            .is_empty();

        Ok(!pod_exists
            && !secret_exists
            && !configmap_exists
            && !service_exists
            && !pvc_exists
            && !ingress_exists)
    }
}
