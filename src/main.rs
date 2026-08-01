mod app;
mod bridge;
mod models;

use app::App;
use leptos::prelude::*;

fn main() {
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    mount_to_body(|| view! { <App /> });

    if let Some(window) = web_sys::window() {
        if let Ok(event) = web_sys::Event::new("virya:ready") {
            let _ = window.dispatch_event(&event);
        }
    }
}
