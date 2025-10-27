lazy_static::lazy_static! {
    static ref RESOURCE_UNAME_REGEX: regex::Regex = regex::Regex::new(r"^(?<type>([A-Za-z0-9][-A-Za-z0-9_.]*)?[A-Za-z0-9])\.mcp-orchestrator\.egoavara\.net/(?<name>([A-Za-z0-9][-A-Za-z0-9_.]*)?[A-Za-z0-9])$").unwrap();
}
pub fn resource_relpath(r#typeof: &str, name: &str) -> String {
    format!("{}.mcp-orchestrator.egoavara.net/{}", r#typeof, name)
}

pub fn filter_relpath<S: AsRef<str>>(key: S) -> Option<(String, String)> {
    RESOURCE_UNAME_REGEX
        .captures(key.as_ref())
        .map(|cap| (cap["type"].to_string(), cap["name"].to_string()))
}
