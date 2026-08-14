fn valid_native_push_installation_id(value: &str) -> bool {
    let Some(raw) = value.strip_prefix("android-") else {
        return false;
    };
    uuid::Uuid::parse_str(raw).is_ok() && value.len() <= 160
}

fn read_native_push_installation_id(app_data_dir: &Path) -> Option<String> {
    let value = std::fs::read_to_string(app_data_dir.join(NATIVE_PUSH_INSTALLATION_FILE)).ok()?;
    let value = value.trim();
    valid_native_push_installation_id(value).then(|| value.to_owned())
}

fn ensure_native_push_installation_id(app_data_dir: &Path) -> Result<String, AppError> {
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
fn native_push_permission(app: &AppHandle) -> Result<String, String> {
    crate::push_plugin::permission(app)
}

#[cfg(not(target_os = "android"))]
fn native_push_permission(_app: &AppHandle) -> Result<String, String> {
    Err("android_push_unavailable".to_owned())
}

#[cfg(target_os = "android")]
fn request_native_push_permission(app: &AppHandle) -> Result<String, String> {
    crate::push_plugin::request_permission(app)
}

#[cfg(not(target_os = "android"))]
fn request_native_push_permission(_app: &AppHandle) -> Result<String, String> {
    Err("android_push_unavailable".to_owned())
}

#[cfg(target_os = "android")]
fn native_push_token(app: &AppHandle) -> Result<String, String> {
    crate::push_plugin::token(app)
}

#[cfg(not(target_os = "android"))]
fn native_push_token(_app: &AppHandle) -> Result<String, String> {
    Err("android_push_unavailable".to_owned())
}

#[cfg(target_os = "android")]
fn delete_native_push_token(app: &AppHandle) -> Result<(), String> {
    crate::push_plugin::delete_token(app)
}

#[cfg(not(target_os = "android"))]
fn delete_native_push_token(_app: &AppHandle) -> Result<(), String> {
    Ok(())
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

async fn current_native_push_status(
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
            state
                .api
                .fan_disable_android_push(&profile, &installation_id)
                .await?;
        }
        delete_native_push_token(app).map_err(AppError::InvalidInput)?;
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
pub(crate) async fn fan_push_status(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<FanPushStatus, AppError> {
    Ok(current_native_push_status(&state, &app, None).await)
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
    let token_delete_error = delete_native_push_token(&app).err();
    let remote_error = match read_native_push_installation_id(&state.app_data_dir) {
        Some(installation_id) => state
            .api
            .fan_disable_android_push(&profile, &installation_id)
            .await
            .err()
            .map(|error| error.to_string()),
        None => None,
    };
    if token_delete_error.is_none() && remote_error.is_none() {
        persist_push_state(&state, &profile, false, true).await?;
    }
    let detail = token_delete_error
        .map(|error| format!("token_delete_unconfirmed:{error}"))
        .or_else(|| remote_error.map(|error| format!("remote_disable_unconfirmed:{error}")));
    Ok(current_native_push_status(&state, &app, detail).await)
}

