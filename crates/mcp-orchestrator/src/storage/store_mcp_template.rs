use std::collections::{BTreeMap, HashMap};

use chrono::{DateTime, Duration, Utc};
use k8s_openapi::api::core::v1::{ConfigMap, Container, EnvVar, Pod, PodSpec};
use kube::{
    Api, Client, Resource, ResourceExt,
    api::{DeleteParams, ListParams, ObjectMeta, PostParams},
};
use proto::mcp::orchestrator::v1;
use rmcp::transport::streamable_http_server::SessionId;

use super::label_query::{LabelQuery, build_label_query};
use super::labels::setup_labels;
use crate::{
    error::AppError,
    storage::{
        ResourceLimitStore, SecretData, SecretStore,
        labels::{
            LABEL_SESSION_ID, is_managed_label, label_dependency, label_dependency_query,
            label_dependency_tuple,
        },
        resource_type::{
            RESOURCE_TYPE_MCP_SERVER, RESOURCE_TYPE_MCP_TEMPLATE, RESOURCE_TYPE_NAMESPACE,
            RESOURCE_TYPE_PREFIX_MCP_TEMPLATE, RESOURCE_TYPE_RESOURCE_LIMIT, RESOURCE_TYPE_SECRET,
        },
        store::KubeStore,
        store_authorization::{self, AuthorizationData, AuthorizationStore},
        util_delete::{DeleteOption, DeleteResult},
        util_list::ListOption,
        util_name::{decode_k8sname, encode_k8sname},
        utils::{
            add_safe_finalizer, data_elem, del_safe_finalizer, interval_timeout, parse_data_elem,
        },
    },
};

const FINALIZER_NAME: &str = "mcp-orchestrator.egoavara.net/mcp-template";
const DATA_IMAGE: &str = "image";
const DATA_COMMAND: &str = "command";
const DATA_ARGS: &str = "args";
const DATA_SECRET_ENVS: &str = "secret_env";
const DATA_RESOURCE_LIMIT_NAME: &str = "resource_limit_name";
const DATA_AUTHORIZATION_NAME: &str = "authorization_name";
const DATA_VOLUME_MOUNTS: &str = "volume_mounts";
const DATA_SECRET_MOUNTS: &str = "secret_mounts";

fn data_env_var(name: &str) -> String {
    format!("env_{}", name)
}
fn parse_env_var(key: &str) -> Option<String> {
    if key.starts_with("env_") {
        let key = key.trim_start_matches("env_");
        Some(key.to_string())
    } else {
        None
    }
}

