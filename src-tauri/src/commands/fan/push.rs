fn valid_native_push_installation_id(value: &str) -> bool {
    let Some(raw) = value.strip_prefix("android-") else {
        return false;
    };
    uuid::Uuid::parse_str(raw).is_ok() && value.len() <= 160
}

pub(crate) fn read_native_push_installation_id(app_data_dir: &Path) -> Option<String> {
    let value = std::fs::read_to_string(app_data_dir.join(NATIVE_PUSH_INSTALLATION_FILE)).ok()?;
    let value = value.trim();
    valid_native_push_installation_id(value).then(|| value.to_owned())
}

pub(crate) fn ensure_native_push_installation_id(app_data_dir: &Path) -> Result<String, AppError> {
    if let Some(value) = read_native_push_installation_id(app_data_dir) {
        return Ok(value);
    }
    let value = format!("android-{}", uuid::Uuid::new_v4());
    let path = app_data_dir.join(NATIVE_PUSH_INSTALLATION_FILE);
    let temporary = app_data_dir.join(format!(".{NATIVE_PUSH_INSTALLATION_FILE}.tmp"));
    std::fs::write(&temporary, format!("{value}\n"))?;
    std::fs::rename(temporary, path)?;
    Ok(value)
}

#[cfg(target_os = "android")]
pub(crate) fn native_push_permission(app: &AppHandle) -> Result<String, String> {
    crate::push_plugin::permission(app)
}

#[cfg(not(target_os = "android"))]
pub(crate) fn native_push_permission(_app: &AppHandle) -> Result<String, String> {
    Err("android_push_unavailable".to_owned())
}

#[cfg(target_os = "android")]
pub(crate) fn request_native_push_permission(app: &AppHandle) -> Result<String, String> {
    crate::push_plugin::request_permission(app)
}

#[cfg(not(target_os = "android"))]
pub(crate) fn request_native_push_permission(_app: &AppHandle) -> Result<String, String> {
    Err("android_push_unavailable".to_owned())
}

#[cfg(target_os = "android")]
pub(crate) fn native_firebase_configured(app: &AppHandle) -> Result<bool, String> {
    crate::push_plugin::firebase_configured(app)
}

#[cfg(not(target_os = "android"))]
pub(crate) fn native_firebase_configured(_app: &AppHandle) -> Result<bool, String> {
    Ok(false)
}

#[cfg(target_os = "android")]
pub(crate) fn native_push_token(app: &AppHandle) -> Result<String, String> {
    crate::push_plugin::token(app)
}

#[cfg(not(target_os = "android"))]
pub(crate) fn native_push_token(_app: &AppHandle) -> Result<String, String> {
    Err("android_push_unavailable".to_owned())
}

#[cfg(target_os = "android")]
fn take_native_push_target(app: &AppHandle) -> Result<Option<String>, String> {
    crate::push_plugin::take_launch_target(app)
}

#[cfg(not(target_os = "android"))]
fn take_native_push_target(_app: &AppHandle) -> Result<Option<String>, String> {
    Ok(None)
}

#[cfg(target_os = "android")]
pub(crate) fn open_native_push_settings(app: &AppHandle) -> Result<(), String> {
    crate::push_plugin::open_notification_settings(app)
}

#[cfg(not(target_os = "android"))]
pub(crate) fn open_native_push_settings(_app: &AppHandle) -> Result<(), String> {
    Err("android_push_unavailable".to_owned())
}


/// Writes the new push state to the vault and publishes it as the live fan
/// session. Returns `Ok(None)` when the session moved on while the vault write
/// was in flight, meaning nothing was published.
///
/// `expected` is the session the caller reconciles for, or `None` for a
/// user-initiated command that reconciles whatever is live.
async fn persist_push_state(
    state: &State<'_, AppState>,
    expected: Option<&Arc<FanProfile>>,
    profile: &FanProfile,
    desired: bool,
    sync_ok: bool,
) -> Result<Option<Arc<FanProfile>>, AppError> {
    let mut updated = profile.clone();
    updated.push_enabled = desired;
    updated.push_last_sync_ok = sync_ok;
    persist_fan(state, &updated).await?;
    let updated = Arc::new(updated);
    // `fan_lock` deliberately does not queue behind `fan_mutation`, so the fan
    // can log out while the vault write above is running. Publishing
    // unconditionally here would put the session back and silently undo that
    // logout, so the live slot is re-checked under its own write guard.
    let mut session = state.fan_session.write().await;
    let still_current = session
        .as_ref()
        .is_some_and(|current| expected.is_none_or(|expected| Arc::ptr_eq(current, expected)));
    if !still_current {
        return Ok(None);
    }
    *session = Some(updated.clone());
    Ok(Some(updated))
}

