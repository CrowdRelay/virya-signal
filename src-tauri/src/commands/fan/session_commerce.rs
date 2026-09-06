#[tauri::command]
pub(crate) async fn fan_status(state: State<'_, AppState>) -> Result<FanSessionStatus, AppError> {
    let mode = crate::device_unlock::effective_mode(&state).await;
    let session = state.fan_session.read().await;
    // Both facts are read under one guard and the phase is derived from them,
    // so a status can never report a phase that disagrees with the very fields
    // it is reporting alongside it.
    let configured = vault::fan_exists(&state.app_data_dir);
    let unlocked = session.is_some();
    Ok(FanSessionStatus {
        configured,
        unlocked,
        session: session.as_ref().map(|profile| profile.as_ref().into()),
        pin_unlock: mode.pin,
        device_unlock: mode.device,
        device_unlock_supported: state.device_unlock_supported,
        phase: FanSessionPhase::resolve(configured, unlocked),
    })
}

#[tauri::command]
pub(crate) async fn fan_unlock(
    state: State<'_, AppState>,
    app: AppHandle,
    pin: String,
) -> Result<FanSessionStatus, AppError> {
    let _mutation = state.fan_mutation.lock().await;
    let epoch = state.fan_session_epoch.load(Ordering::Relaxed);
    validate_pin(&pin)?;
    let app_data_dir = state.app_data_dir.clone();
    let vault_pin = Zeroizing::new(pin);
    let pin_for_session = vault_pin.clone();
    let (profile, vault_password) = run_blocking(move || {
        let password = vault::fan_password(&app_data_dir, vault_pin.as_str())?;
        let profile = vault::load_fan_with_password(&app_data_dir, password.as_ref())?;
        Ok((profile, password))
    })
    .await?;
    if epoch != state.fan_session_epoch.load(Ordering::Relaxed) {
        drop(_mutation);
        return Err(AppError::Locked);
    }
    *state.fan_session.write().await = Some(Arc::new(profile));
    *state.fan_pin.write().await = Some(pin_for_session);
    *state.fan_vault_password.write().await = Some(vault_password);
    *state.pending_fan_confirmation.lock().await = None;
    state.wallet_qr_tokens.write().await.clear();
    drop(_mutation);
    let status = fan_status(state.clone()).await?;

    // The encrypted profile is already unlocked and safe to render. Push
    // reconciliation can perform several network requests, so it must not
    // hold the login screen hostage. The background task is intentionally
    // session-bound so a fast logout/login cannot persist stale state.
    let session = state.fan_session.read().await.clone();
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        match sync_native_push_if_desired(&state, &app, session).await {
            // `Locked` here only means the fan logged out before reconciliation
            // finished, which is the expected outcome of a fast logout, not a
            // degraded sync.
            Ok(_) | Err(AppError::Locked) => {}
            Err(error) => {
                eprintln!("[virya:push-sync] unlock reconciliation degraded: {error}");
            }
        }
    });
    Ok(status)
}

#[tauri::command]
pub(crate) async fn fan_lock(state: State<'_, AppState>) -> Result<FanSessionStatus, AppError> {
    // Locking must serialize with `fan_mutation` so that an in-flight
    // `fan_unlock` (Argon2 + vault load) or `fan_wallets` (QR token write)
    // cannot write the session back after we clear it. The push
    // reconciliation spawned by unlock runs AFTER `fan_mutation` is dropped,
    // so it does not hold this lock; the only long holder is
    // `fan_delete_account` (server round-trip), which is an edge case where
    // waiting is acceptable.
    let _mutation = state.fan_mutation.lock().await;
    *state.fan_session.write().await = None;
    *state.fan_pin.write().await = None;
    *state.fan_vault_password.write().await = None;
    *state.pending_fan_confirmation.lock().await = None;
    state.wallet_qr_tokens.write().await.clear();
    *state.fan_sections_cache.write().await = None;
    state.fan_session_epoch.fetch_add(1, Ordering::Relaxed);
    drop(_mutation);
    fan_status(state).await
}

