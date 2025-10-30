use std::collections::{BTreeMap, HashMap};
use std::f32::consts::E;
use std::vec;

use super::label_query::LabelQuery;
use super::labels::setup_labels;
use crate::storage::SecretData;
use crate::storage::annotations::{ANNOTATION_DESCRIPTION, annotation_description};
use crate::storage::label_query::build_label_query;
use crate::storage::labels::{
    LABEL_AUTH_TYPE_OF, decode_label, decode_label_map, decode_label_optmap, label_auth_type_of,
    label_dependency, label_dependency_query, label_dependency_tuple,
};
use crate::storage::resource_type::{
    RESOURCE_TYPE_AUTHORIZATION, RESOURCE_TYPE_NAMESPACE, RESOURCE_TYPE_PREFIX_AUTHORIZATION,
    RESOURCE_TYPE_PREFIX_AUTHORIZATION_SA,
};
use crate::storage::util_list::ListOption;
use crate::storage::util_name::{decode_k8sname, encode_k8sname};
use crate::storage::utils::{
    add_safe_finalizer, data_elem, data_elem_jsonstr, data_elem_ojsonstr, data_secret,
    del_safe_finalizer, parse_data_elem, parse_secret_elem, pick_created_at, pick_deleted_at,
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
use k8s_openapi::api::authentication::v1::{TokenRequest, TokenRequestSpec};
use k8s_openapi::api::core::v1::{
    Affinity, ConfigMap, ResourceRequirements, Secret, ServiceAccount,
};
use k8s_openapi::api::flowcontrol::v1::ServiceAccountSubject;
use k8s_openapi::apimachinery::pkg::api::resource::Quantity;
use k8s_openapi::apimachinery::pkg::apis::meta::v1::OwnerReference;
use kube::Resource;
use kube::{
    Api, Client, ResourceExt,
    api::{DeleteParams, ListParams, ObjectMeta, PostParams},
};
use proto::mcp::orchestrator::v1::{self, AuthorizationType, VolumeLimit};

const DATA_DATA: &str = "data";
const DATA_SA_NAME: &str = "service_account_name";

pub struct AuthorizationData {
    pub raw: Secret,
    pub namespace: String,
    pub labels: HashMap<String, String>,
    pub name: String,
    pub r#type: AuthorizationType,
    pub data: serde_json::Value,
    pub sa_name: Option<String>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl AuthorizationData {
    pub fn try_from_secret(raw: Secret) -> Result<Self, AppError> {
        Ok(Self {
            raw: raw.clone(),
            namespace: raw.namespace().unwrap_or_else(|| "default".to_string()),
            name: decode_k8sname(RESOURCE_TYPE_PREFIX_AUTHORIZATION, &raw.name_any()).ok_or_else(
                || {
                    AppError::Internal(format!(
                        "Failed to decode secret name: {}, it must start with {}-",
                        raw.name_any(),
                        RESOURCE_TYPE_PREFIX_AUTHORIZATION
                    ))
                },
            )?,
            labels: raw
                .labels()
                .iter()
                .map(|(k, v)| (k.clone(), v.clone()))
                .collect(),
            r#type: decode_label_optmap(
                raw.metadata.labels.as_ref(),
                LABEL_AUTH_TYPE_OF,
                AuthorizationType::from_str_name,
            )?,
            data: parse_secret_elem(&raw.data, "data")?,
            sa_name: parse_secret_elem(&raw.data, "service_account_name")?,
            created_at: pick_created_at(&raw),
            deleted_at: pick_deleted_at(&raw),
        })
    }
}

pub struct AuthorizationStore {
    client: Client,
    namespace: String,
}

impl AuthorizationStore {
    pub fn new(client: Client, namespace: String) -> Self {
        Self { client, namespace }
    }

    fn api(&self) -> Api<Secret> {
        Api::namespaced(self.client.clone(), &self.namespace)
    }

    pub async fn create<L: Iterator<Item = (String, String)>>(
        &self,
        name: &str,
        labels: L,
        r#type: AuthorizationType,
        data: &serde_json::Value,
    ) -> Result<AuthorizationData, AppError> {
        let api_secret = self.api();
        let api_sa = Api::<ServiceAccount>::namespaced(self.client.clone(), &self.namespace);
        let sa_name = encode_k8sname(RESOURCE_TYPE_PREFIX_AUTHORIZATION_SA, name);
        let data_vec = vec![
            data_secret(DATA_DATA, data)?,
            data_secret(DATA_SA_NAME, &sa_name)?,
        ];

        let secret = api_secret
            .create(
                &PostParams::default(),
                &Secret {
                    metadata: ObjectMeta {
                        name: Some(encode_k8sname(RESOURCE_TYPE_PREFIX_AUTHORIZATION, name)),
                        labels: Some(
                            setup_labels(RESOURCE_TYPE_AUTHORIZATION, labels)
                                .chain(vec![
                                    label_dependency_tuple(
                                        RESOURCE_TYPE_NAMESPACE,
                                        &self.namespace,
                                    ),
                                    label_auth_type_of(r#type),
                                ])
                                .collect(),
                        ),
                        ..Default::default()
                    },
                    data: Some(data_vec.into_iter().collect()),
                    ..Default::default()
                },
            )
            .await
            .map_err(AppError::from)?;

        let sa_owner_ref = secret.controller_owner_ref(&()).ok_or_else(|| {
            AppError::Internal(format!(
                "Failed to set owner reference for service account: {}",
                name
            ))
        })?;
        api_sa
            .create(
                &PostParams::default(),
                &ServiceAccount {
                    metadata: ObjectMeta {
                        name: Some(sa_name),
                        owner_references: Some(vec![OwnerReference { ..sa_owner_ref }]),
                        ..Default::default()
                    },
                    ..Default::default()
                },
            )
            .await
            .map_err(AppError::from)?;

        AuthorizationData::try_from_secret(secret)
    }

    pub async fn get(&self, name: &str) -> Result<Option<AuthorizationData>, AppError> {
        let name = encode_k8sname(RESOURCE_TYPE_PREFIX_AUTHORIZATION, name);
        self.api()
            .get_opt(&name)
            .await
            .map_err(AppError::from)?
            .and_then(|x| {
                if is_managed_label(RESOURCE_TYPE_AUTHORIZATION, x.labels()) {
                    Some(x)
                } else {
                    None
                }
            })
            .map(AuthorizationData::try_from_secret)
            .transpose()
    }

    pub async fn list(
        &self,
        subtype: Option<AuthorizationType>,
        queries: &[LabelQuery],
        option: ListOption,
    ) -> Result<(Vec<AuthorizationData>, Option<String>, bool), AppError> {
        let mut queries = queries.to_vec();
        if let Some(subtype) = subtype {
            queries.push(LabelQuery::equal(
                LABEL_AUTH_TYPE_OF,
                AuthorizationType::Anonymous.as_str_name(),
            ));
        }
        let label_query = build_label_query(RESOURCE_TYPE_AUTHORIZATION, &queries)?;
        let lp = option.to_list_param(label_query);
        let list = self.api().list(&lp).await.map_err(AppError::from)?;
        Ok((
            list.items
                .into_iter()
                .take(option.get_limit())
                .map(AuthorizationData::try_from_secret)
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
        let name = encode_k8sname(RESOURCE_TYPE_PREFIX_AUTHORIZATION, name);
        let api = self.api();
        let option = option.unwrap_or_default();

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
                // TODO: get_opt 사용 전환
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

    pub async fn generate_token(
        &self,
        name: &str,
        audience: &str,
        duration: Option<Duration>,
    ) -> Result<(String, DateTime<Utc>), AppError> {
        let audience = audience.to_string();
        let token = Api::<ServiceAccount>::namespaced(self.client.clone(), &self.namespace)
            .create_token_request(
                &encode_k8sname(RESOURCE_TYPE_PREFIX_AUTHORIZATION_SA, name),
                &PostParams::default(),
                &TokenRequest {
                    spec: TokenRequestSpec {
                        audiences: vec![audience],
                        expiration_seconds: duration.map(|d| d.num_seconds() as i64),
                        bound_object_ref: None,
                    },
                    ..Default::default()
                },
            )
            .await
            .map_err(|e| AppError::Internal(format!("Failed to get service account: {}", e)))?;
        let Some(status) = &token.status else {
            return Err(AppError::Internal(
                "TokenRequest status is missing".to_string(),
            ));
        };
        Ok((status.token.clone(), status.expiration_timestamp.0.clone()))
    }
}
