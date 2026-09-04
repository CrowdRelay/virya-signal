#[tauri::command]
pub(crate) async fn fan_status(state: State<'_, AppState>) -> Result<FanSessionStatus, AppError> {
    let session = state.fan_session.read().await;
    Ok(FanSessionStatus {
        configured: vault::fan_exists(&state.app_data_dir),
        unlocked: session.is_some(),
        session: session.as_ref().map(|profile| profile.as_ref().into()),
    })
}

#[tauri::command]
pub(crate) async fn fan_unlock(
    state: State<'_, AppState>,
    app: AppHandle,
    pin: String,
) -> Result<FanSessionStatus, AppError> {
    let _mutation = state.fan_mutation.lock().await;
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
    drop(_mutation);
    fan_status(state).await
}

#[tauri::command]
pub(crate) async fn fan_delete_account(
    state: State<'_, AppState>,
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

    // Clear all in-memory session material.
    *state.fan_session.write().await = None;
    *state.fan_pin.write().await = None;
    *state.fan_vault_password.write().await = None;
    *state.pending_fan_confirmation.lock().await = None;
    state.wallet_qr_tokens.write().await.clear();
    *state.fan_sections_cache.write().await = None;

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
    state.wallet_qr_tokens.write().await.clear();
    *state.fan_sections_cache.write().await = None;
    let app_data_dir = state.app_data_dir.clone();
    run_blocking(move || vault::remove_fan(&app_data_dir)).await?;
    drop(_mutation);
    fan_status(state).await
}

