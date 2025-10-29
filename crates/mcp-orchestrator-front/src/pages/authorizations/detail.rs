use crate::api::authorizations::{delete_authorization, get_authorization};
use crate::components::{ConfirmDialog, ErrorMessage, Loading};
use crate::models::authorization::Authorization;
use crate::routes::Route;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub namespace: String,
    pub name: String,
}

#[derive(Debug)]
enum LoadState {
    Loading,
    Loaded(Authorization),
    Error(String),
}

#[function_component(AuthorizationDetail)]
pub fn authorization_detail(props: &Props) -> Html {
    let load_state = use_state(|| LoadState::Loading);
    let show_delete_confirm = use_state(|| false);
    let is_deleting = use_state(|| false);
    let delete_error = use_state(|| Option::<String>::None);
    let navigator = use_navigator().unwrap();

    let namespace = props.namespace.clone();
    let name = props.name.clone();

    {
        let load_state = load_state.clone();
        let namespace = namespace.clone();
        let name = name.clone();
        use_effect_with((namespace.clone(), name.clone()), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match get_authorization(namespace, name).await {
                    Ok(authorization) => load_state.set(LoadState::Loaded(authorization)),
                    Err(e) => load_state.set(LoadState::Error(e)),
                }
            });
            || ()
        });
    }

    let on_delete_click = {
        let show_delete_confirm = show_delete_confirm.clone();
        Callback::from(move |_| {
            show_delete_confirm.set(true);
        })
    };

    let on_delete_confirm = {
        let is_deleting = is_deleting.clone();
        let delete_error = delete_error.clone();
        let show_delete_confirm = show_delete_confirm.clone();
        let navigator = navigator.clone();
        let namespace = namespace.clone();
        let name = name.clone();

        Callback::from(move |_| {
            is_deleting.set(true);
            let is_deleting = is_deleting.clone();
            let delete_error = delete_error.clone();
            let show_delete_confirm = show_delete_confirm.clone();
            let navigator = navigator.clone();
            let namespace = namespace.clone();
            let name = name.clone();

            wasm_bindgen_futures::spawn_local(async move {
                match delete_authorization(namespace, name).await {
                    Ok(_) => {
                        navigator.push(&Route::AuthorizationList);
                    }
                    Err(e) => {
                        delete_error.set(Some(e));
                        is_deleting.set(false);
                        show_delete_confirm.set(false);
                    }
                }
            });
        })
    };

    let on_delete_cancel = {
        let show_delete_confirm = show_delete_confirm.clone();
        Callback::from(move |_| {
            show_delete_confirm.set(false);
        })
    };

    html! {
        <div class="container">
            <div class="header">
                <h1>{ "Authorization Details" }</h1>
                <div style="display: flex; gap: 0.75rem;">
                    <button
                        class="btn-danger"
                        onclick={on_delete_click}
                        disabled={*is_deleting}
                    >
                        { "Delete Authorization" }
                    </button>
                    <Link<Route> to={Route::AuthorizationList}>
                        <button class="btn-secondary">{ "Back to List" }</button>
                    </Link<Route>>
                </div>
            </div>

            { if let Some(error) = &*delete_error {
                html! { <ErrorMessage message={error.clone()} /> }
            } else { html! {} }}

            { match &*load_state {
                LoadState::Loading => html! { <Loading /> },
                LoadState::Error(e) => html! { <ErrorMessage message={e.clone()} /> },
                LoadState::Loaded(authorization) => {
                    let type_name = match authorization.auth_type {
                        0 => "Anonymous",
                        1 => "Kubernetes Service Account",
                        _ => "Unknown",
                    };
                    html! {
                        <div class="detail-card">
                            <div class="detail-section">
                                <h2>{ "Basic Information" }</h2>
                                <div class="detail-grid">
                                    <div class="detail-field">
                                        <label>{ "Name:" }</label>
                                        <span>{ &authorization.name }</span>
                                    </div>
                                    <div class="detail-field">
                                        <label>{ "Namespace:" }</label>
                                        <span>{ &authorization.namespace }</span>
                                    </div>
                                    <div class="detail-field">
                                        <label>{ "Type:" }</label>
                                        <span class="tag">{ type_name }</span>
                                    </div>
                                    <div class="detail-field">
                                        <label>{ "Created At:" }</label>
                                        <span>{ &authorization.created_at }</span>
                                    </div>
                                </div>
                            </div>

                            { if !authorization.data.is_empty() && authorization.data != "null" {
                                html! {
                                    <div class="detail-section">
                                        <h2>{ "Configuration Data" }</h2>
                                        <pre class="code-block">{ &authorization.data }</pre>
                                    </div>
                                }
                            } else {
                                html! {}
                            }}

                            { if !authorization.labels.is_empty() {
                                html! {
                                    <div class="detail-section">
                                        <h2>{ "Labels" }</h2>
                                        <div class="tags">
                                            { for authorization.labels.iter().map(|(k, v)| {
                                                html! {
                                                    <span class="tag" key={k.clone()}>
                                                        { format!("{}={}", k, v) }
                                                    </span>
                                                }
                                            })}
                                        </div>
                                    </div>
                                }
                            } else {
                                html! {}
                            }}
                        </div>
                    }
                }
            }}

            <ConfirmDialog
                title="Delete Authorization"
                message={format!("Are you sure you want to delete authorization '{}'? This action cannot be undone.", &name)}
                on_confirm={on_delete_confirm}
                on_cancel={on_delete_cancel}
                show={*show_delete_confirm}
            />
        </div>
    }
}