pub struct McpTemplateData {
    pub raw: ConfigMap,
    pub namespace: String,
    pub name: String,
    pub labels: HashMap<String, String>,
    pub image: String,
    pub command: Vec<String>,
    pub args: Vec<String>,
    pub envs: HashMap<String, String>,
    pub secret_envs: Vec<String>,
    pub resource_limit_name: String,
    pub authorization_name: String,
    pub volume_mounts: Vec<v1::VolumeMount>,
    pub secret_mounts: Vec<v1::SecretMount>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl McpTemplateData {
    pub fn try_from_config_map(cm: ConfigMap) -> Result<Self, AppError> {
        let image: String = parse_data_elem(&cm.data, DATA_IMAGE)?;
        let command: Vec<String> = parse_data_elem(&cm.data, DATA_COMMAND)?;
        let args: Vec<String> = parse_data_elem(&cm.data, DATA_ARGS)?;
        let secret_envs: Vec<String> = parse_data_elem(&cm.data, DATA_SECRET_ENVS)?;
        let resource_limit_name: String = parse_data_elem(&cm.data, DATA_RESOURCE_LIMIT_NAME)?;
        let authorization_name: String = parse_data_elem(&cm.data, DATA_AUTHORIZATION_NAME)?;
        let volume_mounts: Vec<v1::VolumeMount> = parse_data_elem(&cm.data, DATA_VOLUME_MOUNTS)?;
        let secret_mounts: Vec<v1::SecretMount> = parse_data_elem(&cm.data, DATA_SECRET_MOUNTS)?;

        let mut envs: HashMap<String, String> = HashMap::new();
        if let Some(data) = &cm.data {
            for (key, value) in data.iter() {
                let Some(key) = parse_env_var(key) else {
                    continue;
                };
                envs.insert(key, value.clone());
            }
        }

        Ok(Self {
            namespace: cm.namespace().unwrap_or_else(|| "default".to_string()),
            name: decode_k8sname(RESOURCE_TYPE_PREFIX_MCP_TEMPLATE, &cm.name_any()).ok_or_else(
                || {
                    AppError::Internal(format!(
                        "Failed to decode configmap name: {}, it must start with {}-",
                        cm.name_any(),
                        RESOURCE_TYPE_PREFIX_MCP_TEMPLATE
                    ))
                },
            )?,
            labels: cm
                .labels()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            image,
            command,
            args,
            envs,
            secret_envs,
            resource_limit_name,
            authorization_name,
            volume_mounts,
            secret_mounts,
            created_at: cm
                .creation_timestamp()
                .map(|x| x.0)
                .unwrap_or_else(Utc::now),
            deleted_at: cm.meta().deletion_timestamp.clone().map(|x| x.0),
            raw: cm,
        })
    }

    async fn load_secrets(
        store: SecretStore,
        names: &Vec<String>,
        mounts: &Vec<v1::SecretMount>,
    ) -> Result<HashMap<String, SecretData>, AppError> {
        let mut secrets: HashMap<String, SecretData> = HashMap::new();
        for name in names {
            if secrets.contains_key(name) {
                continue;
            }
            let Some(secret) = store.get(name).await? else {
                return Err(AppError::Internal(format!(
                    "Secret {} not found for McpTemplate",
                    name
                )));
            };
            secrets.insert(name.clone(), secret);
        }
        for mount in mounts {
            let name = &mount.name;
            if secrets.contains_key(name) {
                continue;
            }
            let Some(secret) = store.get(name).await? else {
                return Err(AppError::Internal(format!(
                    "Secret {} not found for McpTemplate",
                    name
                )));
            };
            secrets.insert(name.clone(), secret);
        }
        Ok(secrets)
    }

    pub async fn get_authorization(
        &self,
        client: &KubeStore,
    ) -> Result<AuthorizationData, AppError> {
        let store_auth = client.authorization(Some(self.namespace.clone()));
        let Some(authorization) = store_auth.get(&self.authorization_name).await? else {
            tracing::error!("Authorization {} not found", self.authorization_name);
            return Err(AppError::Internal(format!(
                "Authorization {} required by McpTemplate {}/{} not found",
                self.authorization_name, self.namespace, self.name
            )));
        };
        Ok(authorization)
    }

    pub async fn to_pod(
        &self,
        session_id: &SessionId,
        client: &KubeStore,
    ) -> Result<(Pod, AuthorizationData), AppError> {
        let resource_limit_store = client.resource_limits();
        let store_auth = client.authorization(Some(self.namespace.clone()));

        let Some(resource_limit) = resource_limit_store.get(&self.resource_limit_name).await?
        else {
            tracing::error!("ResourceLimit {} not found", self.resource_limit_name);
            return Err(AppError::Internal(format!(
                "ResourceLimit {} required by McpTemplate {}/{} not found",
                self.resource_limit_name, self.namespace, self.name
            )));
        };
        let secrets = McpTemplateData::load_secrets(
            client.secrets(Some(self.namespace.clone())),
            &self.secret_envs,
            &self.secret_mounts,
        )
        .await?;

        let authorization = self.get_authorization(client).await?;

        let mut envs = HashMap::new();

        for (key, value) in self.envs.iter() {
            envs.insert(
                key.clone(),
                EnvVar {
                    name: key.clone(),
                    value: Some(value.clone()),
                    ..Default::default()
                },
            );
        }
        for secret_name in self.secret_envs.iter() {
            let secret = secrets.get(secret_name).ok_or_else(|| {
                AppError::Internal(format!(
                    "Secret {} not found for McpTemplate {}/{}",
                    secret_name, self.namespace, self.name
                ))
            })?;
            for (key, _) in secret.raw.data.as_ref().unwrap_or(&BTreeMap::new()).iter() {
                envs.insert(
                    key.clone(),
                    EnvVar {
                        name: key.clone(),
                        value_from: Some(k8s_openapi::api::core::v1::EnvVarSource {
                            secret_key_ref: Some(k8s_openapi::api::core::v1::SecretKeySelector {
                                name: secret.raw.name_any(),
                                key: key.clone(),
                                ..Default::default()
                            }),
                            ..Default::default()
                        }),
                        ..Default::default()
                    },
                );
            }
        }

        let envs = envs.into_values().collect();
        let requirement = resource_limit.to_resource_requirements();
        tracing::debug!(
            "Creating Pod for session {} with resource limit {:?}",
            session_id,
            requirement
        );

        let mut pod_spec = PodSpec {
            containers: vec![Container {
                name: "main".to_string(),
                image: Some(self.image.clone()),
                command: Some(self.command.clone()),
                args: Some(self.args.clone()),
                stdin: Some(true),
                tty: Some(false),
                env: Some(envs),
                resources: Some(requirement),
                ..Default::default()
            }],
            service_account_name: authorization.sa_name.clone(),
            node_selector: resource_limit.node_selector.clone(),
            affinity: resource_limit.node_affinity.clone(),
            ..Default::default()
        };

        Ok((
            Pod {
                metadata: ObjectMeta {
                    name: Some(session_id.to_string()),
                    namespace: Some(self.namespace.clone()),
                    labels: Some(
                        setup_labels(RESOURCE_TYPE_MCP_SERVER, std::iter::empty())
                            .chain(vec![(LABEL_SESSION_ID.to_string(), session_id.to_string())])
                            .collect(),
                    ),
                    owner_references: Some(vec![self.raw.controller_owner_ref(&()).unwrap()]),
                    ..Default::default()
                },
                spec: Some(pod_spec),
                ..Default::default()
            },
            authorization,
        ))
    }
}

pub struct McpTemplateStore {
    client: Client,
    target_namespace: String,
    default_namespace: String,
}

pub struct McpTemplateCreate {
    pub image: String,
    pub command: Vec<String>,
    pub args: Vec<String>,
    pub envs: HashMap<String, String>,
    pub secret_envs: Vec<String>,
    pub resource_limit_name: String,
    pub authorization_name: String,
    pub volume_mounts: Vec<v1::VolumeMount>,
    pub secret_mounts: Vec<v1::SecretMount>,
}

impl McpTemplateStore {
    pub fn new(
        client: Client,
        target_namespace: impl Into<String>,
        default_namespace: impl Into<String>,
    ) -> Self {
        Self {
            client,
            target_namespace: target_namespace.into(),
            default_namespace: default_namespace.into(),
        }
    }

