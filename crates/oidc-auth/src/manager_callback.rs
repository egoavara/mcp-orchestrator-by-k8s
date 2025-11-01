use std::borrow::Cow;

use axum::{
    Json,
    body::Body,
    extract::Query,
    response::{IntoResponse, IntoResponseParts, Response, ResponseParts},
};
use axum_extra::extract::{CookieJar, cookie::Cookie};
use http::StatusCode;
use kube::client::AuthError;
use openidconnect::{AuthorizationCode, OAuth2TokenResponse, PkceCodeVerifier, RedirectUrl};
use serde::{Deserialize, Serialize};

use crate::{
    manager::AuthManager,
    manager_authorize::{
        PKCE_COOKIE_REDIRECT_URI_KEY, PKCE_COOKIE_STATE_KEY, PKCE_COOKIE_VERIFIER_KEY,
    },
};

#[derive(Debug, Deserialize)]
pub struct CallbackQuery {
    pub code: String,
    pub state: String,
}

#[derive(Debug, Serialize)]
pub struct CallbackResponse {
    pub access_token: String,
    pub id_token: Option<String>,
    pub refresh_token: Option<String>,
    pub expires_in: Option<u64>,
}

impl AuthManager {
    pub async fn callback(
        &self,
        cookies: CookieJar,
        Query(query): Query<CallbackQuery>,
    ) -> Result<Response<Body>, Response<Body>> {
        let cookie_verifier = cookies.get(PKCE_COOKIE_VERIFIER_KEY).ok_or_else(|| {
            (StatusCode::UNAUTHORIZED, "Missing PKCE verifier cookie").into_response()
        })?;
        let cookie_state = cookies.get(PKCE_COOKIE_STATE_KEY).ok_or_else(|| {
            (StatusCode::UNAUTHORIZED, "Missing PKCE state cookie").into_response()
        })?;
        let cookie_redirect_uri = cookies.get(PKCE_COOKIE_REDIRECT_URI_KEY).ok_or_else(|| {
            (StatusCode::UNAUTHORIZED, "Missing PKCE redirect URI cookie").into_response()
        })?;

        let state = cookie_state.value().to_string();
        let verifier = cookie_verifier.value().to_string();
        let redirect_uri =
            RedirectUrl::new(cookie_redirect_uri.value().to_string()).map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Invalid redirect URI in cookie: {}", e),
                )
                    .into_response()
            })?;

        if state != query.state {
            return Err((StatusCode::UNAUTHORIZED, "State mismatch").into_response());
        }

        let pkce_verifier = PkceCodeVerifier::new(verifier);

        let token_response = self
            .oidc
            .exchange_code(AuthorizationCode::new(query.code))
            .map_err(|e| {
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    format!("Failed to create token request: {}", e),
                )
                    .into_response()
            })?
            .set_pkce_verifier(pkce_verifier)
            .set_redirect_uri(Cow::Borrowed(&redirect_uri))
            .request_async(&self.client)
            .await
            .map_err(|e| {
                (
                    StatusCode::UNAUTHORIZED,
                    format!("Token exchange failed: {}", e),
                )
                    .into_response()
            })?;

        let token_resp = CallbackResponse {
            access_token: token_response.access_token().secret().clone(),
            id_token: token_response
                .extra_fields()
                .id_token()
                .map(|t| t.to_string()),
            refresh_token: token_response.refresh_token().map(|t| t.secret().clone()),
            expires_in: token_response.expires_in().map(|d| d.as_secs()),
        };

        let response_cookies = cookies
            .remove(Cookie::from(PKCE_COOKIE_VERIFIER_KEY))
            .remove(Cookie::from(PKCE_COOKIE_STATE_KEY))
            .remove(Cookie::from(PKCE_COOKIE_REDIRECT_URI_KEY));



        Ok((response_cookies, Json(token_resp)).into_response())
    }
}
