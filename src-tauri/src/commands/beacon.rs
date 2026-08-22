//! Native Latarnik (Beacon) session and member surface.
//!
//! The Beacon bearer is Stronghold-only. The WASM UI receives bounded member
//! DTOs and session summaries, never the bearer or raw persisted capability.

use std::sync::Arc;

use tauri::{AppHandle, Manager, State};
use url::Url;
use zeroize::Zeroizing;

use crate::{
    AppError, AppState, PendingBeaconConfirmation,
    api::BeaconPreferencesInput,
    commands::fan::{
        ensure_native_push_installation_id, native_push_permission, native_push_token,
        open_native_push_settings, read_native_push_installation_id,
        request_native_push_permission,
    },
    models::{
        BeaconEngagementResult, BeaconHomeData, BeaconMutationResult, BeaconPressRequestsData,
        BeaconPressRoomData, BeaconProfile, BeaconReleasesData, BeaconSessionStatus, FanPushStatus,
        SignalNewsFeed,
    },
    session::{beacon_profile, persist_beacon, run_blocking},
    validation::{validate_api_base, validate_pin},
    vault,
};

const MAX_INVITE_TOKEN_BYTES: usize = 128;
const MAX_DETAILS_BYTES: usize = 2_000;
const MAX_DELIVERY_FIELD_BYTES: usize = 200;
const BEACON_TOPICS: &[&str] = &[
    "shows",
    "press_materials",
    "releases",
    "interviews",
    "accreditation",
];

fn normalize_locale(value: &str) -> Result<String, AppError> {
    let value = value.trim();
    let valid = matches!(value.len(), 2 | 5)
        && value.as_bytes().first().is_some_and(u8::is_ascii_lowercase)
        && value.as_bytes().get(1).is_some_and(u8::is_ascii_lowercase)
        && (value.len() == 2
            || (value.as_bytes().get(2) == Some(&b'-')
                && value.as_bytes().get(3).is_some_and(u8::is_ascii_uppercase)
                && value.as_bytes().get(4).is_some_and(u8::is_ascii_uppercase)));
    if valid {
        Ok(value.to_owned())
    } else {
        Err(AppError::InvalidInput("invalid_beacon_locale".to_owned()))
    }
}

fn normalize_topics(values: Vec<String>) -> Result<Vec<String>, AppError> {
    let mut normalized = values
        .into_iter()
        .map(|value| value.trim().to_owned())
        .collect::<Vec<_>>();
    if normalized.is_empty()
        || normalized
            .iter()
            .any(|value| !BEACON_TOPICS.contains(&value.as_str()))
    {
        return Err(AppError::InvalidInput("invalid_beacon_topics".to_owned()));
    }
    normalized.sort();
    normalized.dedup();
    Ok(normalized)
}

fn validate_radius(radius_km: i32) -> Result<i32, AppError> {
    if (10..=500).contains(&radius_km) {
        Ok(radius_km)
    } else {
        Err(AppError::InvalidInput("invalid_beacon_radius".to_owned()))
    }
}

