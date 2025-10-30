use crate::api::authorizations::create_authorization;
use crate::components::{ErrorMessage, FormField, NamespaceSelector};
use crate::models::authorization::AuthorizationFormData;
use crate::models::SessionState;
use crate::routes::Route;
use crate::utils::validation::validate_name;
use std::collections::HashMap;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[function_component(AuthorizationCreate)]
pub fn authorization_create() -> Html {
    let form_data = use_state(AuthorizationFormData::default);
    let errors = use_state(HashMap::<String, String>::new);
    let is_submitting = use_state(|| false);
    let submit_error = use_state(|| Option::<String>::None);
    let navigator = use_navigator().unwrap();
    let (session_state, _) = use_store::<SessionState>();
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

    let on_type_change = {
        let form_data = form_data.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            let value = match select.value().as_str() {
                "kubernetes-service-account" => 1,
                _ => 1,
            };
            let mut data = (*form_data).clone();
            data.auth_type = value;
            form_data.set(data);
        })
    };

    let on_data_change = {
        let form_data = form_data.clone();
        Callback::from(move |e: Event| {
            let textarea: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
            let value = textarea.value();
            let mut data = (*form_data).clone();
            data.data = if value.is_empty() { None } else { Some(value) };
            form_data.set(data);
        })
    };

    let on_submit = {
        let form_data = form_data.clone();
        let errors = errors.clone();
        let is_submitting = is_submitting.clone();
        let submit_error = submit_error.clone();
        let navigator = navigator.clone();
        let namespace_value = namespace_value.clone();

        Callback::from(move |e: SubmitEvent| {
            e.prevent_default();

            if !errors.is_empty() {
                return;
            }

            let mut data = (*form_data).clone();

            let mut validation_errors = HashMap::new();
            if let Some(error) = validate_name(&data.name) {
                validation_errors.insert("name".to_string(), error);
            }

            if !validation_errors.is_empty() {
                errors.set(validation_errors);
                return;
            }

            data.namespace = Some(namespace_value.clone());

            is_submitting.set(true);
            let is_submitting = is_submitting.clone();
            let submit_error = submit_error.clone();
            let navigator = navigator.clone();

            wasm_bindgen_futures::spawn_local(async move {
                match create_authorization(data).await {
                    Ok(authorization) => {
                        navigator.push(&Route::AuthorizationDetail {
                            namespace: authorization.namespace,
                            name: authorization.name,
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
                <h1>{ "Create Authorization" }</h1>
                <Link<Route> to={Route::AuthorizationList}>
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
                    label="Authorization Name *"
                    error={errors.get("name").cloned()}
                >
                    <input
                        type="text"
                        value={form_data.name.clone()}
                        onchange={on_name_change}
                        required={true}
                        placeholder="my-auth"
                    />
                    <small class="form-help">{ "Lowercase alphanumeric and hyphens only" }</small>
                </FormField>

                <div class="field">
                    <label>{ "Type *" }</label>
                    <select onchange={on_type_change} value="kubernetes-service-account">
                        <option value="kubernetes-service-account">{ "Kubernetes Service Account" }</option>
                    </select>
                    <small class="form-help">{ "Authorization type for MCP server access (Anonymous type cannot be created)" }</small>
                </div>

                <div class="field">
                    <label>{ "Data (JSON, optional)" }</label>
                    <textarea
                        value={form_data.data.clone().unwrap_or_default()}
                        onchange={on_data_change}
                        placeholder="{}"
                        rows="4"
                    />
                    <small class="form-help">{ "Additional configuration data in JSON format" }</small>
                </div>

                <div class="form-actions">
                    <button
                        type="submit"
                        class="btn-primary"
                        disabled={*is_submitting || !errors.is_empty()}
                    >
                        { if *is_submitting { "Creating..." } else { "Create Authorization" } }
                    </button>
                    <Link<Route> to={Route::AuthorizationList}>
                        <button type="button" class="btn-secondary">{ "Cancel" }</button>
                    </Link<Route>>
                </div>
            </form>
        </div>
    }
}
