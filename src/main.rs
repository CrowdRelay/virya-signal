mod app;
mod bridge;
mod models;

use app::App;
use leptos::prelude::*;
use wasm_bindgen::prelude::*;

#[wasm_bindgen(inline_js = r#"
export function viryaAppMounted() {
  document.documentElement?.setAttribute('data-virya-ready', 'true');
  window.__VIRYA_BOOT__?.ready?.();
  window.dispatchEvent(new Event('virya:ready'));
}
"#)]
extern "C" {
    #[wasm_bindgen(js_name = viryaAppMounted)]
    fn virya_app_mounted();
}

fn main() {
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App /> });
    virya_app_mounted();
}
