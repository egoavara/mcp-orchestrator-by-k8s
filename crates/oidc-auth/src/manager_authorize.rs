use std::collections::HashMap;

use axum::{
    body::Body,
    extract::Query,
    handler::Handler,
    response::{IntoResponse, Redirect, Response},
};
use axum_extra::extract::{
    CookieJar,
    cookie::{Cookie, SameSite},
};
use http::Request;
use openidconnect::{
    ClientId, ClientSecret, CsrfToken, Nonce, PkceCodeChallenge, RedirectUrl, Scope,
    core::CoreAuthenticationFlow,
};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{AuthError, manager::AuthManager};

pub(crate) const PKCE_COOKIE_VERIFIER_KEY: &str = "pkce_verifier";
pub(crate) const PKCE_COOKIE_STATE_KEY: &str = "pkce_state";
pub(crate) const PKCE_COOKIE_REDIRECT_URI_KEY: &str = "redirect_uri";
pub(self) const PKCE_COOKIE_MAX_AGE_SECONDS: i64 = 600;

impl AuthManager {
    pub async fn authorize(
        &self,
        cookiejar: CookieJar,
        query: serde_json::Map<String, serde_json::Value>,
    ) -> Result<Response<Body>, AuthError> {
        let (pkce_challenge, pkce_verifier) = PkceCodeChallenge::new_random_sha256();

        let mut auth_request = self.oidc.authorize_url(
            CoreAuthenticationFlow::AuthorizationCode,
            CsrfToken::new_random,
            Nonce::new_random,
        );

        for scope in &self.config.client.scopes {
            auth_request = auth_request.add_scope(Scope::new(scope.to_string()));
        }
        let (mut auth_url, csrf_token, _nonce) =
            auth_request.set_pkce_challenge(pkce_challenge).url();

        let mut previous_query = auth_url
            .query_pairs()
            .map(|(key, val)| (key.to_string(), serde_json::Value::String(val.to_string())))
            .collect::<HashMap<_, _>>();

        let redirect_uri_from_query = query
            .get("redirect_uri")
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| self.config.client.redirect.clone());

        // PKCE 를 위한 값 제외하고는 query 파라미터에서 가져오기
        for (key, value) in query {
            if previous_query.contains_key(&key) {
                continue;
            }
            previous_query.insert(key.to_string(), value.clone());
        }
        let query_string = serde_qs::to_string(&previous_query)
            .map_err(|e| AuthError::FailedPassthroughQueryParam(e))?;
        auth_url.set_query(Some(&query_string));

        let state = csrf_token.secret().clone();

        let cookie_verifier =
            Cookie::build((PKCE_COOKIE_VERIFIER_KEY, pkce_verifier.secret().clone()))
                .path("/")
                .max_age(time::Duration::seconds(PKCE_COOKIE_MAX_AGE_SECONDS))
                .same_site(SameSite::Lax)
                .http_only(true)
                .build();
        let cookie_state = Cookie::build((PKCE_COOKIE_STATE_KEY, state.clone()))
            .path("/")
            .max_age(time::Duration::seconds(PKCE_COOKIE_MAX_AGE_SECONDS))
            .same_site(SameSite::Lax)
            .http_only(true)
            .build();
        let redirect_uri = Cookie::build((
            PKCE_COOKIE_REDIRECT_URI_KEY,
            redirect_uri_from_query,
        ))
        .path("/")
        .max_age(time::Duration::seconds(PKCE_COOKIE_MAX_AGE_SECONDS))
        .same_site(SameSite::Lax)
        .http_only(true)
        .build();

        let updated_cookies = cookiejar
            .add(cookie_verifier)
            .add(cookie_state)
            .add(redirect_uri);

        Ok((updated_cookies, Redirect::to(&auth_url.to_string())).into_response())
    }
}
