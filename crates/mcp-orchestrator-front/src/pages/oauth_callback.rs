use crate::api::client::get_base_url;
use crate::models::state::AuthState;
use gloo_net::http::Request;
use serde::Deserialize;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[derive(Deserialize)]
struct CallbackResponse {
    access_token: String,
    id_token: Option<String>,
    refresh_token: Option<String>,
    expires_in: Option<u64>,
}

#[function_component(OAuthCallback)]
pub fn oauth_callback() -> Html {
    let (_, auth_dispatch) = use_store::<AuthState>();
    let navigator = use_navigator().unwrap();
    let error = use_state(|| None::<String>);

    {
        let error = error.clone();
        let auth_dispatch = auth_dispatch.clone();
        let navigator = navigator.clone();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                log::info!("OAuth callback page loaded");
                
                let window = web_sys::window().unwrap();
                let location = window.location();
                let search = location.search().unwrap();
                
                log::info!("URL search params: {}", search);

                let params: Vec<(String, String)> = search
                    .trim_start_matches('?')
                    .split('&')
                    .filter_map(|pair| {
                        let mut parts = pair.split('=');
                        let key = parts.next()?.to_string();
                        let value = parts.next()?.to_string();
                        Some((key, value))
                    })
                    .collect();

                log::info!("Parsed params: {:?}", params);

                let code = params
                    .iter()
                    .find(|(k, _)| k == "code")
                    .map(|(_, v)| v.clone());
                let state = params
                    .iter()
                    .find(|(k, _)| k == "state")
                    .map(|(_, v)| v.clone());

                log::info!("Code: {:?}, State: {:?}", code, state);

                if code.is_none() || state.is_none() {
                    log::error!("Missing code or state parameter");
                    error.set(Some("Missing code or state parameter".to_string()));
                    return;
                }

                let url = format!(
                    "{}/oauth/callback?code={}&state={}",
                    get_base_url(),
                    code.as_ref().unwrap(),
                    state.as_ref().unwrap()
                );

                log::info!("Calling backend token exchange: {}", url);

                match Request::get(&url).send().await {
                    Ok(response) => {
                        log::info!("Received response with status: {}", response.status());
                        
                        if response.ok() {
                            match response.json::<CallbackResponse>().await {
                                Ok(callback_resp) => {
                                    log::info!("Successfully parsed token response");
                                    auth_dispatch.reduce_mut(|state| {
                                        state.access_token = Some(callback_resp.access_token);
                                    });
                                    log::info!("Navigating to home page");
                                    navigator.push(&crate::routes::Route::Home);
                                }
                                Err(e) => {
                                    log::error!("Failed to parse response: {}", e);
                                    error.set(Some(format!("Failed to parse response: {}", e)));
                                }
                            }
                        } else {
                            let status = response.status();
                            let status_text = response.status_text();
                            let body = response.text().await.unwrap_or_default();
                            log::error!("Token exchange failed: {} - {} - Body: {}", status, status_text, body);
                            error.set(Some(format!(
                                "Token exchange failed: {} - {} - {}",
                                status, status_text, body
                            )));
                        }
                    }
                    Err(e) => {
                        log::error!("Network error: {}", e);
                        error.set(Some(format!("Network error: {}", e)));
                    }
                }
            });
            || ()
        });
    }

    html! {
        <div class="container" style="text-align: center; padding-top: 50px;">
            {
                if let Some(err) = (*error).clone() {
                    html! {
                        <>
                            <h2>{"Authentication Failed"}</h2>
                            <p>{err}</p>
                            <a href="/" class="btn btn-primary">{"Go Home"}</a>
                        </>
                    }
                } else {
                    html! {
                        <>
                            <h2>{"Completing Authentication..."}</h2>
                            <p>{"Please wait while we complete your login."}</p>
                        </>
                    }
                }
            }
        </div>
    }
}
