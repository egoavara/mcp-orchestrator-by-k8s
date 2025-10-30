use crate::api::resource_limits::create_resource_limit;
use crate::components::{ErrorMessage, FormField};
use crate::routes::Route;
use crate::utils::validation::{validate_cpu, validate_memory, validate_name};
use proto_web::{CreateResourceLimitRequest, ResourceLimit as ProtoResourceLimit};
use std::collections::HashMap;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Default, Clone, PartialEq)]
struct ResourceLimitFormData {
    name: String,
    description: String,
    cpu: String,
    memory: String,
    cpu_limit: String,
    memory_limit: String,
    node_selector_yaml: String,
    node_affinity_yaml: String,
}

#[function_component(ResourceLimitCreate)]
pub fn resource_limit_create() -> Html {
    let form_data = use_state(ResourceLimitFormData::default);
    let errors = use_state(HashMap::<String, String>::new);
    let is_submitting = use_state(|| false);
    let submit_error = use_state(|| Option::<String>::None);
    let navigator = use_navigator().unwrap();

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

    let on_description_change = {
        let form_data = form_data.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let mut data = (*form_data).clone();
            data.description = input.value();
            form_data.set(data);
        })
    };

    let on_cpu_change = {
        let form_data = form_data.clone();
        let errors = errors.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let value = input.value();
            let mut data = (*form_data).clone();
            data.cpu = value.clone();
            form_data.set(data);

            let mut new_errors = (*errors).clone();
            if !value.is_empty() {
                if let Some(error) = validate_cpu(&value) {
                    new_errors.insert("cpu".to_string(), error);
                } else {
                    new_errors.remove("cpu");
                }
            } else {
                new_errors.remove("cpu");
            }
            errors.set(new_errors);
        })
    };

    let on_memory_change = {
        let form_data = form_data.clone();
        let errors = errors.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let value = input.value();
            let mut data = (*form_data).clone();
            data.memory = value.clone();
            form_data.set(data);

            let mut new_errors = (*errors).clone();
            if !value.is_empty() {
                if let Some(error) = validate_memory(&value) {
                    new_errors.insert("memory".to_string(), error);
                } else {
                    new_errors.remove("memory");
                }
            } else {
                new_errors.remove("memory");
            }
            errors.set(new_errors);
        })
    };

    let on_cpu_limit_change = {
        let form_data = form_data.clone();
        let errors = errors.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let value = input.value();
            let mut data = (*form_data).clone();
            data.cpu_limit = value.clone();
            form_data.set(data);

            let mut new_errors = (*errors).clone();
            if !value.is_empty() {
                if let Some(error) = validate_cpu(&value) {
                    new_errors.insert("cpu_limit".to_string(), error);
                } else {
                    new_errors.remove("cpu_limit");
                }
            } else {
                new_errors.remove("cpu_limit");
            }
            errors.set(new_errors);
        })
    };

    let on_memory_limit_change = {
        let form_data = form_data.clone();
        let errors = errors.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlInputElement = e.target_unchecked_into();
            let value = input.value();
            let mut data = (*form_data).clone();
            data.memory_limit = value.clone();
            form_data.set(data);

            let mut new_errors = (*errors).clone();
            if !value.is_empty() {
                if let Some(error) = validate_memory(&value) {
                    new_errors.insert("memory_limit".to_string(), error);
                } else {
                    new_errors.remove("memory_limit");
                }
            } else {
                new_errors.remove("memory_limit");
            }
            errors.set(new_errors);
        })
    };

    let on_node_selector_change = {
        let form_data = form_data.clone();
        let errors = errors.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
            let value = input.value();
            let mut data = (*form_data).clone();
            data.node_selector_yaml = value.clone();
            form_data.set(data);

            let mut new_errors = (*errors).clone();
            if !value.trim().is_empty() {
                match serde_yaml::from_str::<serde_yaml::Value>(&value) {
                    Ok(_) => {
                        new_errors.remove("node_selector_yaml");
                    }
                    Err(e) => {
                        new_errors.insert(
                            "node_selector_yaml".to_string(),
                            format!("Invalid YAML: {}", e),
                        );
                    }
                }
            } else {
                new_errors.remove("node_selector_yaml");
            }
            errors.set(new_errors);
        })
    };

    let on_node_affinity_change = {
        let form_data = form_data.clone();
        let errors = errors.clone();
        Callback::from(move |e: Event| {
            let input: web_sys::HtmlTextAreaElement = e.target_unchecked_into();
            let value = input.value();
            let mut data = (*form_data).clone();
            data.node_affinity_yaml = value.clone();
            form_data.set(data);

            let mut new_errors = (*errors).clone();
            if !value.trim().is_empty() {
                match serde_yaml::from_str::<serde_yaml::Value>(&value) {
                    Ok(_) => {
                        new_errors.remove("node_affinity_yaml");
                    }
                    Err(e) => {
                        new_errors.insert(
                            "node_affinity_yaml".to_string(),
                            format!("Invalid YAML: {}", e),
                        );
                    }
                }
            } else {
                new_errors.remove("node_affinity_yaml");
            }
            errors.set(new_errors);
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
            if data.cpu.is_empty() {
                validation_errors.insert("cpu".to_string(), "CPU is required".to_string());
            }
            if data.memory.is_empty() {
                validation_errors.insert("memory".to_string(), "Memory is required".to_string());
            }

            if !validation_errors.is_empty() {
                errors.set(validation_errors);
                return;
            }

            is_submitting.set(true);
            let is_submitting = is_submitting.clone();
            let submit_error = submit_error.clone();
            let navigator = navigator.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let node_selector = if !data.node_selector_yaml.trim().is_empty() {
                    match serde_yaml::from_str::<HashMap<String, String>>(&data.node_selector_yaml)
                    {
                        Ok(map) => map,
                        Err(e) => {
                            submit_error.set(Some(format!("Invalid node_selector YAML: {}", e)));
                            is_submitting.set(false);
                            return;
                        }
                    }
                } else {
                    HashMap::new()
                };

                let node_affinity = if !data.node_affinity_yaml.trim().is_empty() {
                    match serde_yaml::from_str::<serde_json::Value>(&data.node_affinity_yaml) {
                        Ok(yaml_value) => match serde_json::to_string(&yaml_value) {
                            Ok(json_str) => Some(json_str),
                            Err(e) => {
                                submit_error
                                    .set(Some(format!("Failed to serialize node_affinity: {}", e)));
                                is_submitting.set(false);
                                return;
                            }
                        },
                        Err(e) => {
                            submit_error.set(Some(format!("Invalid node_affinity YAML: {}", e)));
                            is_submitting.set(false);
                            return;
                        }
                    }
                } else {
                    None
                };

                let request = CreateResourceLimitRequest {
                    name: data.name.clone(),
                    description: data.description.clone(),
                    limits: Some(ProtoResourceLimit {
                        cpu: data.cpu.clone(),
                        memory: data.memory.clone(),
                        cpu_limit: if data.cpu_limit.is_empty() {
                            None
                        } else {
                            Some(data.cpu_limit.clone())
                        },
                        memory_limit: if data.memory_limit.is_empty() {
                            None
                        } else {
                            Some(data.memory_limit.clone())
                        },
                        ephemeral_storage: None,
                        volumes: HashMap::new(),
                        node_selector,
                        node_affinity,
                    }),
                    labels: HashMap::new(),
                };

                match create_resource_limit(request).await {
                    Ok(limit) => {
                        navigator.push(&Route::ResourceLimitDetail { name: limit.name });
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
            <div class="header">
                <h1>{ "Create Resource Limit" }</h1>
                <Link<Route> to={Route::ResourceLimitList}>
                    <button class="btn-secondary">{ "Cancel" }</button>
                </Link<Route>>
            </div>

            { if let Some(error) = &*submit_error {
                html! { <ErrorMessage message={error.clone()} /> }
            } else { html! {} }}

            <form onsubmit={on_submit} class="form">
                <FormField
                    label="Name *"
                    error={errors.get("name").cloned()}
                >
                    <input
                        type="text"
                        value={form_data.name.clone()}
                        onchange={on_name_change}
                        required={true}
                        placeholder="my-resource-limit"
                    />
                    <small class="form-help">{ "Lowercase alphanumeric and hyphens only" }</small>
                </FormField>

                <FormField
                    label="Description"
                    error={Option::<String>::None}
                >
                    <input
                        type="text"
                        value={form_data.description.clone()}
                        onchange={on_description_change}
                        placeholder="Description of this resource limit"
                    />
                </FormField>

                <div class="form-section">
                    <h3>{ "Resource Requests (Guaranteed)" }</h3>

                    <FormField
                        label="CPU *"
                        error={errors.get("cpu").cloned()}
                    >
                        <input
                            type="text"
                            value={form_data.cpu.clone()}
                            onchange={on_cpu_change}
                            required={true}
                            placeholder="2 or 500m"
                        />
                        <small class="form-help">{ "CPU cores (e.g., '2') or millicores (e.g., '500m')" }</small>
                    </FormField>

                    <FormField
                        label="Memory *"
                        error={errors.get("memory").cloned()}
                    >
                        <input
                            type="text"
                            value={form_data.memory.clone()}
                            onchange={on_memory_change}
                            required={true}
                            placeholder="4Gi or 512Mi"
                        />
                        <small class="form-help">{ "Memory size (e.g., '4Gi', '512Mi')" }</small>
                    </FormField>
                </div>

                <div class="form-section">
                    <h3>{ "Resource Limits (Maximum, Optional)" }</h3>

                    <FormField
                        label="CPU Limit"
                        error={errors.get("cpu_limit").cloned()}
                    >
                        <input
                            type="text"
                            value={form_data.cpu_limit.clone()}
                            onchange={on_cpu_limit_change}
                            placeholder="4 or 1000m"
                        />
                        <small class="form-help">{ "Maximum CPU (must be >= CPU request)" }</small>
                    </FormField>

                    <FormField
                        label="Memory Limit"
                        error={errors.get("memory_limit").cloned()}
                    >
                        <input
                            type="text"
                            value={form_data.memory_limit.clone()}
                            onchange={on_memory_limit_change}
                            placeholder="8Gi or 1024Mi"
                        />
                        <small class="form-help">{ "Maximum memory (must be >= memory request)" }</small>
                    </FormField>
                </div>

                <div class="form-section">
                    <h3>{ "Node Scheduling (Optional)" }</h3>
                    <p class="form-help">{ "Configure pod placement using Kubernetes scheduling. Leave empty to schedule on any available node." }</p>

                    <FormField
                        label="Node Selector (YAML)"
                        error={errors.get("node_selector_yaml").cloned()}
                    >
                        <textarea
                            value={form_data.node_selector_yaml.clone()}
                            onchange={on_node_selector_change}
                            rows="6"
                            placeholder="gpu: \"true\"\nregion: us-west"
                            style="font-family: monospace; font-size: 12px;"
                        />
                    </FormField>

                    <FormField
                        label="Node Affinity (YAML)"
                        error={errors.get("node_affinity_yaml").cloned()}
                    >
                        <textarea
                            value={form_data.node_affinity_yaml.clone()}
                            onchange={on_node_affinity_change}
                            rows="12"
                            placeholder="nodeAffinity:\n  requiredDuringSchedulingIgnoredDuringExecution:\n    nodeSelectorTerms:\n    - matchExpressions:\n      - key: topology.kubernetes.io/zone\n        operator: In\n        values:\n        - us-west-1a"
                            style="font-family: monospace; font-size: 12px;"
                        />
                    </FormField>
                </div>

                <div class="form-actions">
                    <button
                        type="submit"
                        class="btn-primary"
                        disabled={*is_submitting || !errors.is_empty()}
                    >
                        { if *is_submitting { "Creating..." } else { "Create Resource Limit" } }
                    </button>
                    <Link<Route> to={Route::ResourceLimitList}>
                        <button type="button" class="btn-secondary">{ "Cancel" }</button>
                    </Link<Route>>
                </div>
            </form>
        </div>
    }
}
