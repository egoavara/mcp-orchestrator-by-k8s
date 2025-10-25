
pub const ANNOTATION_PREFIX: &str = "mcp-orchestrator.egoavara.net";
pub const ANNOTATION_DESCRIPTION: &str = "mcp-orchestrator.egoavara.net/description";

pub fn annotation_description(description: &str) -> (String, String) {
    (ANNOTATION_DESCRIPTION.to_string(), description.to_string())
}
