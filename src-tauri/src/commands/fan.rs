//! Fan-facing session lifecycle: signup/confirmation, event interests,
//! referral progress, admission passes and the ticket wallet (including
//! locally-rendered QR codes, whose raw tokens never leave the native
//! process once fetched).

use std::{collections::HashMap, sync::Arc};

use futures_util::{StreamExt, stream};
use qrcode::{QrCode, render::svg};
use tauri::State;
use zeroize::Zeroizing;

use crate::{
    AppError, AppState, MAX_SECRET_BYTES,
    api::{SignalMerchBundleCatalog, TicketCheckoutInput, TicketCheckoutStart, TicketSaleOffer},
    models::{
        AdmissionPass, AreaChallenge, AreaClaimResult, AreaPositionSample, FanAuthResult,
        FanConfirmationInput, FanEventInterest, FanHomeData, FanProfile, FanSessionStatus, FanSignupInput,
        MerchCatalog, PublicEvent, ReferralProgress, TicketWallet, TicketWalletApi, WalletBatch,
        WalletCredential, WalletQrCredential, WalletTicket,
    },
    session::{fan_profile, persist_fan, run_blocking},
    validation::{bounded_secret, validate_fan_confirmation, validate_fan_signup, validate_pin},
    vault,
};

const MAX_WALLETS: usize = 24;
const WALLET_FETCH_CONCURRENCY: usize = 8;

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
pub(crate) async fn fan_forget(state: State<'_, AppState>) -> Result<FanSessionStatus, AppError> {
    let _mutation = state.fan_mutation.lock().await;
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
    state.wallet_qr_tokens.write().await.remove(&checkout.order_id);
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
    profile.cached_wallets.retain(|entry| entry.order.order_id.as_str() != order_id.as_str());
    profile.cached_wallets.push(wallet.clone());
    profile.cached_wallet_qr.retain(|entry| entry.order_id.as_str() != order_id.as_str());
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
pub(crate) async fn fan_wallets(state: State<'_, AppState>) -> Result<WalletBatch, AppError> {
    let profile = fan_profile(&state).await?;
    let api = state.api.clone();
    let api_base_url = profile.api_base_url.clone();
    let requests = profile.wallets.iter().cloned().map(move |credential| {
        let api = api.clone();
        let api_base_url = api_base_url.clone();
        let expected_order_id = credential.order_id.clone();
        async move {
            let result = api
                .ticket_wallet(&api_base_url, &credential.order_id, &credential.checkout_token)
                .await
                .and_then(|value| {
                    if value.order.order_id.as_str() == credential.order_id.as_str() {
                        Ok(value)
                    } else {
                        Err(AppError::InvalidInput(
                            crate::i18n::tr("native_wrong_order_wallet").into(),
                        ))
                    }
                });
            (expected_order_id, result)
        }
    });
    let results = stream::iter(requests)
        .buffered(WALLET_FETCH_CONCURRENCY)
        .collect::<Vec<_>>()
        .await;
    let request_count = results.len();
    let mut wallets = Vec::with_capacity(request_count);
    let mut wallet_tokens = Vec::with_capacity(request_count);
    let mut live_snapshots = Vec::with_capacity(request_count);
    let mut failed_orders = Vec::new();
    let mut first_error = None;
    for (order_id, result) in results {
        match result {
            Ok(wallet) => {
                let order_id = wallet.order.order_id.clone();
                let (wallet, tokens, qr_credentials) = prepare_wallet(wallet);
                live_snapshots.push((wallet.clone(), qr_credentials));
                wallets.push(wallet);
                wallet_tokens.push((order_id, tokens));
            }
            Err(error) => {
                failed_orders.push(order_id);
                if first_error.is_none() {
                    first_error = Some(error);
                }
            }
        }
    }

    // Stronghold keeps the last public wallet snapshot plus only still-valid QR
    // credentials. Checkout secrets remain in their canonical credential list;
    // cached QR tokens stay encrypted/zeroized and never enter the WebView payload.
    let mut cached_count = 0usize;
    for order_id in &failed_orders {
        if let Some(mut cached) = profile
            .cached_wallets
            .iter()
            .find(|wallet| wallet.order.order_id.as_str() == order_id.as_str())
            .cloned()
        {
            cached.cached = true;
            for ticket in &mut cached.tickets {
                ticket.qr_available = profile.cached_wallet_qr.iter().any(|entry| {
                    entry.order_id.as_str() == order_id.as_str()
                        && entry.public_reference.as_str() == ticket.public_reference.as_str()
                        && wallet_qr_credential_valid(entry)
                });
            }
            wallets.push(cached);
            cached_count += 1;
        }
    }
    if wallets.is_empty()
        && let Some(error) = first_error
    {
        return Err(error);
    }

    let configured_orders = profile
        .wallets
        .iter()
        .map(|wallet| wallet.order_id.clone())
        .collect::<std::collections::HashSet<_>>();
    let mut cached_tokens = state.wallet_qr_tokens.write().await;
    cached_tokens.retain(|order_id, _| configured_orders.contains(order_id));
    cached_tokens.extend(wallet_tokens);
    drop(cached_tokens);

    if !live_snapshots.is_empty() {
        let _mutation = state.fan_mutation.lock().await;
        let latest = fan_profile(&state).await?;
        if latest.fan_session_token == profile.fan_session_token {
            let configured = latest
                .wallets
                .iter()
                .map(|wallet| wallet.order_id.clone())
                .collect::<std::collections::HashSet<_>>();
            let mut updated = latest.as_ref().clone();
            updated
                .cached_wallets
                .retain(|wallet| configured.contains(&wallet.order.order_id));
            for (mut snapshot, qr_credentials) in live_snapshots {
                snapshot.cached = false;
                let order_id = snapshot.order.order_id.clone();
                updated
                    .cached_wallets
                    .retain(|wallet| wallet.order.order_id.as_str() != order_id.as_str());
                updated.cached_wallets.push(snapshot);
                updated.cached_wallet_qr.retain(|entry| entry.order_id.as_str() != order_id.as_str());
                updated.cached_wallet_qr.extend(qr_credentials);
            }
            if updated.cached_wallets.len() > MAX_WALLETS {
                updated.cached_wallets.truncate(MAX_WALLETS);
            }
            updated.cached_wallet_qr.retain(wallet_qr_credential_valid);
            if updated.cached_wallet_qr.len() > MAX_WALLETS.saturating_mul(8) {
                updated.cached_wallet_qr.truncate(MAX_WALLETS.saturating_mul(8));
            }
            persist_fan(&state, &updated).await?;
            *state.fan_session.write().await = Some(Arc::new(updated));
        }
    }

    Ok(WalletBatch {
        failed_count: failed_orders.len(),
        cached_count,
        wallets,
    })
}

#[tauri::command]
pub(crate) async fn render_wallet_qr(
    state: State<'_, AppState>,
    order_id: String,
    public_reference: String,
) -> Result<String, AppError> {
    let order_id = uuid::Uuid::parse_str(order_id.trim())
        .map(|value| value.to_string())
        .map_err(|_| AppError::InvalidInput(crate::i18n::tr("native_invalid_order_id").into()))?;
    let public_reference = public_reference.trim();
    if public_reference.is_empty() || public_reference.len() > 200 {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_ticket_reference_invalid").into(),
        ));
    }
    let live_token = state
        .wallet_qr_tokens
        .read()
        .await
        .get(&order_id)
        .and_then(|tickets| tickets.get(public_reference))
        .cloned();
    let token = match live_token {
        Some(token) => token,
        None => {
            let profile = fan_profile(&state).await?;
            profile
                .cached_wallet_qr
                .iter()
                .find(|entry| {
                    entry.order_id.as_str() == order_id.as_str()
                        && entry.public_reference.as_str() == public_reference
                        && wallet_qr_credential_valid(entry)
                })
                .map(|entry| Zeroizing::new(entry.token.clone()))
                .ok_or(AppError::NotFound)?
        }
    };
    run_blocking(move || render_qr(token.as_str())).await
}

