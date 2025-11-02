use crate::api::client::get_base_url;
use crate::models::state::OAuthConfig;
use gloo_net::http::Request;
use serde::Deserialize;

#[derive(Deserialize)]
struct OAuthMetadata {
    issuer: String,
    authorization_endpoint: String,
    token_endpoint: String,
}

pub async fn check_oauth_config() -> Result<Option<OAuthConfig>, String> {
    let url = format!("{}/.well-known/oauth-authorization-server", get_base_url());

    let response = Request::get(&url)
        .send()
        .await
        .map_err(|e| format!("Network error: {}", e))?;

    if response.status() == 404 {
        return Ok(None);
    }

    if !response.ok() {
        return Err(format!(
            "HTTP error: status {} - {}",
            response.status(),
            response.status_text()
        ));
    }

    let metadata: OAuthMetadata = response
        .json()
        .await
        .map_err(|e| format!("JSON parse error: {}", e))?;

    Ok(Some(OAuthConfig {
        issuer: metadata.issuer,
        authorization_endpoint: metadata.authorization_endpoint,
        token_endpoint: metadata.token_endpoint,
    }))
}
