mod app;
mod env;

use cfg_if::cfg_if;
use leptos::*;
use wasm_bindgen::prelude::*;
use web_sys::HtmlElement;

pub const MOUNT: &str = "ya-ya-exetension-mount";

#[wasm_bindgen]
pub fn main() {
    _ = console_log::init_with_level({
        cfg_if! {if #[cfg(debug_assertions)] {
            log::Level::Debug
        }else {
            log::Level::Info
        }}
    });
    console_error_panic_hook::set_once();
    log::info!("init log content");
    mount_app().expect("mount app")
}

fn mount_app() -> Result<(), JsValue> {
    let doc = web_sys::window()
        .and_then(|w| w.document())
        .ok_or_else(|| JsValue::from_str("document or winodw"))?;

    let app_el = doc.create_element("div")?;
    app_el.set_id(MOUNT);

    let bod = doc.body().expect("body");
    bod.append_with_node_1(&app_el.clone().into())?;

    let ht_el = app_el
        .dyn_ref::<HtmlElement>()
        .cloned()
        .ok_or_else(|| JsValue::from_str("app element ref"))?;

    mount_to(ht_el, app::App);

    Ok(())
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = "sendMessage")]
    pub async fn send_message() -> JsValue;
}
