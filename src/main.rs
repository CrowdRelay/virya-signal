mod app;
mod bridge;
mod models;

use app::App;
use leptos::prelude::*;
use wasm_bindgen::{JsCast, JsValue};

fn property(target: &JsValue, name: &str) -> Option<JsValue> {
    js_sys::Reflect::get(target, &JsValue::from_str(name)).ok()
}

fn function(target: &JsValue, name: &str) -> Option<js_sys::Function> {
    property(target, name)?.dyn_into::<js_sys::Function>().ok()
}

fn call0(target: &JsValue, name: &str) {
    if let Some(function) = function(target, name) {
        let _ = function.call0(target);
    }
}

fn call1(target: &JsValue, name: &str, value: &str) {
    if let Some(function) = function(target, name) {
        let _ = function.call1(target, &JsValue::from_str(value));
    }
}

fn call2(target: &JsValue, name: &str, first: &str, second: &str) {
    if let Some(function) = function(target, name) {
        let _ = function.call2(
            target,
            &JsValue::from_str(first),
            &JsValue::from_str(second),
        );
    }
}

fn global() -> JsValue {
    js_sys::global().into()
}

fn virya_boot_phase(phase: &str) {
    let global = global();
    if let Some(boot) = property(&global, "__VIRYA_BOOT__") {
        call1(&boot, "phase", phase);
    }
}

fn virya_app_mounted() {
    let global = global();
    if let Some(document) = property(&global, "document") {
        if let Some(document_element) = property(&document, "documentElement") {
            call2(
                &document_element,
                "setAttribute",
                "data-virya-ready",
                "true",
            );
        }
    }
    if let Some(boot) = property(&global, "__VIRYA_BOOT__") {
        call0(&boot, "ready");
    }
}

fn main() {
    console_error_panic_hook::set_once();
    virya_boot_phase("wasm-entered");
    mount_to_body(|| view! { <App /> });
    virya_app_mounted();
}
