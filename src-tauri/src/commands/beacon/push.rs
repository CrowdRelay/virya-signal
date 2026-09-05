// Beacon push lifecycle: desired-state persistence, status reporting and the
// register/disable reconciliation against CrowdRelay.
//
// Split out of `commands/beacon.rs` for the same reason as the fan equivalent:
// the session/member surface and the notification state machine are separate
// concerns, and keeping them in one file was what pushed that module past the
// size contract. Included rather than a module of its own so it keeps sharing
// the imports, helpers and `AppState` handles the session commands use.

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
    cached_config: Option<&FanPushConfigApi>,
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
    // Reuse the config a sync/enable already fetched instead of a second
    // round-trip on cellular. Only fetch when the caller had no reason to.
    let (backend_enabled, config_detail) = match cached_config {
        Some(config) => (config.enabled && config.android_fcm, None),
        None => match state.api.beacon_push_config(&profile).await {
            Ok(config) => (config.enabled && config.android_fcm, None),
            Err(error) => (false, Some(format!("push_config_unavailable:{error}"))),
        },
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
) -> Result<Option<FanPushConfigApi>, AppError> {
    let profile = beacon_profile(state).await?;
    if !state.native_push_available || !cfg!(target_os = "android") {
        return Ok(None);
    }
    if !profile.push_enabled {
        if profile.push_last_sync_ok {
            return Ok(None);
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
        return Ok(None);
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
    Ok(Some(config))
}

#[tauri::command]
pub(crate) async fn beacon_push_sync(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<FanPushStatus, AppError> {
    let _mutation = state.beacon_mutation.lock().await;
    match sync_native_push_if_desired(&state, &app).await {
        Ok(config) => Ok(current_push_status(&state, &app, None, config.as_ref()).await),
        Err(AppError::Forbidden) => Ok(current_push_status(
            &state,
            &app,
            Some("notification_permission_denied".to_owned()),
            None,
        )
        .await),
        Err(AppError::Conflict(detail)) => {
            Ok(current_push_status(&state, &app, Some(detail), None).await)
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
        return Ok(current_push_status(
            &state,
            &app,
            Some("android_push_unavailable".to_owned()),
            None,
        )
        .await);
    }
    let config = state.api.beacon_push_config(&profile).await?;
    if !config.enabled || !config.android_fcm {
        return Ok(current_push_status(
            &state,
            &app,
            Some("push_delivery_not_live".to_owned()),
            Some(&config),
        )
        .await);
    }
    let permission = request_native_push_permission(&app).map_err(AppError::InvalidInput)?;
    if permission != "granted" {
        return Ok(current_push_status(
            &state,
            &app,
            Some("notification_permission_denied".to_owned()),
            Some(&config),
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
    Ok(current_push_status(&state, &app, None, Some(&config)).await)
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
    Ok(current_push_status(&state, &app, detail, None).await)
}

#[tauri::command]
pub(crate) async fn beacon_push_open_settings(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<FanPushStatus, AppError> {
    if !state.native_push_available || !cfg!(target_os = "android") {
        return Ok(current_push_status(
            &state,
            &app,
            Some("android_push_unavailable".to_owned()),
            None,
        )
        .await);
    }
    let _mutation = state.beacon_mutation.lock().await;
    let profile = beacon_profile(&state).await?;
    let _ = persist_push_state(&state, &profile, true, false).await?;
    open_native_push_settings(&app).map_err(AppError::InvalidInput)?;
    Ok(current_push_status(
        &state,
        &app,
        Some("notification_settings_opened".to_owned()),
        None,
    )
    .await)
}