/// Retires the two staged capabilities that belong to the fan identity itself.
///
/// `fan_lock` deliberately keeps both: locking is temporary, and a mailed
/// confirmation link or a Synesthesia handoff that arrived while the vault was
/// closed is still the same person's to spend once they unlock. Forgetting and
/// deleting are not temporary — leaving either behind would hand a one-time
/// credential for a destroyed account to whoever sets this device up next.
async fn clear_fan_identity_capabilities(state: &State<'_, AppState>) {
    *state.pending_fan_confirm_token.lock().await = None;
    *state.pending_synesthesia_handoff.lock().await = None;
}

#[tauri::command]
pub(crate) async fn fan_delete_account(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<FanSessionStatus, AppError> {
    let _mutation = state.fan_mutation.lock().await;
    let profile = state
        .fan_session
        .read()
        .await
        .clone()
        .ok_or(AppError::Locked)?;

    // Remove the local vault first. If the server delete succeeds but the
    // vault removal fails (e.g. disk error), the user would be locked out
    // of a vault that can never sync again — the worst outcome. By removing
    // the vault first, a server-delete failure leaves an orphaned server
    // account (which support can clean up) but the device is in a clean
    // state. The profile is already in memory, so the server call still
    // has the session token it needs.
    let app_data_dir = state.app_data_dir.clone();
    run_blocking(move || vault::remove_fan(&app_data_dir)).await?;
    forget_device_unlock(&state, &app).await;

    // Clear all in-memory session material.
    *state.fan_session.write().await = None;
    *state.fan_pin.write().await = None;
    *state.fan_vault_password.write().await = None;
    *state.pending_fan_confirmation.lock().await = None;
    clear_fan_identity_capabilities(&state).await;
    state.wallet_qr_tokens.write().await.clear();
    *state.fan_sections_cache.write().await = None;
    state.fan_session_epoch.fetch_add(1, Ordering::Relaxed);

    // The device FCM token is shared with Staff mode, so the server removes
    // only the fan endpoint association. A failure here leaves an orphaned
    // server account but the device is already clean.
    if let Err(error) = state.api.fan_delete_account(&profile).await {
        eprintln!("[virya:fan] server-side delete failed after vault removal: {error}");
    }
    drop(_mutation);
    fan_status(state).await
}

#[tauri::command]
pub(crate) async fn fan_forget(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<FanSessionStatus, AppError> {
    let _mutation = state.fan_mutation.lock().await;
    let profile = state
        .fan_session
        .read()
        .await
        .clone()
        .ok_or(AppError::Locked)?;
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
    // Do not delete the device-scoped FCM token here: an unlocked staff
    // profile may intentionally use the same installation for reminders.
    // Requiring an unlocked fan session guarantees we still have the bearer
    // needed to confirm remote audience cleanup before the vault is removed.
    *state.fan_session.write().await = None;
    *state.fan_pin.write().await = None;
    *state.fan_vault_password.write().await = None;
    *state.pending_fan_confirmation.lock().await = None;
    clear_fan_identity_capabilities(&state).await;
    state.wallet_qr_tokens.write().await.clear();
    *state.fan_sections_cache.write().await = None;
    state.fan_session_epoch.fetch_add(1, Ordering::Relaxed);
    let app_data_dir = state.app_data_dir.clone();
    run_blocking(move || vault::remove_fan(&app_data_dir)).await?;
    // The seal outlives the snapshot it opens unless it is dropped with it. A
    // leftover keystore key and unlock record would then be offered to the next
    // fan on this device as a way into a vault that is gone.
    forget_device_unlock(&state, &app).await;
    drop(_mutation);
    fan_status(state).await
}

/// What will open this vault after it is written.
///
/// A confirmation can arrive with a PIN the fan chose, or with nothing at all
/// when they took the mailed link and the device can seal a password for them.
/// Both end in the same 32 bytes handed to the snapshot layer; only the source
/// differs, so this is the one place that has to know which it was.
pub(crate) enum FanCredential {
    Pin(Zeroizing<String>),
    Device,
}

async fn persist_confirmed_fan(
    state: &AppState,
    app: &AppHandle,
    input: FanConfirmationInput,
    credential: FanCredential,
) -> Result<(), AppError> {
    let epoch = state.fan_session_epoch.load(Ordering::Relaxed);
    let (_result, session_token, canonical_email, canonical_name) =
        state.api.fan_confirm(&input).await?;
    let profile = FanProfile {
        api_base_url: input.api_base_url,
        area_wallet_id: uuid::Uuid::new_v4().to_string(),
        email: canonical_email,
        display_name: canonical_name,
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
    let (vault_password, session_pin) = match credential {
        FanCredential::Pin(pin) => {
            let vault_pin = pin.clone();
            // replace_fan already derived this password to encrypt the vault.
            // Argon2 dominates the confirm step, so deriving it a second time
            // from the same pin and salt doubled the wait before the app could
            // show anything.
            let password = run_blocking(move || {
                vault::replace_fan(&app_data_dir, vault_pin.as_str(), &stored_profile)
            })
            .await?;
            crate::device_unlock::write_mode(
                state,
                crate::device_unlock::UnlockMode {
                    pin: true,
                    device: false,
                },
            )
            .await?;
            // A PIN confirmation replaces whatever opened this vault before it,
            // so a seal left from an earlier account would open a snapshot that
            // no longer exists.
            let _ = crate::device_unlock::forget(app, &state.app_data_dir);
            (password, Some(pin))
        }
        FanCredential::Device => {
            let password = vault::random_vault_password()?;
            let sealing = password.clone();
            let write_dir = app_data_dir.clone();
            run_blocking(move || {
                vault::replace_fan_with_password(&write_dir, sealing.as_ref(), &stored_profile)
            })
            .await?;
            // Sealing after the vault is written: a seal for a snapshot that
            // failed to land is a key to nothing, and the vault write is the
            // step that can still fail on a full disk. A keystore that refuses
            // the real key takes the snapshot with it rather than leaving one
            // nobody can open.
            seal_or_discard_fan_vault(state, app, password.as_ref()).await?;
            crate::device_unlock::write_mode(
                state,
                crate::device_unlock::UnlockMode {
                    pin: false,
                    device: true,
                },
            )
            .await?;
            (password, None)
        }
    };
    if epoch != state.fan_session_epoch.load(Ordering::Relaxed) {
        return Err(AppError::Locked);
    }
    *state.fan_session.write().await = Some(Arc::new(profile));
    *state.fan_pin.write().await = session_pin;
    *state.fan_vault_password.write().await = Some(vault_password);
    state.wallet_qr_tokens.write().await.clear();
    // A different fan now owns this vault; the previous fan's panels must not
    // paint under the new session.
    *state.fan_sections_cache.write().await = None;
    Ok(())
}

#[tauri::command]
pub(crate) async fn fan_prepare_confirmation(
    state: State<'_, AppState>,
    mut api_base_url: String,
    pin: String,
) -> Result<(), AppError> {
    let _mutation = state.fan_mutation.lock().await;
    api_base_url = api_base_url.trim().to_owned();
    validate_api_base(&api_base_url)?;

    if pin.is_empty() {
        // A repeat prepare for the same endpoint keeps whatever credential the
        // first one recorded. The borrow ends here so the write below is not
        // taken against a lock this scope still holds.
        {
            let pending = state.pending_fan_confirmation.lock().await;
            if pending
                .as_ref()
                .is_some_and(|pending| pending.api_base_url == api_base_url)
            {
                return Ok(());
            }
        }
        // Otherwise an empty PIN means the fan asked this device to hold the
        // password. Without a keystore there is nothing to hold it with.
        if !state.device_unlock_supported {
            return Err(AppError::InvalidPin);
        }
        *state.pending_fan_confirmation.lock().await = Some(PendingFanConfirmation {
            api_base_url,
            pin: None,
        });
        return Ok(());
    }

    validate_pin(&pin)?;
    *state.pending_fan_confirmation.lock().await = Some(PendingFanConfirmation {
        api_base_url,
        pin: Some(Zeroizing::new(pin)),
    });
    Ok(())
}

#[tauri::command]
pub(crate) async fn fan_clear_pending_confirmation(
    state: State<'_, AppState>,
) -> Result<(), AppError> {
    let _mutation = state.fan_mutation.lock().await;
    *state.pending_fan_confirmation.lock().await = None;
    Ok(())
}

/// Retires a pending mailed confirmation link across the whole boundary.
///
/// There are two holders — the Rust token slot and the native Android app-link
/// holder — and `Ok(())` from this command is the app telling the fan that
/// neither of them can produce the link again. That claim has to be earned:
/// the native holder is cleared *first*, and a refusal returns the error
/// instead of being swallowed. Clearing the Rust slot on a native failure would
/// be the worst of both, because the next resume tick re-reads the native
/// holder and the confirm panel reappears with no explanation.
///
/// Leaving both slots populated on failure is the honest outcome: the link is
/// still pending, the panel is still the truth, and the fan can try again.
#[tauri::command]
pub(crate) async fn fan_clear_pending_confirm_link(
    state: State<'_, AppState>,
    _app: AppHandle,
) -> Result<(), AppError> {
    #[cfg(target_os = "android")]
    crate::push_plugin::clear_fan_confirm_app_link(&_app).map_err(AppError::InvalidInput)?;
    *state.pending_fan_confirm_token.lock().await = None;
    Ok(())
}

#[tauri::command]
pub(crate) async fn fan_confirm(
    state: State<'_, AppState>,
    app: AppHandle,
    mut input: FanConfirmationInput,
    pin: String,
) -> Result<FanSessionStatus, AppError> {
    let _mutation = state.fan_mutation.lock().await;
    validate_fan_confirmation(&mut input, &pin)?;
    persist_confirmed_fan(&state, &app, input, FanCredential::Pin(Zeroizing::new(pin))).await?;
    *state.pending_fan_confirmation.lock().await = None;
    drop(_mutation);
    fan_status(state).await
}

/// Picks up a confirmation link Android delivered to the app instead of the
/// browser, and reports whether one is now waiting.
///
/// The token stays native: the WebView only learns that a mailed capability is
/// pending, the same way a scanned QR never reaches it. `fan_confirm_link`
/// spends it once the fan has entered the PIN.
#[tauri::command]
pub(crate) async fn fan_take_confirm_link(
    state: State<'_, AppState>,
    _app: AppHandle,
) -> Result<bool, AppError> {
    #[cfg(target_os = "android")]
    if let Some(token) =
        crate::push_plugin::take_fan_confirm_app_link(&_app).map_err(AppError::InvalidInput)?
    {
        *state.pending_fan_confirm_token.lock().await = Some(Zeroizing::new(token));
    }
    Ok(state.pending_fan_confirm_token.lock().await.is_some())
}

/// Spends the token a confirmation link delivered, with the PIN the fan just
/// chose. Mirrors `fan_confirm_scanned`, except the capability came from the
/// mailed link rather than the camera.
#[tauri::command]
pub(crate) async fn fan_confirm_link(
    state: State<'_, AppState>,
    app: AppHandle,
    mut api_base_url: String,
    pin: Option<String>,
) -> Result<FanSessionStatus, AppError> {
    let _mutation = state.fan_mutation.lock().await;
    if state.fan_session.read().await.is_some() {
        drop(_mutation);
        return fan_status(state).await;
    }
    let token = state
        .pending_fan_confirm_token
        .lock()
        .await
        .clone()
        .ok_or(AppError::NotConfigured)?;
    // A confirmation started on this device already knows where it is talking
    // to; a link opened on a fresh install carries the shell's own base URL.
    if let Some(pending) = state.pending_fan_confirmation.lock().await.as_ref() {
        api_base_url = pending.api_base_url.clone();
    }
    let mut input = FanConfirmationInput {
        api_base_url,
        email: String::new(),
        display_name: None,
        token: token.to_string(),
    };
    // No PIN means the fan took the link and the device seals the password for
    // them. That is only offered where a keystore can actually hold it — the
    // alternative would be a vault password sitting on disk in the clear, which
    // is worse than any prompt.
    let credential = match pin {
        Some(pin) => {
            validate_fan_confirmation(&mut input, &pin)?;
            FanCredential::Pin(Zeroizing::new(pin))
        }
        None => {
            if !state.device_unlock_supported {
                return Err(AppError::InvalidPin);
            }
            validate_fan_confirmation_token_only(&mut input)?;
            FanCredential::Device
        }
    };
    // A one-time token that the server has already consumed (409) or expired
    // (404) can never succeed on a retry. Clearing it here prevents the next
    // resume tick from re-offering the same spent panel — the "never re-open a
    // dismissed action" invariant, applied to the terminal-failure path that
    // the success path already handled.
    if let Err(error) = persist_confirmed_fan(&state, &app, input, credential).await {
        if matches!(error, AppError::Conflict(_) | AppError::NotFound) {
            *state.pending_fan_confirm_token.lock().await = None;
        }
        return Err(error);
    }
    *state.pending_fan_confirm_token.lock().await = None;
    *state.pending_fan_confirmation.lock().await = None;
    drop(_mutation);
    fan_status(state).await
}

#[tauri::command]
pub(crate) async fn fan_confirm_scanned(
    state: State<'_, AppState>,
    app: AppHandle,
    token: String,
) -> Result<FanSessionStatus, AppError> {
    let _mutation = state.fan_mutation.lock().await;

    if state.fan_session.read().await.is_some() {
        drop(_mutation);
        return fan_status(state).await;
    }

    let (api_base_url, pin) = {
        let pending = state.pending_fan_confirmation.lock().await;
        let pending = pending.as_ref().ok_or(AppError::InvalidPin)?;
        (pending.api_base_url.clone(), pending.pin.clone())
    };

    let mut input = FanConfirmationInput {
        api_base_url,
        email: String::new(),
        display_name: None,
        token,
    };
    // The credential was chosen when the scan was prepared, so the camera path
    // does not ask again — it spends whatever the fan already picked.
    let credential = match pin {
        Some(pin) => {
            validate_fan_confirmation(&mut input, pin.as_str())?;
            FanCredential::Pin(pin)
        }
        None => {
            validate_fan_confirmation_token_only(&mut input)?;
            FanCredential::Device
        }
    };
    persist_confirmed_fan(&state, &app, input, credential).await?;
    *state.pending_fan_confirmation.lock().await = None;
    drop(_mutation);
    fan_status(state).await
}

#[tauri::command]
pub(crate) async fn fan_request_access(
    state: State<'_, AppState>,
    mut api_base_url: String,
    mut email: String,
    mut locale: String,
) -> Result<serde_json::Value, AppError> {
    api_base_url = api_base_url.trim().to_owned();
    email = email.trim().to_ascii_lowercase();
    locale = locale.trim().to_owned();
    crate::validation::validate_api_base(&api_base_url)?;
    if !crate::validation::valid_email(&email)
        || locale.is_empty()
        || locale.len() > 16
        || !locale
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_enter_valid_email").into(),
        ));
    }
    state
        .api
        .fan_request_access(&api_base_url, &email, &locale)
        .await
}

