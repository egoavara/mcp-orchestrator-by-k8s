use yew::prelude::*;
use web_sys::window;

#[derive(Properties, PartialEq)]
pub struct CopyConfigDialogProps {
    pub show: bool,
    pub config_json: String,
    pub on_close: Callback<()>,
}

#[function_component(CopyConfigDialog)]
pub fn copy_config_dialog(props: &CopyConfigDialogProps) -> Html {
    let copy_status = use_state(|| String::new());

    let on_copy = {
        let config_json = props.config_json.clone();
        let copy_status = copy_status.clone();
        Callback::from(move |_| {
            if let Some(window) = window() {
                let navigator = window.navigator();
                let clipboard = navigator.clipboard();
                let config = config_json.clone();
                let status = copy_status.clone();
                wasm_bindgen_futures::spawn_local(async move {
                    match wasm_bindgen_futures::JsFuture::from(clipboard.write_text(&config))
                        .await
                    {
                        Ok(_) => status.set("Copied to clipboard!".to_string()),
                        Err(_) => status.set("Failed to copy".to_string()),
                    }
                });
            }
        })
    };

    let on_backdrop_click = {
        let on_close = props.on_close.clone();
        Callback::from(move |_| {
            on_close.emit(());
        })
    };

    let on_close_click = {
        let on_close = props.on_close.clone();
        Callback::from(move |_| {
            on_close.emit(());
        })
    };

    if !props.show {
        return html! {};
    }

    html! {
        <div class="modal-overlay" onclick={on_backdrop_click}>
            <div class="modal-content" style="max-width: 600px;" onclick={|e: MouseEvent| e.stop_propagation()}>
                <div class="modal-header">
                    <h2>{ "MCP Server Configuration" }</h2>
                    <button class="close-button" onclick={on_close_click.clone()}>{ "×" }</button>
                </div>
                <div class="modal-body">
                    <p style="margin-bottom: 1rem;">{ "Copy this configuration to your MCP client settings:" }</p>
                    <pre style="background: #1e1e1e; color: #d4d4d4; padding: 1rem; border-radius: 4px; overflow-x: auto; font-size: 0.9rem;">
                        <code>{ &props.config_json }</code>
                    </pre>
                    { if !copy_status.is_empty() {
                        html! {
                            <div style="margin-top: 0.5rem; color: green; font-weight: bold;">
                                { &*copy_status }
                            </div>
                        }
                    } else {
                        html! {}
                    }}
                </div>
                <div class="modal-footer">
                    <button class="btn-primary" onclick={on_copy}>
                        { "📋 Copy to Clipboard" }
                    </button>
                    <button class="btn-secondary" onclick={on_close_click}>
                        { "Close" }
                    </button>
                </div>
            </div>
        </div>
    }
}
