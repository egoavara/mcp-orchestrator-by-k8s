use proto_web::SecretResponse;
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct Secret {
    pub name: String,
    pub namespace: String,
    pub keys: Vec<String>,
    pub labels: HashMap<String, String>,
    pub created_at: String,
    pub deleted_at: Option<String>,
}

impl From<SecretResponse> for Secret {
    fn from(response: SecretResponse) -> Self {
        Self {
            name: response.name,
            namespace: response.namespace,
            keys: response.keys,
            labels: response.labels,
            created_at: response.created_at,
            deleted_at: response.deleted_at,
        }
    }
}