/// `persist_push_state` for user-initiated commands, where a session that
/// disappeared mid-command is a real failure the caller must surface.
async fn persist_push_state_now(
    state: &State<'_, AppState>,
    profile: &FanProfile,
    desired: bool,
    sync_ok: bool,
) -> Result<Arc<FanProfile>, AppError> {
    persist_push_state(state, None, profile, desired, sync_ok)
        .await?
        .ok_or(AppError::Locked)
}

async fn session_is_current(
    state: &State<'_, AppState>,
    expected: Option<&Arc<FanProfile>>,
) -> bool {
    match expected {
        Some(expected) => state
            .fan_session
            .read()
            .await
            .as_ref()
            .is_some_and(|current| Arc::ptr_eq(current, expected)),
        None => true,
    }
}

/// Takes `fan_mutation` for the legs of reconciliation that write push state,
/// locally or remotely, so a user-initiated sync can never interleave its
/// remote call with a background one and leave the two sides disagreeing.
///
/// Returns `None` when the session the caller reconciles for is no longer the
/// live one and the leg must be abandoned. The inner `Option` is the guard:
/// user-initiated callers pass `expected == None` because they already own the
/// lock, which is not reentrant.
async fn lock_for_push_mutation<'a>(
    state: &'a State<'_, AppState>,
    expected: Option<&Arc<FanProfile>>,
) -> Option<Option<MutexGuard<'a, ()>>> {
    let Some(expected) = expected else {
        return Some(None);
    };
    let guard = state.fan_mutation.lock().await;
    state
        .fan_session
        .read()
        .await
        .as_ref()
        .is_some_and(|current| Arc::ptr_eq(current, expected))
        .then_some(Some(guard))
}

