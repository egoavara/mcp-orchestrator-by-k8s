use crate::api::APICaller;
use crate::components::{ErrorMessage, FormField, NamespaceSelector};
use crate::models::state::AuthState;
use crate::models::SessionState;
use crate::routes::Route;
use crate::utils::validation::validate_name;
use proto_web::CreateSecretRequest;
use std::collections::HashMap;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[derive(Default, Clone, PartialEq)]
struct SecretFormData {
    name: String,
    data: Vec<(String, String)>,
    labels: Vec<(String, String)>,
}

#[function_component(SecretCreate)]
pub fn secret_create() -> Html {
    let form_data = use_state(SecretFormData::default);
    let errors = use_state(HashMap::<String, String>::new);
    let is_submitting = use_state(|| false);
    let submit_error = use_state(|| Option::<String>::None);
    let navigator = use_navigator().unwrap();
    let (session_state, _) = use_store::<SessionState>();
    let (auth_state, _) = use_store::<AuthState>();
    let namespace = session_state.selected_namespace.clone();

    let namespace_value = namespace.clone().unwrap_or_else(|| "default".to_string());

    let on_name_change = {
        let form_data = form_data.clone();
        let errors = errors.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let value = input.value();
            let mut data = (*form_data).clone();
            data.name = value.clone();
            form_data.set(data);

            let mut new_errors = (*errors).clone();
            if let Some(error) = validate_name(&value) {
                new_errors.insert("name".to_string(), error);
            } else {
                new_errors.remove("name");
            }
            errors.set(new_errors);
        })
    };

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

    let on_submit = {
        let form_data = form_data.clone();
        let errors = errors.clone();
        let is_submitting = is_submitting.clone();
        let submit_error = submit_error.clone();
        let navigator = navigator.clone();
        let namespace_value = namespace_value.clone();
        let auth_state = auth_state.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            if !errors.is_empty() {
                return;
            }

            let data = (*form_data).clone();

            let mut validation_errors = HashMap::new();
            if let Some(error) = validate_name(&data.name) {
                validation_errors.insert("name".to_string(), error);
            }
            if data.data.is_empty() {
                validation_errors.insert(
                    "data".to_string(),
                    "At least one key-value pair is required".to_string(),
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
            let namespace_value = namespace_value.clone();
            let auth_state = auth_state.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let data_map: HashMap<String, String> = data
                    .data
                    .into_iter()
                    .filter(|(k, v)| !k.is_empty() && !v.is_empty())
                    .collect();

                let labels_map: HashMap<String, String> = data
                    .labels
                    .into_iter()
                    .filter(|(k, v)| !k.is_empty() && !v.is_empty())
                    .collect();

                let request = CreateSecretRequest {
                    namespace: Some(namespace_value.clone()),
                    name: data.name.clone(),
                    labels: labels_map,
                    data: data_map,
                };

                let api = APICaller::new(auth_state.access_token.clone());
                match api.create_secret(request).await {
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

    html! {
        <div class="container">
            <NamespaceSelector />

            <div class="header">
                <h1>{ "Create Secret" }</h1>
                <Link<Route> to={Route::SecretList}>
                    <button class="btn-secondary">{ "Cancel" }</button>
                </Link<Route>>
            </div>

            { if let Some(error) = &*submit_error {
                html! { <ErrorMessage message={error.clone()} /> }
            } else { html! {} }}

            <form onsubmit={on_submit} class="form">
                <div class="field">
                    <label>{ "Namespace:" }</label>
                    <span class="namespace-badge">{ &namespace_value }</span>
                </div>

                <FormField
                    label="Secret Name *"
                    error={errors.get("name").cloned()}
                >
                    <input
                        type="text"
                        value={form_data.name.clone()}
                        onchange={on_name_change}
                        required={true}
                        placeholder="my-secret"
                    />
                    <small class="form-help">{ "Lowercase alphanumeric and hyphens only" }</small>
                </FormField>

                <div class="form-section">
                    <label class="section-label">{ "Secret Data * (values will be encrypted)" }</label>
                    <small class="form-help">{ "Add key-value pairs for sensitive data" }</small>

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
                        { if *is_submitting { "Creating..." } else { "Create Secret" } }
                    </button>
                    <Link<Route> to={Route::SecretList}>
                        <button type="button" class="btn-secondary">{ "Cancel" }</button>
                    </Link<Route>>
                </div>
            </form>
        </div>
    }
}
