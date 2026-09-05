//! Operator (owner/staff) session lifecycle and every CrowdRelay command that
//! requires an authenticated operator bearer token.

use std::{path::Path, sync::Arc};

use tauri::{AppHandle, Manager, State};
use zeroize::Zeroizing;

use crate::{
    AppError, AppState,
    models::{
        ConcertQrOverview, CreateQrCampaignInput, FanPushStatus, IssuePassInput, OperatorProfile,
        OperatorSessionPhase, OperatorSignalOverview, PublicEventsResult, SessionStatus,
        ShowChecklist, StaffEventDashboard, TicketingOverview,
    },
    session::{
        load_operator_signal_cache, operator_profile, persist_operator_signal_cache, run_blocking,
    },
    validation::{
        validate_api_base, validate_campaign, validate_issue_pass, validate_new_operator_pin,
        validate_operator_profile, validate_pin,
    },
    vault,
};

const OPERATOR_PUSH_PREFERENCE_FILE: &str = "staff-push-preference-v1.json";

fn operator_signal_cache_fallback_allowed(error: &AppError) -> bool {
    matches!(
        error,
        AppError::Network(_)
            | AppError::Remote {
                status: 500..=599,
                ..
            }
    )
}

#[derive(Clone, Copy, Debug, Default, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct OperatorPushPreference {
    desired: bool,
    last_sync_ok: bool,
}

fn read_operator_push_preference(app_data_dir: &Path) -> OperatorPushPreference {
    let path = app_data_dir.join(OPERATOR_PUSH_PREFERENCE_FILE);
    let Ok(bytes) = std::fs::read(&path) else {
        return OperatorPushPreference::default();
    };
    if bytes.len() > 256 {
        return OperatorPushPreference::default();
    }
    match serde_json::from_slice(&bytes) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("[virya:operator-push] corrupted preference reset to default: {error}");
            let _ = std::fs::remove_file(&path);
            OperatorPushPreference::default()
        }
    }
}

fn persist_operator_push_preference(
    app_data_dir: &Path,
    preference: OperatorPushPreference,
) -> Result<(), AppError> {
    std::fs::create_dir_all(app_data_dir)?;
    let path = app_data_dir.join(OPERATOR_PUSH_PREFERENCE_FILE);
    let temporary = app_data_dir.join(format!(".{OPERATOR_PUSH_PREFERENCE_FILE}.tmp"));
    std::fs::write(&temporary, serde_json::to_vec(&preference)?)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        std::fs::set_permissions(&temporary, std::fs::Permissions::from_mode(0o600))?;
    }
    // Plain rename replaces atomically on Unix. The explicit remove only
    // exists for platforms where rename fails on an existing target; removing
    // unconditionally would let a crash between remove and rename re-enable
    // notifications the operator had turned off.
    #[cfg(windows)]
    {
        if path.exists() {
            std::fs::remove_file(&path)?;
        }
    }
    std::fs::rename(temporary, path)?;
    Ok(())
}

