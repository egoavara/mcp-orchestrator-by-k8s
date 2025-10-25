use std::collections::BTreeMap;


use crate::{error::AppError, storage::resource_uname::resource_relpath};

pub const LABEL_PREFIX: &str = "mcp-orchestrator.egoavara.net";
pub const LABEL_CUSTOM_PREFIX: &str = "custom.mcp-orchestrator.egoavara.net";

pub const LABEL_MANAGED_BY: &str = "app.kubernetes.io/managed-by";
pub const LABEL_MANAGED_BY_VALUE: &str = "mcp-orchestrator";
pub const LABEL_MANAGED_BY_QUERY: &str = "app.kubernetes.io/managed-by=mcp-orchestrator";

pub const LABEL_TYPE_OF: &str = "mcp-orchestrator.egoavara.net/type-of";

pub const TYPE_MCP_TEMPLATE: &str = "mcp-template";
pub const TYPE_RESOURCE_LIMIT: &str = "resource-limit";
pub const TYPE_MCP_SERVER: &str = "mcp-server";

lazy_static::lazy_static! {
    pub static ref LABEL_REGEX: regex::Regex = regex::Regex::new(r"^(([A-Za-z0-9][-A-Za-z0-9_.]*)?[A-Za-z0-9])/(([A-Za-z0-9][-A-Za-z0-9_.]*)?[A-Za-z0-9])$")
    .unwrap();
    pub static ref SIMPLE_LABEL_REGEX: regex::Regex = regex::Regex::new(r"^(([A-Za-z0-9][-A-Za-z0-9_.]*)?[A-Za-z0-9])$")
    .unwrap();
}

pub fn label_dependency(r#typeof: &str, name: &str) -> impl Iterator<Item = (String, String)> {
    std::iter::once(label_dependency_tuple(r#typeof, name))
}
pub fn label_dependency_tuple(r#typeof: &str, name: &str) -> (String, String) {
    (
        resource_relpath(r#typeof, name),
        "1".to_string(),
    )
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
pub fn label_filter<A: AsRef<str>, B: AsRef<str>>(keyval: (A, B)) -> Option<(A, B)> {
    let (key, value) = keyval;
    if LABEL_REGEX.is_match(key.as_ref()) {
        Some((key, value))
    } else {
        None
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

pub fn remove_prefix_from_labels(labels: BTreeMap<String, String>) -> BTreeMap<String, String> {
    labels
        .into_iter()
        .filter_map(|(k, v)| {
            if k == LABEL_MANAGED_BY || k == LABEL_TYPE_OF {
                None
            } else if let Some(stripped) = k.strip_prefix(&format!("{}/", LABEL_CUSTOM_PREFIX)) {
                Some((stripped.to_string(), v))
            } else {
                Some((k, v))
            }
        })
        .collect()
}
