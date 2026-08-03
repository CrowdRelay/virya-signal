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
    models::{
        AdmissionPass, FanAuthResult, FanConfirmationInput, FanEventInterest, FanProfile,
        FanSessionStatus, FanSignupInput, PublicEvent, ReferralProgress, TicketWallet,
        TicketWalletApi, WalletBatch, WalletCredential, WalletTicket,
    },
    session::{fan_profile, persist_fan, run_blocking},
    validation::{bounded_secret, validate_fan_confirmation, validate_fan_signup, validate_pin},
    vault,
};

const MAX_WALLETS: usize = 24;

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
    };
    let app_data_dir = state.app_data_dir.clone();
    let stored_profile = profile.clone();
    let vault_pin = pin.clone();
    run_blocking(move || vault::save_fan(&app_data_dir, vault_pin.as_str(), &stored_profile))
        .await?;
    *state.fan_session.write().await = Some(Arc::new(profile));
    *state.fan_pin.write().await = Some(pin);
    state.wallet_qr_tokens.write().await.clear();
    Ok(result)
}

#[tauri::command]
pub(crate) async fn fan_area_wallet(
    state: State<'_, AppState>,
) -> Result<crate::models::AreaWallet, AppError> {
    let profile = fan_profile(&state).await?;
    state.api.fan_area_wallet(&profile).await
}

