use crate::api::namespaces::get_namespace;
use crate::components::{ErrorMessage, Loading};
use crate::models::namespace::Namespace;
use crate::routes::Route;
use yew::prelude::*;
use yew_router::prelude::*;

#[derive(Properties, PartialEq)]
pub struct Props {
    pub name: String,
}

#[derive(Debug)]
enum LoadState {
    Loading,
    Loaded(Namespace),
    Error(String),
}

#[function_component(NamespaceDetail)]
pub fn namespace_detail(props: &Props) -> Html {
    let load_state = use_state(|| LoadState::Loading);
    let name = props.name.clone();

    {
        let load_state = load_state.clone();
        let name = name.clone();
        use_effect_with(name.clone(), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match get_namespace(&name).await {
                    Ok(namespace) => load_state.set(LoadState::Loaded(namespace)),
                    Err(e) => load_state.set(LoadState::Error(e)),
                }
            });
            || ()
        });
    }

    html! {
        <div class="container">
            <div class="header">
                <h1>{ "Namespace Details" }</h1>
                <Link<Route> to={Route::NamespaceList}>
                    <button class="btn-secondary">{ "Back to List" }</button>
                </Link<Route>>
            </div>

            { match &*load_state {
                LoadState::Loading => html! { <Loading /> },
                LoadState::Error(e) => html! { <ErrorMessage message={e.clone()} /> },
                LoadState::Loaded(namespace) => html! {
                    <div class="detail-card">
                        <div class="detail-section">
                            <h2>{ "Basic Information" }</h2>
                            <div class="detail-grid">
                                <div class="detail-field">
                                    <label>{ "Name:" }</label>
                                    <span>{ &namespace.name }</span>
                                </div>
                                <div class="detail-field">
                                    <label>{ "Status:" }</label>
                                    { if namespace.deleted_at.is_some() {
                                        html! { <span class="badge badge-danger">{ "Deleted" }</span> }
                                    } else {
                                        html! { <span class="badge badge-success">{ "Active" }</span> }
                                    }}
                                </div>
                                <div class="detail-field">
                                    <label>{ "Created At:" }</label>
                                    <span>{ &namespace.created_at }</span>
                                </div>
                                { if let Some(deleted_at) = &namespace.deleted_at {
                                    html! {
                                        <div class="detail-field">
                                            <label>{ "Deleted At:" }</label>
                                            <span>{ deleted_at }</span>
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }}
                            </div>
                        </div>

                        { if !namespace.labels.is_empty() {
                            html! {
                                <div class="detail-section">
                                    <h2>{ "Labels" }</h2>
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
                }
            }}
        </div>
    }
}
