use proto_web::NamespaceResponse;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Namespace {
    pub name: String,
    pub labels: HashMap<String, String>,
    pub created_at: String,
    pub deleted_at: Option<String>,
}

impl From<NamespaceResponse> for Namespace {
    fn from(response: NamespaceResponse) -> Self {
        Self {
            name: response.name,
            labels: response.labels,
            created_at: response.created_at,
            deleted_at: response.deleted_at,
        }
    }
}