#[tauri::command]
pub(crate) async fn fan_area_wallet(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<crate::models::AreaWallet, AppError> {
    let profile = fan_profile(&state).await?;
    let wallet = state.api.fan_area_wallet(&profile).await?;
    remember_fan_sections(app, {
        let wallet = wallet.clone();
        move |snapshot| snapshot.area = Some(wallet)
    });
    Ok(wallet)
}

#[tauri::command]
pub(crate) async fn fan_area_challenge(
    state: State<'_, AppState>,
    drop_id: String,
) -> Result<AreaChallenge, AppError> {
    let profile = fan_profile(&state).await?;
    state.api.fan_area_challenge(&profile, &drop_id).await
}

#[tauri::command]
pub(crate) async fn fan_area_claim(
    state: State<'_, AppState>,
    drop_id: String,
    challenge: String,
    samples: Vec<AreaPositionSample>,
) -> Result<AreaClaimResult, AppError> {
    let challenge = challenge.trim();
    if !(40..=2048).contains(&challenge.len())
        || samples.len() < 3
        || samples.len() > 8
        || samples.iter().any(|sample| {
            !sample.lat.is_finite()
                || !(-90.0..=90.0).contains(&sample.lat)
                || !sample.lng.is_finite()
                || !(-180.0..=180.0).contains(&sample.lng)
                || !sample.accuracy.is_finite()
                || !(0.0..=10_000.0).contains(&sample.accuracy)
                || sample.captured_at == 0
        })
    {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_area_claim_invalid").into(),
        ));
    }
    let profile = fan_profile(&state).await?;
    state
        .api
        .fan_area_claim(&profile, &drop_id, challenge, &samples)
        .await
}

