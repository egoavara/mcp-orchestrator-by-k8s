pub fn encode_k8sname(prefix: &str, name: &str) -> String {
    format!("{}-{}", prefix, name)
}

pub fn decode_k8sname(prefix: &str, full_name: &str) -> Option<String> {
    full_name
        .strip_prefix(&format!("{}-", prefix))
        .map(|s| s.to_string())
}
