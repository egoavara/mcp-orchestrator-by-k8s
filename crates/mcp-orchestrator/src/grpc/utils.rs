use std::collections::BTreeMap;

use crate::storage::{label_query::LabelQuery, labels::label_filter};
use proto::mcp::orchestrator::v1::{self, LabelKeyValue, LabelKeyValues};

pub fn convert_label_query(label: v1::LabelQuery) -> Vec<LabelQuery> {
    let mut queries = Vec::new();
    for LabelKeyValue { key, value } in label.equal {
        queries.push(LabelQuery::Equal { key, value });
    }
    for LabelKeyValue { key, value } in label.not_equal {
        queries.push(LabelQuery::NotEqual { key, value });
    }
    for key in label.contain_key {
        queries.push(LabelQuery::ContainKey { key });
    }
    for key in label.not_contain_key {
        queries.push(LabelQuery::NotContainKey { key });
    }
    for LabelKeyValues { key, values } in label.r#in {
        queries.push(LabelQuery::In { key, values });
    }
    for LabelKeyValues { key, values } in label.not_in {
        queries.push(LabelQuery::NotIn { key, values });
    }
    queries
}

pub fn convert_label(
    labels: Option<BTreeMap<String, String>>,
) -> std::collections::HashMap<String, String> {
    if let Some(labels) = labels {
        return labels
            .into_iter()
            .filter_map(label_filter)
            .collect::<std::collections::HashMap<String, String>>();
    }
    Default::default()
}
