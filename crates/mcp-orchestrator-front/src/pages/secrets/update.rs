use crate::api::secrets::{get_secret, update_secret};
use crate::components::{ErrorMessage, Loading};
use crate::models::{secret::Secret, SessionState};
use crate::routes::Route;
use proto_web::{SecretUpdateStrategy, UpdateSecretRequest};
use std::collections::HashMap;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub namespace: String,
    pub name: String,
}

#[derive(Default, Clone, PartialEq)]
struct SecretFormData {
    data: Vec<(String, String)>,
    strategy: SecretUpdateStrategy,
}

#[function_component(SecretUpdate)]
pub fn secret_update(props: &Props) -> Html {
    let form_data = use_state(SecretFormData::default);
    let errors = use_state(HashMap::<String, String>::new);
    let is_submitting = use_state(|| false);
    let submit_error = use_state(|| Option::<String>::None);
    let secret_data = use_state(|| None::<Secret>);
    let is_loading = use_state(|| true);
    let load_error = use_state(|| Option::<String>::None);
    let navigator = use_navigator().unwrap();
    let (_session_state, _) = use_store::<SessionState>();

    {
        let namespace = props.namespace.clone();
        let name = props.name.clone();
        let secret_data = secret_data.clone();
        let is_loading = is_loading.clone();
        let load_error = load_error.clone();

        use_effect_with((namespace.clone(), name.clone()), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match get_secret(&namespace, &name).await {
                    Ok(secret) => {
                        secret_data.set(Some(secret));
                        is_loading.set(false);
                    }
                    Err(e) => {
                        load_error.set(Some(e));
                        is_loading.set(false);
                    }
                }
            });
        });
    }

    let on_add_data = {
        let form_data = form_data.clone();
        Callback::from(move |_| {
            let mut data = (*form_data).clone();
            data.data.push(("".to_string(), "".to_string()));
            form_data.set(data);
        })
    };

    let on_remove_data = {
        let form_data = form_data.clone();
        move |index: usize| {
            let mut data = (*form_data).clone();
            data.data.remove(index);
            form_data.set(data);
        }
    };

    let on_data_key_change = {
        let form_data = form_data.clone();
        move |index: usize, value: String| {
            let mut data = (*form_data).clone();
            if let Some(item) = data.data.get_mut(index) {
                item.0 = value;
            }
            form_data.set(data);
        }
    };

    let on_data_value_change = {
        let form_data = form_data.clone();
        move |index: usize, value: String| {
            let mut data = (*form_data).clone();
            if let Some(item) = data.data.get_mut(index) {
                item.1 = value;
            }
            form_data.set(data);
        }
    };

    let on_strategy_change = {
        let form_data = form_data.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            let value = select.value();
            let mut data = (*form_data).clone();
            data.strategy = match value.as_str() {
                "replace" => SecretUpdateStrategy::Replace,
                "merge" => SecretUpdateStrategy::Merge,
                "patch" => SecretUpdateStrategy::Patch,
                _ => SecretUpdateStrategy::Unspecified,
            };
            form_data.set(data);
        })
    };

    let on_submit = {
        let form_data = form_data.clone();
        let errors = errors.clone();
        let is_submitting = is_submitting.clone();
        let submit_error = submit_error.clone();
        let navigator = navigator.clone();
        let namespace = props.namespace.clone();
        let name = props.name.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            if !errors.is_empty() {
                return;
            }

            let data = (*form_data).clone();

            let mut validation_errors = HashMap::new();
            if data.data.is_empty() {
                validation_errors.insert(
                    "data".to_string(),
                    "At least one key-value pair is required".to_string(),
                );
            }
            if data.strategy == SecretUpdateStrategy::Unspecified {
                validation_errors.insert(
                    "strategy".to_string(),
                    "Update strategy is required".to_string(),
                );
            }

            if !validation_errors.is_empty() {
                errors.set(validation_errors);
                return;
            }

            is_submitting.set(true);
            let is_submitting = is_submitting.clone();
            let submit_error = submit_error.clone();
            let navigator = navigator.clone();
            let namespace = namespace.clone();
            let name = name.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let data_map: HashMap<String, String> = data
                    .data
                    .into_iter()
                    .filter(|(k, v)| !k.is_empty() && !v.is_empty())
                    .collect();

                let request = UpdateSecretRequest {
                    namespace: Some(namespace.clone()),
                    name: name.clone(),
                    data: data_map,
                    strategy: data.strategy as i32,
                };

                match update_secret(request).await {
                    Ok(secret) => {
                        navigator.push(&Route::SecretDetail {
                            namespace: secret.namespace,
                            name: secret.name,
                        });
                    }
                    Err(e) => {
                        submit_error.set(Some(e));
                        is_submitting.set(false);
                    }
                }
            });
        })
    };

    if *is_loading {
        return html! { <Loading /> };
    }

    if let Some(error) = &*load_error {
        return html! {
            <div class="container">
                <ErrorMessage message={error.clone()} />
                <Link<Route> to={Route::SecretList}>
                    <button class="btn-secondary">{ "Back to Secrets" }</button>
                </Link<Route>>
            </div>
        };
    }

    let secret = secret_data.as_ref().unwrap();

    html! {
        <div class="container">
            <div class="header">
                <h1>{ format!("Update Secret: {}", &props.name) }</h1>
                <Link<Route> to={Route::SecretDetail { namespace: props.namespace.clone(), name: props.name.clone() }}>
                    <button class="btn-secondary">{ "Cancel" }</button>
                </Link<Route>>
            </div>

            { if let Some(error) = &*submit_error {
                html! { <ErrorMessage message={error.clone()} /> }
            } else { html! {} }}

            <form onsubmit={on_submit} class="form">
                <div class="field">
                    <label>{ "Namespace:" }</label>
                    <span class="namespace-badge">{ &props.namespace }</span>
                </div>

                <div class="field">
                    <label>{ "Secret Name:" }</label>
                    <span class="namespace-badge">{ &props.name }</span>
                </div>

                <div class="form-section">
                    <label class="section-label">{ "Update Strategy *" }</label>
                    <select onchange={on_strategy_change} required={true}>
                        <option value="" selected={form_data.strategy == SecretUpdateStrategy::Unspecified}>
                            { "Select strategy..." }
                        </option>
                        <option value="replace" selected={form_data.strategy == SecretUpdateStrategy::Replace}>
                            { "Replace - Replace all existing keys" }
                        </option>
                        <option value="merge" selected={form_data.strategy == SecretUpdateStrategy::Merge}>
                            { "Merge - Add new keys, keep existing" }
                        </option>
                        <option value="patch" selected={form_data.strategy == SecretUpdateStrategy::Patch}>
                            { "Patch - Update specified keys only" }
                        </option>
                    </select>
                    <small class="form-help">
                        { "REPLACE: All existing keys will be deleted. " }
                        { "MERGE: New keys added, existing keys preserved. " }
                        { "PATCH: Only specified keys will be updated." }
                    </small>
                    { if let Some(error) = errors.get("strategy") {
                        html! { <p class="error-text">{ error }</p> }
                    } else { html! {} }}
                </div>

                <div class="form-section">
                    <label class="section-label">{ "Current Keys (read-only)" }</label>
                    <div class="tags">
                        { for secret.keys.iter().map(|key| {
                            html! {
                                <span class="tag">{ key }</span>
                            }
                        })}
                    </div>
                </div>

                <div class="form-section">
                    <label class="section-label">{ "New/Updated Data * (values will be encrypted)" }</label>
                    <small class="form-help">{ "Add key-value pairs to create or update" }</small>

                    { for form_data.data.iter().enumerate().map(|(index, (key, value))| {
                        let on_key_change = {
                            let on_data_key_change = on_data_key_change.clone();
                            Callback::from(move |e: Event| {
                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                on_data_key_change(index, input.value());
                            })
                        };
                        let on_value_change = {
                            let on_data_value_change = on_data_value_change.clone();
                            Callback::from(move |e: Event| {
                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                on_data_value_change(index, input.value());
                            })
                        };
                        let on_remove = {
                            let on_remove_data = on_remove_data.clone();
                            Callback::from(move |_| on_remove_data(index))
                        };

                        html! {
                            <div class="label-row" key={index}>
                                <input
                                    type="text"
                                    value={key.clone()}
                                    onchange={on_key_change}
                                    placeholder="key"
                                    class="label-key"
                                />
                                <span>{ "=" }</span>
                                <input
                                    type="password"
                                    value={value.clone()}
                                    onchange={on_value_change}
                                    placeholder="value (hidden)"
                                    class="label-value"
                                />
                                <button
                                    type="button"
                                    onclick={on_remove}
                                    class="btn-danger-small"
                                >
                                    { "×" }
                                </button>
                            </div>
                        }
                    })}

                    <button
                        type="button"
                        onclick={on_add_data}
                        class="btn-secondary-small"
                    >
                        { "+ Add Key-Value Pair" }
                    </button>

                    { if let Some(error) = errors.get("data") {
                        html! { <p class="error-text">{ error }</p> }
                    } else { html! {} }}
                </div>

                <div class="form-actions">
                    <button
                        type="submit"
                        class="btn-primary"
                        disabled={*is_submitting || !errors.is_empty()}
                    >
                        { if *is_submitting { "Updating..." } else { "Update Secret" } }
                    </button>
                    <Link<Route> to={Route::SecretDetail { namespace: props.namespace.clone(), name: props.name.clone() }}>
                        <button type="button" class="btn-secondary">{ "Cancel" }</button>
                    </Link<Route>>
                </div>
            </form>
        </div>
    }
}