#[tauri::command]
pub(crate) async fn fan_unpublish_synesthesia_leaderboard(
    state: State<'_, AppState>,
) -> Result<bool, AppError> {
    let _mutation = state.fan_mutation.lock().await;
    let profile = fan_profile(&state).await?;
    state.api.fan_unpublish_synesthesia_leaderboard(&profile).await
}

#[tauri::command]
pub(crate) async fn fan_cached_home(
    state: State<'_, AppState>,
) -> Result<Option<FanHomeData>, AppError> {
    let profile = fan_profile(&state).await?;
    let password = state
        .fan_vault_password
        .read()
        .await
        .as_ref()
        .map(|value| Zeroizing::new(value.to_vec()))
        .ok_or(AppError::Locked)?;
    let app_data_dir = state.app_data_dir.clone();
    let snapshot = match run_blocking(move || {
        vault::load_fan_home_cache_with_password(&app_data_dir, password.as_ref())
    })
    .await
    {
        Ok(snapshot) => snapshot,
        Err(error) => {
            eprintln!("[virya:fan-cache] encrypted home snapshot ignored: {error}");
            None
        }
    };
    let Some(snapshot) = snapshot else { return Ok(None); };
    let home = state
        .api
        .seed_fan_home_snapshot(&profile, snapshot.home, snapshot.stored_at_unix_secs)
        .await;
    if home.is_some() {
        state.api.report_signal_startup_ready_once(&profile.api_base_url, true);
    }
    Ok(home)
}

