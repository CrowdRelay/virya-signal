const VIRYA_API_BASE: &str = "https://signal-api.virya.music/v1/";
const VIRYA_STAFF_GATE_URL: &str = "https://virya.music/api/staff/qr/login";
const VIRYA_STAFF_GATE_ORIGIN: &str = "https://virya.music";

const TENANT_API_BASE: &str = match option_env!("VIRYA_SIGNAL_API_BASE") {
    Some(value) if !value.is_empty() => value,
    _ => VIRYA_API_BASE,
};
const TENANT_STAFF_GATE_URL: &str = match option_env!("VIRYA_SIGNAL_STAFF_GATE_URL") {
    Some(value) if !value.is_empty() => value,
    _ => VIRYA_STAFF_GATE_URL,
};
const TENANT_STAFF_GATE_ORIGIN: &str = match option_env!("VIRYA_SIGNAL_STAFF_GATE_ORIGIN") {
    Some(value) if !value.is_empty() => value,
    _ => VIRYA_STAFF_GATE_ORIGIN,
};

#[cfg(debug_assertions)]
pub(crate) const API_BASE: &str = match option_env!("VIRYA_SIGNAL_E2E_API_BASE") {
    Some(value) if !value.is_empty() => value,
    _ => TENANT_API_BASE,
};
#[cfg(not(debug_assertions))]
pub(crate) const API_BASE: &str = TENANT_API_BASE;

pub(crate) const DEFAULT_COUNTRY_CODE: &str = match option_env!("VIRYA_SIGNAL_DEFAULT_COUNTRY_CODE")
{
    Some(value) if !value.is_empty() => value,
    _ => "PL",
};

#[cfg(debug_assertions)]
pub(crate) const STAFF_GATE_URL: &str = match option_env!("VIRYA_SIGNAL_E2E_STAFF_GATE_URL") {
    Some(value) if !value.is_empty() => value,
    _ => TENANT_STAFF_GATE_URL,
};
#[cfg(not(debug_assertions))]
pub(crate) const STAFF_GATE_URL: &str = TENANT_STAFF_GATE_URL;

#[cfg(debug_assertions)]
pub(crate) const STAFF_GATE_ORIGIN: &str = match option_env!("VIRYA_SIGNAL_E2E_STAFF_GATE_ORIGIN") {
    Some(value) if !value.is_empty() => value,
    _ => TENANT_STAFF_GATE_ORIGIN,
};
#[cfg(not(debug_assertions))]
pub(crate) const STAFF_GATE_ORIGIN: &str = TENANT_STAFF_GATE_ORIGIN;
