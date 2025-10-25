pub fn resource_relpath(r#typeof: &str, name: &str) -> String {
    format!("{}.mcp-orchestrator.egoavara.net/{}", r#typeof, name)
}

pub fn resource_fullpath(r#typeof: &str, namespace: &str, name: &str) -> String {
    format!(
        "{}.mcp-orchestrator.egoavara.net/{}.{}",
        r#typeof, namespace, name
    )
}
