use crate::api::templates::list_templates;
use crate::components::{ErrorMessage, Loading, NamespaceSelector};
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
    let namespace = session_state
        .selected_namespace
        .clone()
        .unwrap_or_else(|| "default".to_string());

    {
        let load_state = load_state.clone();
        let namespace = namespace.clone();
        use_effect_with(namespace.clone(), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match list_templates(&namespace).await {
                    Ok(templates) => load_state.set(LoadState::Loaded(templates)),
                    Err(e) => load_state.set(LoadState::Error(e)),
                }
            });
            || ()
        });
    }

    html! {
        <div class="container">
            <NamespaceSelector />

            <div class="header">
                <h1>{ "MCP Templates" }</h1>
                <Link<Route> to={Route::TemplateCreate}>
                    <button class="btn-primary">{ "Create Template" }</button>
                </Link<Route>>
            </div>

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
