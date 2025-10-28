use crate::api::namespaces::list_namespaces;
use crate::components::{ErrorMessage, Loading};
use crate::models::namespace::Namespace;
use crate::routes::Route;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Debug)]
enum LoadState {
    Loading,
    Loaded(Vec<Namespace>),
    Error(String),
}

#[function_component(NamespaceList)]
pub fn namespace_list() -> Html {
    let load_state = use_state(|| LoadState::Loading);

    {
        let load_state = load_state.clone();
        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match list_namespaces().await {
                    Ok(namespaces) => load_state.set(LoadState::Loaded(namespaces)),
                    Err(e) => load_state.set(LoadState::Error(e)),
                }
            });
            || ()
        });
    }

    html! {
        <div class="container">
            <div class="header">
                <h1>{ "Namespaces" }</h1>
                <Link<Route> to={Route::NamespaceCreate}>
                    <button class="btn-primary">{ "Create Namespace" }</button>
                </Link<Route>>
            </div>

            { match &*load_state {
                LoadState::Loading => html! { <Loading /> },
                LoadState::Error(e) => html! { <ErrorMessage message={e.clone()} /> },
                LoadState::Loaded(namespaces) => html! {
                    <div class="namespace-list">
                        { if namespaces.is_empty() {
                            html! {
                                <div class="empty-state">
                                    <p>{ "No namespaces found. Create your first namespace to get started." }</p>
                                </div>
                            }
                        } else {
                            html! {
                                <div class="grid">
                                    { for namespaces.iter().map(|namespace| {
                                        let name = namespace.name.clone();
                                        html! {
                                            <div class="card" key={name.clone()}>
                                                <div class="card-header">
                                                    <h3>{ &namespace.name }</h3>
                                                    { if namespace.deleted_at.is_some() {
                                                        html! { <span class="badge badge-danger">{ "Deleted" }</span> }
                                                    } else {
                                                        html! { <span class="badge badge-success">{ "Active" }</span> }
                                                    }}
                                                </div>
                                                <div class="card-body">
                                                    <div class="field">
                                                        <label>{ "Created:" }</label>
                                                        <span>{ &namespace.created_at }</span>
                                                    </div>
                                                    { if !namespace.labels.is_empty() {
                                                        html! {
                                                            <div class="field">
                                                                <label>{ "Labels:" }</label>
                                                                <div class="tags">
                                                                    { for namespace.labels.iter().map(|(k, v)| {
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
                                                <div class="card-footer">
                                                    <Link<Route> to={Route::NamespaceDetail { name: name.clone() }}>
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
