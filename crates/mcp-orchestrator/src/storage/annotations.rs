pub const ANNOTATION_DESCRIPTION: &str = "mcp-orchestrator.egoavara.net/description";
pub const ANNOTATION_LAST_ACCESS_AT: &str = "mcp-orchestrator.egoavara.net/last-access-at";

pub fn annotation_description(description: &str) -> (String, String) {
    (ANNOTATION_DESCRIPTION.to_string(), description.to_string())
}
