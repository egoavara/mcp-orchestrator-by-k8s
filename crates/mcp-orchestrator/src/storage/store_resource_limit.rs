use std::collections::{BTreeMap, HashMap};
use std::vec;

use super::label_query::LabelQuery;
use super::labels::setup_labels;
use crate::storage::annotations::{ANNOTATION_DESCRIPTION, annotation_description};
use crate::storage::label_query::build_label_query;
use crate::storage::labels::{label_dependency, label_dependency_query};
use crate::storage::resource_type::{
    RESOURCE_TYPE_MCP_TEMPLATE, RESOURCE_TYPE_NAMESPACE, RESOURCE_TYPE_PREFIX_RESOURCE_LIMIT,
    RESOURCE_TYPE_RESOURCE_LIMIT,
};
use crate::storage::util_list::ListOption;
use crate::storage::util_name::{decode_k8sname, encode_k8sname};
use crate::storage::utils::{
    add_safe_finalizer, data_elem, data_elem_jsonstr, data_elem_ojsonstr, del_safe_finalizer,
    parse_data_elem,
};
use crate::{
    error::AppError,
    storage::{
        labels::is_managed_label,
        util_delete::{DeleteOption, DeleteResult},
        utils::interval_timeout,
    },
};
use chrono::{DateTime, Duration, Utc};
use k8s_openapi::api::core::v1::{Affinity, ConfigMap, ResourceRequirements};
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use kube::Resource;
use kube::{
    Api, Client, ResourceExt,
    api::{DeleteParams, ListParams, ObjectMeta, PostParams},
};
use proto::mcp::orchestrator::v1::{self, VolumeLimit};

const FINALIZER_NAME: &str = "mcp-orchestrator.egoavara.net/resource-limit";
const DATA_CPU: &str = "cpu";
const DATA_CPU_LIMIT: &str = "cpu_limit";
const DATA_MEMORY: &str = "memory";
const DATA_MEMORY_LIMIT: &str = "memory_limit";
const DATA_EPHEMERAL_STORAGE: &str = "ephemeral_storage";
const DATA_VOLUMES: &str = "volumes";
const DATA_NODE_SELECTOR: &str = "node_selector";
const DATA_NODE_AFFINITY: &str = "node_affinity";

