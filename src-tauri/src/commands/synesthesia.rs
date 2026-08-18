//! Native Synesthesia -> Signal completion handoff.
//!
//! This path is intentionally independent from every QR/scanner parser. Android
//! delivers a verified virya.music App Link, Rust re-validates it, and only the
//! short-lived handoff capability is held in native memory until a fan session
//! is unlocked.

use tauri::{AppHandle, State};
#[cfg(any(target_os = "android", test))]
use url::Url;
#[cfg(any(target_os = "android", test))]
use zeroize::Zeroizing;

use crate::{AppError, AppState, session::fan_profile};

#[cfg(any(target_os = "android", test))]
fn handoff_from_app_link(value: &str) -> Result<Zeroizing<String>, AppError> {
    let value = value.trim();
    if value.len() > 1024 || !value.is_ascii() {
        return Err(AppError::InvalidInput(
            "invalid_synesthesia_app_link".to_owned(),
        ));
    }
    let url = Url::parse(value)?;
    if url.scheme() != "https"
        || !matches!(
            url.host_str(),
            Some("virya.music") | Some("www.virya.music")
        )
        || url.port().is_some()
        || url.username() != ""
        || url.password().is_some()
        || !matches!(
            url.path().trim_end_matches('/'),
            "/my-signal" | "/pl/my-signal"
        )
    {
        return Err(AppError::InvalidInput(
            "invalid_synesthesia_app_link".to_owned(),
        ));
    }

    let query = url.query_pairs().collect::<Vec<_>>();
    if query.len() != 1 || query[0].0 != "source" || query[0].1 != "synesthesia" {
        return Err(AppError::InvalidInput(
            "invalid_synesthesia_app_link".to_owned(),
        ));
    }

    let fragment = url
        .fragment()
        .ok_or_else(|| AppError::InvalidInput("missing_synesthesia_handoff".to_owned()))?;
    let handoff = fragment
        .strip_prefix("handoff=")
        .ok_or_else(|| AppError::InvalidInput("invalid_synesthesia_handoff".to_owned()))?;
    if handoff.len() != 64 || !handoff.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(AppError::InvalidInput(
            "invalid_synesthesia_handoff".to_owned(),
        ));
    }
    Ok(Zeroizing::new(handoff.to_ascii_lowercase()))
}

#[tauri::command]
pub(crate) async fn fan_take_synesthesia_app_link(
    state: State<'_, AppState>,
    _app: AppHandle,
) -> Result<bool, AppError> {
    #[cfg(target_os = "android")]
    if let Some(link) =
        crate::push_plugin::take_synesthesia_app_link(&_app).map_err(AppError::InvalidInput)?
    {
        let handoff = handoff_from_app_link(&link)?;
        *state.pending_synesthesia_handoff.lock().await = Some(handoff);
    }
    Ok(state.pending_synesthesia_handoff.lock().await.is_some())
}

async fn clear_if_same(state: &AppState, expected: &str) {
    let mut pending = state.pending_synesthesia_handoff.lock().await;
    if pending
        .as_ref()
        .is_some_and(|value| value.as_str() == expected)
    {
        *pending = None;
    }
}

/// Returns one of: "none", "linked", "expired".
/// Retryable transport/session failures remain errors and keep the pending
/// capability. Terminal invalid/expired/conflicting capabilities are retired.
#[tauri::command]
pub(crate) async fn fan_link_pending_synesthesia(
    state: State<'_, AppState>,
) -> Result<String, AppError> {
    let handoff = {
        let pending = state.pending_synesthesia_handoff.lock().await;
        pending.as_ref().cloned()
    };
    let Some(handoff) = handoff else {
        return Ok("none".to_owned());
    };

    let profile = fan_profile(&state).await?;
    match state
        .api
        .fan_link_synesthesia_handoff(profile.as_ref(), handoff.as_str())
        .await
    {
        Ok(true) => {
            clear_if_same(&state, handoff.as_str()).await;
            Ok("linked".to_owned())
        }
        Ok(false) => {
            clear_if_same(&state, handoff.as_str()).await;
            Ok("expired".to_owned())
        }
        Err(AppError::Conflict(_)) | Err(AppError::InvalidInput(_)) | Err(AppError::NotFound) => {
            clear_if_same(&state, handoff.as_str()).await;
            Ok("expired".to_owned())
        }
        Err(error) => Err(error),
    }
}

#[cfg(test)]
mod tests {
    use super::handoff_from_app_link;

    const CODE: &str = "0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

    #[test]
    fn accepts_only_the_synesthesia_my_signal_fragment_contract() {
        let valid = format!("https://virya.music/pl/my-signal/?source=synesthesia#handoff={CODE}");
        let handoff = match handoff_from_app_link(&valid) {
            Ok(handoff) => handoff,
            Err(error) => panic!("valid Synesthesia app link was rejected: {error}"),
        };
        assert_eq!(handoff.as_str(), CODE);

        for invalid in [
            format!("https://evil.example/pl/my-signal/?source=synesthesia#handoff={CODE}"),
            format!("https://virya.music/pl/my-signal/?source=qr#handoff={CODE}"),
            format!("https://virya.music/pl/my-signal/?source=synesthesia&x=1#handoff={CODE}"),
            format!("https://virya.music/pl/my-signal/?source=synesthesia#token={CODE}"),
            "https://virya.music/pl/my-signal/?source=synesthesia#handoff=short".to_owned(),
        ] {
            assert!(handoff_from_app_link(&invalid).is_err(), "{invalid}");
        }
    }
}