fn clear_operator_push_preference(app_data_dir: &Path) -> Result<(), AppError> {
    let path = app_data_dir.join(OPERATOR_PUSH_PREFERENCE_FILE);
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

#[tauri::command]
pub(crate) async fn session_status(state: State<'_, AppState>) -> Result<SessionStatus, AppError> {
    let session = state.session.read().await;
    let phase = *state.operator_phase.read().await;
    Ok(SessionStatus {
        configured: vault::exists(&state.app_data_dir),
        unlocked: session.is_some(),
        session: session.as_ref().map(|profile| profile.as_ref().into()),
        phase,
    })
}

#[tauri::command]
pub(crate) async fn configure(
    state: State<'_, AppState>,
    pin: String,
    mut profile: OperatorProfile,
) -> Result<SessionStatus, AppError> {
    let _mutation = state.operator_mutation.lock().await;
    validate_operator_profile(&mut profile)?;
    validate_new_operator_pin(&pin)?;
    state.api.validate(&profile).await?;
    let app_data_dir = state.app_data_dir.clone();
    let stored_profile = profile.clone();
    let pin = Zeroizing::new(pin);
    let vault_pin = pin.clone();
    let persisted_profile = run_blocking(move || {
        vault::save_verified(&app_data_dir, vault_pin.as_str(), &stored_profile)
    })
    .await?;
    let password_dir = state.app_data_dir.clone();
    let password_pin = pin.clone();
    let vault_password =
        run_blocking(move || vault::operator_password(&password_dir, password_pin.as_str()))
            .await?;
    let _show_mutation = state.show_mode_mutation.lock().await;
    *state.session.write().await = Some(Arc::new(persisted_profile));
    *state.operator_pin.write().await = Some(pin);
    *state.operator_vault_password.write().await = Some(vault_password);
    *state.operator_phase.write().await = OperatorSessionPhase::Active;
    *state.show_mode_store.write().await = None;
    drop(_show_mutation);
    drop(_mutation);
    session_status(state).await
}

#[tauri::command]
pub(crate) async fn unlock(
    state: State<'_, AppState>,
    pin: String,
) -> Result<SessionStatus, AppError> {
    let _mutation = state.operator_mutation.lock().await;
    validate_pin(&pin)?;
    if !vault::exists(&state.app_data_dir) {
        return Err(AppError::NotConfigured);
    }
    let app_data_dir = state.app_data_dir.clone();
    let pin = Zeroizing::new(pin);
    let vault_pin = pin.clone();
    // Argon2 is the whole cost of a staff unlock. Derive the vault password
    // once and open the profile with it: deriving it a second time from the
    // same pin and salt doubled the wait on the login screen for nothing.
    let (profile, vault_password) = run_blocking(move || {
        let password = vault::operator_password(&app_data_dir, vault_pin.as_str())?;
        let profile = vault::load_operator_with_password(&app_data_dir, password.as_ref())?;
        Ok((profile, password))
    })
    .await?;
    let _show_mutation = state.show_mode_mutation.lock().await;
    *state.session.write().await = Some(Arc::new(profile));
    *state.operator_pin.write().await = Some(pin);
    *state.operator_vault_password.write().await = Some(vault_password);
    *state.operator_phase.write().await = OperatorSessionPhase::Active;
    *state.show_mode_store.write().await = None;
    drop(_show_mutation);
    drop(_mutation);
    session_status(state).await
}

#[tauri::command]
pub(crate) async fn lock(state: State<'_, AppState>) -> Result<SessionStatus, AppError> {
    // Locking only drops in-memory session material, so it deliberately does not
    // queue behind the mutation locks. A background reconciliation can hold those
    // for the length of a network call, and "log me out" must not wait for it.
    // An in-flight mutation already owns a cloned profile; anything starting
    // after this point sees a locked session.
    *state.session.write().await = None;
    *state.operator_pin.write().await = None;
    *state.operator_vault_password.write().await = None;
    *state.operator_phase.write().await = OperatorSessionPhase::Locked;
    *state.show_mode_store.write().await = None;
    *state.operator_sections_cache.write().await = None;
    session_status(state).await
}

#[tauri::command]
pub(crate) async fn forget_device(state: State<'_, AppState>) -> Result<SessionStatus, AppError> {
    let _mutation = state.operator_mutation.lock().await;
    let _push_mutation = state.operator_push_mutation.lock().await;
    let _show_mutation = state.show_mode_mutation.lock().await;

    // A removed staff profile must not leave a live, authenticated push endpoint
    // behind. Forget therefore requires the profile to be unlocked: without its
    // bearer there is no safe way to prove remote endpoint cleanup. `lock` stays
    // available for immediate local privacy without destroying that capability.
    let profile = state.session.read().await.clone().ok_or(AppError::Locked)?;
    if let Some(installation_id) = super::fan::read_native_push_installation_id(&state.app_data_dir)
    {
        let response = state
            .api
            .operator_disable_android_push(&profile, &installation_id)
            .await?;
        if response.registered {
            return Err(AppError::Conflict(
                "staff_push_disable_not_confirmed".to_owned(),
            ));
        }
    }

    clear_operator_push_preference(&state.app_data_dir)?;
    *state.session.write().await = None;
    *state.operator_pin.write().await = None;
    *state.operator_vault_password.write().await = None;
    *state.operator_phase.write().await = OperatorSessionPhase::Unconfigured;
    *state.show_mode_store.write().await = None;
    *state.operator_sections_cache.write().await = None;
    let app_data_dir = state.app_data_dir.clone();
    run_blocking(move || vault::remove(&app_data_dir)).await?;
    drop(_show_mutation);
    drop(_push_mutation);
    drop(_mutation);
    session_status(state).await
}

#[tauri::command]
pub(crate) async fn operator_events(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<PublicEventsResult, AppError> {
    let profile = operator_profile(&state).await?;
    let result = state.api.operator_events(&profile).await?;
    remember_operator_sections(app, {
        let events = result.events.clone();
        move |snapshot| snapshot.events = events
    });
    Ok(result)
}

#[tauri::command]
pub(crate) async fn operator_show_checklist(
    state: State<'_, AppState>,
    event_slug: String,
) -> Result<ShowChecklist, AppError> {
    let profile = operator_profile(&state).await?;
    state
        .api
        .operator_show_checklist(&profile, &event_slug)
        .await
}

#[tauri::command]
pub(crate) async fn operator_update_show_checklist(
    state: State<'_, AppState>,
    event_slug: String,
    item_key: String,
    status: String,
) -> Result<ShowChecklist, AppError> {
    if !matches!(status.as_str(), "pending" | "done" | "blocked" | "skipped") {
        return Err(AppError::InvalidInput(
            "invalid checklist status".to_owned(),
        ));
    }
    let profile = operator_profile(&state).await?;
    state
        .api
        .operator_update_show_checklist(&profile, &event_slug, &item_key, &status)
        .await
}

async fn sync_operator_push(
    state: &State<'_, AppState>,
    app: &AppHandle,
    request_permission: bool,
) -> Result<FanPushStatus, AppError> {
    let profile = operator_profile(state).await?;
    let supported = cfg!(target_os = "android") && state.native_push_available;
    let preference = read_operator_push_preference(&state.app_data_dir);
    if !supported {
        return Ok(FanPushStatus {
            supported: false,
            permission: "unsupported".to_owned(),
            detail: Some("android_push_unavailable".to_owned()),
            ..FanPushStatus::default()
        });
    }

    let permission = if request_permission {
        super::fan::request_native_push_permission(app)
    } else {
        super::fan::native_push_permission(app)
    }
    .map_err(AppError::InvalidInput)?;

    if !preference.desired {
        if !preference.last_sync_ok {
            if let Some(installation_id) =
                super::fan::read_native_push_installation_id(&state.app_data_dir)
            {
                let response = state
                    .api
                    .operator_disable_android_push(&profile, &installation_id)
                    .await?;
                if response.registered {
                    return Err(AppError::Conflict(
                        "staff_push_disable_not_confirmed".to_owned(),
                    ));
                }
            }
            persist_operator_push_preference(
                &state.app_data_dir,
                OperatorPushPreference {
                    desired: false,
                    last_sync_ok: true,
                },
            )?;
        }
        return Ok(FanPushStatus {
            supported,
            backend_enabled: true,
            enabled: false,
            permission,
            transport: Some("android_fcm".to_owned()),
            detail: None,
        });
    }

    let config = state.api.operator_push_config(&profile).await?;
    let backend_enabled = config.enabled && config.android_fcm;
    if !backend_enabled {
        persist_operator_push_preference(
            &state.app_data_dir,
            OperatorPushPreference {
                desired: true,
                last_sync_ok: false,
            },
        )?;
        return Ok(FanPushStatus {
            supported,
            backend_enabled: false,
            permission,
            transport: Some("android_fcm".to_owned()),
            detail: Some("push_delivery_not_live".to_owned()),
            ..FanPushStatus::default()
        });
    }
    if permission != "granted" {
        persist_operator_push_preference(
            &state.app_data_dir,
            OperatorPushPreference {
                desired: true,
                last_sync_ok: false,
            },
        )?;
        return Ok(FanPushStatus {
            supported,
            backend_enabled,
            enabled: false,
            permission,
            transport: Some("android_fcm".to_owned()),
            detail: Some("notification_permission_denied".to_owned()),
        });
    }
    let token = super::fan::native_push_token(app).map_err(AppError::InvalidInput)?;
    let installation_id = super::fan::ensure_native_push_installation_id(&state.app_data_dir)?;
    let response = state
        .api
        .operator_register_android_push(&profile, &installation_id, &token)
        .await?;
    persist_operator_push_preference(
        &state.app_data_dir,
        OperatorPushPreference {
            desired: true,
            last_sync_ok: response.registered,
        },
    )?;
    Ok(FanPushStatus {
        supported,
        backend_enabled,
        enabled: response.registered,
        permission,
        transport: Some("android_fcm".to_owned()),
        detail: (!response.registered).then(|| "push_registration_not_confirmed".to_owned()),
    })
}

#[tauri::command]
pub(crate) async fn operator_push_sync(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<FanPushStatus, AppError> {
    let _push_mutation = state.operator_push_mutation.lock().await;
    sync_operator_push(&state, &app, false).await
}

#[tauri::command]
pub(crate) async fn operator_push_enable(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<FanPushStatus, AppError> {
    let _push_mutation = state.operator_push_mutation.lock().await;
    operator_profile(&state).await?;
    persist_operator_push_preference(
        &state.app_data_dir,
        OperatorPushPreference {
            desired: true,
            last_sync_ok: false,
        },
    )?;
    sync_operator_push(&state, &app, true).await
}

#[tauri::command]
pub(crate) async fn operator_push_open_settings(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<FanPushStatus, AppError> {
    let _push_mutation = state.operator_push_mutation.lock().await;
    operator_profile(&state).await?;
    persist_operator_push_preference(
        &state.app_data_dir,
        OperatorPushPreference {
            desired: true,
            last_sync_ok: false,
        },
    )?;
    super::fan::open_native_push_settings(&app).map_err(AppError::InvalidInput)?;
    // Opening Android Settings backgrounds the WebView. Do not start a remote
    // sync while the app is losing focus: that used to leave the checklist UI
    // stuck in a syncing state until a long request timed out. The existing
    // resume effect completes the user's enable intent after Settings returns.
    Ok(super::fan::current_native_push_status(
        &state,
        &app,
        Some("notification_settings_opened".to_owned()),
        None,
    )
    .await)
}

#[tauri::command]
pub(crate) async fn operator_push_disable(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<FanPushStatus, AppError> {
    let _push_mutation = state.operator_push_mutation.lock().await;
    let profile = operator_profile(&state).await?;
    persist_operator_push_preference(
        &state.app_data_dir,
        OperatorPushPreference {
            desired: false,
            last_sync_ok: false,
        },
    )?;
    let mut detail = None;
    if let Some(installation_id) = super::fan::read_native_push_installation_id(&state.app_data_dir)
    {
        match state
            .api
            .operator_disable_android_push(&profile, &installation_id)
            .await
        {
            Ok(response) if !response.registered => {
                persist_operator_push_preference(
                    &state.app_data_dir,
                    OperatorPushPreference {
                        desired: false,
                        last_sync_ok: true,
                    },
                )?;
            }
            Ok(_) => detail = Some("staff_push_disable_not_confirmed".to_owned()),
            Err(error) => detail = Some(format!("remote_disable_unconfirmed:{error}")),
        }
    } else {
        persist_operator_push_preference(
            &state.app_data_dir,
            OperatorPushPreference {
                desired: false,
                last_sync_ok: true,
            },
        )?;
    }
    let permission =
        super::fan::native_push_permission(&app).unwrap_or_else(|_| "unknown".to_owned());
    Ok(FanPushStatus {
        supported: cfg!(target_os = "android") && state.native_push_available,
        backend_enabled: true,
        enabled: false,
        permission,
        transport: Some("android_fcm".to_owned()),
        detail,
    })
}

#[tauri::command]
pub(crate) async fn operator_qr(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<ConcertQrOverview, AppError> {
    let profile = operator_profile(&state).await?;
    let overview = state.api.operator_qr(&profile).await?;
    remember_operator_sections(app, {
        let overview = overview.clone();
        move |snapshot| snapshot.qr = Some(overview)
    });
    Ok(overview)
}

#[tauri::command]
pub(crate) async fn staff_event_dashboard(
    state: State<'_, AppState>,
    event_slug: String,
) -> Result<StaffEventDashboard, AppError> {
    let profile = operator_profile(&state).await?;
    state.api.staff_event_dashboard(&profile, &event_slug).await
}

#[tauri::command]
pub(crate) async fn ticketing_overview(
    state: State<'_, AppState>,
    event_slug: String,
) -> Result<TicketingOverview, AppError> {
    let profile = operator_profile(&state).await?;
    state.api.ticketing_overview(&profile, &event_slug).await
}

#[tauri::command]
pub(crate) async fn redeem_admission(
    state: State<'_, AppState>,
    event_slug: String,
    code: String,
) -> Result<serde_json::Value, AppError> {
    let profile = operator_profile(&state).await?;
    state
        .api
        .redeem_admission(&profile, &event_slug, &code)
        .await
}

#[tauri::command]
pub(crate) async fn redeem_coupon(
    state: State<'_, AppState>,
    code: String,
    order_reference: String,
) -> Result<serde_json::Value, AppError> {
    let profile = operator_profile(&state).await?;
    state
        .api
        .redeem_coupon(&profile, &code, &order_reference)
        .await
}

#[tauri::command]
pub(crate) async fn issue_pass(
    state: State<'_, AppState>,
    mut input: IssuePassInput,
) -> Result<serde_json::Value, AppError> {
    validate_issue_pass(&mut input)?;
    let profile = operator_profile(&state).await?;
    state.api.issue_pass(&profile, &input).await
}

#[tauri::command]
pub(crate) async fn revoke_pass(
    state: State<'_, AppState>,
    public_reference: String,
) -> Result<serde_json::Value, AppError> {
    let profile = operator_profile(&state).await?;
    state.api.revoke_pass(&profile, &public_reference).await
}

#[tauri::command]
pub(crate) async fn create_qr_campaign(
    state: State<'_, AppState>,
    mut input: CreateQrCampaignInput,
) -> Result<serde_json::Value, AppError> {
    validate_campaign(&mut input)?;
    let profile = operator_profile(&state).await?;
    state.api.create_qr_campaign(&profile, &input).await
}

#[tauri::command]
pub(crate) async fn revoke_qr_campaign(
    state: State<'_, AppState>,
    campaign_id: String,
) -> Result<serde_json::Value, AppError> {
    let profile = operator_profile(&state).await?;
    state.api.revoke_qr_campaign(&profile, &campaign_id).await
}

#[tauri::command]
pub(crate) async fn public_events(
    state: State<'_, AppState>,
    api_base_url: String,
) -> Result<PublicEventsResult, AppError> {
    validate_api_base(&api_base_url)?;
    state.api.public_events(&api_base_url).await
}

#[tauri::command]
pub(crate) async fn public_cities(
    state: State<'_, AppState>,
    api_base_url: String,
) -> Result<String, AppError> {
    validate_api_base(&api_base_url)?;
    let result = state.api.public_cities(&api_base_url).await?;
    serde_json::to_string(&result).map_err(AppError::from)
}

/// Returns every operator panel's last known value straight from the vault, so
/// a cold Latarnik paints from disk while its six live requests are still in
/// flight instead of holding six skeletons.
#[tauri::command]
pub(crate) async fn operator_cached_sections(
    state: State<'_, AppState>,
) -> Result<Option<vault::OperatorSectionsCacheSnapshot>, AppError> {
    // Prove the session is unlocked before the in-memory mirror is served.
    // Locking clears the vault password, and a decrypted mirror must never
    // outlive it.
    let password = state
        .operator_vault_password
        .read()
        .await
        .as_ref()
        .map(|value| Zeroizing::new(value.to_vec()))
        .ok_or(AppError::Locked)?;
    if let Some(snapshot) = state.operator_sections_cache.read().await.clone() {
        return Ok(Some(snapshot));
    }
    let app_data_dir = state.app_data_dir.clone();
    let snapshot = match run_blocking(move || {
        vault::load_operator_sections_cache_with_password(&app_data_dir, password.as_ref())
    })
    .await
    {
        Ok(snapshot) => snapshot,
        Err(error) => {
            eprintln!("[virya:operator-cache] encrypted panel snapshot ignored: {error}");
            None
        }
    };
    if let Some(snapshot) = snapshot.clone() {
        *state.operator_sections_cache.write().await = Some(snapshot);
    }
    Ok(snapshot)
}

/// Folds one refreshed panel into the encrypted operator snapshot. Spawned and
/// serialized behind its own lock, for the same reason as the fan equivalent:
/// the write only has to land before the next cold start.
fn remember_operator_sections(
    app: AppHandle,
    apply: impl FnOnce(&mut vault::OperatorSectionsCacheSnapshot) + Send + 'static,
) {
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        let _mutation = state.operator_sections_cache_mutation.lock().await;
        let Some(password) = state
            .operator_vault_password
            .read()
            .await
            .as_ref()
            .map(|value| Zeroizing::new(value.to_vec()))
        else {
            return;
        };
        let mut snapshot = state
            .operator_sections_cache
            .read()
            .await
            .clone()
            .unwrap_or_default();
        apply(&mut snapshot);
        snapshot.stored_at_unix_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|value| value.as_secs())
            .unwrap_or(0);
        *state.operator_sections_cache.write().await = Some(snapshot.clone());
        let app_data_dir = state.app_data_dir.clone();
        if let Err(error) = run_blocking(move || {
            vault::save_operator_sections_cache_with_password(
                &app_data_dir,
                password.as_ref(),
                &snapshot,
            )
        })
        .await
        {
            eprintln!("[virya:operator-cache] encrypted panel snapshot save degraded: {error}");
        }
    });
}

#[tauri::command]
pub(crate) async fn operator_signal_overview(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<OperatorSignalOverview, AppError> {
    let profile = operator_profile(&state).await?;
    match state.api.operator_signal_overview(&profile).await {
        Ok(overview) => {
            if let Err(error) = persist_operator_signal_cache(&state, &overview).await {
                eprintln!("[virya:operator-cache] could not persist Signal overview: {error}");
            }
            remember_operator_sections(app, {
                let overview = overview.clone();
                move |snapshot| snapshot.signal = Some(overview)
            });
            Ok(overview)
        }
        Err(network_error) if operator_signal_cache_fallback_allowed(&network_error) => {
            match load_operator_signal_cache(&state).await {
                Ok(Some(mut cached)) => {
                    if !cached
                        .unavailable_sources
                        .iter()
                        .any(|source| source == "offline_cache")
                    {
                        cached.unavailable_sources.push("offline_cache".to_owned());
                    }
                    Ok(cached)
                }
                Ok(None) | Err(_) => Err(network_error),
            }
        }
        Err(error) => Err(error),
    }
}
