#![forbid(unsafe_code)]

mod api;
mod models;
mod vault;

use std::path::PathBuf;

use api::CrowdRelayClient;
use futures::{stream, StreamExt, TryStreamExt};
use models::{
    CitySignal, CreateQrCampaignInput, FanAuthResult, FanConfirmationInput, FanProfile,
    FanSessionStatus, FanSignupInput, IssuePassInput, OperatorProfile, PublicEvent, SessionStatus,
    WalletCredential,
};
use qrcode::{render::svg, QrCode};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;
use thiserror::Error;
use tokio::sync::{Mutex, RwLock};
use zeroize::Zeroizing;

pub struct AppState {
    session: RwLock<Option<OperatorProfile>>,
    operator_mutation: Mutex<()>,
    fan_session: RwLock<Option<FanProfile>>,
    fan_pin: RwLock<Option<Zeroizing<String>>>,
    fan_mutation: Mutex<()>,
    api: CrowdRelayClient,
    app_data_dir: PathBuf,
}

#[derive(Debug, Error)]
pub enum AppError {
    #[error("Profil urządzenia nie jest skonfigurowany")]
    NotConfigured,
    #[error("Nieprawidłowy PIN")]
    InvalidPin,
    #[error("Sesja jest zablokowana")]
    Locked,
    #[error("Token urządzenia jest nieprawidłowy albo nie ma wymaganych uprawnień")]
    Unauthorized,
    #[error("Ta operacja wymaga roli owner")]
    Forbidden,
    #[error("{0}")]
    InvalidInput(String),
    #[error("Konflikt: {0}")]
    Conflict(String),
    #[error("Nie znaleziono danych")]
    NotFound,
    #[error("CrowdRelay HTTP {status}: {detail}")]
    Remote { status: u16, detail: String },
    #[error("Błąd sieci: {0}")]
    Network(#[from] reqwest::Error),
    #[error("Błędny URL: {0}")]
    Url(#[from] url::ParseError),
    #[error("Błąd danych: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Błąd pliku: {0}")]
    Io(#[from] std::io::Error),
    #[error("Błąd magazynu sejfu")]
    StrongholdClient,
    #[error("Wewnętrzny błąd zadania")]
    BackgroundTask,
}

const MAX_SECRET_BYTES: usize = 4096;
const MAX_WALLETS: usize = 24;

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

#[tauri::command]
fn open_external_url(app: AppHandle, url: String) -> Result<(), AppError> {
    let url = url.trim();
    if url.len() > 2048 {
        return Err(AppError::InvalidInput("Link jest zbyt długi".into()));
    }
    let parsed = url::Url::parse(url)?;
    if parsed.scheme() != "https"
        || parsed.host_str().is_none()
        || parsed.username() != ""
        || parsed.password().is_some()
    {
        return Err(AppError::InvalidInput(
            "Można otwierać wyłącznie bezpieczne linki HTTPS".into(),
        ));
    }
    app.opener()
        .open_url(parsed.as_str(), None::<&str>)
        .map_err(|error| AppError::InvalidInput(format!("Nie udało się otworzyć linku: {error}")))
}

#[tauri::command]
async fn session_status(state: State<'_, AppState>) -> Result<SessionStatus, AppError> {
    let session = state.session.read().await;
    Ok(SessionStatus {
        configured: vault::exists(&state.app_data_dir),
        unlocked: session.is_some(),
        session: session.as_ref().map(Into::into),
    })
}

#[tauri::command]
async fn configure(
    state: State<'_, AppState>,
    pin: String,
    mut profile: OperatorProfile,
) -> Result<SessionStatus, AppError> {
    let _mutation = state.operator_mutation.lock().await;
    validate_operator_profile(&mut profile)?;
    validate_pin(&pin)?;
    state.api.validate(&profile).await?;
    let app_data_dir = state.app_data_dir.clone();
    let stored_profile = profile.clone();
    let pin = Zeroizing::new(pin);
    run_blocking(move || vault::save(&app_data_dir, pin.as_str(), &stored_profile)).await?;
    *state.session.write().await = Some(profile);
    drop(_mutation);
    session_status(state).await
}

#[tauri::command]
async fn unlock(state: State<'_, AppState>, pin: String) -> Result<SessionStatus, AppError> {
    let _mutation = state.operator_mutation.lock().await;
    validate_pin(&pin)?;
    let app_data_dir = state.app_data_dir.clone();
    let pin = Zeroizing::new(pin);
    let profile = run_blocking(move || vault::load(&app_data_dir, pin.as_str())).await?;
    *state.session.write().await = Some(profile);
    drop(_mutation);
    session_status(state).await
}

#[tauri::command]
async fn lock(state: State<'_, AppState>) -> Result<SessionStatus, AppError> {
    let _mutation = state.operator_mutation.lock().await;
    *state.session.write().await = None;
    drop(_mutation);
    session_status(state).await
}

#[tauri::command]
async fn forget_device(state: State<'_, AppState>) -> Result<SessionStatus, AppError> {
    let _mutation = state.operator_mutation.lock().await;
    *state.session.write().await = None;
    let app_data_dir = state.app_data_dir.clone();
    run_blocking(move || vault::remove(&app_data_dir)).await?;
    drop(_mutation);
    session_status(state).await
}

#[tauri::command]
async fn operator_events(state: State<'_, AppState>) -> Result<Vec<PublicEvent>, AppError> {
    let profile = operator_profile(&state).await?;
    state.api.operator_events(&profile).await
}

#[tauri::command]
async fn operator_qr(state: State<'_, AppState>) -> Result<serde_json::Value, AppError> {
    let profile = operator_profile(&state).await?;
    state.api.operator_qr(&profile).await
}

#[tauri::command]
async fn ticketing_overview(
    state: State<'_, AppState>,
    event_slug: String,
) -> Result<serde_json::Value, AppError> {
    let profile = operator_profile(&state).await?;
    state.api.ticketing_overview(&profile, &event_slug).await
}

#[tauri::command]
async fn redeem_admission(
    state: State<'_, AppState>,
    event_slug: String,
    code: String,
) -> Result<serde_json::Value, AppError> {
    let profile = operator_profile(&state).await?;
    state
        .api
        .redeem_admission(&profile, &event_slug, &code)
        .await
}

#[tauri::command]
async fn redeem_coupon(
    state: State<'_, AppState>,
    code: String,
    order_reference: String,
) -> Result<serde_json::Value, AppError> {
    let profile = operator_profile(&state).await?;
    state
        .api
        .redeem_coupon(&profile, &code, &order_reference)
        .await
}

#[tauri::command]
async fn issue_pass(
    state: State<'_, AppState>,
    mut input: IssuePassInput,
) -> Result<serde_json::Value, AppError> {
    validate_issue_pass(&mut input)?;
    let profile = operator_profile(&state).await?;
    state.api.issue_pass(&profile, &input).await
}

#[tauri::command]
async fn revoke_pass(
    state: State<'_, AppState>,
    public_reference: String,
) -> Result<serde_json::Value, AppError> {
    let profile = operator_profile(&state).await?;
    state.api.revoke_pass(&profile, &public_reference).await
}

#[tauri::command]
async fn create_qr_campaign(
    state: State<'_, AppState>,
    mut input: CreateQrCampaignInput,
) -> Result<serde_json::Value, AppError> {
    validate_campaign(&mut input)?;
    let profile = operator_profile(&state).await?;
    state.api.create_qr_campaign(&profile, &input).await
}

#[tauri::command]
async fn revoke_qr_campaign(
    state: State<'_, AppState>,
    campaign_id: String,
) -> Result<serde_json::Value, AppError> {
    let profile = operator_profile(&state).await?;
    state.api.revoke_qr_campaign(&profile, &campaign_id).await
}

#[tauri::command]
async fn public_events(
    state: State<'_, AppState>,
    api_base_url: String,
) -> Result<Vec<PublicEvent>, AppError> {
    validate_api_base(&api_base_url)?;
    state.api.public_events(&api_base_url).await
}

#[tauri::command]
async fn public_cities(
    state: State<'_, AppState>,
    api_base_url: String,
) -> Result<Vec<CitySignal>, AppError> {
    validate_api_base(&api_base_url)?;
    state.api.public_cities(&api_base_url).await
}

#[tauri::command]
async fn fan_status(state: State<'_, AppState>) -> Result<FanSessionStatus, AppError> {
    let session = state.fan_session.read().await;
    Ok(FanSessionStatus {
        configured: vault::fan_exists(&state.app_data_dir),
        unlocked: session.is_some(),
        session: session.as_ref().map(Into::into),
    })
}

#[tauri::command]
async fn fan_unlock(state: State<'_, AppState>, pin: String) -> Result<FanSessionStatus, AppError> {
    let _mutation = state.fan_mutation.lock().await;
    validate_pin(&pin)?;
    let app_data_dir = state.app_data_dir.clone();
    let vault_pin = Zeroizing::new(pin);
    let pin_for_session = vault_pin.clone();
    let profile = run_blocking(move || vault::load_fan(&app_data_dir, vault_pin.as_str())).await?;
    *state.fan_session.write().await = Some(profile);
    *state.fan_pin.write().await = Some(pin_for_session);
    drop(_mutation);
    fan_status(state).await
}

#[tauri::command]
async fn fan_lock(state: State<'_, AppState>) -> Result<FanSessionStatus, AppError> {
    let _mutation = state.fan_mutation.lock().await;
    *state.fan_session.write().await = None;
    *state.fan_pin.write().await = None;
    drop(_mutation);
    fan_status(state).await
}

#[tauri::command]
async fn fan_forget(state: State<'_, AppState>) -> Result<FanSessionStatus, AppError> {
    let _mutation = state.fan_mutation.lock().await;
    *state.fan_session.write().await = None;
    *state.fan_pin.write().await = None;
    let app_data_dir = state.app_data_dir.clone();
    run_blocking(move || vault::remove_fan(&app_data_dir)).await?;
    drop(_mutation);
    fan_status(state).await
}

#[tauri::command]
async fn fan_signup(
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
        *state.fan_session.write().await = Some(profile);
        *state.fan_pin.write().await = Some(pin);
    }
    Ok(result)
}

#[tauri::command]
async fn fan_confirm(
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
    *state.fan_session.write().await = Some(profile);
    *state.fan_pin.write().await = Some(pin);
    Ok(result)
}

#[tauri::command]
async fn fan_events(state: State<'_, AppState>) -> Result<Vec<PublicEvent>, AppError> {
    let profile = fan_profile(&state).await?;
    state.api.fan_events(&profile).await
}

#[tauri::command]
async fn fan_referral(state: State<'_, AppState>) -> Result<serde_json::Value, AppError> {
    let profile = fan_profile(&state).await?;
    state.api.fan_referral(&profile).await
}

#[tauri::command]
async fn fan_interests(state: State<'_, AppState>) -> Result<serde_json::Value, AppError> {
    let profile = fan_profile(&state).await?;
    state.api.fan_interests(&profile).await
}

#[tauri::command]
async fn fan_admission_pass(
    state: State<'_, AppState>,
) -> Result<Option<serde_json::Value>, AppError> {
    let _mutation = state.fan_mutation.lock().await;
    let mut profile = fan_profile(&state).await?;
    match state.api.fan_admission_pass(&profile).await {
        Ok(value) => Ok(value),
        Err(AppError::Unauthorized | AppError::NotFound) => {
            profile.pass_session_token = None;
            persist_fan(&state, &profile).await?;
            *state.fan_session.write().await = Some(profile);
            Ok(None)
        }
        Err(error) => Err(error),
    }
}

#[tauri::command]
async fn fan_register_interest(
    state: State<'_, AppState>,
    event_slug: String,
) -> Result<serde_json::Value, AppError> {
    let profile = fan_profile(&state).await?;
    state.api.register_interest(&profile, &event_slug).await
}

#[tauri::command]
async fn fan_claim_pass(
    state: State<'_, AppState>,
    claim_token: String,
) -> Result<serde_json::Value, AppError> {
    let _mutation = state.fan_mutation.lock().await;
    let claim_token = bounded_secret(claim_token, "token wejściówki")?;
    let mut profile = fan_profile(&state).await?;
    let (pass, pass_session_token) = state.api.claim_pass(&profile, claim_token.as_str()).await?;
    profile.pass_session_token = Some(pass_session_token);
    persist_fan(&state, &profile).await?;
    *state.fan_session.write().await = Some(profile);
    Ok(pass)
}

#[tauri::command]
async fn fan_admission_qr(state: State<'_, AppState>) -> Result<serde_json::Value, AppError> {
    let profile = fan_profile(&state).await?;
    let mut value = state.api.admission_qr(&profile).await?;
    attach_single_qr(&mut value)?;
    Ok(value)
}

#[tauri::command]
async fn fan_import_wallet(
    state: State<'_, AppState>,
    order_id: String,
    checkout_token: String,
) -> Result<serde_json::Value, AppError> {
    let _mutation = state.fan_mutation.lock().await;
    let order_id = uuid::Uuid::parse_str(order_id.trim())
        .map(|value| value.to_string())
        .map_err(|_| AppError::InvalidInput("Nieprawidłowy identyfikator zamówienia".into()))?;
    let checkout_token = bounded_secret(checkout_token, "token zamówienia")?;
    let mut profile = fan_profile(&state).await?;
    let already_imported = profile
        .wallets
        .iter()
        .any(|wallet| wallet.order_id == order_id);
    if !already_imported && profile.wallets.len() >= MAX_WALLETS {
        return Err(AppError::InvalidInput(format!(
            "Portfel może zawierać maksymalnie {MAX_WALLETS} zamówienia"
        )));
    }
    let mut wallet = state
        .api
        .ticket_wallet(&profile.api_base_url, &order_id, checkout_token.as_str())
        .await?;
    attach_wallet_qrs(&mut wallet)?;
    profile.wallets.retain(|wallet| wallet.order_id != order_id);
    profile.wallets.push(WalletCredential {
        order_id,
        checkout_token: checkout_token.to_string(),
    });
    persist_fan(&state, &profile).await?;
    *state.fan_session.write().await = Some(profile);
    Ok(wallet)
}

#[tauri::command]
async fn fan_wallets(state: State<'_, AppState>) -> Result<Vec<serde_json::Value>, AppError> {
    let profile = fan_profile(&state).await?;
    let api = state.api.clone();
    let api_base_url = profile.api_base_url.clone();
    let requests = profile.wallets.iter().cloned().map(move |wallet| {
        let api = api.clone();
        let api_base_url = api_base_url.clone();
        async move {
            let mut value = api
                .ticket_wallet(&api_base_url, &wallet.order_id, &wallet.checkout_token)
                .await?;
            attach_wallet_qrs(&mut value)?;
            Ok::<_, AppError>(value)
        }
    });
    stream::iter(requests).buffered(4).try_collect().await
}

#[tauri::command]
async fn fan_request_delivery(
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

fn attach_wallet_qrs(value: &mut serde_json::Value) -> Result<(), AppError> {
    let Some(tickets) = value
        .get_mut("tickets")
        .and_then(serde_json::Value::as_array_mut)
    else {
        return Ok(());
    };
    for ticket in tickets {
        if let Some(token) = ticket.get("qr_token").and_then(serde_json::Value::as_str) {
            let svg = render_qr(token)?;
            ticket["qr_svg"] = serde_json::Value::String(svg);
        }
    }
    Ok(())
}

fn render_qr(token: &str) -> Result<String, AppError> {
    if token.is_empty() || token.len() > MAX_SECRET_BYTES {
        return Err(AppError::InvalidInput("Nieprawidłowy token QR".into()));
    }
    let code = QrCode::new(token.as_bytes())
        .map_err(|_| AppError::InvalidInput("Nie udało się wygenerować kodu QR".into()))?;
    Ok(code
        .render::<svg::Color>()
        .min_dimensions(320, 320)
        .dark_color(svg::Color("#080808"))
        .light_color(svg::Color("#ffffff"))
        .build())
}

async fn operator_profile(state: &State<'_, AppState>) -> Result<OperatorProfile, AppError> {
    state.session.read().await.clone().ok_or(AppError::Locked)
}

async fn fan_profile(state: &State<'_, AppState>) -> Result<FanProfile, AppError> {
    state
        .fan_session
        .read()
        .await
        .clone()
        .ok_or(AppError::Locked)
}

async fn persist_fan(state: &State<'_, AppState>, profile: &FanProfile) -> Result<(), AppError> {
    let pin = state.fan_pin.read().await.clone().ok_or(AppError::Locked)?;
    let app_data_dir = state.app_data_dir.clone();
    let profile = profile.clone();
    run_blocking(move || vault::save_fan(&app_data_dir, pin.as_str(), &profile)).await
}

async fn run_blocking<T, F>(task: F) -> Result<T, AppError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, AppError> + Send + 'static,
{
    tokio::task::spawn_blocking(task)
        .await
        .map_err(|_| AppError::BackgroundTask)?
}

fn validate_operator_profile(profile: &mut OperatorProfile) -> Result<(), AppError> {
    profile.display_name = profile.display_name.trim().to_owned();
    profile.api_base_url = profile.api_base_url.trim().to_owned();
    profile.bearer_token = profile.bearer_token.trim().to_owned();
    if profile.display_name.is_empty() || profile.display_name.chars().count() > 80 {
        return Err(AppError::InvalidInput(
            "Nieprawidłowa nazwa urządzenia".into(),
        ));
    }
    if profile.bearer_token.len() < 24 || profile.bearer_token.len() > 512 {
        return Err(AppError::InvalidInput(
            "Nieprawidłowy token urządzenia".into(),
        ));
    }
    validate_api_base(&profile.api_base_url)
}

fn validate_fan_signup(input: &mut FanSignupInput, pin: &str) -> Result<(), AppError> {
    validate_pin(pin)?;
    input.api_base_url = input.api_base_url.trim().to_owned();
    input.email = input.email.trim().to_ascii_lowercase();
    input.city_slug = input.city_slug.trim().to_owned();
    input.locale = input.locale.trim().to_owned();
    input.policy_version = input.policy_version.trim().to_owned();
    input.display_name = clean_optional(input.display_name.take());
    input.referral_code = clean_optional(input.referral_code.take());
    validate_api_base(&input.api_base_url)?;
    if !valid_email(&input.email)
        || !valid_slug(&input.city_slug)
        || input.locale.is_empty()
        || input.locale.len() > 16
        || input.policy_version.is_empty()
        || input.policy_version.len() > 64
        || input
            .display_name
            .as_ref()
            .is_some_and(|value| value.chars().count() > 100)
        || input
            .referral_code
            .as_ref()
            .is_some_and(|value| value.len() > 128)
    {
        return Err(AppError::InvalidInput(
            "Uzupełnij poprawnie dane fana".into(),
        ));
    }
    Ok(())
}

fn validate_fan_confirmation(input: &mut FanConfirmationInput, pin: &str) -> Result<(), AppError> {
    validate_pin(pin)?;
    input.api_base_url = input.api_base_url.trim().to_owned();
    input.email = input.email.trim().to_ascii_lowercase();
    input.token = input.token.trim().to_owned();
    input.display_name = clean_optional(input.display_name.take());
    validate_api_base(&input.api_base_url)?;
    if !valid_email(&input.email)
        || !(24..=MAX_SECRET_BYTES).contains(&input.token.len())
        || input
            .display_name
            .as_ref()
            .is_some_and(|value| value.chars().count() > 100)
    {
        return Err(AppError::InvalidInput(
            "Nieprawidłowy e-mail lub token".into(),
        ));
    }
    Ok(())
}

fn validate_issue_pass(input: &mut IssuePassInput) -> Result<(), AppError> {
    input.event_slug = input.event_slug.trim().to_owned();
    input.pool_slug = input.pool_slug.trim().to_owned();
    input.fan_email = input.fan_email.trim().to_ascii_lowercase();
    if !valid_slug(&input.event_slug)
        || !valid_slug(&input.pool_slug)
        || !valid_email(&input.fan_email)
        || !(1..=720).contains(&input.claim_expires_hours)
    {
        return Err(AppError::InvalidInput(
            "Nieprawidłowe dane wejściówki".into(),
        ));
    }
    Ok(())
}

fn validate_campaign(input: &mut CreateQrCampaignInput) -> Result<(), AppError> {
    input.event_slug = input.event_slug.trim().to_owned();
    input.label = input.label.trim().to_owned();
    input.valid_from = input.valid_from.trim().to_owned();
    input.valid_until = input.valid_until.trim().to_owned();
    if !valid_slug(&input.event_slug)
        || input.label.is_empty()
        || input.label.chars().count() > 100
        || input.valid_from.len() > 64
        || input.valid_until.len() > 64
        || input.valid_until.as_str() <= input.valid_from.as_str()
        || input.max_checkins == Some(0)
    {
        return Err(AppError::InvalidInput(
            "Nieprawidłowe dane kampanii QR".into(),
        ));
    }
    Ok(())
}

fn validate_pin(pin: &str) -> Result<(), AppError> {
    if (6..=128).contains(&pin.chars().count()) {
        Ok(())
    } else {
        Err(AppError::InvalidInput(
            "PIN musi mieć co najmniej 6 znaków".into(),
        ))
    }
}

fn validate_api_base(value: &str) -> Result<(), AppError> {
    let parsed = url::Url::parse(value.trim())?;
    let allowed_scheme =
        parsed.scheme() == "https" || (cfg!(debug_assertions) && parsed.scheme() == "http");
    if !allowed_scheme {
        return Err(AppError::InvalidInput("API musi używać HTTPS".into()));
    }
    if parsed.host_str().is_none()
        || parsed.username() != ""
        || parsed.password().is_some()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
    {
        return Err(AppError::InvalidInput(
            "Nieprawidłowy bazowy URL API".into(),
        ));
    }
    Ok(())
}

fn valid_email(value: &str) -> bool {
    if value.len() > 320 || value.chars().any(char::is_whitespace) {
        return false;
    }
    let mut parts = value.split('@');
    let (Some(local), Some(domain), None) = (parts.next(), parts.next(), parts.next()) else {
        return false;
    };
    !local.is_empty()
        && local.len() <= 64
        && !domain.starts_with('.')
        && !domain.ends_with('.')
        && domain.split('.').all(|part| !part.is_empty())
        && domain.contains('.')
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

fn valid_slug(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 200
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
}

fn bounded_secret(value: String, label: &str) -> Result<Zeroizing<String>, AppError> {
    let value = Zeroizing::new(value.trim().to_owned());
    if value.is_empty() || value.len() > MAX_SECRET_BYTES {
        Err(AppError::InvalidInput(format!("Nieprawidłowy {label}")))
    } else {
        Ok(value)
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(mobile)]
            app.handle().plugin(tauri_plugin_barcode_scanner::init())?;
            let app_data_dir = app.path().app_local_data_dir()?;
            app.manage(AppState {
                session: RwLock::new(None),
                operator_mutation: Mutex::new(()),
                fan_session: RwLock::new(None),
                fan_pin: RwLock::new(None),
                fan_mutation: Mutex::new(()),
                api: CrowdRelayClient::new()?,
                app_data_dir,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_external_url,
            session_status,
            configure,
            unlock,
            lock,
            forget_device,
            operator_events,
            operator_qr,
            ticketing_overview,
            redeem_admission,
            redeem_coupon,
            issue_pass,
            revoke_pass,
            create_qr_campaign,
            revoke_qr_campaign,
            public_events,
            public_cities,
            fan_status,
            fan_unlock,
            fan_lock,
            fan_forget,
            fan_signup,
            fan_confirm,
            fan_events,
            fan_referral,
            fan_interests,
            fan_admission_pass,
            fan_register_interest,
            fan_claim_pass,
            fan_admission_qr,
            fan_import_wallet,
            fan_wallets,
            fan_request_delivery,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Virya Signal");
}
