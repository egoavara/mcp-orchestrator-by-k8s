use crate::api::APICaller;
use crate::components::{ConfirmDialog, ErrorMessage, Loading};
use crate::models::resource_limit::ResourceLimit;
use crate::models::state::AuthState;
use crate::routes::Route;
use yew::prelude::*;
use yew_router::prelude::*;
use yewdux::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub name: String,
}

#[derive(Debug)]
enum LoadState {
    Loading,
    Loaded(ResourceLimit),
    Error(String),
}

#[function_component(ResourceLimitDetail)]
pub fn resource_limit_detail(props: &Props) -> Html {
    let load_state = use_state(|| LoadState::Loading);
    let show_delete_confirm = use_state(|| false);
    let is_deleting = use_state(|| false);
    let delete_error = use_state(|| Option::<String>::None);
    let navigator = use_navigator().unwrap();
    let (auth_state, _) = use_store::<AuthState>();

    let name = props.name.clone();

    {
        let load_state = load_state.clone();
        let name = name.clone();
        let auth_state = auth_state.clone();
        use_effect_with(name.clone(), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                let api = APICaller::new(auth_state.access_token.clone());
                match api.get_resource_limit(&name).await {
                    Ok(limit) => load_state.set(LoadState::Loaded(limit)),
                    Err(e) => load_state.set(LoadState::Error(e)),
                }
            });
            || ()
        });
    }

    let on_delete_click = {
        let show_delete_confirm = show_delete_confirm.clone();
        Callback::from(move |_| {
            show_delete_confirm.set(true);
        })
    };

    let on_delete_confirm = {
        let is_deleting = is_deleting.clone();
        let delete_error = delete_error.clone();
        let show_delete_confirm = show_delete_confirm.clone();
        let navigator = navigator.clone();
        let name = name.clone();
        let auth_state = auth_state.clone();

        Callback::from(move |_| {
            is_deleting.set(true);
            let is_deleting = is_deleting.clone();
            let delete_error = delete_error.clone();
            let show_delete_confirm = show_delete_confirm.clone();
            let navigator = navigator.clone();
            let name = name.clone();
            let auth_state = auth_state.clone();

            wasm_bindgen_futures::spawn_local(async move {
                let api = APICaller::new(auth_state.access_token.clone());
                match api.delete_resource_limit(&name).await {
                    Ok(_) => {
                        navigator.push(&Route::ResourceLimitList);
                    }
                    Err(e) => {
                        delete_error.set(Some(e));
                        is_deleting.set(false);
                        show_delete_confirm.set(false);
                    }
                }
            });
        })
    };

    let on_delete_cancel = {
        let show_delete_confirm = show_delete_confirm.clone();
        Callback::from(move |_| {
            show_delete_confirm.set(false);
        })
    };

    html! {
        <div class="container">
            <div class="header">
                <h1>{ "Resource Limit Details" }</h1>
                <div style="display: flex; gap: 0.75rem;">
                    <button
                        class="btn-danger"
                        onclick={on_delete_click}
                        disabled={*is_deleting}
                    >
                        { "Delete" }
                    </button>
                    <Link<Route> to={Route::ResourceLimitList}>
                        <button class="btn-secondary">{ "Back to List" }</button>
                    </Link<Route>>
                </div>
            </div>

            { if let Some(error) = &*delete_error {
                html! { <ErrorMessage message={error.clone()} /> }
            } else { html! {} }}

            { match &*load_state {
                LoadState::Loading => html! { <Loading /> },
                LoadState::Error(e) => html! { <ErrorMessage message={e.clone()} /> },
                LoadState::Loaded(limit) => html! {
                    <div class="detail-card">
                        <div class="detail-section">
                            <h2>{ "Basic Information" }</h2>
                            <div class="detail-grid">
                                <div class="detail-field">
                                    <label>{ "Name:" }</label>
                                    <span>{ &limit.name }</span>
                                </div>
                                <div class="detail-field">
                                    <label>{ "Status:" }</label>
                                    { if limit.deleted_at.is_some() {
                                        html! { <span class="badge badge-danger">{ "Deleted" }</span> }
                                    } else {
                                        html! { <span class="badge badge-success">{ "Active" }</span> }
                                    }}
                                </div>
                                <div class="detail-field">
                                    <label>{ "Created At:" }</label>
                                    <span>{ &limit.created_at }</span>
                                </div>
                                { if !limit.description.is_empty() {
                                    html! {
                                        <div class="detail-field">
                                            <label>{ "Description:" }</label>
                                            <span>{ &limit.description }</span>
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }}
                            </div>
                        </div>

                        <div class="detail-section">
                            <h2>{ "Resource Requests (Guaranteed)" }</h2>
                            <div class="detail-grid">
                                <div class="detail-field">
                                    <label>{ "CPU:" }</label>
                                    <span>{ &limit.limits.cpu }</span>
                                </div>
                                <div class="detail-field">
                                    <label>{ "Memory:" }</label>
                                    <span>{ &limit.limits.memory }</span>
                                </div>
                            </div>
                        </div>

                        { if limit.limits.cpu_limit.is_some() || limit.limits.memory_limit.is_some() {
                            html! {
                                <div class="detail-section">
                                    <h2>{ "Resource Limits (Maximum)" }</h2>
                                    <div class="detail-grid">
                                        { if let Some(cpu_limit) = &limit.limits.cpu_limit {
                                            html! {
                                                <div class="detail-field">
                                                    <label>{ "CPU Limit:" }</label>
                                                    <span>{ cpu_limit }</span>
                                                </div>
                                            }
                                        } else {
                                            html! {}
                                        }}
                                        { if let Some(memory_limit) = &limit.limits.memory_limit {
                                            html! {
                                                <div class="detail-field">
                                                    <label>{ "Memory Limit:" }</label>
                                                    <span>{ memory_limit }</span>
                                                </div>
                                            }
                                        } else {
                                            html! {}
                                        }}
                                    </div>
                                </div>
                            }
                        } else {
                            html! {}
                        }}

                        { if limit.limits.node_selector.is_some() || limit.limits.node_affinity.is_some() {
                            html! {
                                <div class="detail-section">
                                    <h2>{ "Node Scheduling Configuration" }</h2>
                                    { if let Some(node_selector_yaml) = &limit.limits.node_selector {
                                        html! {
                                            <div class="detail-field" style="margin-bottom: 1.5rem;">
                                                <label>{ "Node Selector:" }</label>
                                                <pre style="background: var(--bg-primary); color: var(--text-primary); padding: 1rem; border-radius: 4px; overflow-x: auto; font-family: monospace; font-size: 12px; border: 1px solid var(--border-color);">
                                                    { node_selector_yaml }
                                                </pre>
                                            </div>
                                        }
                                    } else {
                                        html! {}
                                    }}
                                    { if let Some(node_affinity_yaml) = &limit.limits.node_affinity {
                                        html! {
                                            <div class="detail-field">
                                                <label>{ "Node Affinity:" }</label>
                                                <pre style="background: var(--bg-primary); color: var(--text-primary); padding: 1rem; border-radius: 4px; overflow-x: auto; font-family: monospace; font-size: 12px; border: 1px solid var(--border-color);">
                                                    { node_affinity_yaml }
                                                </pre>
                                            </div>
                                        }
                                    } else {
                                        html! {}
                                    }}
                                </div>
                            }
                        } else {
                            html! {}
                        }}

                        { if !limit.labels.is_empty() {
                            html! {
                                <div class="detail-section">
                                    <h2>{ "Labels" }</h2>
                                    <div class="tags">
                                        { for limit.labels.iter().map(|(k, v)| {
                                            html! {
                                                <span class="tag" key={k.clone()}>
                                                    { format!("{}={}", k, v) }
                                                </span>
                                            }
                                        })}
                                    </div>
                                </div>
                            }
                        } else {
                            html! {}
                        }}
                    </div>
                }
            }}

            <ConfirmDialog
                title="Delete Resource Limit"
                message={format!("Are you sure you want to delete resource limit '{}'? This may affect templates using this limit.", &name)}
                on_confirm={on_delete_confirm}
                on_cancel={on_delete_cancel}
                show={*show_delete_confirm}
            />
        </div>
    }
}
