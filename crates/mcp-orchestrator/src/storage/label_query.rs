use super::labels::LABEL_CUSTOM_PREFIX;

#[derive(Debug, Clone)]
pub enum LabelQuery {
    Equal { key: String, value: String },
    NotEqual { key: String, value: String },
    In { key: String, values: Vec<String> },
    NotIn { key: String, values: Vec<String> },
    ContainKey { key: String },
    NotContainKey { key: String },
}

impl LabelQuery {
    pub fn with_prefix(self) -> Self {
        match self {
            LabelQuery::Equal { key, value } => LabelQuery::Equal {
                key: add_prefix_to_key(&key),
                value,
            },
            LabelQuery::NotEqual { key, value } => LabelQuery::NotEqual {
                key: add_prefix_to_key(&key),
                value,
            },
            LabelQuery::In { key, values } => LabelQuery::In {
                key: add_prefix_to_key(&key),
                values,
            },
            LabelQuery::NotIn { key, values } => LabelQuery::NotIn {
                key: add_prefix_to_key(&key),
                values,
            },
            LabelQuery::ContainKey { key } => LabelQuery::ContainKey {
                key: add_prefix_to_key(&key),
            },
            LabelQuery::NotContainKey { key } => LabelQuery::NotContainKey {
                key: add_prefix_to_key(&key),
            },
        }
    }
}

fn add_prefix_to_key(key: &str) -> String {
    format!("{}/{}", LABEL_CUSTOM_PREFIX, key)
}

impl LabelQuery {
    pub fn to_selector_string(&self) -> String {
        match self {
            LabelQuery::Equal { key, value } => format!("{}={}", key, value),
            LabelQuery::NotEqual { key, value } => format!("{}!={}", key, value),
            LabelQuery::In { key, values } => {
                format!("{} in ({})", key, values.join(","))
            }
            LabelQuery::NotIn { key, values } => {
                format!("{} notin ({})", key, values.join(","))
            }
            LabelQuery::ContainKey { key } => key.clone(),
            LabelQuery::NotContainKey { key } => format!("!{}", key),
        }
    }
}

pub fn build_label_selector(queries: &[LabelQuery]) -> String {
    queries
        .iter()
        .map(|q| q.to_selector_string())
        .collect::<Vec<_>>()
        .join(",")
}
