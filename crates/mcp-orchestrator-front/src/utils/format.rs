pub fn format_timestamp(timestamp: &Option<String>) -> String {
    match timestamp {
        Some(ts) if !ts.is_empty() => ts.clone(),
        _ => "N/A".to_string(),
    }
}

pub fn format_relative_time(timestamp: &Option<String>) -> String {
    format_timestamp(timestamp)
}

pub fn truncate_string(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len])
    }
}
