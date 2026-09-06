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
        // Re-check the vault password immediately before publishing the
        // snapshot. `fan_lock` clears `fan_vault_password` and
        // `fan_sections_cache` without holding `fan_sections_cache_mutation`,
        // so a background `remember_fan_sections` that captured the password
        // before the lock can still be in flight here. Without this guard, the
        // stale snapshot would re-populate the cache after the lock cleared it,
        // and the next unlock would serve the previous fan's panels.
        if state.fan_vault_password.read().await.is_none() {
            return;
        }
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
) -> Result<PublicEventsResult, AppError> {
    let profile = fan_profile(&state).await?;
    // The snapshot carries its own freshness answer: it is the last list that
    // arrived, painted while the live request is still in flight, so nothing
    // in this process has validated it.
    Ok(state.api.public_events_snapshot(&profile.api_base_url).await)
}

#[tauri::command]
pub(crate) async fn fan_cached_merch_catalog(
    state: State<'_, AppState>,
) -> Result<Option<MerchCatalogResult>, AppError> {
    let profile = fan_profile(&state).await?;
    Ok(state.api.public_merch_snapshot(&profile.api_base_url).await)
}

#[tauri::command]
pub(crate) async fn fan_events(
    state: State<'_, AppState>,
) -> Result<PublicEventsResult, AppError> {
    let profile = fan_profile(&state).await?;
    state.api.fan_events(&profile).await
}

#[tauri::command]
pub(crate) async fn fan_merch_catalog(
    state: State<'_, AppState>,
) -> Result<MerchCatalogResult, AppError> {
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
