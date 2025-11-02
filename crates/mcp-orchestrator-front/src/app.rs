use crate::api::auth::check_oauth_config;
use crate::components::Layout;
use crate::models::state::AuthState;
use crate::pages::{
    AuthorizationCreate, AuthorizationDetail, AuthorizationList, Home, NamespaceCreate,
    NamespaceDetail, NamespaceList, OAuthCallback, ResourceLimitCreate, ResourceLimitDetail,
    ResourceLimitList, SecretCreate, SecretDetail, SecretList, SecretUpdate, TemplateDetail,
    TemplateForm, TemplateList,
};
use crate::routes::Route;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[function_component(App)]
pub fn app() -> Html {
    let (auth_state, auth_dispatch) = use_store::<AuthState>();
    let initialized = use_state(|| false);

    {
        let initialized = initialized.clone();
        let auth_dispatch = auth_dispatch.clone();
        use_effect_with((), move |_| {
            if !*initialized {
                wasm_bindgen_futures::spawn_local(async move {
                    match check_oauth_config().await {
                        Ok(oauth_config) => {
                            let oauth_required = oauth_config.is_some();
                            auth_dispatch.reduce_mut(|state| {
                                state.oauth_config = oauth_config;
                                state.oauth_required = oauth_required;
                            });
                            initialized.set(true);
                        }
                        Err(e) => {
                            log::error!("Failed to check OAuth config: {}", e);
                            auth_dispatch.reduce_mut(|state| {
                                state.oauth_required = false;
                            });
                            initialized.set(true);
                        }
                    }
                });
            }
            || ()
        });
    }

    let window = web_sys::window().unwrap();
    let location = window.location();
    let pathname = location.pathname().unwrap_or_default();
    let is_callback = pathname == "/callback";

    if !*initialized {
        return html! {
            <div class="container" style="text-align: center; padding-top: 50px;">
                <p>{"Loading..."}</p>
            </div>
        };
    }

    if !is_callback && auth_state.oauth_required && auth_state.access_token.is_none() {
        if let Some(config) = &auth_state.oauth_config {
            let origin = location.origin().unwrap();
            let redirect_uri = format!("{}/callback", origin);
            let encoded_redirect_uri = urlencoding::encode(&redirect_uri);
            let auth_url = format!(
                "{}?response_type=code&client_id=mcp-orchestrator&redirect_uri={}",
                config.authorization_endpoint, encoded_redirect_uri
            );
            log::info!("Redirecting to OAuth authorization endpoint: {}", auth_url);
            
            return html! {
                <div class="container" style="text-align: center; padding-top: 50px;">
                    <h2>{"Authentication Required"}</h2>
                    <p>{"Please log in to continue"}</p>
                    <a href={auth_url} class="btn btn-primary">{"Log In"}</a>
                </div>
            };
        }
    }

    html! {
        <BrowserRouter>
            <Layout>
                <Switch<Route> render={switch} />
            </Layout>
        </BrowserRouter>
    }
}

fn switch(routes: Route) -> Html {
    match routes {
        Route::Home => html! { <Home /> },
        Route::NamespaceList => html! { <NamespaceList /> },
        Route::NamespaceCreate => html! { <NamespaceCreate /> },
        Route::NamespaceDetail { name } => html! { <NamespaceDetail name={name} /> },
        Route::TemplateList => html! { <TemplateList /> },
        Route::TemplateCreate => html! { <TemplateForm /> },
        Route::TemplateDetail { namespace, name } => html! {
            <TemplateDetail namespace={namespace} name={name} />
        },
        Route::ServerList => {
            html! { <div class="container"><h2>{"Server List - Coming Soon"}</h2></div> }
        }
        Route::ServerDetail { namespace, name } => {
            html! { <div class="container"><h2>{format!("Server Detail: {}/{} - Coming Soon", namespace, name)}</h2></div> }
        }
        Route::SecretList => html! { <SecretList /> },
        Route::SecretCreate => html! { <SecretCreate /> },
        Route::SecretDetail { namespace, name } => {
            html! { <SecretDetail namespace={namespace} name={name} /> }
        }
        Route::SecretEdit { namespace, name } => {
            html! { <SecretUpdate namespace={namespace} name={name} /> }
        }
        Route::ResourceLimitList => html! { <ResourceLimitList /> },
        Route::ResourceLimitCreate => html! { <ResourceLimitCreate /> },
        Route::ResourceLimitDetail { name } => html! { <ResourceLimitDetail name={name} /> },
        Route::AuthorizationList => html! { <AuthorizationList /> },
        Route::AuthorizationCreate => html! { <AuthorizationCreate /> },
        Route::AuthorizationDetail { namespace, name } => {
            html! { <AuthorizationDetail namespace={namespace} name={name} /> }
        }
        Route::OAuthCallback => html! { <OAuthCallback /> },
        Route::NotFound => html! { <div class="container"><h2>{"404 - Page Not Found"}</h2></div> },
    }
}