fn valid_invite_token(value: &str) -> bool {
    (24..=MAX_INVITE_TOKEN_BYTES).contains(&value.len())
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn normalize_invite(value: &str) -> Result<Zeroizing<String>, AppError> {
    let value = value.trim();
    if let Ok(url) = Url::parse(value) {
        if url.scheme() != "https"
            || !matches!(
                url.host_str(),
                Some("virya.music") | Some("www.virya.music")
            )
            || url.username() != ""
            || url.password().is_some()
            || url.fragment().is_some()
            || !matches!(
                url.path().trim_end_matches('/'),
                "/latarnik" | "/pl/latarnik"
            )
        {
            return Err(AppError::InvalidInput(
                "invalid_latarnik_invite_url".to_owned(),
            ));
        }
        let mut invite = None;
        for (key, candidate) in url.query_pairs() {
            if key == "invite" {
                if invite.is_some() {
                    return Err(AppError::InvalidInput(
                        "duplicate_latarnik_invite".to_owned(),
                    ));
                }
                invite = Some(candidate.into_owned());
            } else {
                return Err(AppError::InvalidInput(
                    "invalid_latarnik_invite_url".to_owned(),
                ));
            }
        }
        let invite =
            invite.ok_or_else(|| AppError::InvalidInput("missing_latarnik_invite".to_owned()))?;
        if valid_invite_token(&invite) {
            return Ok(Zeroizing::new(invite));
        }
        return Err(AppError::InvalidInput("invalid_latarnik_invite".to_owned()));
    }
    if valid_invite_token(value) {
        Ok(Zeroizing::new(value.to_owned()))
    } else {
        Err(AppError::InvalidInput("invalid_latarnik_invite".to_owned()))
    }
}

fn validate_event_id(value: &str) -> Result<String, AppError> {
    uuid::Uuid::parse_str(value.trim())
        .map(|value| value.to_string())
        .map_err(|_| AppError::InvalidInput("invalid_latarnik_event".to_owned()))
}

fn validate_campaign_id(value: &str) -> Result<String, AppError> {
    uuid::Uuid::parse_str(value.trim())
        .map(|value| value.to_string())
        .map_err(|_| AppError::InvalidInput("invalid_latarnik_campaign".to_owned()))
}

fn bounded_optional(value: Option<String>, max: usize) -> Result<Option<String>, AppError> {
    let value = value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty());
    if value.as_ref().is_some_and(|value| value.len() > max) {
        return Err(AppError::InvalidInput("latarnik_text_too_long".to_owned()));
    }
    Ok(value)
}

fn https_url(value: &str) -> Result<String, AppError> {
    let value = value.trim();
    let url = Url::parse(value)?;
    if value.len() > 2048
        || url.scheme() != "https"
        || url.host_str().is_none()
        || url.username() != ""
        || url.password().is_some()
    {
        return Err(AppError::InvalidInput("invalid_coverage_url".to_owned()));
    }
    Ok(value.to_owned())
}

async fn status_from_state(state: &State<'_, AppState>) -> BeaconSessionStatus {
    let session = state.beacon_session.read().await;
    BeaconSessionStatus {
        configured: vault::beacon_exists(&state.app_data_dir),
        unlocked: session.is_some(),
        session: session.as_ref().map(|profile| profile.as_ref().into()),
    }
}

#[tauri::command]
pub(crate) async fn beacon_status(
    state: State<'_, AppState>,
) -> Result<BeaconSessionStatus, AppError> {
    Ok(status_from_state(&state).await)
}

async fn persist_exchanged_beacon(
    state: &AppState,
    api_base_url: String,
    invite_token: &str,
    pin: Zeroizing<String>,
    radius_km: i32,
    locale: String,
    topics: Vec<String>,
) -> Result<(), AppError> {
    let exchange = state
        .api
        .beacon_exchange(&api_base_url, invite_token, radius_km, &locale, &topics)
        .await?;
    let profile = BeaconProfile {
        api_base_url,
        beacon_id: exchange.beacon_id,
        display_name: exchange.display_name,
        beacon_kind: exchange.beacon_kind,
        bearer_token: exchange.bearer_token,
        session_id: exchange.session_id,
        client_kind: exchange.client_kind,
        expires_at: exchange.expires_at,
        // A just-minted session has its own token hash, so no Beacon push
        // endpoint can exist for it yet. Recording that as already reconciled
        // keeps the first unlock from firing a pointless remote disable and a
        // Stronghold rewrite before the member has touched notifications.
        push_enabled: false,
        push_last_sync_ok: true,
    };
    let app_data_dir = state.app_data_dir.clone();
    let stored = profile.clone();
    let vault_pin = pin.clone();
    let vault_password =
        run_blocking(move || vault::replace_beacon(&app_data_dir, vault_pin.as_str(), &stored))
            .await?;
    // Native storage is authoritative. UI state is unlocked only after the
    // Stronghold write has succeeded and can be reopened with this PIN.
    *state.beacon_session.write().await = Some(Arc::new(profile));
    *state.beacon_pin.write().await = Some(pin);
    *state.beacon_vault_password.write().await = Some(vault_password);
    Ok(())
}

