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
    let profile = run_blocking(move || vault::load_fan(&app_data_dir, vault_pin.as_str())).await?;
    *state.fan_session.write().await = Some(Arc::new(profile));
    *state.fan_pin.write().await = Some(pin_for_session);
    state.wallet_qr_tokens.write().await.clear();
    drop(_mutation);
    if let Err(error) = sync_native_push_if_desired(&state, &app).await {
        eprintln!("[virya:push-sync] unlock reconciliation degraded: {error}");
    }
    fan_status(state).await
}

#[tauri::command]
pub(crate) async fn fan_lock(state: State<'_, AppState>) -> Result<FanSessionStatus, AppError> {
    let _mutation = state.fan_mutation.lock().await;
    *state.fan_session.write().await = None;
    *state.fan_pin.write().await = None;
    state.wallet_qr_tokens.write().await.clear();
    drop(_mutation);
    fan_status(state).await
}

#[tauri::command]
pub(crate) async fn fan_forget(
    state: State<'_, AppState>,
    app: AppHandle,
) -> Result<FanSessionStatus, AppError> {
    let _mutation = state.fan_mutation.lock().await;
    if let Some(profile) = state.fan_session.read().await.clone() {
        if let Some(installation_id) = read_native_push_installation_id(&state.app_data_dir)
            && let Err(error) = state
                .api
                .fan_disable_android_push(&profile, &installation_id)
                .await
        {
            eprintln!("[virya:push-disable] remote disable before forget degraded: {error}");
        }
        if let Err(error) = delete_native_push_token(&app) {
            eprintln!("[virya:push-disable] local token delete before forget degraded: {error}");
        }
    }
    *state.fan_session.write().await = None;
    *state.fan_pin.write().await = None;
    state.wallet_qr_tokens.write().await.clear();
    let app_data_dir = state.app_data_dir.clone();
    run_blocking(move || vault::remove_fan(&app_data_dir)).await?;
    drop(_mutation);
    fan_status(state).await
}

#[tauri::command]
pub(crate) async fn fan_signup(
    state: State<'_, AppState>,
    mut input: FanSignupInput,
    pin: String,
) -> Result<FanAuthResult, AppError> {
    let _mutation = state.fan_mutation.lock().await;
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
        run_blocking(move || vault::save_fan(&app_data_dir, vault_pin.as_str(), &stored_profile))
            .await?;
        *state.fan_session.write().await = Some(Arc::new(profile));
        *state.fan_pin.write().await = Some(pin);
        state.wallet_qr_tokens.write().await.clear();
    }
    Ok(result)
}

#[tauri::command]
pub(crate) async fn fan_confirm(
    state: State<'_, AppState>,
    mut input: FanConfirmationInput,
    pin: String,
) -> Result<FanAuthResult, AppError> {
    let _mutation = state.fan_mutation.lock().await;
    validate_fan_confirmation(&mut input, &pin)?;
    let pin = Zeroizing::new(pin);
    let (result, session_token) = state.api.fan_confirm(&input).await?;
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
    run_blocking(move || vault::replace_fan(&app_data_dir, vault_pin.as_str(), &stored_profile))
        .await?;
    *state.fan_session.write().await = Some(Arc::new(profile));
    *state.fan_pin.write().await = Some(pin);
    state.wallet_qr_tokens.write().await.clear();
    Ok(result)
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
    state: State<'_, AppState>,
) -> Result<crate::models::AreaWallet, AppError> {
    let profile = fan_profile(&state).await?;
    state.api.fan_area_wallet(&profile).await
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
pub(crate) async fn fan_home(state: State<'_, AppState>) -> Result<FanHomeData, AppError> {
    let profile = fan_profile(&state).await?;
    state.api.fan_home(&profile).await
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
pub(crate) async fn fan_referral(state: State<'_, AppState>) -> Result<ReferralProgress, AppError> {
    let profile = fan_profile(&state).await?;
    state.api.fan_referral(&profile).await
}

#[tauri::command]
pub(crate) async fn fan_interests(
    state: State<'_, AppState>,
) -> Result<Vec<FanEventInterest>, AppError> {
    let profile = fan_profile(&state).await?;
    state.api.fan_interests(&profile).await
}

#[tauri::command]
pub(crate) async fn fan_admission_pass(
    state: State<'_, AppState>,
) -> Result<Option<AdmissionPass>, AppError> {
    let _mutation = state.fan_mutation.lock().await;
    let mut profile = fan_profile(&state).await?.as_ref().clone();
    match state.api.fan_admission_pass(&profile).await {
        Ok(value) => Ok(value),
        Err(AppError::Unauthorized | AppError::NotFound) => {
            profile.pass_session_token = None;
            persist_fan(&state, &profile).await?;
            *state.fan_session.write().await = Some(Arc::new(profile));
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

#[tauri::command]
