// Entry that does not go through a PIN.
//
// These three commands are the whole surface of device unlock: open a vault
// the keystore is holding a password for, start holding one, and stop. The
// vault format and every other fan command are the same either way — only the
// source of the 32 bytes differs, which is why this is a file of its own
// rather than a branch threaded through the session commands.
//
// Included into `commands/fan.rs`, so an inner doc comment is not available
// here: the tokens land inside that module rather than starting a new one.

/// Opens the vault with the password the keystore is holding.
///
/// Mirrors `fan_unlock` exactly, except that nothing is asked of the fan: the
/// keystore returns the password, the snapshot opens with it, and the session
/// is live. A missing or unopenable seal is not an error the fan can act on —
/// it means this device can no longer let them in without their PIN — so it
/// reports `Locked` and the gate falls back to the PIN prompt.
#[tauri::command]
pub(crate) async fn fan_device_unlock(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<FanSessionStatus, AppError> {
    let _mutation = state.fan_mutation.lock().await;
    if state.fan_session.read().await.is_some() {
        drop(_mutation);
        return fan_status(state).await;
    }
    if !crate::device_unlock::effective_mode(&state).await.device {
        return Err(AppError::Locked);
    }
    let password = match crate::device_unlock::open(&app, &state.app_data_dir) {
        Ok(password) => password,
        Err(error) => {
            // The keystore refused, so this seal is not a way in any more —
            // a restore onto another device, a cleared credential store. Drop
            // the cache so the gate stops offering it and falls back to the
            // PIN prompt for the rest of this process.
            crate::device_unlock::invalidate_cache(&state).await;
            return Err(error);
        }
    };
    let app_data_dir = state.app_data_dir.clone();
    let loading = password.clone();
    let profile =
        run_blocking(move || vault::load_fan_with_password(&app_data_dir, loading.as_ref()))
            .await?;
    *state.fan_session.write().await = Some(Arc::new(profile));
    *state.fan_pin.write().await = None;
    *state.fan_vault_password.write().await = Some(password);
    *state.pending_fan_confirmation.lock().await = None;
    state.wallet_qr_tokens.write().await.clear();
    drop(_mutation);
    fan_status(state).await
}

/// Seals the current session's vault password so the next launch needs no PIN.
///
/// The password is already in memory from the unlock that produced this
/// session, so there is nothing to derive and nothing to ask for. The PIN keeps
/// working: this adds a second way in, it does not replace the first.
#[tauri::command]
pub(crate) async fn fan_enable_device_unlock(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<FanSessionStatus, AppError> {
    let _mutation = state.fan_mutation.lock().await;
    if !state.device_unlock_supported {
        return Err(AppError::NotConfigured);
    }
    let password = state
        .fan_vault_password
        .read()
        .await
        .clone()
        .ok_or(AppError::Locked)?;
    // Whether a PIN also opens this vault is not changed by adding a seal, so
    // it is carried from the mode already in memory rather than re-read.
    let pin = crate::device_unlock::effective_mode(&state).await.pin;
    crate::device_unlock::seal(&app, &state.app_data_dir, password.as_ref())?;
    crate::device_unlock::write_mode(
        &state,
        crate::device_unlock::UnlockMode { pin, device: true },
    )
    .await?;
    drop(_mutation);
    fan_status(state).await
}

/// Turns device unlock off, leaving a PIN as the way in.
///
/// A vault created from a mailed link has no PIN behind it, so switching back
/// is a re-key rather than a flag: the profile is read with the sealed
/// password and written again under Argon2 over the new PIN. The seal is
/// dropped only after that write lands, because a cleared keystore key and a
/// vault still sealed with the old password is a device nobody can open.
#[tauri::command]
pub(crate) async fn fan_disable_device_unlock(
    state: State<'_, AppState>,
    app: AppHandle,
    pin: String,
) -> Result<FanSessionStatus, AppError> {
    let _mutation = state.fan_mutation.lock().await;
    validate_pin(&pin)?;
    let profile = state
        .fan_session
        .read()
        .await
        .clone()
        .ok_or(AppError::Locked)?;
    let app_data_dir = state.app_data_dir.clone();
    let stored_profile = profile.as_ref().clone();
    let vault_pin = Zeroizing::new(pin);
    let session_pin = vault_pin.clone();
    let password = run_blocking(move || {
        vault::replace_fan(&app_data_dir, vault_pin.as_str(), &stored_profile)
    })
    .await?;
    crate::device_unlock::write_mode(
        &state,
        crate::device_unlock::UnlockMode {
            pin: true,
            device: false,
        },
    )
            .await?;
    crate::device_unlock::forget(&app, &state.app_data_dir)?;
    *state.fan_pin.write().await = Some(session_pin);
    *state.fan_vault_password.write().await = Some(password);
    drop(_mutation);
    fan_status(state).await
}

/// Drops every trace of device unlock. Best effort on purpose: this runs while
/// a profile is being removed, and a keystore that refuses to delete a key must
/// not turn "forget me" into an error the fan cannot clear.
async fn forget_device_unlock(state: &AppState, app: &AppHandle) {
    if let Err(error) = crate::device_unlock::forget(app, &state.app_data_dir) {
        eprintln!("[virya:device-unlock] seal was not cleared: {error}");
    }
    if let Err(error) = crate::device_unlock::clear_mode(state).await {
        eprintln!("[virya:device-unlock] unlock record was not cleared: {error}");
    }
}