#[tauri::command]
pub(crate) async fn fan_home(state: State<'_, AppState>) -> Result<FanHomeData, AppError> {
    let profile = fan_profile(&state).await?;
    let home = state.api.fan_home(&profile).await?;
    if !home.stale && home.has_supported_schema() {
        state.api.report_signal_startup_ready_once(&profile.api_base_url, false);
        if let Some(password) = state.fan_vault_password.read().await.as_ref() {
            let snapshot = vault::FanHomeCacheSnapshot {
                stored_at_unix_secs: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map(|value| value.as_secs())
                    .unwrap_or(0),
                home: home.clone(),
            };
            let password = Zeroizing::new(password.to_vec());
            let app_data_dir = state.app_data_dir.clone();
            // The snapshot only has to be on disk before the next cold start,
            // not before Home paints. Encrypting and writing it inline added a
            // blocking file write to every refresh the fan waited on.
            tauri::async_runtime::spawn(async move {
                if let Err(error) = run_blocking(move || {
                    vault::save_fan_home_cache_with_password(
                        &app_data_dir,
                        password.as_ref(),
                        &snapshot,
                    )
                })
                .await
                {
                    eprintln!("[virya:fan-cache] encrypted home snapshot save degraded: {error}");
                }
            });
        }
    }
    Ok(home)
}

