use std::vec;

use chrono::{DateTime, Duration, Utc};
use k8s_openapi::api::core::v1::{ConfigMap, Secret};
use kube::{
    Api, Client, Resource, ResourceExt,
    api::{DeleteParams, ListParams, ObjectMeta, PostParams},
};

use super::label_query::{LabelQuery, build_label_query};
use super::labels::setup_labels;
use crate::{
    error::AppError,
    storage::{
        labels::{is_managed_label, label_dependency, label_dependency_query},
        resource_type::{
            RESOURCE_TYPE_MCP_TEMPLATE, RESOURCE_TYPE_NAMESPACE, RESOURCE_TYPE_PREFIX_SECRET,
            RESOURCE_TYPE_SECRET,
        },
        util_delete::{DeleteOption, DeleteResult},
        util_list::ListOption,
        util_name::{decode_k8sname, encode_k8sname},
        utils::{add_safe_finalizer, del_safe_finalizer, interval_timeout},
    },
};

const FINALIZER_NAME: &str = "mcp-orchestrator.egoavara.net/secret";

pub struct SecretData {
    pub raw: Secret,
    pub namespace: String,
    pub name: String,
    pub labels: std::collections::HashMap<String, String>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl SecretData {
    pub fn try_from_secret(secret: Secret) -> Result<Self, AppError> {
        Ok(Self {
            namespace: secret.namespace().unwrap_or_else(|| "default".to_string()),
            name: decode_k8sname(RESOURCE_TYPE_PREFIX_SECRET, &secret.name_any()).ok_or_else(
                || {
                    AppError::Internal(format!(
                        "Failed to decode secret name: {}, it must start with {}-",
                        secret.name_any(),
                        RESOURCE_TYPE_PREFIX_SECRET
                    ))
                },
            )?,
            labels: secret
                .labels()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            created_at: secret
                .creation_timestamp()
                .map(|x| x.0)
                .unwrap_or_else(Utc::now),
            deleted_at: secret.meta().deletion_timestamp.clone().map(|x| x.0),
            raw: secret,
        })
    }
}

pub struct SecretStore {
    client: Client,
    namespace: String,
}

impl SecretStore {
    pub fn new(client: Client, default_namespace: impl Into<String>) -> Self {
        Self {
            client,
            namespace: default_namespace.into(),
        }
    }

    fn api(&self) -> Api<Secret> {
        Api::namespaced(self.client.clone(), &self.namespace)
    }

    pub async fn create<
        L: Iterator<Item = (String, String)>,
        B: Iterator<Item = (String, String)>,
    >(
        &self,
        name: &str,
        secret_type: Option<String>,
        labels: L,
        data: B,
    ) -> Result<SecretData, AppError> {
        let name = encode_k8sname(RESOURCE_TYPE_PREFIX_SECRET, name);
        let secret = Secret {
            metadata: ObjectMeta {
                name: Some(name.clone()),
                labels: Some(
                    setup_labels(RESOURCE_TYPE_SECRET, labels)
                        .chain(label_dependency(RESOURCE_TYPE_NAMESPACE, &self.namespace))
                        .collect(),
                ),
                annotations: Some(vec![].into_iter().collect()),
                ..Default::default()
            },
            string_data: Some(data.collect()),
            type_: secret_type,
            ..Default::default()
        };

        self.api()
            .create(&PostParams::default(), &secret)
            .await
            .map_err(AppError::from)
            .and_then(SecretData::try_from_secret)
    }

    pub async fn get(&self, name: &str) -> Result<Option<SecretData>, AppError> {
        let name = encode_k8sname(RESOURCE_TYPE_PREFIX_SECRET, name);
        self.api()
            .get_opt(&name)
            .await
            .map_err(AppError::from)?
            .and_then(|x| {
                if is_managed_label(RESOURCE_TYPE_SECRET, x.labels()) {
                    Some(x)
                } else {
                    None
                }
            })
            .map(SecretData::try_from_secret)
            .transpose()
    }

    pub async fn list(
        &self,
        queries: &[LabelQuery],
        option: ListOption,
    ) -> Result<(Vec<SecretData>, Option<String>, bool), AppError> {
        let label_query = build_label_query(RESOURCE_TYPE_SECRET, queries)?;
        let lp = option.to_list_param(label_query);
        let list = self.api().list(&lp).await.map_err(AppError::from)?;
        Ok((
            list.items
                .into_iter()
                .take(option.get_limit())
                .map(SecretData::try_from_secret)
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
        let name = encode_k8sname(RESOURCE_TYPE_PREFIX_SECRET, name);
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
        let raw_name = encode_k8sname(RESOURCE_TYPE_PREFIX_SECRET, name);
        let api = Api::<Secret>::namespaced(self.client.clone(), &self.namespace);
        let Some(_secret) = api.get_opt(&raw_name).await? else {
            return Ok(false);
        };

        let has_dep_mcp_templates = self.has_dep_mcp_templates(name).await?;
        Ok(!has_dep_mcp_templates)
    }

    async fn has_dep_mcp_templates(&self, name: &str) -> Result<bool, AppError> {
        let mcp_template_store = Api::<ConfigMap>::namespaced(self.client.clone(), &self.namespace);

        let label = build_label_query(
            RESOURCE_TYPE_MCP_TEMPLATE,
            &[label_dependency_query(RESOURCE_TYPE_SECRET, name)],
        )?
        .to_string();
        let lp = ListParams::default().labels(&label).limit(1);
        let list = mcp_template_store.list(&lp).await.map_err(AppError::from)?;
        Ok(!list.items.is_empty())
    }
}
