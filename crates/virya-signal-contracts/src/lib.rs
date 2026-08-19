//! Stable shared contracts used by both the WebAssembly UI and the Tauri shell.
//!
//! Keep provider/API compatibility normalization in the native model layer. This
//! crate is intentionally limited to already-normalized IPC/control-plane DTOs so
//! the two sides cannot silently drift while external wire quirks remain isolated.

pub mod autopilot;
pub mod fan;
pub mod ops;
pub mod push;
