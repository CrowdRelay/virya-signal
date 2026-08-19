const VIRYA_API_BASE: &str = "https://signal-api.virya.music/v1/";

const TENANT_API_BASE: &str = match option_env!("VIRYA_SIGNAL_API_BASE") {
    Some(value) if !value.is_empty() => value,
    _ => VIRYA_API_BASE,
};

// Emulator E2E remains a debug-only highest-priority override. Ordinary debug
// and release builds share the same tenant default so a branded build cannot
// accidentally talk to VIRYA from one UI path and its own backend from another.
#[cfg(debug_assertions)]
pub(crate) const API_BASE: &str = match option_env!("VIRYA_SIGNAL_E2E_API_BASE") {
    Some(value) if !value.is_empty() => value,
    _ => TENANT_API_BASE,
};
#[cfg(not(debug_assertions))]
pub(crate) const API_BASE: &str = TENANT_API_BASE;
