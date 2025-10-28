use crate::{
    error::AppError,
    storage::labels::{LABEL_MANAGED_BY, LABEL_MANAGED_BY_VALUE, LABEL_TYPE_OF, label_fullpath},
};

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
    pub fn to_selector_string(&self) -> Result<String, AppError> {
        match self {
            LabelQuery::Equal { key, value } => Ok(format!("{}={}", label_fullpath(key)?, value)),
            LabelQuery::NotEqual { key, value } => {
                Ok(format!("{}!={}", label_fullpath(key)?, value))
            }
            LabelQuery::In { key, values } => Ok(format!(
                "{} in ({})",
                label_fullpath(key)?,
                values.join(",")
            )),
            LabelQuery::NotIn { key, values } => Ok(format!(
                "{} notin ({})",
                label_fullpath(key)?,
                values.join(",")
            )),
            LabelQuery::ContainKey { key } => Ok(label_fullpath(key)?),
            LabelQuery::NotContainKey { key } => Ok(format!("!{}", label_fullpath(key)?)),
        }
    }
}

pub fn build_label_query(r#typeof: &str, queries: &[LabelQuery]) -> Result<String, AppError> {
    queries
        .iter()
        .map(|q| q.to_selector_string())
        .chain(vec![
            Ok(format!("{}={}", LABEL_MANAGED_BY, LABEL_MANAGED_BY_VALUE)),
            Ok(format!("{}={}", LABEL_TYPE_OF, r#typeof)),
        ])
        .collect::<Result<Vec<_>, _>>()
        .map(|x| {
            let query = x.join(",");
            tracing::trace!("Built label query: {}", query);
            query
        })
}
