use crate::api::APICaller;
use crate::components::{ConfirmDialog, ErrorMessage, Loading};
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
    let show_delete_dialog = use_state(|| false);
    let navigator = use_navigator().unwrap();
    let (auth_state, _) = use_store::<AuthState>();

    {
        let load_state = load_state.clone();
        let namespace = props.namespace.clone();
        let name = props.name.clone();
        let auth_state = auth_state.clone();

        use_effect_with((namespace.clone(), name.clone()), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let api = APICaller::new(auth_state.access_token.clone());
                match api.get_template(&namespace, &name).await {
                    Ok(template) => load_state.set(LoadState::Loaded(Box::new(template))),
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
                LoadState::Loaded(template) => html! {
                    <>
                        <div class="header">
                            <div>
                                <h1>{ &template.name }</h1>
                                <span class="namespace-badge">{ &template.namespace }</span>
                            </div>
                            <div class="button-group">
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

                        <ConfirmDialog
                            show={*show_delete_dialog}
                            title="Delete Template"
                            message={format!("Are you sure you want to delete template '{}/{}'? This action cannot be undone.", template.namespace, template.name)}
                            on_confirm={on_delete_confirm}
                            on_cancel={on_delete_cancel}
                        />
                    </>
                }
            }}
        </div>
    }
}