#[tauri::command]
pub(crate) async fn beacon_exchange_invite(
    state: State<'_, AppState>,
    mut api_base_url: String,
    invite: String,
    pin: String,
    radius_km: i32,
    locale: String,
    topics: Vec<String>,
) -> Result<BeaconSessionStatus, AppError> {
    let _mutation = state.beacon_mutation.lock().await;
    api_base_url = api_base_url.trim().to_owned();
    validate_api_base(&api_base_url)?;
    validate_pin(&pin)?;
    let radius_km = validate_radius(radius_km)?;
    let locale = normalize_locale(&locale)?;
    let topics = normalize_topics(topics)?;
    let invite = normalize_invite(&invite)?;
    persist_exchanged_beacon(
        &state,
        api_base_url,
        invite.as_str(),
        Zeroizing::new(pin),
        radius_km,
        locale,
        topics,
    )
    .await?;
    *state.pending_beacon_confirmation.lock().await = None;
    *state.pending_beacon_link.lock().await = None;
    drop(_mutation);
    beacon_status(state).await
}

#[tauri::command]
pub(crate) async fn beacon_prepare_invite(
    state: State<'_, AppState>,
    mut api_base_url: String,
    pin: String,
    radius_km: i32,
    locale: String,
    topics: Vec<String>,
) -> Result<(), AppError> {
    let _mutation = state.beacon_mutation.lock().await;
    api_base_url = api_base_url.trim().to_owned();
    validate_api_base(&api_base_url)?;
    validate_pin(&pin)?;
    *state.pending_beacon_confirmation.lock().await = Some(PendingBeaconConfirmation {
        api_base_url,
        pin: Zeroizing::new(pin),
        radius_km: validate_radius(radius_km)?,
        locale: normalize_locale(&locale)?,
        topics: normalize_topics(topics)?,
    });
    Ok(())
}

/// Drops only the staged PIN/preferences of an abandoned scanner ceremony. A
/// queued App Link capability is a separate invitation and must survive a
/// cancelled or failed camera pass.
#[tauri::command]
pub(crate) async fn beacon_clear_pending_confirmation(
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let _mutation = state.beacon_mutation.lock().await;
    *state.pending_beacon_confirmation.lock().await = None;
    Ok(())
}

/// Discards the whole pending invitation, including a queued App Link.
#[tauri::command]
pub(crate) async fn beacon_clear_pending_invite(
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let _mutation = state.beacon_mutation.lock().await;
    *state.pending_beacon_confirmation.lock().await = None;
    *state.pending_beacon_link.lock().await = None;
    Ok(())
}

#[tauri::command]
pub(crate) async fn beacon_confirm_scanned(
    state: State<'_, AppState>,
    token: String,
) -> Result<BeaconSessionStatus, AppError> {
    let _mutation = state.beacon_mutation.lock().await;
    if state.beacon_session.read().await.is_some() {
        drop(_mutation);
        return beacon_status(state).await;
    }
    let pending = {
        let pending = state.pending_beacon_confirmation.lock().await;
        let pending = pending.as_ref().ok_or(AppError::InvalidPin)?;
        PendingBeaconConfirmation {
            api_base_url: pending.api_base_url.clone(),
            pin: pending.pin.clone(),
            radius_km: pending.radius_km,
            locale: pending.locale.clone(),
            topics: pending.topics.clone(),
        }
    };
    let invite = normalize_invite(&token)?;
    persist_exchanged_beacon(
        &state,
        pending.api_base_url,
        invite.as_str(),
        pending.pin,
        pending.radius_km,
        pending.locale,
        pending.topics,
    )
    .await?;
    *state.pending_beacon_confirmation.lock().await = None;
    *state.pending_beacon_link.lock().await = None;
    drop(_mutation);
    beacon_status(state).await
}

#[tauri::command]
pub(crate) async fn beacon_take_app_link(
    state: State<'_, AppState>,
    _app: AppHandle,
) -> Result<bool, AppError> {
    if !cfg!(target_os = "android") {
        return Ok(state.pending_beacon_link.lock().await.is_some());
    }
    #[cfg(target_os = "android")]
    if let Some(link) = crate::push_plugin::take_app_link(&_app).map_err(AppError::InvalidInput)? {
        let invite = normalize_invite(&link)?;
        *state.pending_beacon_link.lock().await = Some(invite);
    }
    Ok(state.pending_beacon_link.lock().await.is_some())
}

