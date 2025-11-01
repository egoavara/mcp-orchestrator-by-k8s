use crate::api::APICaller;
use crate::components::{ConfirmDialog, ErrorMessage, Loading};
use crate::models::secret::Secret;
use crate::models::state::AuthState;
use crate::routes::Route;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub namespace: String,
    pub name: String,
}

#[derive(Debug)]
enum LoadState {
    Loading,
    Loaded(Secret),
    Error(String),
}

#[function_component(SecretDetail)]
pub fn secret_detail(props: &Props) -> Html {
    let load_state = use_state(|| LoadState::Loading);
    let show_delete_confirm = use_state(|| false);
    let is_deleting = use_state(|| false);
    let delete_error = use_state(|| Option::<String>::None);
    let navigator = use_navigator().unwrap();
    let (auth_state, _) = use_store::<AuthState>();

    let namespace = props.namespace.clone();
    let name = props.name.clone();

    {
        let load_state = load_state.clone();
        let namespace = namespace.clone();
        let name = name.clone();
        let auth_state = auth_state.clone();
        use_effect_with((namespace.clone(), name.clone()), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let api = APICaller::new(auth_state.access_token.clone());
                match api.get_secret(&namespace, &name).await {
                    Ok(secret) => load_state.set(LoadState::Loaded(secret)),
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
        let auth_state = auth_state.clone();

        Callback::from(move |_| {
            is_deleting.set(true);
            let is_deleting = is_deleting.clone();
            let delete_error = delete_error.clone();
            let show_delete_confirm = show_delete_confirm.clone();
            let navigator = navigator.clone();
            let namespace = namespace.clone();
            let name = name.clone();
            let auth_state = auth_state.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let api = APICaller::new(auth_state.access_token.clone());
                match api.delete_secret(&namespace, &name).await {
                    Ok(_) => {
                        navigator.push(&Route::SecretList);
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
                <h1>{ "Secret Details" }</h1>
                <div style="display: flex; gap: 0.75rem;">
                    <Link<Route> to={Route::SecretEdit { namespace: namespace.clone(), name: name.clone() }}>
                        <button class="btn-primary">{ "Edit Secret" }</button>
                    </Link<Route>>
                    <button
                        class="btn-danger"
                        onclick={on_delete_click}
                        disabled={*is_deleting}
                    >
                        { "Delete Secret" }
                    </button>
                    <Link<Route> to={Route::SecretList}>
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
                LoadState::Loaded(secret) => html! {
                    <div class="detail-card">
                        <div class="detail-section">
                            <h2>{ "Basic Information" }</h2>
                            <div class="detail-grid">
                                <div class="detail-field">
                                    <label>{ "Name:" }</label>
                                    <span>{ &secret.name }</span>
                                </div>
                                <div class="detail-field">
                                    <label>{ "Namespace:" }</label>
                                    <span>{ &secret.namespace }</span>
                                </div>
                                <div class="detail-field">
                                    <label>{ "Created At:" }</label>
                                    <span>{ &secret.created_at }</span>
                                </div>
                            </div>
                        </div>

                        <div class="detail-section">
                            <h2>{ "Secret Keys (Values Hidden)" }</h2>
                            <div class="tags">
                                { for secret.keys.iter().map(|key| {
                                    html! {
                                        <span class="tag" key={key.clone()}>
                                            { key }
                                        </span>
                                    }
                                })}
                            </div>
                            <p class="form-help">{ "Secret values are never exposed through the UI for security reasons" }</p>
                        </div>

                        { if !secret.labels.is_empty() {
                            html! {
                                <div class="detail-section">
                                    <h2>{ "Labels" }</h2>
                                    <div class="tags">
                                        { for secret.labels.iter().map(|(k, v)| {
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
            }}

            <ConfirmDialog
                title="Delete Secret"
                message={format!("Are you sure you want to delete secret '{}'? This action cannot be undone.", &name)}
                on_confirm={on_delete_confirm}
                on_cancel={on_delete_cancel}
                show={*show_delete_confirm}
            />
        </div>
    }
}
