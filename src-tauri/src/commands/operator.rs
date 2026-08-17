//! Operator (owner/staff) session lifecycle and every CrowdRelay command that
//! requires an authenticated operator bearer token.

use std::{path::Path, sync::Arc};

use tauri::{AppHandle, State};
use zeroize::Zeroizing;

use crate::{
    AppError, AppState,
    models::{
        AutopilotAuthorityRequest, AutopilotChiefOfStaff, AutopilotMutation, ConcertQrOverview,
        CreateQrCampaignInput, FanPushStatus, IssuePassInput, OperatorAutopilotOverview,
        OperatorOpsOverview, OperatorProfile, OperatorSignalOverview, OpsRetryResult, PublicEvent,
        SessionStatus, ShowChecklist, StaffEventDashboard, TicketingOverview,
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
    let Ok(bytes) = std::fs::read(path) else {
        return OperatorPushPreference::default();
    };
    if bytes.len() > 256 {
        return OperatorPushPreference::default();
    }
    serde_json::from_slice(&bytes).unwrap_or_default()
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
    if path.exists() {
        std::fs::remove_file(&path)?;
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
    Ok(SessionStatus {
        configured: vault::exists(&state.app_data_dir),
        unlocked: session.is_some(),
        session: session.as_ref().map(|profile| profile.as_ref().into()),
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
    let app_data_dir = state.app_data_dir.clone();
    let pin = Zeroizing::new(pin);
    let vault_pin = pin.clone();
    let profile = run_blocking(move || vault::load(&app_data_dir, vault_pin.as_str())).await?;
    let password_dir = state.app_data_dir.clone();
    let password_pin = pin.clone();
    let vault_password =
        run_blocking(move || vault::operator_password(&password_dir, password_pin.as_str()))
            .await?;
    let _show_mutation = state.show_mode_mutation.lock().await;
    *state.session.write().await = Some(Arc::new(profile));
    *state.operator_pin.write().await = Some(pin);
    *state.operator_vault_password.write().await = Some(vault_password);
    *state.show_mode_store.write().await = None;
    drop(_show_mutation);
    drop(_mutation);
    session_status(state).await
}

#[tauri::command]
pub(crate) async fn lock(state: State<'_, AppState>) -> Result<SessionStatus, AppError> {
    let _mutation = state.operator_mutation.lock().await;
    let _show_mutation = state.show_mode_mutation.lock().await;
    *state.session.write().await = None;
    *state.operator_pin.write().await = None;
    *state.operator_vault_password.write().await = None;
    *state.show_mode_store.write().await = None;
    drop(_show_mutation);
    drop(_mutation);
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
    *state.show_mode_store.write().await = None;
    let app_data_dir = state.app_data_dir.clone();
    run_blocking(move || vault::remove(&app_data_dir)).await?;
    drop(_show_mutation);
    drop(_push_mutation);
    drop(_mutation);
    session_status(state).await
}

#[tauri::command]
pub(crate) async fn operator_events(
    state: State<'_, AppState>,
) -> Result<Vec<PublicEvent>, AppError> {
    let profile = operator_profile(&state).await?;
    state.api.operator_events(&profile).await
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
    super::fan::open_native_push_settings(&app).map_err(AppError::InvalidInput)?;
    // Opening Android Settings backgrounds the WebView. Do not start a remote
    // sync while the app is losing focus: that used to leave the checklist UI
    // stuck in a syncing state until a long request timed out. The existing
    // resume effect completes the user's enable intent after Settings returns.
    Ok(super::fan::current_native_push_status(
        &state,
        &app,
        Some("notification_settings_opened".to_owned()),
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
pub(crate) async fn operator_qr(state: State<'_, AppState>) -> Result<ConcertQrOverview, AppError> {
    let profile = operator_profile(&state).await?;
    state.api.operator_qr(&profile).await
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
) -> Result<Vec<PublicEvent>, AppError> {
    validate_api_base(&api_base_url)?;
    state.api.public_events(&api_base_url).await
}

#[tauri::command]
pub(crate) async fn public_cities(
    state: State<'_, AppState>,
    api_base_url: String,
) -> Result<String, AppError> {
    validate_api_base(&api_base_url)?;
    let cities = state.api.public_cities(&api_base_url).await?;
    serde_json::to_string(&cities).map_err(AppError::from)
}

#[tauri::command]
pub(crate) async fn operator_signal_overview(
    state: State<'_, AppState>,
) -> Result<OperatorSignalOverview, AppError> {
    let profile = operator_profile(&state).await?;
    match state.api.operator_signal_overview(&profile).await {
        Ok(overview) => {
            if let Err(error) = persist_operator_signal_cache(&state, &overview).await {
                eprintln!("[virya:operator-cache] could not persist Signal overview: {error}");
            }
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

#[tauri::command]
pub(crate) async fn operator_ops_overview(
    state: State<'_, AppState>,
) -> Result<OperatorOpsOverview, AppError> {
    let profile = operator_profile(&state).await?;
    state.api.operator_ops_overview(&profile).await
}

#[tauri::command]
pub(crate) async fn operator_autopilot_overview(
    state: State<'_, AppState>,
) -> Result<OperatorAutopilotOverview, AppError> {
    let profile = operator_profile(&state).await?;
    state.api.operator_autopilot_overview(&profile).await
}

#[tauri::command]
pub(crate) async fn operator_autopilot_chief_of_staff(
    state: State<'_, AppState>,
) -> Result<AutopilotChiefOfStaff, AppError> {
    let profile = operator_profile(&state).await?;
    state.api.operator_autopilot_chief_of_staff(&profile).await
}

#[tauri::command]
pub(crate) async fn operator_autopilot_set_authority(
    state: State<'_, AppState>,
    context: String,
    enabled: bool,
    autonomy_level: String,
    minimum_confidence_basis_points: u16,
    max_actions_24h: u32,
    expected_version: i64,
) -> Result<AutopilotMutation, AppError> {
    let profile = operator_profile(&state).await?;
    state
        .api
        .operator_autopilot_set_authority(
            &profile,
            context.trim(),
            AutopilotAuthorityRequest {
                enabled,
                autonomy_level,
                minimum_confidence_basis_points,
                max_actions_24h,
                expected_version,
            },
        )
        .await
}

#[tauri::command]
pub(crate) async fn operator_autopilot_assign(
    state: State<'_, AppState>,
    action_id: String,
    member_key: String,
) -> Result<AutopilotMutation, AppError> {
    let profile = operator_profile(&state).await?;
    state
        .api
        .operator_autopilot_assign(&profile, action_id.trim(), member_key.trim())
        .await
}

#[tauri::command]
pub(crate) async fn operator_autopilot_approve(
    state: State<'_, AppState>,
    action_id: String,
) -> Result<AutopilotMutation, AppError> {
    let profile = operator_profile(&state).await?;
    state
        .api
        .operator_autopilot_approve(&profile, action_id.trim())
        .await
}

#[tauri::command]
pub(crate) async fn operator_autopilot_cancel(
    state: State<'_, AppState>,
    action_id: String,
) -> Result<AutopilotMutation, AppError> {
    let profile = operator_profile(&state).await?;
    state
        .api
        .operator_autopilot_cancel(&profile, action_id.trim())
        .await
}

#[tauri::command]
pub(crate) async fn operator_retry(
    state: State<'_, AppState>,
    target_kind: String,
    target_id: String,
) -> Result<OpsRetryResult, AppError> {
    let profile = operator_profile(&state).await?;
    state
        .api
        .operator_retry(&profile, target_kind.trim(), target_id.trim())
        .await
}