#[tauri::command]
pub(crate) async fn beacon_exchange_pending(
    state: State<'_, AppState>,
    mut api_base_url: String,
    pin: String,
    radius_km: i32,
    locale: String,
    topics: Vec<String>,
) -> Result<BeaconSessionStatus, AppError> {
    let _mutation = state.beacon_mutation.lock().await;
    api_base_url = api_base_url.trim().to_owned();
    validate_api_base(&api_base_url)?;
    validate_pin(&pin)?;
    let radius_km = validate_radius(radius_km)?;
    let locale = normalize_locale(&locale)?;
    let topics = normalize_topics(topics)?;
    let invite = state
        .pending_beacon_link
        .lock()
        .await
        .as_ref()
        .cloned()
        .ok_or_else(|| AppError::InvalidInput("missing_latarnik_invite".to_owned()))?;
    persist_exchanged_beacon(
        &state,
        api_base_url,
        invite.as_str(),
        Zeroizing::new(pin),
        radius_km,
        locale,
        topics,
    )
    .await?;
    *state.pending_beacon_link.lock().await = None;
    // A completed exchange also retires any PIN staged for an abandoned
    // scanner ceremony; leaving it resident serves nothing.
    *state.pending_beacon_confirmation.lock().await = None;
    drop(_mutation);
    beacon_status(state).await
}

#[tauri::command]
pub(crate) async fn beacon_unlock(
    state: State<'_, AppState>,
    app: AppHandle,
    pin: String,
) -> Result<BeaconSessionStatus, AppError> {
    let _mutation = state.beacon_mutation.lock().await;
    validate_pin(&pin)?;
    let pin = Zeroizing::new(pin);
    let app_data_dir = state.app_data_dir.clone();
    let vault_pin = pin.clone();
    let (profile, vault_password) = run_blocking(move || {
        let password = vault::beacon_password(&app_data_dir, vault_pin.as_str())?;
        let profile = vault::load_beacon_with_password(&app_data_dir, password.as_ref())?;
        Ok((profile, password))
    })
    .await?;
    *state.beacon_session.write().await = Some(Arc::new(profile));
    *state.beacon_pin.write().await = Some(pin);
    *state.beacon_vault_password.write().await = Some(vault_password);
    drop(_mutation);
    let status = beacon_status(state).await?;

    // Unlock is a local vault operation. Do not delay the first usable screen
    // on backend push configuration/token registration; reconcile it in the
    // background while keeping session mutations serialized.
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        let _mutation = state.beacon_mutation.lock().await;
        if let Err(error) = sync_native_push_if_desired(&state, &app).await {
            eprintln!("[virya:beacon-push-sync] unlock reconciliation degraded: {error}");
        }
    });
    Ok(status)
}

#[tauri::command]
pub(crate) async fn beacon_lock(
    state: State<'_, AppState>,
) -> Result<BeaconSessionStatus, AppError> {
    // Locking only drops in-memory session material, so it deliberately does not
    // queue behind `beacon_mutation`. The push reconciliation spawned by unlock
    // can hold that lock for the length of several network calls, and "log me
    // out" must not wait for it. An in-flight mutation already owns a cloned
    // profile; anything starting after this point sees a locked session.
    *state.beacon_session.write().await = None;
    *state.beacon_pin.write().await = None;
    *state.beacon_vault_password.write().await = None;
    *state.pending_beacon_confirmation.lock().await = None;
    beacon_status(state).await
}

#[tauri::command]
pub(crate) async fn beacon_home(state: State<'_, AppState>) -> Result<BeaconHomeData, AppError> {
    let profile = beacon_profile(&state).await?;
    state.api.beacon_me(&profile).await
}

#[tauri::command]
pub(crate) async fn beacon_news(state: State<'_, AppState>) -> Result<SignalNewsFeed, AppError> {
    let _ = beacon_profile(&state).await?;
    state.api.signal_news().await
}

