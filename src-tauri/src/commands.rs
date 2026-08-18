//! Every `#[tauri::command]` handler, grouped by the domain it belongs to.
//! `lib.rs` only wires these into `tauri::generate_handler!` and owns
//! `AppState`; the actual request handling lives here.

pub(crate) mod beacon;
pub(crate) mod fan;
pub(crate) mod misc;
pub(crate) mod operator;
pub(crate) mod pairing;
pub(crate) mod show_mode;
pub(crate) mod synesthesia;
