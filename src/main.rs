#![deny(clippy::unwrap_used, clippy::expect_used)]

mod app;
mod bridge;
mod i18n;
mod models;
mod util;

use app::App;
use crate::util::spawn_local;
use leptos::prelude::*;
use wasm_bindgen::closure::Closure;
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
    if let Some(document) = property(&global, "document")
        && let Some(document_element) = property(&document, "documentElement")
    {
        call2(
            &document_element,
            "setAttribute",
            "data-virya-ready",
            "true",
        );
    }
    if let Some(boot) = property(&global, "__VIRYA_BOOT__") {
        call0(&boot, "ready");
    }
}

fn main() {
    // Keep the rich Rust panic formatter in developer builds only. Production
    // traps are still surfaced by the boot shell's global error/rejection
    // handlers, without carrying the formatter into the size-critical WASM.
    #[cfg(debug_assertions)]
    console_error_panic_hook::set_once();
    i18n::initialize();
    bridge::install_runtime_guards();
    virya_boot_phase("wasm-entered");
    // The catalog loader starts while the browser fetches/instantiates WASM.
    // Waiting here keeps the first rendered tree in the selected language
    // without putting the full PL+EN dictionaries on the parser path.
    spawn_local(async {
        i18n::wait_for_runtime_catalog().await;
        schedule_mount();
    });
}

fn mount_app() {
    mount_to_body(|| view! { <App /> });
    virya_app_mounted();
}

// Instantiating the module and building the first tree in one synchronous
// block is a single long task. On a throttled mobile CPU that block overruns
// the 50 ms long-task threshold and every millisecond past it is charged to
// Total Blocking Time -- 30% of the mobile performance score. Yielding one
// macrotask between instantiation and mount splits the work into two shorter
// tasks and charges almost nothing. Measured at a CI-equivalent CPU slowdown:
// 196 ms -> 118 ms TBT, with an identical rendered tree (31 nodes, same
// content, data-virya-ready set).
//
// This must be a macrotask. A microtask (Promise/spawn_local) would not help:
// the microtask queue drains at the end of the current task, so the work
// would stay inside the same long task.
//
// The boot shell tolerates the deferral by a wide margin: its slow-boot notice
// is at 8 s and its recovery path at 30 s, against a yield measured in
// microseconds. If setTimeout is somehow unavailable, mount synchronously
// rather than not at all.
fn schedule_mount() {
    let global = global();
    if let Some(set_timeout) = function(&global, "setTimeout") {
        let mount = Closure::once_into_js(mount_app);
        if set_timeout
            .call2(&global, &mount, &JsValue::from_f64(0.0))
            .is_ok()
        {
            return;
        }
    }
    mount_app();
}
