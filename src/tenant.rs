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

pub(crate) const DEFAULT_COUNTRY_CODE: &str =
    match option_env!("VIRYA_SIGNAL_DEFAULT_COUNTRY_CODE") {
        Some(value) if !value.is_empty() => value,
        _ => "PL",
    };

pub(crate) const POLICY_VERSION: &str = match option_env!("VIRYA_SIGNAL_POLICY_VERSION") {
    Some(value) if !value.is_empty() => value,
    _ => "2026-07",
};

// The shipped catalog currently supports PL and EN. International branded
// builds can therefore default to English without forking the application;
// adding DE/CS catalogs later does not change the tenant-config seam.
pub(crate) const DEFAULT_LANGUAGE: &str = match option_env!("VIRYA_SIGNAL_DEFAULT_LANGUAGE") {
    Some(value) if !value.is_empty() => value,
    _ => "pl",
};