pub(crate) async fn current_native_push_status(
    state: &State<'_, AppState>,
    app: &AppHandle,
    detail: Option<String>,
    cached_config: Option<&FanPushConfigApi>,
) -> FanPushStatus {
    let profile = match fan_profile(state).await {
        Ok(value) => value,
        Err(_) => {
            return FanPushStatus {
                supported: cfg!(target_os = "android") && state.native_push_available,
                permission: "unknown".to_owned(),
                detail: Some("fan_locked".to_owned()),
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
    // Reuse the config a sync/enable already fetched instead of a second
    // round-trip on cellular. Only fetch when the caller had no reason to.
    let (backend_enabled, config_detail) = match cached_config {
        Some(config) => (config.enabled && config.android_fcm, None),
        None => match state.api.fan_push_config(&profile).await {
            Ok(config) => (config.enabled && config.android_fcm, None),
            Err(error) => (false, Some(format!("push_config_unavailable:{error}"))),
        },
    };
    let firebase_configured = native_firebase_configured(app).unwrap_or(false);
    let provider_detail = (!firebase_configured).then(|| "firebase_not_configured".to_owned());
    FanPushStatus {
        supported,
        backend_enabled,
        enabled: profile.push_enabled
            && profile.push_last_sync_ok
            && backend_enabled
            && firebase_configured
            && permission == "granted",
        permission,
        transport: Some("android_fcm".to_owned()),
        detail: detail.or(config_detail).or(provider_detail),
    }
}

async fn sync_native_push_if_desired(
    state: &State<'_, AppState>,
    app: &AppHandle,
    expected: Option<Arc<FanProfile>>,
) -> Result<Option<FanPushConfigApi>, AppError> {
    let profile = fan_profile(state).await?;
    let expected_ref = expected.as_ref();
    if !session_is_current(state, expected_ref).await {
        return Ok(None);
    }
    if !state.native_push_available || !cfg!(target_os = "android") {
        return Ok(None);
    }
    if !profile.push_enabled {
        if profile.push_last_sync_ok {
            return Ok(None);
        }
        let Some(_mutation) = lock_for_push_mutation(state, expected_ref).await else {
            return Ok(None);
        };
        if let Some(installation_id) = read_native_push_installation_id(&state.app_data_dir) {
            let response = state
                .api
                .fan_disable_android_push(&profile, &installation_id)
                .await?;
            if response.registered {
                return Err(AppError::Conflict(
                    "fan_push_disable_not_confirmed".to_owned(),
                ));
            }
        }
        // FCM tokens are device-scoped and shared by fan/staff audiences.
        // Disabling the fan audience must never delete the provider token, or
        // staff reminders on the same installation would silently break.
        persist_push_state(state, expected_ref, &profile, false, true).await?;
        return Ok(None);
    }
    if !session_is_current(state, expected_ref).await {
        return Ok(None);
    }
    let config = state.api.fan_push_config(&profile).await?;
    // The config read above is the only remaining lock-free network leg. Past
    // this point every branch writes push state, so the rest runs under one
    // `fan_mutation` hold; the local Android calls in between do not block.
    let Some(_mutation) = lock_for_push_mutation(state, expected_ref).await else {
        return Ok(Some(config));
    };
    if !config.enabled || !config.android_fcm {
        let _ = persist_push_state(state, expected_ref, &profile, true, false).await;
        return Err(AppError::Conflict("push_delivery_not_live".to_owned()));
    }
    let permission = native_push_permission(app).map_err(AppError::InvalidInput)?;
    if permission != "granted" {
        let _ = persist_push_state(state, expected_ref, &profile, true, false).await;
        return Err(AppError::Forbidden);
    }
    let token = native_push_token(app).map_err(AppError::InvalidInput)?;
    let installation_id = ensure_native_push_installation_id(&state.app_data_dir)?;
    let response = state
        .api
        .fan_register_android_push(&profile, &installation_id, &token)
        .await?;
    if !response.registered {
        let _ = persist_push_state(state, expected_ref, &profile, true, false).await;
        return Err(AppError::Conflict(
            "push_registration_not_confirmed".to_owned(),
        ));
    }
    persist_push_state(state, expected_ref, &profile, true, true).await?;
    Ok(Some(config))
}

#[tauri::command]
pub(crate) async fn fan_push_sync(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<FanPushStatus, AppError> {
    let _mutation = state.fan_mutation.lock().await;
    match sync_native_push_if_desired(&state, &app, None).await {
        Ok(config) => {
            Ok(current_native_push_status(&state, &app, None, config.as_ref()).await)
        }
        Err(AppError::Forbidden) => Ok(current_native_push_status(
            &state,
            &app,
            Some("notification_permission_denied".to_owned()),
            None,
        )
        .await),
        Err(AppError::Conflict(detail)) => {
            Ok(current_native_push_status(&state, &app, Some(detail), None).await)
        }
        Err(error) => Err(error),
    }
}

#[tauri::command]
pub(crate) async fn fan_push_status(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<FanPushStatus, AppError> {
    fan_push_sync(state, app).await
}

#[tauri::command]
pub(crate) async fn fan_push_enable(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<FanPushStatus, AppError> {
    let _mutation = state.fan_mutation.lock().await;
    let profile = fan_profile(&state).await?;
    if !state.native_push_available || !cfg!(target_os = "android") {
        return Ok(current_native_push_status(
            &state,
            &app,
            Some("android_push_unavailable".to_owned()),
            None,
        )
        .await);
    }
    let config = state.api.fan_push_config(&profile).await?;
    if !config.enabled || !config.android_fcm {
        return Ok(current_native_push_status(
            &state,
            &app,
            Some("push_delivery_not_live".to_owned()),
            Some(&config),
        )
        .await);
    }
    let permission = request_native_push_permission(&app).map_err(AppError::InvalidInput)?;
    if permission != "granted" {
        return Ok(current_native_push_status(
            &state,
            &app,
            Some("notification_permission_denied".to_owned()),
            Some(&config),
        )
        .await);
    }
    // Persist desired intent before the remote write. If the network response is
    // lost, the next unlock retries registration with the current FCM token.
    let profile = persist_push_state_now(&state, &profile, true, false).await?;
    let token = native_push_token(&app).map_err(AppError::InvalidInput)?;
    let installation_id = ensure_native_push_installation_id(&state.app_data_dir)?;
    let response = state
        .api
        .fan_register_android_push(&profile, &installation_id, &token)
        .await?;
    if !response.registered {
        return Err(AppError::Conflict(
            "push_registration_not_confirmed".to_owned(),
        ));
    }
    persist_push_state_now(&state, &profile, true, true).await?;
    Ok(current_native_push_status(&state, &app, None, Some(&config)).await)
}

#[tauri::command]
pub(crate) async fn fan_push_disable(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<FanPushStatus, AppError> {
    let _mutation = state.fan_mutation.lock().await;
    let profile = fan_profile(&state).await?;
    let profile = persist_push_state_now(&state, &profile, false, false).await?;
    let detail = match read_native_push_installation_id(&state.app_data_dir) {
        Some(installation_id) => match state
            .api
            .fan_disable_android_push(&profile, &installation_id)
            .await
        {
            Ok(response) if !response.registered => {
                persist_push_state_now(&state, &profile, false, true).await?;
                None
            }
            Ok(_) => Some("fan_push_disable_not_confirmed".to_owned()),
            Err(error) => Some(format!("remote_disable_unconfirmed:{error}")),
        },
        None => {
            persist_push_state_now(&state, &profile, false, true).await?;
            None
        }
    };
    Ok(current_native_push_status(&state, &app, detail, None).await)
}

#[tauri::command]
pub(crate) async fn fan_push_take_target(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<Option<String>, AppError> {
    if !state.native_push_available || !cfg!(target_os = "android") {
        return Ok(None);
    }
    take_native_push_target(&app).map_err(AppError::InvalidInput)
}

#[tauri::command]
pub(crate) async fn fan_push_open_settings(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<FanPushStatus, AppError> {
    if !state.native_push_available || !cfg!(target_os = "android") {
        return Ok(current_native_push_status(
            &state,
            &app,
            Some("android_push_unavailable".to_owned()),
            None,
        )
        .await);
    }
    let _mutation = state.fan_mutation.lock().await;
    let profile = fan_profile(&state).await?;
    // Persist the user's desired state before Android backgrounds/remounts the
    // WebView. fan_push_sync will reconcile FCM/backend registration on resume.
    let _ = persist_push_state_now(&state, &profile, true, false).await?;
    open_native_push_settings(&app).map_err(AppError::InvalidInput)?;
    Ok(current_native_push_status(
        &state,
        &app,
        Some("notification_settings_opened".to_owned()),
        None,
    )
    .await)
}


#[tauri::command]
pub(crate) async fn fan_push_preferences(
    state: State<'_, AppState>,
) -> Result<FanPushPreferences, AppError> {
    let profile = fan_profile(&state).await?;
    state.api.fan_push_preferences(&profile).await
}

#[tauri::command]
pub(crate) async fn fan_push_update_preferences(
    state: State<'_, AppState>,
    preferences: FanPushPreferencesUpdate,
) -> Result<FanPushPreferences, AppError> {
    let _mutation = state.fan_mutation.lock().await;
    let profile = fan_profile(&state).await?;
    state.api.fan_update_push_preferences(&profile, &preferences).await
}

/// Stores the fan's city and nearby-show preference.
///
/// Serialized behind `fan_mutation` with the other fan writes, and the reply is
/// the server's own view rather than what the app asked for: the answer to
/// "will this reach me" is the server's to give.
#[tauri::command]
pub(crate) async fn fan_set_location(
    state: State<'_, AppState>,
    city_slug: String,
    nearby_gigs_enabled: bool,
    radius_km: u16,
) -> Result<FanLocationState, AppError> {
    let _mutation = state.fan_mutation.lock().await;
    let profile = fan_profile(&state).await?;
    state
        .api
        .fan_set_location(&profile, city_slug.trim(), nearby_gigs_enabled, radius_km)
        .await
}
