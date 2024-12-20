mod app;

use leptos::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    log::info!("init log popup");
    mount_app()
}

fn mount_app() {
    mount_to_body(app::App);
}

#[wasm_bindgen(module = "/src/lib.js")]
extern "C" {
    #[wasm_bindgen(js_name = "sendMessage")]
    pub async fn send_message(msg: JsValue) -> JsValue;
}