#[tauri::command]
pub(crate) async fn beacon_preferences_update(
    state: State<'_, AppState>,
    radius_km: i32,
    locale: String,
    topics: Vec<String>,
    nearby_gigs_enabled: bool,
) -> Result<crate::models::BeaconPreferences, AppError> {
    let _mutation = state.beacon_mutation.lock().await;
    let profile = beacon_profile(&state).await?;
    let locale = normalize_locale(&locale)?;
    let topics = normalize_topics(topics)?;
    let input = BeaconPreferencesInput {
        radius_km: validate_radius(radius_km)?,
        locale: &locale,
        topics: &topics,
        nearby_gigs_enabled,
    };
    state.api.beacon_preferences(&profile, &input).await
}

#[tauri::command]
pub(crate) async fn beacon_press_room(
    state: State<'_, AppState>,
    event_id: Option<String>,
) -> Result<BeaconPressRoomData, AppError> {
    let profile = beacon_profile(&state).await?;
    let event_id = event_id.as_deref().map(validate_event_id).transpose()?;
    state
        .api
        .beacon_press_room(&profile, event_id.as_deref())
        .await
}

#[tauri::command]
pub(crate) async fn beacon_press_requests(
    state: State<'_, AppState>,
) -> Result<BeaconPressRequestsData, AppError> {
    let profile = beacon_profile(&state).await?;
    state.api.beacon_press_requests(&profile).await
}

#[tauri::command]
pub(crate) async fn beacon_press_request_create(
    state: State<'_, AppState>,
    event_id: Option<String>,
    request_kind: String,
    details: Option<String>,
) -> Result<BeaconMutationResult, AppError> {
    let _mutation = state.beacon_mutation.lock().await;
    let profile = beacon_profile(&state).await?;
    let event_id = event_id.as_deref().map(validate_event_id).transpose()?;
    let request_kind = request_kind.trim();
    if !matches!(
        request_kind,
        "press_photo" | "wav" | "clean_version" | "interview" | "accreditation" | "custom"
    ) {
        return Err(AppError::InvalidInput(
            "invalid_press_request_kind".to_owned(),
        ));
    }
    let details = bounded_optional(details, MAX_DETAILS_BYTES)?;
    state
        .api
        .beacon_create_press_request(
            &profile,
            event_id.as_deref(),
            request_kind,
            details.as_deref(),
        )
        .await
}

#[tauri::command]
pub(crate) async fn beacon_engagement(
    state: State<'_, AppState>,
    event_id: String,
    action: String,
    help_kind: Option<String>,
    help_details: Option<String>,
) -> Result<BeaconEngagementResult, AppError> {
    let _mutation = state.beacon_mutation.lock().await;
    let profile = beacon_profile(&state).await?;
    let event_id = validate_event_id(&event_id)?;
    let action = action.trim();
    if !matches!(
        action,
        "opened" | "interested" | "helping" | "completed" | "declined"
    ) {
        return Err(AppError::InvalidInput(
            "invalid_beacon_engagement".to_owned(),
        ));
    }
    let help_kind = bounded_optional(help_kind, 32)?;
    if help_kind.as_ref().is_some_and(|value| {
        !matches!(
            value.as_str(),
            "article" | "radio" | "podcast" | "photos" | "share" | "contact" | "other"
        )
    }) {
        return Err(AppError::InvalidInput(
            "invalid_beacon_help_kind".to_owned(),
        ));
    }
    if action == "helping" && help_kind.is_none() {
        return Err(AppError::InvalidInput(
            "missing_beacon_help_kind".to_owned(),
        ));
    }
    let help_details = bounded_optional(help_details, MAX_DETAILS_BYTES)?;
    state
        .api
        .beacon_engagement(
            &profile,
            &event_id,
            action,
            help_kind.as_deref(),
            help_details.as_deref(),
        )
        .await
}

