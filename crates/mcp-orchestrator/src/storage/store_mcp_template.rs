use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use k8s_openapi::api::core::v1::ConfigMap;
use kube::{
    Api, Client, Resource, ResourceExt,
    api::{DeleteParams, ListParams, ObjectMeta, PostParams},
};
use proto::mcp::orchestrator::v1;

use super::label_query::{LabelQuery, build_label_query};
use super::labels::setup_labels;
use crate::{
    error::AppError,
    storage::{
        ResourceLimitStore, SecretStore,
        labels::{is_managed_label, label_dependency, label_dependency_tuple},
        resource_type::{
            RESOURCE_TYPE_MCP_TEMPLATE, RESOURCE_TYPE_NAMESPACE, RESOURCE_TYPE_RESOURCE_LIMIT,
            RESOURCE_TYPE_SECRET,
        },
        resource_uname::resource_relpath,
        util_delete::{DeleteOption, DeleteResult},
        util_name::{decode_k8sname, encode_k8sname},
        utils::{add_safe_finalizer, data_elem, interval_timeout, parse_data_elem},
    },
};

const PREFIX: &str = "mt";
const FINALIZER_NAME: &str = "mcp-orchestrator.egoavara.net/mcp-template";
const DATA_IMAGE: &str = "image";
const DATA_COMMAND: &str = "command";
const DATA_ARGS: &str = "args";
const DATA_SECRET_ENV_VARS: &str = "secret";
const DATA_RESOURCE_LIMIT_NAME: &str = "resource_limit_name";
const DATA_VOLUME_MOUNTS: &str = "volume_mounts";

fn data_env_var(name: &str) -> String {
    format!("env:{}", name)
}

