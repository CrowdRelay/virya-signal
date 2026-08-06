//! Staff pairing QR/deep-link payload parsing and the command that turns a
//! scanned pairing code into a configured operator session.

use std::time::{SystemTime, UNIX_EPOCH};

use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use tauri::State;

use crate::{
    AppError, AppState,
    commands::operator::configure,
    models::{OperatorProfile, SessionStatus, StaffPairingPayload},
};

#[tauri::command]
pub(crate) async fn configure_from_pairing(
    state: State<'_, AppState>,
    pin: String,
    payload: String,
) -> Result<SessionStatus, AppError> {
    let pairing = parse_pairing_payload(&payload)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |value| value.as_secs());
    if pairing.version != 1 || pairing.expires_at < now || pairing.expires_at > now + 1800 {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_pairing_code_expired").into(),
        ));
    }
    configure(
        state,
        pin,
        OperatorProfile {
            display_name: pairing.display_name,
            api_base_url: pairing.api_base_url,
            role: pairing.role,
            bearer_token: pairing.bearer_token,
        },
    )
    .await
}

fn parse_pairing_payload(raw: &str) -> Result<StaffPairingPayload, AppError> {
    let raw = raw.trim();
    if raw.is_empty() || raw.len() > 8192 {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_pairing_code_invalid").into(),
        ));
    }
    if raw.starts_with('{') {
        return serde_json::from_str(raw).map_err(AppError::from);
    }
    let url = url::Url::parse(raw)?;
    if url.scheme() != "virya-signal" || url.host_str() != Some("pair") {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_pairing_code_invalid").into(),
        ));
    }
    let encoded = url
        .query_pairs()
        .find_map(|(key, value)| (key == "payload").then_some(value.into_owned()))
        .ok_or_else(|| {
            AppError::InvalidInput(crate::i18n::tr("native_pairing_code_empty").into())
        })?;
    let mut padded = encoded;
    while padded.len() % 4 != 0 {
        padded.push('=');
    }
    let bytes = URL_SAFE.decode(padded).map_err(|_| {
        AppError::InvalidInput(crate::i18n::tr("native_pairing_code_invalid").into())
    })?;
    serde_json::from_slice(&bytes).map_err(AppError::from)
}
