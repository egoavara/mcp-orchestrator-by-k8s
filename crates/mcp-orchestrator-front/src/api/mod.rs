pub mod auth;
pub mod authorizations;
pub mod client;
pub mod namespaces;
pub mod resource_limits;
pub mod secrets;
pub mod templates;

#[derive(Clone, PartialEq)]
pub struct APICaller {
    pub access_token: Option<String>,
}

impl APICaller {
    pub fn new(access_token: Option<String>) -> Self {
        Self { access_token }
    }
}