pub struct ResourceLimitData {
    pub raw: ConfigMap,
    pub name: String,
    pub description: String,
    pub labels: HashMap<String, String>,
    pub cpu: String,
    pub memory: String,
    pub cpu_limit: Option<String>,
    pub memory_limit: Option<String>,
    pub ephemeral_storage: Option<String>,
    pub volumes: HashMap<String, VolumeLimit>,
    pub node_selector: Option<BTreeMap<String, String>>,
    pub node_affinity: Option<Affinity>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl ResourceLimitData {
    pub fn try_from_config_map(cm: ConfigMap) -> Result<Self, AppError> {
        Ok(Self {
            name: decode_k8sname(RESOURCE_TYPE_PREFIX_RESOURCE_LIMIT, &cm.name_any()).ok_or_else(
                || {
                    AppError::Internal(format!(
                        "Failed to decode configmap name: {}, it must start with {}-",
                        cm.name_any(),
                        RESOURCE_TYPE_PREFIX_RESOURCE_LIMIT
                    ))
                },
            )?,
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
            cpu: parse_data_elem(&cm.data, DATA_CPU)?,
            cpu_limit: parse_data_elem(&cm.data, DATA_CPU_LIMIT)?,
            memory: parse_data_elem(&cm.data, DATA_MEMORY)?,
            memory_limit: parse_data_elem(&cm.data, DATA_MEMORY_LIMIT)?,
            volumes: parse_data_elem(&cm.data, DATA_VOLUMES)?,
            ephemeral_storage: parse_data_elem(&cm.data, DATA_EPHEMERAL_STORAGE)?,
            node_selector: parse_data_elem(&cm.data, DATA_NODE_SELECTOR)?,
            node_affinity: parse_data_elem(&cm.data, DATA_NODE_AFFINITY)?,
            created_at: cm
                .creation_timestamp()
                .map(|x| x.0)
                .unwrap_or_else(Utc::now),
            deleted_at: cm.meta().deletion_timestamp.clone().map(|x| x.0),
            raw: cm,
        })
    }

    pub fn to_resource_requirements(&self) -> ResourceRequirements {
        ResourceRequirements {
            requests: Some(
                [
                    Some(("cpu".to_string(), Quantity(self.cpu.clone()))),
                    Some(("memory".to_string(), Quantity(self.memory.clone()))),
                    self.ephemeral_storage
                        .as_ref()
                        .map(|v| ("ephemeral-storage".to_string(), Quantity(v.to_string()))),
                ]
                .into_iter()
                .flatten()
                .collect(),
            ),
            limits: Some(
                [
                    Some((
                        "cpu".to_string(),
                        Quantity(self.cpu_limit.as_ref().cloned().unwrap_or(self.cpu.clone())),
                    )),
                    Some((
                        "memory".to_string(),
                        Quantity(
                            self.memory_limit
                                .as_ref()
                                .cloned()
                                .unwrap_or(self.memory.clone()),
                        ),
                    )),
                ]
                .into_iter()
                .flatten()
                .collect(),
            ),
            ..Default::default()
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
        let name = encode_k8sname(RESOURCE_TYPE_PREFIX_RESOURCE_LIMIT, name);

        let mut data_vec = vec![
            data_elem(DATA_CPU, &data.cpu)?,
            data_elem(DATA_CPU_LIMIT, &data.cpu_limit)?,
            data_elem(DATA_MEMORY, &data.memory)?,
            data_elem(DATA_MEMORY_LIMIT, &data.memory_limit)?,
            data_elem(DATA_VOLUMES, &data.volumes)?,
            data_elem(DATA_EPHEMERAL_STORAGE, &data.ephemeral_storage)?,
            data_elem(DATA_NODE_SELECTOR, &data.node_selector)?,
            data_elem_ojsonstr(DATA_NODE_AFFINITY, data.node_affinity.as_deref())?,
        ];

        let configmap = ConfigMap {
            metadata: ObjectMeta {
                name: Some(name),
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
            data: Some(data_vec.into_iter().collect()),
            ..Default::default()
        };

        self.api()
            .create(&PostParams::default(), &configmap)
            .await
            .map_err(AppError::from)
            .and_then(ResourceLimitData::try_from_config_map)
    }

    pub async fn get(&self, name: &str) -> Result<Option<ResourceLimitData>, AppError> {
        let name = encode_k8sname(RESOURCE_TYPE_PREFIX_RESOURCE_LIMIT, name);
        self.api()
            .get_opt(&name)
            .await
            .map_err(AppError::from)?
            .and_then(|x| {
                if is_managed_label(RESOURCE_TYPE_RESOURCE_LIMIT, x.labels()) {
                    Some(x)
                } else {
                    None
                }
            })
            .map(ResourceLimitData::try_from_config_map)
            .transpose()
    }

    pub async fn list(
        &self,
        queries: &[LabelQuery],
        option: ListOption,
    ) -> Result<(Vec<ResourceLimitData>, Option<String>, bool), AppError> {
        let label_query = build_label_query(RESOURCE_TYPE_RESOURCE_LIMIT, queries)?;
        let lp = option.to_list_param(label_query);
        let list = self.api().list(&lp).await.map_err(AppError::from)?;
        Ok((
            list.items
                .into_iter()
                .take(option.get_limit())
                .map(ResourceLimitData::try_from_config_map)
                .collect::<Result<Vec<_>, _>>()?,
            list.metadata.continue_.clone(),
            option.has_more(&list.metadata),
        ))
    }

    pub async fn delete(
        &self,
        name: &str,
        option: Option<DeleteOption>,
    ) -> Result<DeleteResult, AppError> {
        let name = encode_k8sname(RESOURCE_TYPE_PREFIX_RESOURCE_LIMIT, name);
        let api = self.api();
        let option = option.unwrap_or_default();

        if option.remove_finalizer.unwrap_or_default() {
            del_safe_finalizer(self.api().clone(), &name, FINALIZER_NAME, 5).await?;
        } else {
            add_safe_finalizer(self.api().clone(), &name, FINALIZER_NAME, 5).await?;
        }

        let mut result = api
            .delete(&name, &DeleteParams::default())
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
                api.get(&name).await.map(|_| None).unwrap_or_else(|e| {
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
        let raw_name = encode_k8sname(RESOURCE_TYPE_PREFIX_RESOURCE_LIMIT, name);
        let api = Api::<ConfigMap>::namespaced(self.client.clone(), &self.namespace);
        let Some(_config_map) = api.get_opt(&raw_name).await? else {
            return Ok(false);
        };

        let has_dep_mcp_templates = self.has_dep_mcp_templates(name).await?;
        Ok(!has_dep_mcp_templates)
    }

    async fn has_dep_mcp_templates(&self, name: &str) -> Result<bool, AppError> {
        let mcp_template_store = Api::<ConfigMap>::all(self.client.clone());

        let label = build_label_query(
            RESOURCE_TYPE_MCP_TEMPLATE,
            &[label_dependency_query(RESOURCE_TYPE_RESOURCE_LIMIT, name)],
        )?
        .to_string();
        let lp = ListParams::default().labels(&label).limit(1);
        let list = mcp_template_store.list(&lp).await.map_err(AppError::from)?;
        Ok(!list.items.is_empty())
    }
}
