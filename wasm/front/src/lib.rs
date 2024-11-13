mod app;

use leptos::*;
use wasm_bindgen::prelude::*;
use web_sys::HtmlElement;

const MOUNT: &str = "ya-ya-exetension-mount";

#[wasm_bindgen(start)]
pub fn main() {
    _ = console_log::init_with_level(log::Level::Debug);
    console_error_panic_hook::set_once();
    log::info!("init log content");
    mount_app()
}

fn mount_app() {
    let doc = web_sys::window()
        .and_then(|w| w.document())
        .expect("document");
    let el = doc.create_element("div").expect("create app mount");
    el.set_id(MOUNT);
    let bod = doc.body().expect("body");
    bod.append_with_node_1(&el.clone().into())
        .expect("append app mount");

    let ht_el = el.dyn_ref::<HtmlElement>().cloned().unwrap();

    mount_to(ht_el, app::App);
}

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(js_name = "sendMessage")]
    pub async fn send_message() -> JsValue;
}
