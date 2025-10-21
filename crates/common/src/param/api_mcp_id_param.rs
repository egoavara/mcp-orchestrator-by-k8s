use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct ApiMcpIdParam {
    pub id: String,
}
