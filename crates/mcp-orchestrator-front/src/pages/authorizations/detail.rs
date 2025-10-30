use crate::api::authorizations::{delete_authorization, generate_token, get_authorization};
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

    let show_generate_token = use_state(|| false);
    let is_generating_token = use_state(|| false);
    let token_result = use_state(|| Option::<(String, Option<String>)>::None);
    let token_error = use_state(|| Option::<String>::None);
    let expire_days = use_state(|| String::from("7"));

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

    let on_generate_token_click = {
        let show_generate_token = show_generate_token.clone();
        let token_result = token_result.clone();
        let token_error = token_error.clone();
        Callback::from(move |_| {
            show_generate_token.set(true);
            token_result.set(None);
            token_error.set(None);
        })
    };

    let on_expire_days_change = {
        let expire_days = expire_days.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            expire_days.set(input.value());
        })
    };

    let on_generate_token_submit = {
        let is_generating_token = is_generating_token.clone();
        let token_result = token_result.clone();
        let token_error = token_error.clone();
        let expire_days = expire_days.clone();
        let namespace = namespace.clone();
        let name = name.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            let days = (*expire_days).parse::<i64>().ok();
            if days.is_none() || days.unwrap() < 1 || days.unwrap() > 365 {
                token_error.set(Some("Expire days must be between 1 and 365".to_string()));
                return;
            }

            is_generating_token.set(true);
            token_error.set(None);
            let is_generating_token = is_generating_token.clone();
            let token_result = token_result.clone();
            let token_error = token_error.clone();
            let namespace = namespace.clone();
            let name = name.clone();

            wasm_bindgen_futures::spawn_local(async move {
                match generate_token(namespace, name, days).await {
                    Ok(result) => {
                        token_result.set(Some(result));
                        is_generating_token.set(false);
                    }
                    Err(e) => {
                        token_error.set(Some(e));
                        is_generating_token.set(false);
                    }
                }
            });
        })
    };

    let on_generate_token_close = {
        let show_generate_token = show_generate_token.clone();
        Callback::from(move |_| {
            show_generate_token.set(false);
        })
    };

    let on_copy_token = {
        let token_result = token_result.clone();
        Callback::from(move |_| {
            if let Some((token, _)) = &*token_result {
                let window = web_sys::window().unwrap();
                let navigator = window.navigator();
                let clipboard = navigator.clipboard();
                let _ = clipboard.write_text(token);
            }
        })
    };

    html! {
        <div class="container">
            <div class="header">
                <h1>{ "Authorization Details" }</h1>
                <div style="display: flex; gap: 0.75rem;">
                    { match &*load_state {
                        LoadState::Loaded(auth) if auth.auth_type == 1 => html! {
                            <button
                                class="btn-primary"
                                onclick={on_generate_token_click}
                            >
                                { "Generate Token" }
                            </button>
                        },
                        _ => html! {}
                    }}
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

            { if *show_generate_token {
                let on_close_clone = on_generate_token_close.clone();
                html! {
                    <div class="modal-overlay" onclick={on_generate_token_close.clone()}>
                        <div class="modal-content" onclick={|e: MouseEvent| e.stop_propagation()}>
                            <div class="modal-header">
                                <h2>{ "Generate Token" }</h2>
                                <button class="btn-secondary-small" onclick={on_close_clone}>{ "×" }</button>
                            </div>

                            { if let Some(error) = &*token_error {
                                html! { <ErrorMessage message={error.clone()} /> }
                            } else { html! {} }}

                            { if let Some((token, expire_at)) = &*token_result {
                                html! {
                                    <div class="token-result">
                                        <div class="detail-section">
                                            <h3>{ "Token Generated Successfully" }</h3>
                                            <div class="detail-field">
                                                <label>{ "Token:" }</label>
                                                <div style="display: flex; gap: 0.5rem; align-items: center;">
                                                    <pre class="code-block" style="flex: 1; margin: 0; overflow-x: auto;">{ token }</pre>
                                                    <button class="btn-secondary-small" onclick={on_copy_token}>{ "Copy" }</button>
                                                </div>
                                            </div>
                                            { if let Some(expire) = expire_at {
                                                html! {
                                                    <div class="detail-field">
                                                        <label>{ "Expires At:" }</label>
                                                        <span>{ expire }</span>
                                                    </div>
                                                }
                                            } else { html! {} }}
                                            <p class="form-help">{ "⚠️ Save this token securely. It will not be shown again." }</p>
                                        </div>
                                    </div>
                                }
                            } else {
                                html! {
                                    <form onsubmit={on_generate_token_submit} class="form">
                                        <div class="field">
                                            <label>{ "Expiration (days) *" }</label>
                                            <input
                                                type="number"
                                                value={(*expire_days).clone()}
                                                onchange={on_expire_days_change}
                                                min="1"
                                                max="365"
                                                required={true}
                                            />
                                            <small class="form-help">{ "Token will expire after this many days (1-365)" }</small>
                                        </div>

                                        <div class="form-actions">
                                            <button
                                                type="submit"
                                                class="btn-primary"
                                                disabled={*is_generating_token}
                                            >
                                                { if *is_generating_token { "Generating..." } else { "Generate" } }
                                            </button>
                                            <button
                                                type="button"
                                                class="btn-secondary"
                                                onclick={on_generate_token_close}
                                            >
                                                { "Cancel" }
                                            </button>
                                        </div>
                                    </form>
                                }
                            }}
                        </div>
                    </div>
                }
            } else { html! {} }}
        </div>
    }
}
