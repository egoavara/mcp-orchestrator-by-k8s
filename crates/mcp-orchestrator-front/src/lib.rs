mod app;
mod routes;
mod api;
mod components;
mod pages;
mod models;
mod utils;

use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn start() -> Result<(), JsValue> {
    wasm_logger::init(wasm_logger::Config::default());
    log::info!("MCP Orchestrator UI starting...");
    
    yew::Renderer::<app::App>::new().render();
    
    Ok(())
}