    fn api(&self) -> Api<ConfigMap> {
        Api::namespaced(self.client.clone(), &self.target_namespace)
    }

    pub async fn create<L: Iterator<Item = (String, String)>>(
        &self,
        name: &str,
        labels: L,
        data: McpTemplateCreate,
    ) -> Result<McpTemplateData, AppError> {
        let name = encode_k8sname(RESOURCE_TYPE_PREFIX_MCP_TEMPLATE, name);
        let api = self.api();
        let resource_limit_store =
            ResourceLimitStore::new(self.client.clone(), self.default_namespace.clone());
        let store_authorization =
            AuthorizationStore::new(self.client.clone(), self.target_namespace.clone());

        let resource_limit = resource_limit_store
            .get(&data.resource_limit_name)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "ResourceLimit {} required by McpTemplate {}/{} not found",
                    data.resource_limit_name, self.target_namespace, name
                ))
            })?;
        let secrets = McpTemplateData::load_secrets(
            SecretStore::new(self.client.clone(), self.target_namespace.clone()),
            &data.secret_envs,
            &data.secret_mounts,
        )
        .await?;
        let store_authorization = AuthorizationStore::new(self.client.clone(), self.target_namespace.clone());
        let authorization = store_authorization
            .get(&data.authorization_name)
            .await?
            .ok_or_else(|| {
                AppError::NotFound(format!(
                    "Authorization {} required by McpTemplate {}/{} not found",
                    data.authorization_name, self.target_namespace, name
                ))
            })?;

        let configmap = ConfigMap {
            metadata: ObjectMeta {
                namespace: Some(self.target_namespace.clone()),
                name: Some(name),
                labels: Some(
                    setup_labels(RESOURCE_TYPE_MCP_TEMPLATE, labels)
                        .chain(label_dependency(
                            RESOURCE_TYPE_NAMESPACE,
                            &self.target_namespace,
                        ))
                        .chain(label_dependency(
                            RESOURCE_TYPE_RESOURCE_LIMIT,
                            &resource_limit.name,
                        ))
                        .chain(
                            secrets
                                .keys()
                                .map(|name| label_dependency_tuple(RESOURCE_TYPE_SECRET, name)),
                        )
                        .collect(),
                ),
                ..Default::default()
            },
            data: Some(
                vec![
                    data_elem(DATA_IMAGE, &data.image)?,
                    data_elem(DATA_COMMAND, &data.command)?,
                    data_elem(DATA_ARGS, &data.args)?,
                    data_elem(DATA_SECRET_ENVS, &data.secret_envs)?,
                    data_elem(DATA_RESOURCE_LIMIT_NAME, &data.resource_limit_name)?,
                    data_elem(DATA_AUTHORIZATION_NAME, &data.authorization_name)?,
                    data_elem(DATA_VOLUME_MOUNTS, &data.volume_mounts)?,
                    data_elem(DATA_SECRET_MOUNTS, &data.secret_mounts)?,
                ]
                .into_iter()
                .chain(
                    data.envs
                        .iter()
                        .map(|(key, value)| (data_env_var(key), value.clone())),
                )
                .collect(),
            ),
            ..Default::default()
        };

        api.create(&PostParams::default(), &configmap)
            .await
            .map_err(AppError::from)
            .and_then(McpTemplateData::try_from_config_map)
    }

    pub async fn get(&self, name: &str) -> Result<Option<McpTemplateData>, AppError> {
        let name = encode_k8sname(RESOURCE_TYPE_PREFIX_MCP_TEMPLATE, name);
        self.api()
            .get_opt(&name)
            .await
            .map_err(AppError::from)?
            .and_then(|x| {
                if is_managed_label(RESOURCE_TYPE_MCP_TEMPLATE, x.labels()) {
                    Some(x)
                } else {
                    None
                }
            })
            .map(McpTemplateData::try_from_config_map)
            .transpose()
    }

    pub async fn list(
        &self,
        queries: &[LabelQuery],
        option: ListOption,
    ) -> Result<(Vec<McpTemplateData>, Option<String>, bool), AppError> {
        let label_query = build_label_query(RESOURCE_TYPE_MCP_TEMPLATE, queries)?;
        let lp = option.to_list_param(label_query);
        let list = self.api().list(&lp).await.map_err(AppError::from)?;
        Ok((
            list.items
                .into_iter()
                .take(option.get_limit())
                .map(McpTemplateData::try_from_config_map)
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
        let name = encode_k8sname(RESOURCE_TYPE_PREFIX_MCP_TEMPLATE, name);
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
        let raw_name = encode_k8sname(RESOURCE_TYPE_PREFIX_MCP_TEMPLATE, name);
        let api = Api::<ConfigMap>::namespaced(self.client.clone(), &self.target_namespace);
        let Some(_configmap) = api.get_opt(&raw_name).await? else {
            return Ok(false);
        };

        let has_dep_mcp_server = self.has_dep_mcp_server(name).await?;
        Ok(!has_dep_mcp_server)
    }

    async fn has_dep_mcp_server(&self, name: &str) -> Result<bool, AppError> {
        let mcp_server_store = Api::<Pod>::namespaced(self.client.clone(), &self.target_namespace);

        let label = build_label_query(
            RESOURCE_TYPE_MCP_SERVER,
            &[label_dependency_query(RESOURCE_TYPE_MCP_TEMPLATE, name)],
        )?
        .to_string();
        let lp = ListParams::default().labels(&label).limit(1);
        let list = mcp_server_store.list(&lp).await.map_err(AppError::from)?;
        Ok(!list.items.is_empty())
    }
}