fn prepare_wallet(
    wallet: TicketWalletApi,
) -> (
    TicketWallet,
    HashMap<String, Zeroizing<String>>,
    Vec<WalletQrCredential>,
) {
    let order_id = wallet.order.order_id.clone();
    let mut tokens = HashMap::with_capacity(wallet.tickets.len());
    let mut cached_qr = Vec::with_capacity(wallet.tickets.len());
    let tickets = wallet
        .tickets
        .into_iter()
        .map(|ticket| {
            let qr_available = match ticket.qr_token {
                Some(token) => {
                    let credential = WalletQrCredential {
                        order_id: order_id.clone(),
                        public_reference: ticket.public_reference.clone(),
                        token,
                        expires_at: ticket.qr_expires_at.clone(),
                    };
                    if wallet_qr_credential_valid(&credential) {
                        tokens.insert(
                            ticket.public_reference.clone(),
                            Zeroizing::new(credential.token.clone()),
                        );
                        cached_qr.push(credential);
                        true
                    } else {
                        false
                    }
                }
                None => false,
            };
            WalletTicket {
                ticket_type_name: ticket.ticket_type_name,
                public_reference: ticket.public_reference,
                holder_name: ticket.holder_name,
                holder_email_masked: ticket.holder_email_masked,
                qr_available,
                qr_expires_at: ticket.qr_expires_at,
            }
        })
        .collect();
    cached_qr.retain(wallet_qr_credential_valid);
    (
        TicketWallet {
            order: wallet.order,
            tickets,
            cached: false,
        },
        tokens,
        cached_qr,
    )
}

fn wallet_qr_credential_valid(value: &WalletQrCredential) -> bool {
    use time::format_description::well_known::Rfc3339;
    if value.token.is_empty() || value.token.len() > MAX_SECRET_BYTES {
        return false;
    }
    time::OffsetDateTime::parse(&value.expires_at, &Rfc3339)
        .is_ok_and(|expires_at| expires_at > time::OffsetDateTime::now_utc())
}

