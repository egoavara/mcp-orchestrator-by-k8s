use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Authorization {
    pub namespace: String,
    pub name: String,
    pub labels: HashMap<String, String>,
    #[serde(rename = "type")]
    pub auth_type: i32,
    pub data: String,
    pub created_at: String,
    pub deleted_at: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AuthorizationFormData {
    pub namespace: Option<String>,
    pub name: String,
    pub labels: HashMap<String, String>,
    #[serde(rename = "type")]
    pub auth_type: i32,
    pub data: Option<String>,
}

impl Default for AuthorizationFormData {
    fn default() -> Self {
        Self {
            namespace: None,
            name: String::new(),
            labels: HashMap::new(),
            auth_type: 1,
            data: None,
        }
    }
}

impl AuthorizationFormData {
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self::default()
    }
}