#[tauri::command]
pub(crate) async fn beacon_coverage(
    state: State<'_, AppState>,
    event_id: String,
    coverage_kind: String,
    url: String,
    title: Option<String>,
) -> Result<BeaconMutationResult, AppError> {
    let _mutation = state.beacon_mutation.lock().await;
    let profile = beacon_profile(&state).await?;
    let event_id = validate_event_id(&event_id)?;
    let coverage_kind = coverage_kind.trim();
    if !matches!(
        coverage_kind,
        "article" | "radio" | "video" | "photo" | "social" | "podcast" | "other"
    ) {
        return Err(AppError::InvalidInput("invalid_coverage_kind".to_owned()));
    }
    let url = https_url(&url)?;
    let title = bounded_optional(title, 300)?;
    state
        .api
        .beacon_coverage(&profile, &event_id, coverage_kind, &url, title.as_deref())
        .await
}

#[tauri::command]
pub(crate) async fn beacon_releases(
    state: State<'_, AppState>,
) -> Result<BeaconReleasesData, AppError> {
    let profile = beacon_profile(&state).await?;
    state.api.beacon_releases(&profile).await
}

#[tauri::command]
pub(crate) async fn beacon_release_confirm(
    state: State<'_, AppState>,
    campaign_id: String,
    recipient_name: String,
    recipient_phone: String,
    parcel_locker_code: String,
) -> Result<BeaconMutationResult, AppError> {
    let _mutation = state.beacon_mutation.lock().await;
    let profile = beacon_profile(&state).await?;
    let campaign_id = validate_campaign_id(&campaign_id)?;
    let recipient_name = recipient_name.trim();
    let recipient_phone = recipient_phone.trim();
    let parcel_locker_code = parcel_locker_code.trim();
    if recipient_name.is_empty()
        || recipient_name.len() > MAX_DELIVERY_FIELD_BYTES
        || recipient_phone.is_empty()
        || recipient_phone.len() > 64
        || parcel_locker_code.is_empty()
        || parcel_locker_code.len() > 64
    {
        return Err(AppError::InvalidInput(
            "invalid_release_delivery".to_owned(),
        ));
    }
    state
        .api
        .beacon_confirm_release(
            &profile,
            &campaign_id,
            recipient_name,
            recipient_phone,
            parcel_locker_code,
        )
        .await
}

#[tauri::command]
pub(crate) async fn beacon_release_decline(
    state: State<'_, AppState>,
    campaign_id: String,
) -> Result<BeaconMutationResult, AppError> {
    let _mutation = state.beacon_mutation.lock().await;
    let profile = beacon_profile(&state).await?;
    let campaign_id = validate_campaign_id(&campaign_id)?;
    state
        .api
        .beacon_decline_release(&profile, &campaign_id)
        .await
}

#[tauri::command]
pub(crate) async fn beacon_logout(
    state: State<'_, AppState>,
) -> Result<BeaconSessionStatus, AppError> {
    let _mutation = state.beacon_mutation.lock().await;
    let profile = beacon_profile(&state).await?;
    state.api.beacon_logout(&profile).await?;
    let app_data_dir = state.app_data_dir.clone();
    run_blocking(move || vault::remove_beacon(&app_data_dir)).await?;
    *state.beacon_session.write().await = None;
    *state.beacon_pin.write().await = None;
    *state.beacon_vault_password.write().await = None;
    drop(_mutation);
    beacon_status(state).await
}

#[tauri::command]
pub(crate) async fn beacon_leave(
    state: State<'_, AppState>,
    do_not_contact: bool,
) -> Result<BeaconSessionStatus, AppError> {
    let _mutation = state.beacon_mutation.lock().await;
    let profile = beacon_profile(&state).await?;
    state.api.beacon_leave(&profile, do_not_contact).await?;
    let app_data_dir = state.app_data_dir.clone();
    run_blocking(move || vault::remove_beacon(&app_data_dir)).await?;
    *state.beacon_session.write().await = None;
    *state.beacon_pin.write().await = None;
    *state.beacon_vault_password.write().await = None;
    drop(_mutation);
    beacon_status(state).await
}

async fn persist_push_state(
    state: &State<'_, AppState>,
    profile: &BeaconProfile,
    desired: bool,
    sync_ok: bool,
) -> Result<Arc<BeaconProfile>, AppError> {
    let mut updated = profile.clone();
    updated.push_enabled = desired;
    updated.push_last_sync_ok = sync_ok;
    persist_beacon(state, &updated).await?;
    let updated = Arc::new(updated);
    *state.beacon_session.write().await = Some(updated.clone());
    Ok(updated)
}

