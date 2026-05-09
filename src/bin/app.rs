use leptos::prelude::*;
use qa::app::App;
use wasm_bindgen::prelude::wasm_bindgen;

#[wasm_bindgen(main)]
async fn main() {
    console_error_panic_hook::set_once();
    console_log::init_with_level(log::Level::Debug).unwrap();
    mount_to_body(App);
}
