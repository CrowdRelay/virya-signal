//! Staff pairing QR/deep-link parsing and one-time broker exchange.
//! The QR never contains the durable operator bearer: CrowdRelay mints a
//! revocable per-device session only after the short-lived code is exchanged.

use std::time::{SystemTime, UNIX_EPOCH};

use base64::{Engine as _, engine::general_purpose::URL_SAFE};
use tauri::State;

use crate::{
    AppError, AppState,
    commands::operator::configure,
    models::{OperatorProfile, OperatorRole, SessionStatus, StaffPairingPayload},
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
    if pairing.version != 2
        || pairing.role != OperatorRole::Staff
        || pairing.expires_at < now
        || pairing.expires_at > now + 1800
    {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_pairing_code_expired").into(),
        ));
    }

    let exchange = state
        .api
        .exchange_staff_pairing(&pairing.api_base_url, &pairing.pairing_code)
        .await?;
    if exchange.version != 2
        || exchange.role != OperatorRole::Staff
        || exchange.display_name.trim() != pairing.display_name.trim()
        || exchange.expires_at <= now
        || exchange.session_id.trim().is_empty()
    {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_pairing_code_invalid").into(),
        ));
    }

    configure(
        state,
        pin,
        OperatorProfile {
            display_name: exchange.display_name,
            api_base_url: pairing.api_base_url,
            role: exchange.role,
            bearer_token: exchange.bearer_token,
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
