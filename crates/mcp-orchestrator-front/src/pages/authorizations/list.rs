use crate::api::authorizations::list_authorizations;
use crate::components::{ErrorMessage, Loading, NamespaceSelector};
use crate::models::authorization::Authorization;
use crate::models::SessionState;
use crate::routes::Route;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[derive(Debug)]
enum LoadState {
    Loading,
    Loaded(Vec<Authorization>),
    Error(String),
}

#[function_component(AuthorizationList)]
pub fn authorization_list() -> Html {
    let load_state = use_state(|| LoadState::Loading);
    let (session_state, _) = use_store::<SessionState>();
    let namespace = session_state.selected_namespace.clone();

    {
        let load_state = load_state.clone();
        let namespace = namespace.clone();
        use_effect_with(namespace.clone(), move |ns| {
            let load_state = load_state.clone();
            let namespace = ns.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match list_authorizations(namespace, None).await {
                    Ok(authorizations) => load_state.set(LoadState::Loaded(authorizations)),
                    Err(e) => load_state.set(LoadState::Error(e)),
                }
            });
            || ()
        });
    }

    html! {
        <div class="container">
            <NamespaceSelector />

            <div class="header">
                <h1>{ "Authorizations" }</h1>
                { if namespace.is_some() {
                    html! {
                        <Link<Route> to={Route::AuthorizationCreate}>
                            <button class="btn-primary">{ "Create Authorization" }</button>
                        </Link<Route>>
                    }
                } else {
                    html! {}
                }}
            </div>

            { match &*load_state {
                LoadState::Loading => html! { <Loading /> },
                LoadState::Error(e) => html! { <ErrorMessage message={e.clone()} /> },
                LoadState::Loaded(authorizations) => html! {
                    <div class="authorization-list">
                        { if authorizations.is_empty() {
                            html! {
                                <div class="empty-state">
                                    <p>{ "No authorizations found. Create your first authorization to get started." }</p>
                                </div>
                            }
                        } else {
                            html! {
                                <div class="grid">
                                    { for authorizations.iter().map(|auth| {
                                        let namespace = auth.namespace.clone();
                                        let name = auth.name.clone();
                                        let type_name = match auth.auth_type {
                                            0 => "Anonymous",
                                            1 => "Kubernetes Service Account",
                                            _ => "Unknown",
                                        };
                                        html! {
                                            <div class="card" key={format!("{}/{}", namespace, name)}>
                                                <div class="card-header">
                                                    <h3>{ &auth.name }</h3>
                                                    <span class="namespace-badge">{ &auth.namespace }</span>
                                                </div>
                                                <div class="card-body">
                                                    <div class="field">
                                                        <label>{ "Type:" }</label>
                                                        <span class="tag">{ type_name }</span>
                                                    </div>
                                                    <div class="field">
                                                        <label>{ "Created:" }</label>
                                                        <span>{ &auth.created_at }</span>
                                                    </div>
                                                </div>
                                                <div class="card-footer">
                                                    <Link<Route> to={Route::AuthorizationDetail { namespace: namespace.clone(), name: name.clone() }}>
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
