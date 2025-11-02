use crate::api::APICaller;
use crate::components::{CopyConfigDialog, ErrorMessage, Loading, NamespaceSelector};
use crate::models::authorization::Authorization;
use crate::models::state::AuthState;
use crate::models::template::Template;
use crate::models::SessionState;
use crate::routes::Route;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[derive(Debug)]
enum LoadState {
    Loading,
    Loaded(Vec<Template>),
    Error(String),
}

#[function_component(TemplateList)]
pub fn template_list() -> Html {
    let load_state = use_state(|| LoadState::Loading);
    let (session_state, _) = use_store::<SessionState>();
    let (auth_state, _) = use_store::<AuthState>();
    let namespace = session_state
        .selected_namespace
        .clone()
        .unwrap_or_else(|| "default".to_string());
    
    let show_copy_dialog = use_state(|| false);
    let copy_config = use_state(|| String::new());

    {
        let load_state = load_state.clone();
        let namespace = namespace.clone();
        let api = APICaller::new(auth_state.access_token.clone());
        use_effect_with(namespace.clone(), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match api.list_templates(&namespace).await {
                    Ok(templates) => load_state.set(LoadState::Loaded(templates)),
                    Err(e) => load_state.set(LoadState::Error(e)),
                }
            });
            || ()
        });
    }

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
        let auth_state = auth_state.clone();
        move |template: Template| {
            let show_copy_dialog = show_copy_dialog.clone();
            let copy_config = copy_config.clone();
            let auth_state = auth_state.clone();
            
            wasm_bindgen_futures::spawn_local(async move {
                let api = APICaller::new(auth_state.access_token.clone());
                
                // Load authorization if specified
                let authorization = if let Some(auth_name) = &template.authorization_name {
                    if !auth_name.is_empty() {
                        match api.get_authorization(template.namespace.clone(), auth_name.clone()).await {
                            Ok(auth) => Some(auth),
                            Err(e) => {
                                web_sys::console::error_1(&format!("Failed to load authorization: {}", e).into());
                                None
                            }
                        }
                    } else {
                        None
                    }
                } else {
                    None
                };
                
                let config = generate_mcp_config(&template, &authorization);
                copy_config.set(config);
                show_copy_dialog.set(true);
            });
        }
    };

    let on_close_dialog = {
        let show_copy_dialog = show_copy_dialog.clone();
        Callback::from(move |_| {
            show_copy_dialog.set(false);
        })
    };

    html! {
        <div class="container">
            <NamespaceSelector />

            <div class="header">
                <h1>{ "MCP Templates" }</h1>
                <Link<Route> to={Route::TemplateCreate}>
                    <button class="btn-primary">{ "Create Template" }</button>
                </Link<Route>>
            </div>

            <CopyConfigDialog 
                show={*show_copy_dialog}
                config_json={(*copy_config).clone()}
                on_close={on_close_dialog}
            />

            { match &*load_state {
                LoadState::Loading => html! { <Loading /> },
                LoadState::Error(e) => html! { <ErrorMessage message={e.clone()} /> },
                LoadState::Loaded(templates) => html! {
                    <div class="template-list">
                        { if templates.is_empty() {
                            html! {
                                <div class="empty-state">
                                    <p>{ "No templates found. Create your first template to get started." }</p>
                                </div>
                            }
                        } else {
                            html! {
                                <div class="grid">
                                    { for templates.iter().map(|template| {
                                        let namespace = template.namespace.clone();
                                        let name = template.name.clone();
                                        let template_for_copy = template.clone();
                                        let on_copy_click = {
                                            let on_copy_config = on_copy_config.clone();
                                            Callback::from(move |_| {
                                                on_copy_config(template_for_copy.clone());
                                            })
                                        };
                                        
                                        html! {
                                            <div class="card" key={format!("{}/{}", namespace, name)}>
                                                <div class="card-header">
                                                    <h3>{ &template.name }</h3>
                                                    <span class="namespace-badge">{ &template.namespace }</span>
                                                </div>
                                                <div class="card-body">
                                                    <div class="field">
                                                        <label>{ "Image:" }</label>
                                                        <span>{ &template.image }</span>
                                                    </div>
                                                    { if !template.command.is_empty() {
                                                        html! {
                                                            <div class="field">
                                                                <label>{ "Command:" }</label>
                                                                <code>{ template.command.join(" ") }</code>
                                                            </div>
                                                        }
                                                    } else {
                                                        html! {}
                                                    }}
                                                    <div class="field">
                                                        <label>{ "Created:" }</label>
                                                        <span>{ &template.created_at }</span>
                                                    </div>
                                                </div>
                                                <div class="card-footer">
                                                    <button class="btn-primary" onclick={on_copy_click} title="Copy MCP configuration">
                                                        { "📋 Copy Config" }
                                                    </button>
                                                    <Link<Route> to={Route::TemplateDetail { namespace: namespace.clone(), name: name.clone() }}>
                                                        <button class="btn-secondary">{ "View Details" }</button>
                                                    </Link<Route>>
                                                </div>
                                            </div>
                                        }
                                    }) }
                                </div>
                            }
                        }}
                    </div>
                }
            }}
        </div>
    }
}
