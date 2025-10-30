use crate::api::resource_limits::list_resource_limits;
use crate::components::{ErrorMessage, Loading};
use crate::models::resource_limit::ResourceLimit;
use crate::routes::Route;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Debug)]
enum LoadState {
    Loading,
    Loaded(Vec<ResourceLimit>),
    Error(String),
}

#[function_component(ResourceLimitList)]
pub fn resource_limit_list() -> Html {
    let load_state = use_state(|| LoadState::Loading);

    {
        let load_state = load_state.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match list_resource_limits().await {
                    Ok(limits) => load_state.set(LoadState::Loaded(limits)),
                    Err(e) => load_state.set(LoadState::Error(e)),
                }
            });
            || ()
        });
    }

    html! {
        <div class="container">
            <div class="header">
                <h1>{ "Resource Limits" }</h1>
                <Link<Route> to={Route::ResourceLimitCreate}>
                    <button class="btn-primary">{ "Create Resource Limit" }</button>
                </Link<Route>>
            </div>

            { match &*load_state {
                LoadState::Loading => html! { <Loading /> },
                LoadState::Error(e) => html! { <ErrorMessage message={e.clone()} /> },
                LoadState::Loaded(limits) => html! {
                    <div class="resource-limit-list">
                        { if limits.is_empty() {
                            html! {
                                <div class="empty-state">
                                    <p>{ "No resource limits found. Create your first resource limit to get started." }</p>
                                </div>
                            }
                        } else {
                            html! {
                                <div class="grid">
                                    { for limits.iter().map(|limit| {
                                        let name = limit.name.clone();
                                        html! {
                                            <div class="card" key={name.clone()}>
                                                <div class="card-header">
                                                    <h3>{ &limit.name }</h3>
                                                    { if limit.deleted_at.is_some() {
                                                        html! { <span class="badge badge-danger">{ "Deleted" }</span> }
                                                    } else {
                                                        html! { <span class="badge badge-success">{ "Active" }</span> }
                                                    }}
                                                </div>
                                                <div class="card-body">
                                                    { if !limit.description.is_empty() {
                                                        html! {
                                                            <div class="field">
                                                                <label>{ "Description:" }</label>
                                                                <span>{ &limit.description }</span>
                                                            </div>
                                                        }
                                                    } else {
                                                        html! {}
                                                    }}
                                                    <div class="field">
                                                        <label>{ "CPU:" }</label>
                                                        <span>{ &limit.limits.cpu }</span>
                                                    </div>
                                                    <div class="field">
                                                        <label>{ "Memory:" }</label>
                                                        <span>{ &limit.limits.memory }</span>
                                                    </div>
                                                    <div class="field">
                                                        <label>{ "Created:" }</label>
                                                        <span>{ &limit.created_at }</span>
                                                    </div>
                                                </div>
                                                <div class="card-footer">
                                                    <Link<Route> to={Route::ResourceLimitDetail { name: name.clone() }}>
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