async fn current_push_status(
    state: &State<'_, AppState>,
    app: &AppHandle,
    detail: Option<String>,
) -> FanPushStatus {
    let profile = match beacon_profile(state).await {
        Ok(value) => value,
        Err(_) => {
            return FanPushStatus {
                supported: cfg!(target_os = "android") && state.native_push_available,
                permission: "unknown".to_owned(),
                detail: Some("beacon_locked".to_owned()),
                ..FanPushStatus::default()
            };
        }
    };
    let supported = cfg!(target_os = "android") && state.native_push_available;
    if !supported {
        return FanPushStatus {
            supported: false,
            permission: "unsupported".to_owned(),
            detail: detail.or_else(|| Some("android_push_unavailable".to_owned())),
            ..FanPushStatus::default()
        };
    }
    let permission = native_push_permission(app).unwrap_or_else(|_| "unknown".to_owned());
    let config = state.api.beacon_push_config(&profile).await;
    let (backend_enabled, config_detail) = match config {
        Ok(config) => (config.enabled && config.android_fcm, None),
        Err(error) => (false, Some(format!("push_config_unavailable:{error}"))),
    };
    FanPushStatus {
        supported,
        backend_enabled,
        enabled: profile.push_enabled
            && profile.push_last_sync_ok
            && backend_enabled
            && permission == "granted",
        permission,
        transport: Some("android_fcm".to_owned()),
        detail: detail.or(config_detail),
    }
}

async fn sync_native_push_if_desired(
    state: &State<'_, AppState>,
    app: &AppHandle,
) -> Result<(), AppError> {
    let profile = beacon_profile(state).await?;
    if !state.native_push_available || !cfg!(target_os = "android") {
        return Ok(());
    }
    if !profile.push_enabled {
        if profile.push_last_sync_ok {
            return Ok(());
        }
        if let Some(installation_id) = read_native_push_installation_id(&state.app_data_dir) {
            let response = state
                .api
                .beacon_disable_android_push(&profile, &installation_id)
                .await?;
            if response.registered {
                return Err(AppError::Conflict(
                    "beacon_push_disable_not_confirmed".to_owned(),
                ));
            }
        }
        persist_push_state(state, &profile, false, true).await?;
        return Ok(());
    }
    let config = state.api.beacon_push_config(&profile).await?;
    if !config.enabled || !config.android_fcm {
        let _ = persist_push_state(state, &profile, true, false).await;
        return Err(AppError::Conflict("push_delivery_not_live".to_owned()));
    }
    let permission = native_push_permission(app).map_err(AppError::InvalidInput)?;
    if permission != "granted" {
        let _ = persist_push_state(state, &profile, true, false).await;
        return Err(AppError::Forbidden);
    }
    let token = native_push_token(app).map_err(AppError::InvalidInput)?;
    let installation_id = ensure_native_push_installation_id(&state.app_data_dir)?;
    let response = state
        .api
        .beacon_register_android_push(&profile, &installation_id, &token)
        .await?;
    if !response.registered {
        let _ = persist_push_state(state, &profile, true, false).await;
        return Err(AppError::Conflict(
            "push_registration_not_confirmed".to_owned(),
        ));
    }
    persist_push_state(state, &profile, true, true).await?;
    Ok(())
}

