use crate::api::authorizations::list_authorizations;
use crate::api::resource_limits::list_resource_limits;
use crate::api::secrets::list_secrets;
use crate::api::templates::create_template;
use crate::components::{ErrorMessage, FormField, NamespaceSelector};
use crate::models::authorization::Authorization;
use crate::models::resource_limit::ResourceLimit;
use crate::models::secret::Secret;
use crate::models::template::TemplateFormData;
use crate::models::SessionState;
use crate::routes::Route;
use crate::utils::validation::{validate_docker_image, validate_name};
use std::collections::HashMap;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[function_component(TemplateForm)]
pub fn template_form() -> Html {
    let (session_state, _) = use_store::<SessionState>();
    let namespace = session_state
        .selected_namespace
        .clone()
        .unwrap_or_else(|| "default".to_string());

    let form_data = use_state(|| TemplateFormData {
        namespace: namespace.clone(),
        ..Default::default()
    });
    let errors = use_state(HashMap::<String, String>::new);
    let is_submitting = use_state(|| false);
    let submit_error = use_state(|| Option::<String>::None);
    let navigator = use_navigator().unwrap();

    let resource_limits = use_state(Vec::<ResourceLimit>::new);
    let is_loading_limits = use_state(|| true);

    let secrets = use_state(Vec::<Secret>::new);
    let is_loading_secrets = use_state(|| true);

    let authorizations = use_state(Vec::<Authorization>::new);
    let is_loading_authorizations = use_state(|| true);

    // Load resource limits
    {
        let resource_limits = resource_limits.clone();
        let is_loading_limits = is_loading_limits.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match list_resource_limits().await {
                    Ok(limits) => {
                        resource_limits.set(limits);
                    }
                    Err(e) => {
                        web_sys::console::error_1(
                            &format!("Failed to load resource limits: {}", e).into(),
                        );
                        resource_limits.set(vec![]);
                    }
                }
                is_loading_limits.set(false);
            });
            || ()
        });
    }

    // Load secrets for the current namespace
    {
        let secrets = secrets.clone();
        let is_loading_secrets = is_loading_secrets.clone();
        let namespace = namespace.clone();
        use_effect_with(namespace.clone(), move |ns| {
            let secrets = secrets.clone();
            let is_loading_secrets = is_loading_secrets.clone();
            let namespace = ns.clone();
            wasm_bindgen_futures::spawn_local(async move {
                match list_secrets(&namespace).await {
                    Ok(secret_list) => {
                        secrets.set(secret_list);
                    }
                    Err(e) => {
                        web_sys::console::error_1(&format!("Failed to load secrets: {}", e).into());
                        secrets.set(vec![]);
                    }
                }
                is_loading_secrets.set(false);
            });
            || ()
        });
    }

    // Load authorizations for the current namespace
    {
        let authorizations = authorizations.clone();
        let is_loading_authorizations = is_loading_authorizations.clone();
        let namespace = namespace.clone();
        use_effect_with(namespace.clone(), move |ns| {
            let authorizations = authorizations.clone();
            let is_loading_authorizations = is_loading_authorizations.clone();
            let namespace = Some(ns.clone());
            wasm_bindgen_futures::spawn_local(async move {
                match list_authorizations(namespace, None).await {
                    Ok(auth_list) => {
                        authorizations.set(auth_list);
                    }
                    Err(e) => {
                        web_sys::console::error_1(
                            &format!("Failed to load authorizations: {}", e).into(),
                        );
                        authorizations.set(vec![]);
                    }
                }
                is_loading_authorizations.set(false);
            });
            || ()
        });
    }

    // Update form namespace when session state changes
    {
        let form_data = form_data.clone();
        let namespace = namespace.clone();
        use_effect_with(namespace.clone(), move |ns| {
            let mut data = (*form_data).clone();
            data.namespace = ns.clone();
            form_data.set(data);
            || ()
        });
    }

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

    let on_image_change = {
        let form_data = form_data.clone();
        let errors = errors.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let value = input.value();
            let mut data = (*form_data).clone();
            data.image = value.clone();
            form_data.set(data);

            let mut new_errors = (*errors).clone();
            if let Some(error) = validate_docker_image(&value) {
                new_errors.insert("image".to_string(), error);
            } else {
                new_errors.remove("image");
            }
            errors.set(new_errors);
        })
    };

    let on_command_change = {
        let form_data = form_data.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let value = input.value();
            let mut data = (*form_data).clone();
            data.command = if value.is_empty() {
                Vec::new()
            } else {
                value.split_whitespace().map(|s| s.to_string()).collect()
            };
            form_data.set(data);
        })
    };

    let on_args_change = {
        let form_data = form_data.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let value = input.value();
            let mut data = (*form_data).clone();
            data.args = if value.is_empty() {
                Vec::new()
            } else {
                value.split_whitespace().map(|s| s.to_string()).collect()
            };
            form_data.set(data);
        })
    };

    let env_items = use_state(Vec::<(usize, String, String)>::new);
    let env_counter = use_state(|| 0usize);

    let on_add_env = {
        let env_items = env_items.clone();
        let env_counter = env_counter.clone();
        Callback::from(move |_| {
            let mut items = (*env_items).clone();
            let id = *env_counter;
            items.push((id, String::new(), String::new()));
            env_items.set(items);
            env_counter.set(id + 1);
        })
    };

    let on_remove_env = {
        let env_items = env_items.clone();
        move |id: usize| {
            let mut items = (*env_items).clone();
            items.retain(|(item_id, _, _)| *item_id != id);
            env_items.set(items);
        }
    };

    let on_env_key_change = {
        let env_items = env_items.clone();
        move |id: usize, new_key: String| {
            let mut items = (*env_items).clone();
            if let Some(item) = items.iter_mut().find(|(item_id, _, _)| *item_id == id) {
                item.1 = new_key;
            }
            env_items.set(items);
        }
    };

    let on_env_value_change = {
        let env_items = env_items.clone();
        move |id: usize, new_value: String| {
            let mut items = (*env_items).clone();
            if let Some(item) = items.iter_mut().find(|(item_id, _, _)| *item_id == id) {
                item.2 = new_value;
            }
            env_items.set(items);
        }
    };

    // Sync env_items to form_data.envs
    {
        let form_data = form_data.clone();
        let env_items = env_items.clone();
        use_effect_with(env_items.clone(), move |items| {
            let mut data = (*form_data).clone();
            data.envs = items
                .iter()
                .filter(|(_, k, v)| !k.is_empty() || !v.is_empty())
                .map(|(_, k, v)| (k.clone(), v.clone()))
                .collect();
            form_data.set(data);
            || ()
        });
    }

    let on_add_secret_env = {
        let form_data = form_data.clone();
        let is_loading_secrets = is_loading_secrets.clone();
        Callback::from(move |_| {
            if *is_loading_secrets {
                return;
            }
            let mut data = (*form_data).clone();
            data.secret_envs.push(String::new());
            form_data.set(data);
        })
    };

    let on_remove_secret_env = {
        let form_data = form_data.clone();
        move |index: usize| {
            let mut data = (*form_data).clone();
            data.secret_envs.remove(index);
            form_data.set(data);
        }
    };

    let on_secret_env_change = {
        let form_data = form_data.clone();
        move |index: usize, e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            let value = select.value();
            let mut data = (*form_data).clone();
            if let Some(item) = data.secret_envs.get_mut(index) {
                *item = value;
            }
            form_data.set(data);
        }
    };

    let on_resource_limit_change = {
        let form_data = form_data.clone();
        let errors = errors.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            let value = select.value();
            let mut data = (*form_data).clone();
            data.resource_limit_name = if value.is_empty() {
                None
            } else {
                Some(value.clone())
            };
            form_data.set(data);

            let mut new_errors = (*errors).clone();
            if value.is_empty() {
                new_errors.insert(
                    "resource_limit_name".to_string(),
                    "Resource limit is required".to_string(),
                );
            } else {
                new_errors.remove("resource_limit_name");
            }
            errors.set(new_errors);
        })
    };

    let on_authorization_change = {
        let form_data = form_data.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            let value = select.value();
            let mut data = (*form_data).clone();
            data.authorization_name = if value.is_empty() {
                None
            } else {
                Some(value)
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
            if let Some(error) = validate_docker_image(&data.image) {
                validation_errors.insert("image".to_string(), error);
            }
            if data.resource_limit_name.is_none() {
                validation_errors.insert(
                    "resource_limit_name".to_string(),
                    "Resource limit is required".to_string(),
                );
            }

            if !validation_errors.is_empty() {
                errors.set(validation_errors);
                return;
            }

            is_submitting.set(true);
            let _errors = errors.clone();
            let is_submitting = is_submitting.clone();
            let submit_error = submit_error.clone();
            let navigator = navigator.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let request = data.to_create_request();
                match create_template(request).await {
                    Ok(template) => {
                        navigator.push(&Route::TemplateDetail {
                            namespace: template.namespace,
                            name: template.name,
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
                <h1>{ "Create Template" }</h1>
                <Link<Route> to={Route::TemplateList}>
                    <button class="btn-secondary">{ "Cancel" }</button>
                </Link<Route>>
            </div>

            { if let Some(error) = &*submit_error {
                html! { <ErrorMessage message={error.clone()} /> }
            } else { html! {} }}

            <form onsubmit={on_submit} class="form">
                <div class="field">
                    <label>{ "Namespace:" }</label>
                    <span class="namespace-badge">{ &form_data.namespace }</span>
                </div>

                <FormField
                    label="Template Name *"
                    error={errors.get("name").cloned()}
                >
                    <input
                        type="text"
                        value={form_data.name.clone()}
                        onchange={on_name_change}
                        required={true}
                        placeholder="my-mcp-template"
                    />
                    <small class="form-help">{ "Lowercase alphanumeric and hyphens only, max 63 characters" }</small>
                </FormField>

                <FormField
                    label="Docker Image *"
                    error={errors.get("image").cloned()}
                >
                    <input
                        type="text"
                        value={form_data.image.clone()}
                        onchange={on_image_change}
                        required={true}
                        placeholder="mcp/server:latest"
                    />
                    <small class="form-help">{ "Full image name with tag (e.g., myregistry/myimage:v1.0)" }</small>
                </FormField>

                <FormField
                    label="Command"
                    error={Option::<String>::None}
                >
                    <input
                        type="text"
                        value={form_data.command.join(" ")}
                        onchange={on_command_change}
                        placeholder="/bin/sh -c"
                    />
                    <small class="form-help">{ "Optional command override (space-separated)" }</small>
                </FormField>

                <FormField
                    label="Arguments"
                    error={Option::<String>::None}
                >
                    <input
                        type="text"
                        value={form_data.args.join(" ")}
                        onchange={on_args_change}
                        placeholder="start-server.sh"
                    />
                    <small class="form-help">{ "Optional command arguments (space-separated)" }</small>
                </FormField>

                <div class="form-section">
                    <label class="section-label">{ "Environment Variables" }</label>
                    <small class="form-help">{ "Add key-value pairs for environment variables" }</small>

                    { for env_items.iter().map(|(id, key, value)| {
                        let item_id = *id;

                        let on_key_change = {
                            let on_env_key_change = on_env_key_change.clone();
                            Callback::from(move |e: Event| {
                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                on_env_key_change(item_id, input.value());
                            })
                        };

                        let on_value_change = {
                            let on_env_value_change = on_env_value_change.clone();
                            Callback::from(move |e: Event| {
                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                on_env_value_change(item_id, input.value());
                            })
                        };

                        let on_remove = {
                            let on_remove_env = on_remove_env.clone();
                            Callback::from(move |_| on_remove_env(item_id))
                        };

                        html! {
                            <div class="label-row" key={item_id}>
                                <input
                                    type="text"
                                    value={key.clone()}
                                    onchange={on_key_change}
                                    placeholder="KEY"
                                    class="label-key"
                                />
                                <span>{ "=" }</span>
                                <input
                                    type="text"
                                    value={value.clone()}
                                    onchange={on_value_change}
                                    placeholder="value"
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
                        onclick={on_add_env}
                        class="btn-secondary-small"
                    >
                        { "+ Add Environment Variable" }
                    </button>
                </div>

                <div class="form-section">
                    <label class="section-label">{ "Secret References" }</label>
                    <small class="form-help">{ "Reference secrets from the same namespace" }</small>

                    { if *is_loading_secrets {
                        html! { <p>{ "Loading secrets..." }</p> }
                    } else if secrets.is_empty() {
                        html! {
                            <p class="form-help">
                                { "No secrets available in this namespace. " }
                                <Link<Route> to={Route::SecretCreate}>
                                    { "Create a secret first" }
                                </Link<Route>>
                            </p>
                        }
                    } else {
                        html! {
                            <>
                                { for form_data.secret_envs.iter().enumerate().map(|(index, secret_ref)| {
                                    let on_change = {
                                        let on_secret_env_change = on_secret_env_change.clone();
                                        Callback::from(move |e: Event| {
                                            on_secret_env_change(index, e);
                                        })
                                    };

                                    let on_remove = {
                                        let on_remove_secret_env = on_remove_secret_env.clone();
                                        Callback::from(move |_| on_remove_secret_env(index))
                                    };

                                    html! {
                                        <div class="label-row" key={index}>
                                            <select
                                                onchange={on_change}
                                                style="flex: 1;"
                                            >
                                                <option value="" selected={secret_ref.is_empty()}>{"-- Select a secret --"}</option>
                                                { for secrets.iter().map(|secret| {
                                                    let is_selected = secret_ref == &secret.name;
                                                    html! {
                                                        <option key={secret.name.clone()} value={secret.name.clone()} selected={is_selected}>
                                                            {format!("{} ({} keys)", secret.name, secret.keys.len())}
                                                        </option>
                                                    }
                                                })}
                                            </select>
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
                                    onclick={on_add_secret_env}
                                    class="btn-secondary-small"
                                    disabled={*is_loading_secrets}
                                >
                                    { if *is_loading_secrets { "+ Loading..." } else { "+ Add Secret Reference" } }
                                </button>
                            </>
                        }
                    }}
                </div>

                <FormField
                    label="Resource Limit *"
                    error={errors.get("resource_limit_name").cloned()}
                >
                    if *is_loading_limits {
                        <select disabled={true}>
                            <option>{"Loading resource limits..."}</option>
                        </select>
                    } else if resource_limits.is_empty() {
                        <>
                            <select disabled={true}>
                                <option>{"No resource limits available"}</option>
                            </select>
                            <small class="form-help error-text">
                                { "Please create a resource limit first. " }
                                <Link<Route> to={Route::ResourceLimitCreate}>
                                    { "Create Resource Limit" }
                                </Link<Route>>
                            </small>
                        </>
                    } else {
                        <>
                            <select
                                onchange={on_resource_limit_change}
                                required={true}
                            >
                                <option value="" selected={form_data.resource_limit_name.is_none()}>{"-- Select a resource limit --"}</option>
                                { for resource_limits.iter().map(|limit| {
                                    let is_selected = form_data.resource_limit_name.as_ref() == Some(&limit.name);
                                    html! {
                                        <option key={limit.name.clone()} value={limit.name.clone()} selected={is_selected}>
                                            {format!("{} (CPU: {}, Memory: {})",
                                                limit.name,
                                                limit.limits.cpu,
                                                limit.limits.memory
                                            )}
                                        </option>
                                    }
                                })}
                            </select>
                            <small class="form-help">{ "Select a resource limit configuration for this template" }</small>
                        </>
                    }
                </FormField>

                <FormField
                    label="Authorization"
                    error={Option::<String>::None}
                >
                    if *is_loading_authorizations {
                        <select disabled={true}>
                            <option>{"Loading authorizations..."}</option>
                        </select>
                    } else {
                        <>
                            <select
                                onchange={on_authorization_change}
                            >
                                <option value="" selected={form_data.authorization_name.is_none()}>{"-- Use default (anonymous) --"}</option>
                                { for authorizations.iter().map(|auth| {
                                    let is_selected = form_data.authorization_name.as_ref() == Some(&auth.name);
                                    let type_name = match auth.auth_type {
                                        1 => "Service Account",
                                        _ => "Unknown",
                                    };
                                    html! {
                                        <option key={auth.name.clone()} value={auth.name.clone()} selected={is_selected}>
                                            {format!("{} ({})", auth.name, type_name)}
                                        </option>
                                    }
                                })}
                            </select>
                            <small class="form-help">{ "Optional: Select an authorization for accessing this MCP server. If not selected, anonymous access is used." }</small>
                        </>
                    }
                </FormField>

                <div class="form-actions">
                    <button
                        type="submit"
                        class="btn-primary"
                        disabled={*is_submitting || !errors.is_empty()}
                    >
                        { if *is_submitting { "Creating..." } else { "Create Template" } }
                    </button>
                    <Link<Route> to={Route::TemplateList}>
                        <button type="button" class="btn-secondary">{ "Cancel" }</button>
                    </Link<Route>>
                </div>
            </form>
        </div>
    }
}
