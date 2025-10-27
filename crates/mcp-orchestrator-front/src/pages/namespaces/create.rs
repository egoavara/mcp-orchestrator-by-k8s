use yew::prelude::*;
use yew_router::prelude::*;
use std::collections::HashMap;
use crate::api::namespaces::create_namespace;
use crate::routes::Route;
use crate::components::{FormField, ErrorMessage};
use crate::utils::validation::validate_name;
use proto_web::CreateNamespaceRequest;

#[derive(Default, Clone, PartialEq)]
struct NamespaceFormData {
    name: String,
    labels: Vec<(String, String)>,
}

#[function_component(NamespaceCreate)]
pub fn namespace_create() -> Html {
    let form_data = use_state(NamespaceFormData::default);
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

    let on_add_label = {
        let form_data = form_data.clone();
        Callback::from(move |_| {
            let mut data = (*form_data).clone();
            data.labels.push(("".to_string(), "".to_string()));
            form_data.set(data);
        })
    };

    let on_remove_label = {
        let form_data = form_data.clone();
        move |index: usize| {
            let mut data = (*form_data).clone();
            data.labels.remove(index);
            form_data.set(data);
        }
    };

    let on_label_key_change = {
        let form_data = form_data.clone();
        move |index: usize, value: String| {
            let mut data = (*form_data).clone();
            if let Some(label) = data.labels.get_mut(index) {
                label.0 = value;
            }
            form_data.set(data);
        }
    };

    let on_label_value_change = {
        let form_data = form_data.clone();
        move |index: usize, value: String| {
            let mut data = (*form_data).clone();
            if let Some(label) = data.labels.get_mut(index) {
                label.1 = value;
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
            
            if !validation_errors.is_empty() {
                errors.set(validation_errors);
                return;
            }
            
            is_submitting.set(true);
            let errors = errors.clone();
            let is_submitting = is_submitting.clone();
            let submit_error = submit_error.clone();
            let navigator = navigator.clone();
            
            wasm_bindgen_futures::spawn_local(async move {
                let labels_map: HashMap<String, String> = data.labels
                    .into_iter()
                    .filter(|(k, v)| !k.is_empty() && !v.is_empty())
                    .collect();
                
                let request = CreateNamespaceRequest {
                    name: data.name.clone(),
                    labels: labels_map,
                };
                
                match create_namespace(request).await {
                    Ok(namespace) => {
                        navigator.push(&Route::NamespaceDetail {
                            name: namespace.name,
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
            <div class="header">
                <h1>{ "Create Namespace" }</h1>
                <Link<Route> to={Route::NamespaceList}>
                    <button class="btn-secondary">{ "Cancel" }</button>
                </Link<Route>>
            </div>

            { if let Some(error) = &*submit_error {
                html! { <ErrorMessage message={error.clone()} /> }
            } else { html! {} }}

            <form onsubmit={on_submit} class="form">
                <FormField 
                    label="Namespace Name *"
                    error={errors.get("name").cloned()}
                >
                    <input 
                        type="text"
                        value={form_data.name.clone()}
                        onchange={on_name_change}
                        required={true}
                        placeholder="my-namespace"
                    />
                    <small class="form-help">{ "Lowercase alphanumeric and hyphens only, max 63 characters" }</small>
                </FormField>

                <div class="form-section">
                    <label class="section-label">{ "Labels (optional)" }</label>
                    <small class="form-help">{ "Add key-value pairs to organize and filter namespaces" }</small>
                    
                    { for form_data.labels.iter().enumerate().map(|(index, (key, value))| {
                        let on_key_change = {
                            let on_label_key_change = on_label_key_change.clone();
                            Callback::from(move |e: Event| {
                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                on_label_key_change(index, input.value());
                            })
                        };
                        let on_value_change = {
                            let on_label_value_change = on_label_value_change.clone();
                            Callback::from(move |e: Event| {
                                let input: web_sys::HtmlInputElement = e.target_unchecked_into();
                                on_label_value_change(index, input.value());
                            })
                        };
                        let on_remove = {
                            let on_remove_label = on_remove_label.clone();
                            Callback::from(move |_| on_remove_label(index))
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
                        onclick={on_add_label}
                        class="btn-secondary-small"
                    >
                        { "+ Add Label" }
                    </button>
                </div>

                <div class="form-actions">
                    <button 
                        type="submit"
                        class="btn-primary"
                        disabled={*is_submitting || !errors.is_empty()}
                    >
                        { if *is_submitting { "Creating..." } else { "Create Namespace" } }
                    </button>
                    <Link<Route> to={Route::NamespaceList}>
                        <button type="button" class="btn-secondary">{ "Cancel" }</button>
                    </Link<Route>>
                </div>
            </form>
        </div>
    }
}