#[tauri::command]
pub(crate) async fn fan_events(state: State<'_, AppState>) -> Result<Vec<PublicEvent>, AppError> {
    let profile = fan_profile(&state).await?;
    state.api.fan_events(&profile).await
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
    let claim_token = bounded_secret(claim_token, "token wejściówki")?;
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
        .map_err(|_| AppError::InvalidInput("Nieprawidłowy identyfikator zamówienia".into()))?;
    let checkout_token = bounded_secret(checkout_token, "token zamówienia")?;
    let mut profile = fan_profile(&state).await?.as_ref().clone();
    let already_imported = profile
        .wallets
        .iter()
        .any(|wallet| wallet.order_id == order_id);
    if !already_imported && profile.wallets.len() >= MAX_WALLETS {
        return Err(AppError::InvalidInput(format!(
            "Portfel może zawierać maksymalnie {MAX_WALLETS} zamówienia"
        )));
    }
    let wallet = state
        .api
        .ticket_wallet(&profile.api_base_url, &order_id, checkout_token.as_str())
        .await?;
    if wallet.order.order_id != order_id {
        return Err(AppError::InvalidInput(
            "Backend zwrócił portfel innego zamówienia".into(),
        ));
    }
    let (wallet, wallet_tokens) = prepare_wallet(wallet);
    profile.wallets.retain(|wallet| wallet.order_id != order_id);
    profile.wallets.push(WalletCredential {
        order_id: order_id.clone(),
        checkout_token: checkout_token.to_string(),
    });
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
    let requests = profile.wallets.iter().cloned().map(move |wallet| {
        let api = api.clone();
        let api_base_url = api_base_url.clone();
        async move {
            let value = api
                .ticket_wallet(&api_base_url, &wallet.order_id, &wallet.checkout_token)
                .await?;
            if value.order.order_id != wallet.order_id {
                return Err(AppError::InvalidInput(
                    "Backend zwrócił portfel innego zamówienia".into(),
                ));
            }
            Ok(value)
        }
    });
    let results = stream::iter(requests).buffered(8).collect::<Vec<_>>().await;
    let request_count = results.len();
    let mut wallets = Vec::with_capacity(request_count);
    let mut wallet_tokens = Vec::with_capacity(request_count);
    let mut first_error = None;
    for result in results {
        match result {
            Ok(wallet) => {
                let order_id = wallet.order.order_id.clone();
                let (wallet, tokens) = prepare_wallet(wallet);
                wallets.push(wallet);
                wallet_tokens.push((order_id, tokens));
            }
            Err(error) if first_error.is_none() => first_error = Some(error),
            Err(_) => {}
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
    Ok(WalletBatch {
        failed_count: request_count - wallets.len(),
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
        .map_err(|_| AppError::InvalidInput("Nieprawidłowy identyfikator zamówienia".into()))?;
    let public_reference = public_reference.trim();
    if public_reference.is_empty() || public_reference.len() > 200 {
        return Err(AppError::InvalidInput(
            "Nieprawidłowa referencja biletu".into(),
        ));
    }
    let token = state
        .wallet_qr_tokens
        .read()
        .await
        .get(&order_id)
        .and_then(|tickets| tickets.get(public_reference))
        .cloned()
        .ok_or(AppError::NotFound)?;
    run_blocking(move || render_qr(token.as_str())).await
}

fn prepare_wallet(wallet: TicketWalletApi) -> (TicketWallet, HashMap<String, Zeroizing<String>>) {
    let mut tokens = HashMap::with_capacity(wallet.tickets.len());
    let tickets = wallet
        .tickets
        .into_iter()
        .map(|ticket| {
            let qr_available = ticket.qr_token.is_some_and(|token| {
                let token = Zeroizing::new(token);
                if token.is_empty() || token.len() > MAX_SECRET_BYTES {
                    return false;
                }
                tokens.insert(ticket.public_reference.clone(), token);
                true
            });
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
    (
        TicketWallet {
            order: wallet.order,
            tickets,
        },
        tokens,
    )
}

#[tauri::command]
pub(crate) async fn fan_request_delivery(
    state: State<'_, AppState>,
    order_id: String,
) -> Result<serde_json::Value, AppError> {
    let order_id = uuid::Uuid::parse_str(order_id.trim())
        .map(|value| value.to_string())
        .map_err(|_| AppError::InvalidInput("Nieprawidłowy identyfikator zamówienia".into()))?;
    let profile = fan_profile(&state).await?;
    let wallet = profile
        .wallets
        .iter()
        .find(|wallet| wallet.order_id == order_id)
        .ok_or_else(|| AppError::InvalidInput("Nie znaleziono biletu na urządzeniu".into()))?;
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
        .ok_or_else(|| AppError::InvalidInput("Brak tokenu QR w odpowiedzi backendu".into()))?;
    let svg = render_qr(token)?;
    value["qr_svg"] = serde_json::Value::String(svg);
    Ok(())
}

fn render_qr(token: &str) -> Result<String, AppError> {
    if token.is_empty() || token.len() > MAX_SECRET_BYTES {
        return Err(AppError::InvalidInput("Nieprawidłowy token QR".into()));
    }
    let code = QrCode::new(token.as_bytes())
        .map_err(|_| AppError::InvalidInput("Nie udało się wygenerować kodu QR".into()))?;
    let rendered = code
        .render::<svg::Color>()
        .min_dimensions(320, 320)
        .dark_color(svg::Color("#080808"))
        .light_color(svg::Color("#ffffff"))
        .build();

    // qrcode's SVG renderer prepends an XML declaration. The webview contract
    // expects a standalone <svg> fragment suitable for direct DOM insertion.
    let start = rendered
        .find("<svg")
        .ok_or_else(|| AppError::InvalidInput("Nie udało się wygenerować kodu QR".into()))?;
    let svg = rendered[start..].trim();
    if !svg.ends_with("</svg>") {
        return Err(AppError::InvalidInput(
            "Nie udało się wygenerować kodu QR".into(),
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
                qr_expires_at: "2026-08-01T21:00:00Z".into(),
            }],
        };
        let (public, tokens) = prepare_wallet(wallet);
        assert!(public.tickets[0].qr_available);
        assert_eq!(tokens["VRY-TICKET"].as_str(), "v1.private-token");
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
        let (public, tokens) = prepare_wallet(wallet);
        assert!(!public.tickets[0].qr_available);
        assert!(tokens.is_empty());
    }
}
