use std::{
    collections::{BTreeMap, HashMap},
    f64::consts::E,
};

use chrono::{DateTime, Duration, NaiveDateTime, Utc};
use k8s_openapi::api::core::v1::{Namespace, Pod, Secret};
use kube::{
    Api, Client, Resource, ResourceExt,
    api::{DeleteParams, ListParams, ObjectMeta, Patch, PatchParams, PostParams},
    config,
};
use serde_json::{self, json};
use tracing::info;

use super::label_query::LabelQuery;
use super::labels::{LABEL_MANAGED_BY, LABEL_MANAGED_BY_VALUE, setup_labels};
use crate::{
    error::AppError,
    storage::{
        util_delete::{DeleteOption, DeleteResult},
        label_query::build_label_query,
        labels::{LABEL_MANAGED_BY_QUERY, LABEL_TYPE_OF, is_managed_label},
        resource_type::RESOURCE_TYPE_NAMESPACE,
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
                .unwrap_or_else(|| Utc::now()),
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
        let mut ns = match api.get(&self.default_namespace).await {
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
                api.create(&PostParams::default(), &namespace)
                    .await
                    .map_err(AppError::from)?
            }
            Err(err) => return Err(AppError::from(err)),
        };
        {
            let label_mut = ns.labels_mut();
            label_mut.insert(
                LABEL_MANAGED_BY.to_string(),
                LABEL_MANAGED_BY_VALUE.to_string(),
            );
            label_mut.insert(
                LABEL_TYPE_OF.to_string(),
                RESOURCE_TYPE_NAMESPACE.to_string(),
            );
        }
        ns = api
            .replace(&self.default_namespace, &PostParams::default(), &ns)
            .await
            .map_err(AppError::from)?;
        Ok(ns)
    }

    pub async fn create<L: Iterator<Item = (String, String)>>(
        &self,
        name: &str,
        labels: L,
    ) -> Result<NamespaceData, AppError> {
        let namespace = Namespace {
            metadata: ObjectMeta {
                name: Some(name.to_string()),
                labels: Some(setup_labels(RESOURCE_TYPE_NAMESPACE, labels).collect()),
                ..Default::default()
            },
            ..Default::default()
        };

        self.api
            .create(&PostParams::default(), &namespace)
            .await
            .map(NamespaceData::from_namespace)
            .map_err(AppError::from)
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

    pub async fn list(&self, queries: &[LabelQuery]) -> Result<Vec<NamespaceData>, AppError> {
        let selector = build_label_query(RESOURCE_TYPE_NAMESPACE, &queries)?;
        let lp = ListParams::default().labels(&selector);
        let list = self.api.list(&lp).await.map_err(AppError::from)?;
        Ok(list
            .items
            .into_iter()
            .map(NamespaceData::from_namespace)
            .collect())
    }

    pub async fn delete(
        &self,
        name: &str,
        option: Option<DeleteOption>,
    ) -> Result<DeleteResult, AppError> {
        if name == &self.default_namespace {
            return Err(AppError::ProtectedNamespace(self.default_namespace.clone()));
        }
        let option = option.unwrap_or_default();

        if option.force.unwrap_or_default() {
            del_safe_finalizer(self.api.clone(), name, FINALIZER_NAME, 5).await?;
        } else {
            add_safe_finalizer(self.api.clone(), name, FINALIZER_NAME, 5).await?;
        }

        let mut result = self
            .api
            .delete(name, &DeleteParams::default())
            .await
            .map_err(AppError::from)?
            .map_left(|_x| DeleteResult::Deleting)
            .map_right(|_x| DeleteResult::Deleted)
            .into_inner();
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
        if name == &self.default_namespace {
            return Ok(false);
        }
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

        let pod_exists = pod
            .list_metadata(
                &ListParams::default()
                    .labels(LABEL_MANAGED_BY_QUERY)
                    .limit(1),
            )
            .await
            .map_err(AppError::from)?
            .items
            .len()
            > 0;
        let secret_exists = secret
            .list_metadata(
                &ListParams::default()
                    .labels(LABEL_MANAGED_BY_QUERY)
                    .limit(1),
            )
            .await
            .map_err(AppError::from)?
            .items
            .len()
            > 0;
        let configmap_exists = configmap
            .list_metadata(
                &ListParams::default()
                    .labels(LABEL_MANAGED_BY_QUERY)
                    .limit(1),
            )
            .await
            .map_err(AppError::from)?
            .items
            .len()
            > 0;
        let service_exists = service
            .list_metadata(
                &ListParams::default()
                    .labels(LABEL_MANAGED_BY_QUERY)
                    .limit(1),
            )
            .await
            .map_err(AppError::from)?
            .items
            .len()
            > 0;
        let pvc_exists = pvc
            .list_metadata(
                &ListParams::default()
                    .labels(LABEL_MANAGED_BY_QUERY)
                    .limit(1),
            )
            .await
            .map_err(AppError::from)?
            .items
            .len()
            > 0;
        let ingress_exists = ingress
            .list_metadata(
                &ListParams::default()
                    .labels(LABEL_MANAGED_BY_QUERY)
                    .limit(1),
            )
            .await
            .map_err(AppError::from)?
            .items
            .len()
            > 0;

        Ok(!pod_exists
            && !secret_exists
            && !configmap_exists
            && !service_exists
            && !pvc_exists
            && !ingress_exists)
    }
}
