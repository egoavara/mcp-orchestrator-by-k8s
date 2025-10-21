#![cfg(target_arch = "wasm32")]

#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() -> Result<(), wasm_bindgen::JsValue> {
    Ok(())
}
