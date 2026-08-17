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


async fn persist_push_state(
    state: &State<'_, AppState>,
    profile: &FanProfile,
    desired: bool,
    sync_ok: bool,
) -> Result<Arc<FanProfile>, AppError> {
    let mut updated = profile.clone();
    updated.push_enabled = desired;
    updated.push_last_sync_ok = sync_ok;
    persist_fan(state, &updated).await?;
    let updated = Arc::new(updated);
    *state.fan_session.write().await = Some(updated.clone());
    Ok(updated)
}

pub(crate) async fn current_native_push_status(
    state: &State<'_, AppState>,
    app: &AppHandle,
    detail: Option<String>,
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
    let config = state.api.fan_push_config(&profile).await;
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
    let profile = fan_profile(state).await?;
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
        persist_push_state(state, &profile, false, true).await?;
        return Ok(());
    }
    let config = state.api.fan_push_config(&profile).await?;
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
        .fan_register_android_push(&profile, &installation_id, &token)
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
pub(crate) async fn fan_push_sync(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<FanPushStatus, AppError> {
    let _mutation = state.fan_mutation.lock().await;
    match sync_native_push_if_desired(&state, &app).await {
        Ok(()) => Ok(current_native_push_status(&state, &app, None).await),
        Err(AppError::Forbidden) => Ok(current_native_push_status(
            &state,
            &app,
            Some("notification_permission_denied".to_owned()),
        )
        .await),
        Err(AppError::Conflict(detail)) => {
            Ok(current_native_push_status(&state, &app, Some(detail)).await)
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
        )
        .await);
    }
    let config = state.api.fan_push_config(&profile).await?;
    if !config.enabled || !config.android_fcm {
        return Ok(current_native_push_status(
            &state,
            &app,
            Some("push_delivery_not_live".to_owned()),
        )
        .await);
    }
    let permission = request_native_push_permission(&app).map_err(AppError::InvalidInput)?;
    if permission != "granted" {
        return Ok(current_native_push_status(
            &state,
            &app,
            Some("notification_permission_denied".to_owned()),
        )
        .await);
    }
    // Persist desired intent before the remote write. If the network response is
    // lost, the next unlock retries registration with the current FCM token.
    let profile = persist_push_state(&state, &profile, true, false).await?;
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
    persist_push_state(&state, &profile, true, true).await?;
    Ok(current_native_push_status(&state, &app, None).await)
}

#[tauri::command]
pub(crate) async fn fan_push_disable(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<FanPushStatus, AppError> {
    let _mutation = state.fan_mutation.lock().await;
    let profile = fan_profile(&state).await?;
    let profile = persist_push_state(&state, &profile, false, false).await?;
    let detail = match read_native_push_installation_id(&state.app_data_dir) {
        Some(installation_id) => match state
            .api
            .fan_disable_android_push(&profile, &installation_id)
            .await
        {
            Ok(response) if !response.registered => {
                persist_push_state(&state, &profile, false, true).await?;
                None
            }
            Ok(_) => Some("fan_push_disable_not_confirmed".to_owned()),
            Err(error) => Some(format!("remote_disable_unconfirmed:{error}")),
        },
        None => {
            persist_push_state(&state, &profile, false, true).await?;
            None
        }
    };
    Ok(current_native_push_status(&state, &app, detail).await)
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
        )
        .await);
    }
    let _mutation = state.fan_mutation.lock().await;
    let profile = fan_profile(&state).await?;
    // Persist the user's desired state before Android backgrounds/remounts the
    // WebView. fan_push_sync will reconcile FCM/backend registration on resume.
    let _ = persist_push_state(&state, &profile, true, false).await?;
    open_native_push_settings(&app).map_err(AppError::InvalidInput)?;
    Ok(current_native_push_status(
        &state,
        &app,
        Some("notification_settings_opened".to_owned()),
    )
    .await)
}
