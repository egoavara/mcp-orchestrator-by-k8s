use crate::api::APICaller;
use crate::components::{ErrorMessage, Loading, NamespaceSelector};
use crate::models::secret::Secret;
use crate::models::state::AuthState;
use crate::models::SessionState;
use crate::routes::Route;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[derive(Debug)]
enum LoadState {
    Loading,
    Loaded(Vec<Secret>),
    Error(String),
}

#[function_component(SecretList)]
pub fn secret_list() -> Html {
    let load_state = use_state(|| LoadState::Loading);
    let (session_state, _) = use_store::<SessionState>();
    let (auth_state, _) = use_store::<AuthState>();
    let namespace = session_state.selected_namespace.clone();

    {
        let load_state = load_state.clone();
        let namespace = namespace.clone();
        let api = APICaller::new(auth_state.access_token.clone());
        use_effect_with(namespace.clone(), move |ns| {
            if let Some(namespace) = ns {
                let load_state = load_state.clone();
                let namespace = namespace.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match api.list_secrets(&namespace).await {
                        Ok(secrets) => load_state.set(LoadState::Loaded(secrets)),
                        Err(e) => load_state.set(LoadState::Error(e)),
                    }
                });
            } else {
                load_state.set(LoadState::Error(
                    "Please select a namespace first".to_string(),
                ));
            }
            || ()
        });
    }

    html! {
        <div class="container">
            <NamespaceSelector />

            <div class="header">
                <h1>{ "Secrets" }</h1>
                { if namespace.is_some() {
                    html! {
                        <Link<Route> to={Route::SecretCreate}>
                            <button class="btn-primary">{ "Create Secret" }</button>
                        </Link<Route>>
                    }
                } else {
                    html! {}
                }}
            </div>

            { match &*load_state {
                LoadState::Loading => html! { <Loading /> },
                LoadState::Error(e) => html! { <ErrorMessage message={e.clone()} /> },
                LoadState::Loaded(secrets) => html! {
                    <div class="secret-list">
                        { if secrets.is_empty() {
                            html! {
                                <div class="empty-state">
                                    <p>{ "No secrets found. Create your first secret to get started." }</p>
                                </div>
                            }
                        } else {
                            html! {
                                <div class="grid">
                                    { for secrets.iter().map(|secret| {
                                        let namespace = secret.namespace.clone();
                                        let name = secret.name.clone();
                                        html! {
                                            <div class="card" key={format!("{}/{}", namespace, name)}>
                                                <div class="card-header">
                                                    <h3>{ &secret.name }</h3>
                                                    <span class="namespace-badge">{ &secret.namespace }</span>
                                                </div>
                                                <div class="card-body">
                                                    <div class="field">
                                                        <label>{ "Keys:" }</label>
                                                        <div class="tags">
                                                            { for secret.keys.iter().map(|key| {
                                                                html! {
                                                                    <span class="tag" key={key.clone()}>
                                                                        { key }
                                                                    </span>
                                                                }
                                                            })}
                                                        </div>
                                                    </div>
                                                    <div class="field">
                                                        <label>{ "Created:" }</label>
                                                        <span>{ &secret.created_at }</span>
                                                    </div>
                                                </div>
                                                <div class="card-footer">
                                                    <Link<Route> to={Route::SecretDetail { namespace: namespace.clone(), name: name.clone() }}>
                                                        <button class="btn-secondary">{ "View Details" }</button>
                                                    </Link<Route>>
                                                </div>
                                            </div>
                                        }
                                    }) }
                                </div>
                            }
                        }}
                    </div>
                }
            }}
        </div>
    }
}