#[tauri::command]
pub(crate) async fn fan_request_delivery(
    state: State<'_, AppState>,
    order_id: String,
) -> Result<serde_json::Value, AppError> {
    let order_id = uuid::Uuid::parse_str(order_id.trim())
        .map(|value| value.to_string())
        .map_err(|_| AppError::InvalidInput(crate::i18n::tr("native_invalid_order_id").into()))?;
    let profile = fan_profile(&state).await?;
    let wallet = profile
        .wallets
        .iter()
        .find(|wallet| wallet.order_id.as_str() == order_id.as_str())
        .ok_or_else(|| {
            AppError::InvalidInput(crate::i18n::tr("native_ticket_not_on_device").into())
        })?;
    state
        .api
        .request_ticket_delivery(
            &profile.api_base_url,
            &wallet.order_id,
            &wallet.checkout_token,
        )
        .await
}

fn attach_single_qr(value: &mut serde_json::Value) -> Result<(), AppError> {
    let token = value
        .get("token")
        .and_then(serde_json::Value::as_str)
        .ok_or_else(|| AppError::InvalidInput(crate::i18n::tr("native_qr_token_missing").into()))?;
    let svg = render_qr(token)?;
    value["qr_svg"] = serde_json::Value::String(svg);
    Ok(())
}

fn render_qr(token: &str) -> Result<String, AppError> {
    if token.is_empty() || token.len() > MAX_SECRET_BYTES {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_qr_token_invalid").into(),
        ));
    }
    let code = QrCode::new(token.as_bytes()).map_err(|_| {
        AppError::InvalidInput(crate::i18n::tr("native_qr_generation_failed").into())
    })?;
    let rendered = code
        .render::<svg::Color>()
        .min_dimensions(320, 320)
        .dark_color(svg::Color("#080808"))
        .light_color(svg::Color("#ffffff"))
        .build();

    // qrcode's SVG renderer prepends an XML declaration. The webview contract
    // expects a standalone <svg> fragment suitable for direct DOM insertion.
    let start = rendered.find("<svg").ok_or_else(|| {
        AppError::InvalidInput(crate::i18n::tr("native_qr_generation_failed").into())
    })?;
    let svg = rendered[start..].trim();
    if !svg.ends_with("</svg>") {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_qr_generation_failed").into(),
        ));
    }
    Ok(svg.to_owned())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models;

    fn test_value<T, E>(result: Result<T, E>) -> T
    where
        E: std::fmt::Debug,
    {
        match result {
            Ok(value) => value,
            Err(error) => panic!("test setup failed: {error:?}"),
        }
    }

    #[test]
    fn qr_render_is_bounded_and_produces_svg() {
        let svg = test_value(render_qr("v1.test-token"));
        assert!(svg.starts_with("<svg"));
        assert!(render_qr("").is_err());
        assert!(render_qr(&"x".repeat(MAX_SECRET_BYTES + 1)).is_err());
    }

    #[test]
    fn wallet_tokens_are_split_from_the_webview_payload() {
        let wallet = TicketWalletApi {
            order: models::WalletOrder {
                order_id: "01234567-89ab-cdef-0123-456789abcdef".into(),
                public_reference: "VRY-ORDER".into(),
                event_title: "Virya Live".into(),
                venue: Some("Club".into()),
                starts_at: "2026-08-01T20:00:00Z".into(),
                status: "paid".into(),
            },
            tickets: vec![models::WalletTicketApi {
                ticket_type_name: "Regular".into(),
                public_reference: "VRY-TICKET".into(),
                holder_name: Some("Fan".into()),
                holder_email_masked: "f***@example.com".into(),
                qr_token: Some("v1.private-token".into()),
                qr_expires_at: "2099-08-01T21:00:00Z".into(),
            }],
        };
        let (public, tokens, cached_qr) = prepare_wallet(wallet);
        assert!(public.tickets[0].qr_available);
        assert_eq!(tokens["VRY-TICKET"].as_str(), "v1.private-token");
        assert_eq!(cached_qr.len(), 1);
    }

    #[test]
    fn invalid_wallet_qr_tokens_are_not_cached() {
        let wallet = TicketWalletApi {
            order: models::WalletOrder {
                order_id: "01234567-89ab-cdef-0123-456789abcdef".into(),
                public_reference: "VRY-ORDER".into(),
                event_title: "Virya Live".into(),
                venue: None,
                starts_at: "2026-08-01T20:00:00Z".into(),
                status: "paid".into(),
            },
            tickets: vec![models::WalletTicketApi {
                ticket_type_name: "Regular".into(),
                public_reference: "VRY-TICKET".into(),
                holder_name: None,
                holder_email_masked: "f***@example.com".into(),
                qr_token: Some(String::new()),
                qr_expires_at: "2026-08-01T21:00:00Z".into(),
            }],
        };
        let (public, tokens, cached_qr) = prepare_wallet(wallet);
        assert!(!public.tickets[0].qr_available);
        assert!(tokens.is_empty());
        assert!(cached_qr.is_empty());
    }
}
