use std::collections::HashMap;
use std::vec;

use super::label_query::LabelQuery;
use super::labels::setup_labels;
use crate::storage::annotations::{ANNOTATION_DESCRIPTION, annotation_description};
use crate::storage::label_query::build_label_query;
use crate::storage::labels::label_dependency;
use crate::storage::resource_type::{RESOURCE_TYPE_NAMESPACE, RESOURCE_TYPE_RESOURCE_LIMIT};
use crate::storage::util_name::{decode_k8sname, encode_k8sname};
use crate::storage::utils::{add_safe_finalizer, data_elem, parse_data_elem};
use crate::{
    error::AppError,
    storage::{
        labels::is_managed_label,
        util_delete::{DeleteOption, DeleteResult},
        utils::interval_timeout,
    },
};
use chrono::{DateTime, Duration, Utc};
use k8s_openapi::api::core::v1::ConfigMap;
use kube::Resource;
use kube::{
    Api, Client, ResourceExt,
    api::{DeleteParams, ListParams, ObjectMeta, PostParams},
};
use proto::mcp::orchestrator::v1;

const PREFIX: &str = "rl";
const FINALIZER_NAME: &str = "mcp-orchestrator.egoavara.net/resource-limit";
const DATA_CPU: &str = "cpu";
const DATA_CPU_LIMIT: &str = "cpu_limit";
const DATA_MEMORY: &str = "memory";
const DATA_MEMORY_LIMIT: &str = "memory_limit";
const DATA_EPHEMERAL_STORAGE: &str = "ephemeral_storage";
const DATA_VOLUMES: &str = "volumes";

pub struct ResourceLimitData {
    pub raw: ConfigMap,
    pub name: String,
    pub description: String,
    pub labels: HashMap<String, String>,
    pub limits: v1::ResourceLimit,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl ResourceLimitData {
    pub fn try_from_config_map(cm: ConfigMap) -> Result<Self, AppError> {
        let spec = v1::ResourceLimit {
            cpu: parse_data_elem(&cm.data, DATA_CPU)?,
            cpu_limit: parse_data_elem(&cm.data, DATA_CPU_LIMIT)?,
            memory: parse_data_elem(&cm.data, DATA_MEMORY)?,
            memory_limit: parse_data_elem(&cm.data, DATA_MEMORY_LIMIT)?,
            volumes: parse_data_elem(&cm.data, DATA_VOLUMES)?,
            ephemeral_storage: parse_data_elem(&cm.data, DATA_EPHEMERAL_STORAGE)?,
        };
        Ok(Self {
            name: decode_k8sname(PREFIX, &cm.name_any()).ok_or_else(|| {
                AppError::Internal(format!(
                    "Failed to decode configmap name: {}, it must start with {}-",
                    cm.name_any(),
                    PREFIX
                ))
            })?,
            description: cm
                .annotations()
                .get(&ANNOTATION_DESCRIPTION.to_string())
                .cloned()
                .unwrap_or_default(),
            labels: cm
                .labels()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            limits: spec,
            created_at: cm
                .creation_timestamp()
                .map(|x| x.0)
                .unwrap_or_else(Utc::now),
            deleted_at: cm.meta().deletion_timestamp.clone().map(|x| x.0),
            raw: cm,
        })
    }

    fn try_from_option_config_map(cm_opt: Option<ConfigMap>) -> Result<Option<Self>, AppError> {
        if let Some(cm) = cm_opt {
            Ok(Some(Self::try_from_config_map(cm)?))
        } else {
            Ok(None)
        }
    }
}

pub struct ResourceLimitStore {
    client: Client,
    namespace: String,
}

impl ResourceLimitStore {
    pub fn new(client: Client, namespace: String) -> Self {
        Self { client, namespace }
    }

    fn api(&self) -> Api<ConfigMap> {
        Api::namespaced(self.client.clone(), &self.namespace)
    }

    pub async fn create<L: Iterator<Item = (String, String)>>(
        &self,
        name: &str,
        labels: L,
        description: &str,
        data: &v1::ResourceLimit,
    ) -> Result<ResourceLimitData, AppError> {
        let configmap = ConfigMap {
            metadata: ObjectMeta {
                name: Some(encode_k8sname(PREFIX, name)),
                labels: Some(
                    setup_labels(RESOURCE_TYPE_RESOURCE_LIMIT, labels)
                        .chain(label_dependency(RESOURCE_TYPE_NAMESPACE, &self.namespace))
                        .collect(),
                ),
                annotations: Some(
                    vec![annotation_description(description)]
                        .into_iter()
                        .collect(),
                ),
                ..Default::default()
            },
            data: Some(
                vec![
                    data_elem(DATA_CPU, &data.cpu)?,
                    data_elem(DATA_CPU_LIMIT, &data.cpu_limit)?,
                    data_elem(DATA_MEMORY, &data.memory)?,
                    data_elem(DATA_MEMORY_LIMIT, &data.memory_limit)?,
                    data_elem(DATA_VOLUMES, &data.volumes)?,
                    data_elem(DATA_EPHEMERAL_STORAGE, &data.ephemeral_storage)?,
                ]
                .into_iter()
                .collect(),
            ),
            ..Default::default()
        };

        self.api()
            .create(&PostParams::default(), &configmap)
            .await
            .map_err(AppError::from)
            .and_then(ResourceLimitData::try_from_config_map)
    }

    pub async fn get(&self, name: &str) -> Result<Option<ResourceLimitData>, AppError> {
        self.api()
            .get(&encode_k8sname(PREFIX, name))
            .await
            .map(|x| {
                if is_managed_label(RESOURCE_TYPE_RESOURCE_LIMIT, x.labels()) {
                    Some(x)
                } else {
                    None
                }
            })
            .map_err(AppError::from)
            .and_then(ResourceLimitData::try_from_option_config_map)
    }

    pub async fn list(&self, queries: &[LabelQuery]) -> Result<Vec<ResourceLimitData>, AppError> {
        let selector = build_label_query(RESOURCE_TYPE_RESOURCE_LIMIT, queries)?;
        let lp = ListParams::default().labels(&selector);
        let list = self.api().list(&lp).await.map_err(AppError::from)?;
        list.items
            .into_iter()
            .map(ResourceLimitData::try_from_config_map)
            .collect::<Result<Vec<_>, _>>()
    }

    pub async fn delete(
        &self,
        name: &str,
        option: Option<DeleteOption>,
    ) -> Result<DeleteResult, AppError> {
        let api = self.api();
        let option = option.unwrap_or_default();

        if !option.force.unwrap_or_default() {
            add_safe_finalizer(self.api(), &encode_k8sname(PREFIX, name), FINALIZER_NAME, 5).await?;
        }

        let mut result = api
            .delete(&encode_k8sname(PREFIX, name), &DeleteParams::default())
            .await
            .map_err(AppError::from)?
            .map_left(|_x| DeleteResult::Deleting)
            .map_right(|_x| DeleteResult::Deleted)
            .into_inner();
        if let Some(wait) = option.timeout {
            result = interval_timeout(Duration::milliseconds(300), wait, || async {
                api.get(name).await.map(|_| None).unwrap_or_else(|e| {
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
}
