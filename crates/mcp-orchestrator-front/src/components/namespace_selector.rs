use crate::api::namespaces::list_namespaces;
use crate::models::{Namespace, SessionState};
use yew::prelude::*;
use yewdux::prelude::*;

#[function_component(NamespaceSelector)]
pub fn namespace_selector() -> Html {
    let (state, dispatch) = use_store::<SessionState>();
    let namespaces = use_state(Vec::<Namespace>::new);
    let is_loading = use_state(|| true);

    {
        let namespaces = namespaces.clone();
        let is_loading = is_loading.clone();
        let dispatch = dispatch.clone();
        let has_selected = state.selected_namespace.is_some();

        use_effect_with((), move |_| {
            wasm_bindgen_futures::spawn_local(async move {
                match list_namespaces().await {
                    Ok(ns_list) => {
                        if !ns_list.is_empty() && !has_selected {
                            let first_ns = ns_list[0].name.clone();
                            dispatch.reduce_mut(|state| {
                                state.selected_namespace = Some(first_ns);
                            });
                        }
                        namespaces.set(ns_list);
                    }
                    Err(e) => {
                        web_sys::console::error_1(
                            &format!("Failed to load namespaces: {}", e).into(),
                        );
                        namespaces.set(vec![]);
                    }
                }
                is_loading.set(false);
            });
            || ()
        });
    }

    let on_namespace_change = {
        let dispatch = dispatch.clone();
        Callback::from(move |e: Event| {
            let select: web_sys::HtmlSelectElement = e.target_unchecked_into();
            let value = select.value();
            dispatch.reduce_mut(|state| {
                state.selected_namespace = Some(value);
            });
        })
    };

    let current_namespace = state
        .selected_namespace
        .clone()
        .or_else(|| namespaces.first().map(|ns| ns.name.clone()))
        .unwrap_or_else(|| "default".to_string());

    html! {
        <div class="namespace-selector-bar">
            <label>{"Namespace:"}</label>
            if *is_loading {
                <select disabled={true}>
                    <option>{"Loading..."}</option>
                </select>
            } else if namespaces.is_empty() {
                <select disabled={true}>
                    <option>{"No namespaces"}</option>
                </select>
            } else {
                <select onchange={on_namespace_change} value={current_namespace}>
                    { for namespaces.iter().map(|ns| {
                        html! {
                            <option key={ns.name.clone()} value={ns.name.clone()}>
                                {&ns.name}
                            </option>
                        }
                    })}
                </select>
            }
        </div>
    }
}