async fn persist_confirmed_fan(
    state: &AppState,
    input: FanConfirmationInput,
    pin: Zeroizing<String>,
) -> Result<(), AppError> {
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
    let vault_pin = pin.clone();
    // replace_fan already derived this password to encrypt the vault. Argon2
    // dominates the confirm step, so deriving it a second time from the same
    // pin and salt doubled the wait before the app could show anything.
    let vault_password = run_blocking(move || {
        vault::replace_fan(&app_data_dir, vault_pin.as_str(), &stored_profile)
    })
    .await?;
    *state.fan_session.write().await = Some(Arc::new(profile));
    *state.fan_pin.write().await = Some(pin);
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
        let pending = state.pending_fan_confirmation.lock().await;
        if pending
            .as_ref()
            .is_some_and(|pending| pending.api_base_url == api_base_url)
        {
            return Ok(());
        }
        return Err(AppError::InvalidPin);
    }

    validate_pin(&pin)?;
    *state.pending_fan_confirmation.lock().await = Some(PendingFanConfirmation {
        api_base_url,
        pin: Zeroizing::new(pin),
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

#[tauri::command]
pub(crate) async fn fan_signup(
    state: State<'_, AppState>,
    mut input: FanSignupInput,
    pin: String,
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
    validate_fan_signup(&mut input, &pin)?;
    let pin = Zeroizing::new(pin);
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
        let vault_pin = pin.clone();
        let vault_password = run_blocking(move || {
            vault::save_fan(&app_data_dir, vault_pin.as_str(), &stored_profile)
        })
        .await?;
        *state.fan_session.write().await = Some(Arc::new(profile));
        *state.fan_pin.write().await = Some(pin);
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

#[tauri::command]
pub(crate) async fn fan_confirm(
    state: State<'_, AppState>,
    mut input: FanConfirmationInput,
    pin: String,
) -> Result<FanSessionStatus, AppError> {
    let _mutation = state.fan_mutation.lock().await;
    validate_fan_confirmation(&mut input, &pin)?;
    persist_confirmed_fan(&state, input, Zeroizing::new(pin)).await?;
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
    mut api_base_url: String,
    pin: String,
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
    validate_fan_confirmation(&mut input, &pin)?;
    persist_confirmed_fan(&state, input, Zeroizing::new(pin)).await?;
    *state.pending_fan_confirm_token.lock().await = None;
    *state.pending_fan_confirmation.lock().await = None;
    drop(_mutation);
    fan_status(state).await
}

#[tauri::command]
pub(crate) async fn fan_confirm_scanned(
    state: State<'_, AppState>,
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
    validate_fan_confirmation(&mut input, pin.as_str())?;
    persist_confirmed_fan(&state, input, pin).await?;
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

/// Returns the last known referral, interests, admission pass and AREA wallet
/// without touching the network, so those panels paint from disk on a cold
/// start instead of holding a skeleton for the length of a live request.
#[tauri::command]
pub(crate) async fn fan_cached_sections(
    state: State<'_, AppState>,
) -> Result<Option<vault::FanSectionsCacheSnapshot>, AppError> {
    // Prove the session is unlocked before the in-memory mirror is served.
    // Locking clears the vault password, and a decrypted mirror must never
    // outlive it.
    let password = state
        .fan_vault_password
        .read()
        .await
        .as_ref()
        .map(|value| Zeroizing::new(value.to_vec()))
        .ok_or(AppError::Locked)?;
    if let Some(snapshot) = state.fan_sections_cache.read().await.clone() {
        return Ok(Some(snapshot));
    }
    let app_data_dir = state.app_data_dir.clone();
    let snapshot = match run_blocking(move || {
        vault::load_fan_sections_cache_with_password(&app_data_dir, password.as_ref())
    })
    .await
    {
        Ok(snapshot) => snapshot,
        Err(error) => {
            eprintln!("[virya:fan-cache] encrypted dashboard snapshot ignored: {error}");
            None
        }
    };
    if let Some(snapshot) = snapshot.clone() {
        *state.fan_sections_cache.write().await = Some(snapshot);
    }
    Ok(snapshot)
}

/// Folds one refreshed section into the encrypted dashboard snapshot. The write
/// is spawned and serialized behind its own mutation lock: the snapshot only has
/// to be on disk before the next cold start, never before the fan sees the
/// section it came from.
fn remember_fan_sections(
    app: AppHandle,
    apply: impl FnOnce(&mut vault::FanSectionsCacheSnapshot) + Send + 'static,
) {
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        let _mutation = state.fan_sections_cache_mutation.lock().await;
        let Some(password) = state
            .fan_vault_password
            .read()
            .await
            .as_ref()
            .map(|value| Zeroizing::new(value.to_vec()))
        else {
            return;
        };
        let mut snapshot = state
            .fan_sections_cache
            .read()
            .await
            .clone()
            .unwrap_or_default();
        apply(&mut snapshot);
        snapshot.stored_at_unix_secs = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|value| value.as_secs())
            .unwrap_or(0);
        *state.fan_sections_cache.write().await = Some(snapshot.clone());
        let app_data_dir = state.app_data_dir.clone();
        if let Err(error) = run_blocking(move || {
            vault::save_fan_sections_cache_with_password(
                &app_data_dir,
                password.as_ref(),
                &snapshot,
            )
        })
        .await
        {
            eprintln!("[virya:fan-cache] encrypted dashboard snapshot save degraded: {error}");
        }
    });
}

#[tauri::command]
pub(crate) async fn fan_cached_events(
    state: State<'_, AppState>,
) -> Result<Vec<PublicEvent>, AppError> {
    let profile = fan_profile(&state).await?;
    Ok(state.api.public_events_snapshot(&profile.api_base_url).await)
}

#[tauri::command]
pub(crate) async fn fan_cached_merch_catalog(
    state: State<'_, AppState>,
) -> Result<Option<MerchCatalog>, AppError> {
    let profile = fan_profile(&state).await?;
    Ok(state.api.public_merch_snapshot(&profile.api_base_url).await)
}

#[tauri::command]
pub(crate) async fn fan_events(state: State<'_, AppState>) -> Result<Vec<PublicEvent>, AppError> {
    let profile = fan_profile(&state).await?;
    state.api.fan_events(&profile).await
}

#[tauri::command]
pub(crate) async fn fan_merch_catalog(
    state: State<'_, AppState>,
) -> Result<MerchCatalog, AppError> {
    let profile = fan_profile(&state).await?;
    state.api.public_merch_catalog(&profile.api_base_url).await
}

#[tauri::command]
pub(crate) async fn fan_merch_bundles(
    state: State<'_, AppState>,
) -> Result<SignalMerchBundleCatalog, AppError> {
    state.api.public_merch_bundles().await
}

#[tauri::command]
pub(crate) async fn fan_ticket_sale(
    state: State<'_, AppState>,
    event_slug: String,
) -> Result<Option<TicketSaleOffer>, AppError> {
    let profile = fan_profile(&state).await?;
    match state
        .api
        .public_ticket_sale(&profile.api_base_url, &event_slug)
        .await
    {
        Ok(value) => Ok(Some(value)),
        Err(AppError::NotFound) => Ok(None),
        Err(error) => Err(error),
    }
}

#[tauri::command]
pub(crate) async fn fan_start_ticket_checkout(
    state: State<'_, AppState>,
    mut input: TicketCheckoutInput,
) -> Result<TicketCheckoutStart, AppError> {
    input.normalize()?;
    let _mutation = state.fan_mutation.lock().await;
    let mut profile = fan_profile(&state).await?.as_ref().clone();
    if profile.wallets.len() >= MAX_WALLETS {
        return Err(AppError::InvalidInput(crate::i18n::replace(
            "native_wallet_limit",
            &[("max", MAX_WALLETS.to_string())],
        )));
    }

    let response = state.api.start_ticket_checkout(&profile, &input).await?;
    let checkout = TicketCheckoutStart::from_site(&response)?;
    let checkout_token = Zeroizing::new(response.checkout_token);
    profile
        .wallets
        .retain(|wallet| wallet.order_id != checkout.order_id);
    profile.wallets.push(WalletCredential {
        order_id: checkout.order_id.clone(),
        checkout_token: checkout_token.to_string(),
    });
    profile
        .cached_wallets
        .retain(|wallet| wallet.order.order_id != checkout.order_id);
    profile
        .cached_wallet_qr
        .retain(|entry| entry.order_id != checkout.order_id);
    state
        .wallet_qr_tokens
        .write()
        .await
        .remove(&checkout.order_id);
    persist_fan(&state, &profile).await?;
    *state.fan_session.write().await = Some(Arc::new(profile));
    Ok(checkout)
}

#[tauri::command]
pub(crate) async fn fan_referral(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<ReferralProgress, AppError> {
    let profile = fan_profile(&state).await?;
    let referral = state.api.fan_referral(&profile).await?;
    remember_fan_sections(app, {
        let referral = referral.clone();
        move |snapshot| snapshot.referral = Some(referral)
    });
    Ok(referral)
}

#[tauri::command]
pub(crate) async fn fan_interests(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Vec<FanEventInterest>, AppError> {
    let profile = fan_profile(&state).await?;
    let interests = state.api.fan_interests(&profile).await?;
    remember_fan_sections(app, {
        let interests = interests.clone();
        move |snapshot| snapshot.interests = interests
    });
    Ok(interests)
}

#[tauri::command]
pub(crate) async fn fan_admission_pass(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<Option<AdmissionPass>, AppError> {
    let _mutation = state.fan_mutation.lock().await;
    let mut profile = fan_profile(&state).await?.as_ref().clone();
    match state.api.fan_admission_pass(&profile).await {
        Ok(value) => {
            remember_fan_sections(app, {
                let value = value.clone();
                move |snapshot| snapshot.admission_pass = value
            });
            Ok(value)
        }
        Err(AppError::Unauthorized | AppError::NotFound) => {
            profile.pass_session_token = None;
            persist_fan(&state, &profile).await?;
            *state.fan_session.write().await = Some(Arc::new(profile));
            // The pass is gone server-side; the cached copy must not outlive it.
            remember_fan_sections(app, |snapshot| snapshot.admission_pass = None);
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

#[tauri::command]
pub(crate) async fn fan_register_interest(
    state: State<'_, AppState>,
    event_slug: String,
) -> Result<serde_json::Value, AppError> {
    let profile = fan_profile(&state).await?;
    state.api.register_interest(&profile, &event_slug).await
}

#[tauri::command]
pub(crate) async fn fan_claim_pass(
    state: State<'_, AppState>,
    claim_token: String,
) -> Result<AdmissionPass, AppError> {
    let _mutation = state.fan_mutation.lock().await;
    let claim_token = bounded_secret(claim_token, crate::i18n::tr("native_admission_token_label"))?;
    let mut profile = fan_profile(&state).await?.as_ref().clone();
    let (pass, pass_session_token) = state.api.claim_pass(&profile, claim_token.as_str()).await?;
    profile.pass_session_token = Some(pass_session_token);
    persist_fan(&state, &profile).await?;
    *state.fan_session.write().await = Some(Arc::new(profile));
    Ok(pass)
}

#[tauri::command]
pub(crate) async fn fan_admission_qr(
    state: State<'_, AppState>,
) -> Result<serde_json::Value, AppError> {
    let profile = fan_profile(&state).await?;
    let value = state.api.admission_qr(&profile).await?;
    run_blocking(move || {
        let mut value = value;
        attach_single_qr(&mut value)?;
        Ok(value)
    })
    .await
}

#[tauri::command]
pub(crate) async fn fan_import_wallet(
    state: State<'_, AppState>,
    order_id: String,
    checkout_token: String,
) -> Result<TicketWallet, AppError> {
    let _mutation = state.fan_mutation.lock().await;
    let order_id = uuid::Uuid::parse_str(order_id.trim())
        .map(|value| value.to_string())
        .map_err(|_| AppError::InvalidInput(crate::i18n::tr("native_invalid_order_id").into()))?;
    let checkout_token =
        bounded_secret(checkout_token, crate::i18n::tr("native_order_token_label"))?;
    let mut profile = fan_profile(&state).await?.as_ref().clone();
    let already_imported = profile
        .wallets
        .iter()
        .any(|wallet| wallet.order_id.as_str() == order_id.as_str());
    if !already_imported && profile.wallets.len() >= MAX_WALLETS {
        return Err(AppError::InvalidInput(crate::i18n::replace(
            "native_wallet_limit",
            &[("max", MAX_WALLETS.to_string())],
        )));
    }
    let wallet = state
        .api
        .ticket_wallet(&profile.api_base_url, &order_id, checkout_token.as_str())
        .await?;
    if wallet.order.order_id != order_id {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_wrong_order_wallet").into(),
        ));
    }
    let (wallet, wallet_tokens, wallet_qr) = prepare_wallet(wallet);
    profile.wallets.retain(|wallet| wallet.order_id != order_id);
    profile.wallets.push(WalletCredential {
        order_id: order_id.clone(),
        checkout_token: checkout_token.to_string(),
    });
    profile
        .cached_wallets
        .retain(|entry| entry.order.order_id.as_str() != order_id.as_str());
    profile.cached_wallets.push(wallet.clone());
    profile
        .cached_wallet_qr
        .retain(|entry| entry.order_id.as_str() != order_id.as_str());
    profile.cached_wallet_qr.extend(wallet_qr);
    persist_fan(&state, &profile).await?;
    *state.fan_session.write().await = Some(Arc::new(profile));
    state
        .wallet_qr_tokens
        .write()
        .await
        .insert(order_id, wallet_tokens);
    Ok(wallet)
}
