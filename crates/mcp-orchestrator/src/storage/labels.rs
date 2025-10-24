use std::collections::BTreeMap;

pub const LABEL_PREFIX: &str = "mcp-orchestrator.egoavara.net";
pub const LABEL_CUSTOM_PREFIX: &str = "custom.mcp-orchestrator.egoavara.net";

pub const LABEL_MANAGED_BY: &str = "app.kubernetes.io/managed-by";
pub const LABEL_MANAGED_BY_VALUE: &str = "mcp-orchestrator";

pub const LABEL_TYPE_OF: &str = "mcp-orchestrator.egoavara.net/type-of";

pub const TYPE_MCP_TEMPLATE: &str = "mcp-template";
pub const TYPE_RESOURCE_LIMIT: &str = "resource-limit";
pub const TYPE_MCP_SERVER: &str = "mcp-server";

pub fn add_prefix_to_user_labels(user_labels: BTreeMap<String, String>) -> BTreeMap<String, String> {
    user_labels
        .into_iter()
        .map(|(k, v)| (format!("{}/{}", LABEL_CUSTOM_PREFIX, k), v))
        .collect()
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