pub struct McpTemplateData {
    pub data: ConfigMap,
    pub name: String,
    pub labels: HashMap<String, String>,
    pub image: String,
    pub command: Vec<String>,
    pub args: Vec<String>,
    pub env_vars: Vec<v1::EnvVar>,
    pub secret_env_vars: Vec<v1::SecretEnvVar>,
    pub resource_limit_name: String,
    pub volume_mounts: Vec<v1::VolumeMount>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl McpTemplateData {
    pub fn try_from_config_map(cm: ConfigMap) -> Result<Self, AppError> {
        let image: String = parse_data_elem(&cm.data, DATA_IMAGE)?;
        let command: Vec<String> = parse_data_elem(&cm.data, DATA_COMMAND)?;
        let args: Vec<String> = parse_data_elem(&cm.data, DATA_ARGS)?;
        let secret_env_vars: Vec<v1::SecretEnvVar> =
            parse_data_elem(&cm.data, DATA_SECRET_ENV_VARS)?;
        let resource_limit_name: String = parse_data_elem(&cm.data, DATA_RESOURCE_LIMIT_NAME)?;
        let volume_mounts: Vec<v1::VolumeMount> = parse_data_elem(&cm.data, DATA_VOLUME_MOUNTS)?;

        let mut env_vars: Vec<v1::EnvVar> = vec![];
        if let Some(data) = &cm.data {
            for (key, value) in data.iter() {
                if key.starts_with("env:") {
                    let key = key.trim_start_matches("env:");
                    env_vars.push(v1::EnvVar {
                        key: key.to_string(),
                        value: value.clone(),
                    });
                }
            }
        }

        Ok(Self {
            name: decode_k8sname(PREFIX, &cm.name_any()).ok_or_else(|| {
                AppError::Internal(format!(
                    "Failed to decode configmap name: {}, it must start with {}-",
                    cm.name_any(),
                    PREFIX
                ))
            })?,
            labels: cm
                .labels()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            image,
            command,
            args,
            env_vars,
            secret_env_vars,
            resource_limit_name,
            volume_mounts,
            created_at: cm
                .creation_timestamp()
                .map(|x| x.0)
                .unwrap_or_else(Utc::now),
            deleted_at: cm.meta().deletion_timestamp.clone().map(|x| x.0),
            data: cm,
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

pub struct McpTemplateStore {
    client: Client,
    default_namespace: String,
}

pub struct McpTemplateCreate {
    pub image: String,
    pub command: Vec<String>,
    pub args: Vec<String>,
    pub env_vars: Vec<v1::EnvVar>,
    pub secret_env_vars: Vec<v1::SecretEnvVar>,
    pub resource_limit_name: String,
    pub volume_mounts: Vec<v1::VolumeMount>,
}

impl McpTemplateStore {
    pub fn new(client: Client, default_namespace: impl Into<String>) -> Self {
        Self {
            client,
            default_namespace: default_namespace.into(),
        }
    }

    fn api(&self) -> Api<ConfigMap> {
        Api::namespaced(self.client.clone(), &self.default_namespace)
    }

    pub async fn create<L: Iterator<Item = (String, String)>>(
        &self,
        name: &str,
        labels: L,
        data: McpTemplateCreate,
    ) -> Result<ConfigMap, AppError> {
        let api = self.api();
        let secret_store = SecretStore::new(self.client.clone(), self.default_namespace.clone());
        let resource_limit_store =
            ResourceLimitStore::new(self.client.clone(), self.default_namespace.clone());

        let resource_limit = resource_limit_store
            .get(&data.resource_limit_name)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "ResourceLimit {} required by McpTemplate {}/{} not found",
                    data.resource_limit_name, self.default_namespace, name
                ))
            })?;

        let secrets = futures::future::join_all(
            data.secret_env_vars
                .iter()
                .map(|sev| secret_store.get(&sev.name)),
        )
        .await
        .into_iter()
        .collect::<Result<Vec<_>, AppError>>()?
        .into_iter()
        .flatten()
        .map(|x| (x.name.clone(), x))
        .collect::<HashMap<_, _>>();

        for sev in &data.secret_env_vars {
            if !secrets.contains_key(&sev.name) {
                return Err(AppError::NotFound(format!(
                    "Secret not found ({})",
                    resource_relpath(RESOURCE_TYPE_SECRET, &sev.name),
                )));
            }
        }

        let configmap =
            ConfigMap {
                metadata: ObjectMeta {
                    namespace: Some(self.default_namespace.clone()),
                    name: Some(encode_k8sname(PREFIX, name)),
                    labels: Some(
                        setup_labels(RESOURCE_TYPE_MCP_TEMPLATE, labels)
                            .chain(label_dependency(
                                RESOURCE_TYPE_NAMESPACE,
                                &self.default_namespace,
                            ))
                            .chain(label_dependency(
                                RESOURCE_TYPE_RESOURCE_LIMIT,
                                &data.resource_limit_name,
                            ))
                            .chain(secrets.keys().map(|name| {
                                label_dependency_tuple(RESOURCE_TYPE_SECRET, name)
                            }))
                            .collect(),
                    ),
                    ..Default::default()
                },
                data: Some(
                    vec![
                        data_elem(DATA_IMAGE, &data.image)?,
                        data_elem(DATA_COMMAND, &data.command)?,
                        data_elem(DATA_ARGS, &data.args)?,
                        data_elem(DATA_SECRET_ENV_VARS, &data.secret_env_vars)?,
                        data_elem(DATA_RESOURCE_LIMIT_NAME, &data.resource_limit_name)?,
                        data_elem(DATA_VOLUME_MOUNTS, &data.volume_mounts)?,
                    ]
                    .into_iter()
                    .chain(
                        data.env_vars
                            .into_iter()
                            .map(|envvar| (data_env_var(&envvar.key), envvar.value.clone())),
                    )
                    .collect(),
                ),
                ..Default::default()
            };

        api.create(&PostParams::default(), &configmap)
            .await
            .map_err(AppError::from)
    }

    pub async fn get(&self, name: &str) -> Result<Option<McpTemplateData>, AppError> {
        self.api()
            .get(&encode_k8sname(PREFIX, name))
            .await
            .map(|x| {
                if is_managed_label(RESOURCE_TYPE_MCP_TEMPLATE, x.labels()) {
                    Some(x)
                } else {
                    None
                }
            })
            .map_err(AppError::from)
            .and_then(McpTemplateData::try_from_option_config_map)
    }

    pub async fn list(
        &self,
        queries: &[LabelQuery],
    ) -> Result<Vec<McpTemplateData>, AppError> {
        let selector = build_label_query(RESOURCE_TYPE_MCP_TEMPLATE, queries)?;
        let lp = ListParams::default().labels(&selector);
        let list = self.api().list(&lp).await.map_err(AppError::from)?;
        list.items
            .into_iter()
            .map(McpTemplateData::try_from_config_map)
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
            add_safe_finalizer(self.api(), &encode_k8sname(PREFIX, name), FINALIZER_NAME, 5)
                .await?;
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