#[tauri::command]
pub(crate) async fn beacon_push_sync(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<FanPushStatus, AppError> {
    let _mutation = state.beacon_mutation.lock().await;
    match sync_native_push_if_desired(&state, &app).await {
        Ok(()) => Ok(current_push_status(&state, &app, None).await),
        Err(AppError::Forbidden) => Ok(current_push_status(
            &state,
            &app,
            Some("notification_permission_denied".to_owned()),
        )
        .await),
        Err(AppError::Conflict(detail)) => {
            Ok(current_push_status(&state, &app, Some(detail)).await)
        }
        Err(error) => Err(error),
    }
}

#[tauri::command]
pub(crate) async fn beacon_push_enable(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<FanPushStatus, AppError> {
    let _mutation = state.beacon_mutation.lock().await;
    let profile = beacon_profile(&state).await?;
    if !state.native_push_available || !cfg!(target_os = "android") {
        return Ok(
            current_push_status(&state, &app, Some("android_push_unavailable".to_owned())).await,
        );
    }
    let config = state.api.beacon_push_config(&profile).await?;
    if !config.enabled || !config.android_fcm {
        return Ok(
            current_push_status(&state, &app, Some("push_delivery_not_live".to_owned())).await,
        );
    }
    let permission = request_native_push_permission(&app).map_err(AppError::InvalidInput)?;
    if permission != "granted" {
        return Ok(current_push_status(
            &state,
            &app,
            Some("notification_permission_denied".to_owned()),
        )
        .await);
    }
    let profile = persist_push_state(&state, &profile, true, false).await?;
    let token = native_push_token(&app).map_err(AppError::InvalidInput)?;
    let installation_id = ensure_native_push_installation_id(&state.app_data_dir)?;
    let response = state
        .api
        .beacon_register_android_push(&profile, &installation_id, &token)
        .await?;
    if !response.registered {
        return Err(AppError::Conflict(
            "push_registration_not_confirmed".to_owned(),
        ));
    }
    persist_push_state(&state, &profile, true, true).await?;
    Ok(current_push_status(&state, &app, None).await)
}

#[tauri::command]
pub(crate) async fn beacon_push_disable(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<FanPushStatus, AppError> {
    let _mutation = state.beacon_mutation.lock().await;
    let profile = beacon_profile(&state).await?;
    let profile = persist_push_state(&state, &profile, false, false).await?;
    let detail = match read_native_push_installation_id(&state.app_data_dir) {
        Some(installation_id) => match state
            .api
            .beacon_disable_android_push(&profile, &installation_id)
            .await
        {
            Ok(response) if !response.registered => {
                persist_push_state(&state, &profile, false, true).await?;
                None
            }
            Ok(_) => Some("beacon_push_disable_not_confirmed".to_owned()),
            Err(error) => Some(format!("remote_disable_unconfirmed:{error}")),
        },
        None => {
            persist_push_state(&state, &profile, false, true).await?;
            None
        }
    };
    Ok(current_push_status(&state, &app, detail).await)
}

#[tauri::command]
pub(crate) async fn beacon_push_open_settings(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<FanPushStatus, AppError> {
    if !state.native_push_available || !cfg!(target_os = "android") {
        return Ok(
            current_push_status(&state, &app, Some("android_push_unavailable".to_owned())).await,
        );
    }
    let _mutation = state.beacon_mutation.lock().await;
    let profile = beacon_profile(&state).await?;
    let _ = persist_push_state(&state, &profile, true, false).await?;
    open_native_push_settings(&app).map_err(AppError::InvalidInput)?;
    Ok(current_push_status(
        &state,
        &app,
        Some("notification_settings_opened".to_owned()),
    )
    .await)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invite_parser_accepts_only_latarnik_https_or_raw_capability() {
        let token = "Abcdefghijklmnopqrstuvwxyz_123456";
        assert_eq!(
            normalize_invite(token).ok().map(|v| v.to_string()),
            Some(token.to_owned())
        );
        assert_eq!(
            normalize_invite(&format!("https://virya.music/pl/latarnik?invite={token}"))
                .ok()
                .map(|v| v.to_string()),
            Some(token.to_owned())
        );
        assert!(
            normalize_invite(&format!("https://evil.example/latarnik?invite={token}")).is_err()
        );
        assert!(normalize_invite(&format!("http://virya.music/latarnik?invite={token}")).is_err());
        assert!(normalize_invite("short").is_err());
    }

    #[test]
    fn beacon_topics_are_strict_and_deduplicated() {
        let topics = normalize_topics(vec!["shows".into(), "shows".into(), "accreditation".into()]);
        assert_eq!(
            topics.ok(),
            Some(vec!["accreditation".into(), "shows".into()])
        );
        assert!(normalize_topics(vec!["street_team".into()]).is_err());
    }
}
