use std::collections::BTreeMap;

use proto::mcp::orchestrator::v1::AuthorizationType;

use crate::{
    error::AppError,
    storage::{label_query::LabelQuery, resource_uname::resource_relpath},
};

pub const LABEL_CUSTOM_PREFIX: &str = "custom.mcp-orchestrator.egoavara.net";

pub const LABEL_MANAGED_BY: &str = "app.kubernetes.io/managed-by";
pub const LABEL_MANAGED_BY_VALUE: &str = "mcp-orchestrator";
pub const LABEL_MANAGED_BY_QUERY: &str = "app.kubernetes.io/managed-by=mcp-orchestrator";

pub const LABEL_TYPE_OF: &str = "mcp-orchestrator.egoavara.net/type-of";
pub const LABEL_AUTH_TYPE_OF: &str = "mcp-orchestrator.egoavara.net/auth-type-of";

pub const LABEL_SESSION_ID: &str = "mcp-orchestrator.egoavara.net/session-id";

lazy_static::lazy_static! {
    pub static ref LABEL_REGEX: regex::Regex = regex::Regex::new(r"^(([A-Za-z0-9][-A-Za-z0-9_.]*)?[A-Za-z0-9])/(([A-Za-z0-9][-A-Za-z0-9_.]*)?[A-Za-z0-9])$")
    .unwrap();
    pub static ref SIMPLE_LABEL_REGEX: regex::Regex = regex::Regex::new(r"^(([A-Za-z0-9][-A-Za-z0-9_.]*)?[A-Za-z0-9])$")
    .unwrap();
}

pub fn label_dependency(r#typeof: &str, name: &str) -> impl Iterator<Item = (String, String)> {
    std::iter::once(label_dependency_tuple(r#typeof, name))
}

pub fn label_dependency_query(r#typeof: &str, name: &str) -> LabelQuery {
    let (key, value) = label_dependency_tuple(r#typeof, name);
    LabelQuery::Equal { key, value }
}

pub fn label_dependency_tuple(r#typeof: &str, name: &str) -> (String, String) {
    (resource_relpath(r#typeof, name), "1".to_string())
}

pub fn label_fullpath(key: &str) -> Result<String, AppError> {
    if SIMPLE_LABEL_REGEX.is_match(key) {
        Ok(format!("{}/{}", LABEL_CUSTOM_PREFIX, key))
    } else if LABEL_REGEX.is_match(key) {
        Ok(key.to_string())
    } else {
        Err(AppError::InvalidLabelKey(key.to_string()))
    }
}

pub fn setup_labels<L: Iterator<Item = (String, String)>>(
    r#typeof: &str,
    user_labels: L,
) -> impl Iterator<Item = (String, String)> {
    user_labels
        .map(|(k, v)| (format!("{}/{}", LABEL_CUSTOM_PREFIX, k), v))
        .chain(vec![
            (
                LABEL_MANAGED_BY.to_string(),
                LABEL_MANAGED_BY_VALUE.to_string(),
            ),
            (LABEL_TYPE_OF.to_string(), r#typeof.to_string()),
        ])
}

pub fn is_managed_label(r#typeof: &str, labels: &BTreeMap<String, String>) -> bool {
    let Some(managed_type) = labels.get(LABEL_MANAGED_BY) else {
        return false;
    };
    let Some(target_typeof) = labels.get(LABEL_TYPE_OF) else {
        return false;
    };
    managed_type == LABEL_MANAGED_BY_VALUE && r#typeof == target_typeof
}

#[allow(dead_code)]
pub fn decode_label(
    data: Option<&BTreeMap<String, String>>,
    key: &str,
) -> Result<String, AppError> {
    data.and_then(|d| d.get(key).cloned())
        .ok_or_else(|| AppError::InvalidLabelKey(key.to_string()))
}

#[allow(dead_code)]
pub fn decode_label_map<R, M: FnOnce(&str) -> Result<R, AppError>>(
    data: Option<&BTreeMap<String, String>>,
    key: &str,
    mapper: M,
) -> Result<R, AppError> {
    data.and_then(|d| d.get(key))
        .ok_or_else(|| AppError::InvalidLabelKey(key.to_string()))
        .and_then(|v| mapper(v))
}

pub fn decode_label_optmap<R, M: FnOnce(&str) -> Option<R>>(
    data: Option<&BTreeMap<String, String>>,
    key: &str,
    mapper: M,
) -> Result<R, AppError> {
    data.and_then(|d| d.get(key))
        .ok_or_else(|| AppError::InvalidLabelKey(key.to_string()))
        .and_then(|v| {
            mapper(v).ok_or_else(|| AppError::InvalidLabelValue {
                value: v.to_string(),
                key: key.to_string(),
            })
        })
}

pub fn label_auth_type_of(auth_type: AuthorizationType) -> (String, String) {
    (
        LABEL_AUTH_TYPE_OF.to_string(),
        auth_type.as_str_name().to_string(),
    )
}
