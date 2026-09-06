// Creating a fan profile from scratch.
//
// Split out of `session_commerce.rs` to keep that file under the module limit.
// It shares `FanCredential` and the vault helpers with the confirmation paths,
// which is why it stays an include rather than becoming a module of its own.
//
// Included into `commands/fan.rs`, so an inner doc comment is not available.

#[tauri::command]
pub(crate) async fn fan_signup(
    state: State<'_, AppState>,
    app: AppHandle,
    mut input: FanSignupInput,
    pin: Option<String>,
) -> Result<FanAuthResult, AppError> {
    let _mutation = state.fan_mutation.lock().await;
    // Reject signup if a fan vault already exists. The UI gates signup to
    // `!configured`, but a direct command call could otherwise overwrite the
    // profile while leaving stale `FAN_HOME_CACHE_KEY` /
    // `FAN_SECTIONS_CACHE_KEY` entries from the previous account, leaking
    // cross-account data through the cache.
    if vault::fan_exists(&state.app_data_dir) {
        return Err(AppError::Conflict(
            crate::i18n::tr("native_error_already_configured").into(),
        ));
    }
    validate_fan_signup(&mut input, pin.as_deref())?;
    // Signing up with no PIN is the same offer the mailed link makes, and it is
    // only available where a keystore can actually hold the password.
    if pin.is_none() && !state.device_unlock_supported {
        return Err(AppError::InvalidPin);
    }
    let pin = pin.map(Zeroizing::new);
    let (result, session_token) = state.api.fan_signup(&input).await?;
    if let Some(session_token) = session_token {
        let profile = FanProfile {
            api_base_url: input.api_base_url,
            area_wallet_id: uuid::Uuid::new_v4().to_string(),
            email: input.email,
            display_name: input.display_name,
            fan_session_token: session_token,
            push_enabled: false,
            push_last_sync_ok: false,
            pass_session_token: None,
            wallets: Vec::new(),
            cached_wallets: Vec::new(),
            cached_wallet_qr: Vec::new(),
        };
        let app_data_dir = state.app_data_dir.clone();
        let stored_profile = profile.clone();
        let vault_password = match pin.as_ref() {
            Some(pin) => {
                let vault_pin = pin.clone();
                let password = run_blocking(move || {
                    vault::save_fan(&app_data_dir, vault_pin.as_str(), &stored_profile)
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
                password
            }
            None => {
                let password = vault::random_vault_password()?;
                let sealing = password.clone();
                run_blocking(move || {
                    vault::replace_fan_with_password(
                        &app_data_dir,
                        sealing.as_ref(),
                        &stored_profile,
                    )
                })
                .await?;
                seal_or_discard_fan_vault(&state, &app, password.as_ref()).await?;
                crate::device_unlock::write_mode(
                    &state,
                    crate::device_unlock::UnlockMode {
                        pin: false,
                        device: true,
                    },
                )
            .await?;
                password
            }
        };
        *state.fan_session.write().await = Some(Arc::new(profile));
        *state.fan_pin.write().await = pin;
        *state.fan_vault_password.write().await = Some(vault_password);
        *state.pending_fan_confirmation.lock().await = None;
        state.wallet_qr_tokens.write().await.clear();
    } else {
        *state.pending_fan_confirmation.lock().await = Some(PendingFanConfirmation {
            api_base_url: input.api_base_url.clone(),
            pin,
        });
    }
    Ok(result)
}

