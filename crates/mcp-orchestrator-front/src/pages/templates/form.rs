use crate::api::APICaller;
use crate::components::{ErrorMessage, FormField, NamespaceSelector};
use crate::models::authorization::Authorization;
use crate::models::resource_limit::ResourceLimit;
use crate::models::secret::Secret;
use crate::models::state::AuthState;
use crate::models::template::TemplateFormData;
use crate::models::SessionState;
use crate::routes::Route;
use crate::utils::validation::{validate_docker_image, validate_name, validate_arg_env_key, validate_arg_env_name, validate_arg_env_value};
use std::collections::HashMap;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[function_component(TemplateForm)]
pub fn template_form() -> Html {
    let (session_state, _) = use_store::<SessionState>();
    let (auth_state, _) = use_store::<AuthState>();
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
        let auth_state = auth_state.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let api = APICaller::new(auth_state.access_token.clone());
                match api.list_resource_limits().await {
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
        let auth_state = auth_state.clone();
        use_effect_with(namespace.clone(), move |ns| {
            let secrets = secrets.clone();
            let is_loading_secrets = is_loading_secrets.clone();
            let namespace = ns.clone();
            let auth_state = auth_state.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let api = APICaller::new(auth_state.access_token.clone());
                match api.list_secrets(&namespace).await {
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
        let auth_state = auth_state.clone();
        use_effect_with(namespace.clone(), move |ns| {
            let authorizations = authorizations.clone();
            let is_loading_authorizations = is_loading_authorizations.clone();
            let namespace = Some(ns.clone());
            let auth_state = auth_state.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let api = APICaller::new(auth_state.access_token.clone());
                match api.list_authorizations(namespace, None).await {
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

    // arg_env_items: (id, key, env_name, type)
    let arg_env_items = use_state(Vec::<(usize, String, String, String)>::new);
    let arg_env_counter = use_state(|| 0usize);

    let on_add_arg_env = {
        let arg_env_items = arg_env_items.clone();
        let arg_env_counter = arg_env_counter.clone();
        Callback::from(move |_| {
            let mut items = (*arg_env_items).clone();
            let id = *arg_env_counter;
            items.push((id, String::new(), String::new(), "string".to_string()));
            arg_env_items.set(items);
            arg_env_counter.set(id + 1);
        })
    };

    let on_remove_arg_env = {
        let arg_env_items = arg_env_items.clone();
        move |id: usize| {
            let mut items = (*arg_env_items).clone();
            items.retain(|(item_id, _, _, _)| *item_id != id);
            arg_env_items.set(items);
        }
    };

    let on_arg_env_key_change = {
        let arg_env_items = arg_env_items.clone();
        let errors = errors.clone();
        move |id: usize, new_key: String| {
            let mut items = (*arg_env_items).clone();
            if let Some(item) = items.iter_mut().find(|(item_id, _, _, _)| *item_id == id) {
                item.1 = new_key.clone();
            }
            arg_env_items.set(items);

            // Validate key
            let mut new_errors = (*errors).clone();
            if !new_key.is_empty() {
                if let Some(error) = validate_arg_env_key(&new_key) {
                    new_errors.insert(format!("arg_env_key_{}", id), error);
                } else {
                    new_errors.remove(&format!("arg_env_key_{}", id));
                }
            } else {
                new_errors.remove(&format!("arg_env_key_{}", id));
            }
            errors.set(new_errors);
        }
    };

    let on_arg_env_name_change = {
        let arg_env_items = arg_env_items.clone();
        let errors = errors.clone();
        move |id: usize, new_env_name: String| {
            let mut items = (*arg_env_items).clone();
            if let Some(item) = items.iter_mut().find(|(item_id, _, _, _)| *item_id == id) {
                item.2 = new_env_name.clone();
            }
            arg_env_items.set(items);

            // Validate env name
            let mut new_errors = (*errors).clone();
            if !new_env_name.is_empty() {
                if let Some(error) = validate_arg_env_name(&new_env_name) {
                    new_errors.insert(format!("arg_env_name_{}", id), error);
                } else {
                    new_errors.remove(&format!("arg_env_name_{}", id));
                }
            } else {
                new_errors.remove(&format!("arg_env_name_{}", id));
            }
            errors.set(new_errors);
        }
    };

    let on_arg_env_type_change = {
        let arg_env_items = arg_env_items.clone();
        move |id: usize, new_type: String| {
            let mut items = (*arg_env_items).clone();
            if let Some(item) = items.iter_mut().find(|(item_id, _, _, _)| *item_id == id) {
                item.3 = new_type;
            }
            arg_env_items.set(items);
        }
    };

    // Sync arg_env_items to form_data.arg_envs
    {
        let form_data = form_data.clone();
        let arg_env_items = arg_env_items.clone();
        use_effect_with(arg_env_items.clone(), move |items| {
            let mut data = (*form_data).clone();
            data.arg_envs = items
                .iter()
                .filter(|(_, k, _, _)| !k.is_empty())
                .map(|(_, k, env_name, type_str)| {
                    let value = if env_name.is_empty() {
                        type_str.clone()
                    } else {
                        format!("{}: {}", env_name, type_str)
                    };
                    (k.clone(), value)
                })
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
            if let Some(error) = validate_docker_image(&data.image) {
                validation_errors.insert("image".to_string(), error);
            }
            if data.resource_limit_name.is_none() {
                validation_errors.insert(
                    "resource_limit_name".to_string(),
                    "Resource limit is required".to_string(),
                );
            }
            
            // Validate arg_envs
            for (key, value) in &data.arg_envs {
                if let Some(error) = validate_arg_env_key(key) {
                    validation_errors.insert(format!("arg_env_key_{}", key), error);
                }
                if let Some(error) = validate_arg_env_value(value) {
                    validation_errors.insert(format!("arg_env_value_{}", key), error);
                }
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
            let auth_state = auth_state.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let request = data.into_create_request();
                let api = APICaller::new(auth_state.access_token.clone());
                match api.create_template(request).await {
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
                    <label class="section-label">{ "Argument Environment Variables" }</label>
                    <small class="form-help">{ "Define arguments that will be passed via HTTP headers (arg-{key}) and injected as environment variables in the Pod" }</small>

                    { for arg_env_items.iter().map(|(id, key, env_name, type_str)| {
                        let item_id = *id;
                        let key_error = errors.get(&format!("arg_env_key_{}", item_id)).cloned();
                        let name_error = errors.get(&format!("arg_env_name_{}", item_id)).cloned();

                        let on_key_change = {
                            let on_arg_env_key_change = on_arg_env_key_change.clone();
                            Callback::from(move |e: Event| {
                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                on_arg_env_key_change(item_id, input.value());
                            })
                        };

                        let on_env_name_change = {
                            let on_arg_env_name_change = on_arg_env_name_change.clone();
                            Callback::from(move |e: Event| {
                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                on_arg_env_name_change(item_id, input.value());
                            })
                        };

                        let on_type_change = {
                            let on_arg_env_type_change = on_arg_env_type_change.clone();
                            Callback::from(move |e: Event| {
                                let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
                                on_arg_env_type_change(item_id, select.value());
                            })
                        };

                        let on_remove = {
                            let on_remove_arg_env = on_remove_arg_env.clone();
                            Callback::from(move |_| on_remove_arg_env(item_id))
                        };

                        html! {
                            <div key={item_id}>
                                <div class="label-row">
                                    <input
                                        type="text"
                                        value={key.clone()}
                                        onchange={on_key_change}
                                        placeholder="key (starts with a-z)"
                                        class="label-key"
                                    />
                                    <span>{ "=" }</span>
                                    <input
                                        type="text"
                                        value={env_name.clone()}
                                        onchange={on_env_name_change}
                                        placeholder="ENV_NAME (optional, A-Z0-9_-)"
                                        class="label-value"
                                    />
                                    <span>{ ":" }</span>
                                    <select
                                        onchange={on_type_change}
                                        value={type_str.clone()}
                                    >
                                        <option value="string" selected={type_str == "string"}>{ "string" }</option>
                                        <option value="string?" selected={type_str == "string?"}>{ "string?" }</option>
                                    </select>
                                    <button
                                        type="button"
                                        onclick={on_remove}
                                        class="btn-danger-small"
                                    >
                                        { "×" }
                                    </button>
                                </div>
                                { if key_error.is_some() || name_error.is_some() {
                                    html! {
                                        <div style="margin-top: 0.25rem;">
                                            { if let Some(error) = key_error {
                                                html! { <small class="error-text" style="display: block;">{ format!("Key: {}", error) }</small> }
                                            } else {
                                                html! {}
                                            }}
                                            { if let Some(error) = name_error {
                                                html! { <small class="error-text" style="display: block;">{ format!("ENV_NAME: {}", error) }</small> }
                                            } else {
                                                html! {}
                                            }}
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }}
                                <small class="form-help" style="display: block; margin-top: 0.25rem;">
                                    {{
                                        let type_example = if type_str.as_str() == "string?" { 
                                            "\"value\" or null" 
                                        } else { 
                                            "\"value\"" 
                                        };
                                        let final_env_name = if env_name.is_empty() { 
                                            key.as_str() 
                                        } else { 
                                            env_name.as_str() 
                                        };
                                        
                                        if env_name.is_empty() {
                                            format!("HTTP Header = \"arg-{}: {}\" ⇒ Pod Env = \"{}={}\"", 
                                                key, type_example, key, type_example)
                                        } else {
                                            format!("HTTP Header = \"arg-{}: {}\" ⇒ Pod Env = \"{}={}\"", 
                                                key, type_example, final_env_name, type_example)
                                        }
                                    }}
                                </small>
                            </div>
                        }
                    })}

                    <button
                        type="button"
                        onclick={on_add_arg_env}
                        class="btn-secondary-small"
                    >
                        { "+ Add Argument Environment Variable" }
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
