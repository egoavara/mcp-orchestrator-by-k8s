use crate::api::APICaller;
use crate::components::{ConfirmDialog, CopyConfigDialog, ErrorMessage, Loading};
use crate::models::authorization::Authorization;
use crate::models::state::AuthState;
use crate::models::template::Template;
use crate::routes::Route;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TemplateDetailProps {
    pub namespace: String,
    pub name: String,
}

#[derive(Debug)]
enum LoadState {
    Loading,
    Loaded(Box<Template>),
    Error(String),
}

#[function_component(TemplateDetail)]
pub fn template_detail(props: &TemplateDetailProps) -> Html {
    let load_state = use_state(|| LoadState::Loading);
    let authorization_state = use_state(|| Option::<Authorization>::None);
    let show_delete_dialog = use_state(|| false);
    let show_copy_dialog = use_state(|| false);
    let copy_config = use_state(|| String::new());
    let navigator = use_navigator().unwrap();
    let (auth_state, _) = use_store::<AuthState>();

    {
        let load_state = load_state.clone();
        let authorization_state = authorization_state.clone();
        let namespace = props.namespace.clone();
        let name = props.name.clone();
        let auth_state = auth_state.clone();

        use_effect_with((namespace.clone(), name.clone()), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let api = APICaller::new(auth_state.access_token.clone());
                match api.get_template(&namespace, &name).await {
                    Ok(template) => {
                        // Load authorization if specified
                        if let Some(auth_name) = &template.authorization_name {
                            if !auth_name.is_empty() {
                                match api.get_authorization(template.namespace.clone(), auth_name.clone()).await {
                                    Ok(auth) => authorization_state.set(Some(auth)),
                                    Err(e) => {
                                        web_sys::console::error_1(&format!("Failed to load authorization: {}", e).into());
                                        authorization_state.set(None);
                                    }
                                }
                            }
                        }
                        load_state.set(LoadState::Loaded(Box::new(template)));
                    }
                    Err(e) => load_state.set(LoadState::Error(e)),
                }
            });
            || ()
        });
    }

    let on_delete_click = {
        let show_delete_dialog = show_delete_dialog.clone();
        Callback::from(move |_| {
            show_delete_dialog.set(true);
        })
    };

    let on_delete_confirm = {
        let namespace = props.namespace.clone();
        let name = props.name.clone();
        let navigator = navigator.clone();
        let auth_state = auth_state.clone();
        Callback::from(move |_| {
            let namespace = namespace.clone();
            let name = name.clone();
            let navigator = navigator.clone();
            let auth_state = auth_state.clone();
            wasm_bindgen_futures::spawn_local(async move {
                let api = APICaller::new(auth_state.access_token.clone());
                match api.delete_template(&namespace, &name).await {
                    Ok(_) => {
                        navigator.push(&Route::TemplateList);
                    }
                    Err(e) => {
                        web_sys::console::error_1(&format!("Delete failed: {}", e).into());
                    }
                }
            });
        })
    };

    let on_delete_cancel = {
        let show_delete_dialog = show_delete_dialog.clone();
        Callback::from(move |_| {
            show_delete_dialog.set(false);
        })
    };

    let generate_mcp_config = |template: &Template, authorization: &Option<Authorization>| -> String {
        let current_origin = web_sys::window()
            .and_then(|w| w.location().origin().ok())
            .unwrap_or_else(|| "http://localhost:8080".to_string());
        
        let url = format!("{}/mcp/{}/{}", current_origin, template.namespace, template.name);
        
        let has_arg_envs = !template.arg_envs.is_empty();
        let has_k8s_sa_auth = authorization
            .as_ref()
            .map(|auth| auth.auth_type == 1) // KUBERNETES_SERVICE_ACCOUNT = 1
            .unwrap_or(false);
        
        let mut headers = Vec::new();
        
        // Add arg-env headers
        if has_arg_envs {
            for (key, value) in &template.arg_envs {
                let type_example = if value.contains("?") || value.ends_with("?") {
                    "\"value\" or null"
                } else {
                    "\"value\""
                };
                headers.push(format!("    \"arg-{}\": {}", key, type_example));
            }
        }
        
        // Add Authorization header if K8s SA
        if has_k8s_sa_auth {
            headers.push(format!("    \"Authorization\": \"Bearer <token>\""));
        }
        
        if !headers.is_empty() {
            format!(
r#"{{
  "mcpServers": {{
    "{}": {{
      "type": "remote",
      "url": "{}",
      "headers": {{
{}
      }}
    }}
  }}
}}"#,
                template.name,
                url,
                headers.join(",\n")
            )
        } else {
            format!(
r#"{{
  "mcpServers": {{
    "{}": {{
      "type": "remote",
      "url": "{}"
    }}
  }}
}}"#,
                template.name,
                url
            )
        }
    };

    let on_copy_config = {
        let show_copy_dialog = show_copy_dialog.clone();
        let copy_config = copy_config.clone();
        let authorization_state = authorization_state.clone();
        move |template: &Template| {
            let config = generate_mcp_config(template, &*authorization_state);
            copy_config.set(config);
            show_copy_dialog.set(true);
        }
    };

    let on_close_copy_dialog = {
        let show_copy_dialog = show_copy_dialog.clone();
        Callback::from(move |_| {
            show_copy_dialog.set(false);
        })
    };

    html! {
        <div class="container">
            { match &*load_state {
                LoadState::Loading => html! { <Loading /> },
                LoadState::Error(e) => html! {
                    <>
                        <ErrorMessage message={e.clone()} />
                        <Link<Route> to={Route::TemplateList}>
                            <button class="btn-secondary">{ "Back to List" }</button>
                        </Link<Route>>
                    </>
                },
                LoadState::Loaded(template) => {
                    let template_for_copy = template.as_ref().clone();
                    let on_copy_click = {
                        let on_copy_config = on_copy_config.clone();
                        Callback::from(move |_| {
                            on_copy_config(&template_for_copy);
                        })
                    };
                    
                    html! {
                    <>
                        <div class="header">
                            <div>
                                <h1>{ &template.name }</h1>
                                <span class="namespace-badge">{ &template.namespace }</span>
                            </div>
                            <div class="button-group">
                                <button class="btn-primary" onclick={on_copy_click} title="Copy MCP configuration">
                                    { "📋 Copy Config" }
                                </button>
                                <Link<Route> to={Route::TemplateList}>
                                    <button class="btn-secondary">{ "Back to List" }</button>
                                </Link<Route>>
                                <button class="btn-danger" onclick={on_delete_click}>{ "Delete" }</button>
                            </div>
                        </div>

                        <div class="detail-sections">
                            <section class="detail-section">
                                <h2>{ "Basic Information" }</h2>
                                <div class="field-list">
                                    <div class="field">
                                        <label>{ "Namespace:" }</label>
                                        <span>{ &template.namespace }</span>
                                    </div>
                                    <div class="field">
                                        <label>{ "Name:" }</label>
                                        <span>{ &template.name }</span>
                                    </div>
                                    <div class="field">
                                        <label>{ "Image:" }</label>
                                        <code>{ &template.image }</code>
                                    </div>
                                    <div class="field">
                                        <label>{ "Created At:" }</label>
                                        <span>{ &template.created_at }</span>
                                    </div>
                                </div>
                            </section>

                            { if !template.command.is_empty() || !template.args.is_empty() {
                                html! {
                                    <section class="detail-section">
                                        <h2>{ "Command & Arguments" }</h2>
                                        { if !template.command.is_empty() {
                                            html! {
                                                <div class="field">
                                                    <label>{ "Command:" }</label>
                                                    <code>{ template.command.join(" ") }</code>
                                                </div>
                                            }
                                        } else { html! {} }}
                                        { if !template.args.is_empty() {
                                            html! {
                                                <div class="field">
                                                    <label>{ "Arguments:" }</label>
                                                    <code>{ template.args.join(" ") }</code>
                                                </div>
                                            }
                                        } else { html! {} }}
                                    </section>
                                }
                            } else { html! {} }}

                            { if !template.envs.is_empty() {
                                html! {
                                    <section class="detail-section">
                                        <h2>{ "Environment Variables" }</h2>
                                        <table class="data-table">
                                            <thead>
                                                <tr>
                                                    <th>{ "Key" }</th>
                                                    <th>{ "Value" }</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                { for template.envs.iter().map(|(key, value)| {
                                                    html! {
                                                        <tr key={key.clone()}>
                                                            <td><code>{ key }</code></td>
                                                            <td><code>{ value }</code></td>
                                                        </tr>
                                                    }
                                                }) }
                                            </tbody>
                                        </table>
                                    </section>
                                }
                            } else { html! {} }}

                            { if !template.arg_envs.is_empty() {
                                html! {
                                    <section class="detail-section">
                                        <h2>{ "Argument Environment Variables" }</h2>
                                        <table class="data-table">
                                            <thead>
                                                <tr>
                                                    <th>{ "Key" }</th>
                                                    <th>{ "Value" }</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                { for template.arg_envs.iter().map(|(key, value)| {
                                                    html! {
                                                        <tr key={key.clone()}>
                                                            <td><code>{ key }</code></td>
                                                            <td><code>{ value }</code></td>
                                                        </tr>
                                                    }
                                                }) }
                                            </tbody>
                                        </table>
                                    </section>
                                }
                            } else { html! {} }}

                            { if !template.secret_envs.is_empty() {
                                html! {
                                    <section class="detail-section">
                                        <h2>{ "Secret Environment Variables" }</h2>
                                        <ul class="tag-list">
                                            { for template.secret_envs.iter().map(|secret| {
                                                html! {
                                                    <li key={secret.clone()} class="tag">{ secret }</li>
                                                }
                                            }) }
                                        </ul>
                                    </section>
                                }
                            } else { html! {} }}

                            { if let Some(resource_limit) = &template.resource_limit_name {
                                html! {
                                    <section class="detail-section">
                                        <h2>{ "Resource Limits" }</h2>
                                        <div class="field">
                                            <label>{ "Resource Limit Configuration:" }</label>
                                            <span>{ resource_limit }</span>
                                        </div>
                                    </section>
                                }
                            } else { html! {} }}

                            { if !template.labels.is_empty() {
                                html! {
                                    <section class="detail-section">
                                        <h2>{ "Labels" }</h2>
                                        <table class="data-table">
                                            <thead>
                                                <tr>
                                                    <th>{ "Key" }</th>
                                                    <th>{ "Value" }</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                { for template.labels.iter().map(|(key, value)| {
                                                    html! {
                                                        <tr key={key.clone()}>
                                                            <td><code>{ key }</code></td>
                                                            <td>{ value }</td>
                                                        </tr>
                                                    }
                                                }) }
                                            </tbody>
                                        </table>
                                    </section>
                                }
                            } else { html! {} }}
                        </div>

                        <CopyConfigDialog 
                            show={*show_copy_dialog}
                            config_json={(*copy_config).clone()}
                            on_close={on_close_copy_dialog.clone()}
                        />

                        <ConfirmDialog
                            show={*show_delete_dialog}
                            title="Delete Template"
                            message={format!("Are you sure you want to delete template '{}/{}'? This action cannot be undone.", template.namespace, template.name)}
                            on_confirm={on_delete_confirm}
                            on_cancel={on_delete_cancel}
                        />
                    </>
                    }
                }
            }}
        </div>
    }
}
