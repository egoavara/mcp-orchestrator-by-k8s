use std::vec;

use chrono::{DateTime, Duration, Utc};
use k8s_openapi::api::core::v1::Secret;
use kube::{
    Api, Client, Resource, ResourceExt,
    api::{DeleteParams, ListParams, ObjectMeta, PostParams},
};

use super::label_query::{LabelQuery, build_label_query};
use super::labels::setup_labels;
use crate::{
    error::AppError,
    storage::{
        labels::{is_managed_label, label_dependency},
        resource_type::{RESOURCE_TYPE_NAMESPACE, RESOURCE_TYPE_SECRET},
        resource_uname::resource_fullpath,
        util_delete::{DeleteOption, DeleteResult},
        util_name::{decode_k8sname, encode_k8sname},
        utils::{add_safe_finalizer, interval_timeout},
    },
};

const FINALIZER_NAME: &str = "mcp-orchestrator.egoavara.net/secret";
const PREFIX: &str = "sc";

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
            name: decode_k8sname(PREFIX, &secret.name_any()).ok_or_else(|| {
                AppError::Internal(format!(
                    "Failed to decode secret name: {}, it must start with {}-",
                    secret.name_any(),
                    PREFIX
                ))
            })?,
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

    fn try_from_option_secret(cm_opt: Option<Secret>) -> Result<Option<Self>, AppError> {
        if let Some(cm) = cm_opt {
            Ok(Some(Self::try_from_secret(cm)?))
        } else {
            Ok(None)
        }
    }

    pub fn fullpath(&self) -> String {
        resource_fullpath(RESOURCE_TYPE_SECRET, &self.namespace, &self.name)
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
        let secret = Secret {
            metadata: ObjectMeta {
                name: Some(encode_k8sname(PREFIX, name)),
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
        self.api()
            .get(&encode_k8sname(PREFIX, name))
            .await
            .map(|x| {
                if is_managed_label(RESOURCE_TYPE_SECRET, x.labels()) {
                    Some(x)
                } else {
                    None
                }
            })
            .map_err(AppError::from)
            .and_then(SecretData::try_from_option_secret)
    }

    pub async fn list(&self, queries: &[LabelQuery]) -> Result<Vec<SecretData>, AppError> {
        let selector = build_label_query(RESOURCE_TYPE_SECRET, queries)?;
        let lp = ListParams::default().labels(&selector);
        let list = self.api().list(&lp).await.map_err(AppError::from)?;
        list.items
            .into_iter()
            .map(SecretData::try_from_secret)
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
