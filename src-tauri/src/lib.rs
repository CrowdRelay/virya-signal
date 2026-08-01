#![forbid(unsafe_code)]

mod api;
mod models;
mod vault;

use std::path::PathBuf;

use api::CrowdRelayClient;
use models::{
    CreateQrCampaignInput, DashboardData, FanAuthResult, FanConfirmationInput, FanDashboardData,
    FanProfile, FanSessionStatus, FanSignupInput, IssuePassInput, OperatorProfile, PublicHomeData,
    SessionStatus, WalletCredential,
};
use qrcode::{render::svg, QrCode};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;
use thiserror::Error;
use tokio::sync::RwLock;
use zeroize::Zeroizing;

pub struct AppState {
    session: RwLock<Option<OperatorProfile>>,
    fan_session: RwLock<Option<FanProfile>>,
    fan_pin: RwLock<Option<Zeroizing<String>>>,
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
}

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
    let parsed = url::Url::parse(url.trim())?;
    if parsed.scheme() != "https" || parsed.username() != "" || parsed.password().is_some() {
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
    validate_operator_profile(&mut profile)?;
    state.api.validate(&profile).await?;
    vault::save(&state.app_data_dir, &pin, &profile)?;
    *state.session.write().await = Some(profile);
    session_status(state).await
}

#[tauri::command]
async fn unlock(state: State<'_, AppState>, pin: String) -> Result<SessionStatus, AppError> {
    let profile = vault::load(&state.app_data_dir, &pin)?;
    state.api.validate(&profile).await?;
    *state.session.write().await = Some(profile);
    session_status(state).await
}

#[tauri::command]
async fn lock(state: State<'_, AppState>) -> Result<SessionStatus, AppError> {
    *state.session.write().await = None;
    session_status(state).await
}

#[tauri::command]
async fn forget_device(state: State<'_, AppState>) -> Result<SessionStatus, AppError> {
    *state.session.write().await = None;
    vault::remove(&state.app_data_dir)?;
    session_status(state).await
}

#[tauri::command]
async fn dashboard(state: State<'_, AppState>) -> Result<DashboardData, AppError> {
    let profile = operator_profile(&state).await?;
    state.api.dashboard(&profile).await
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
    input: IssuePassInput,
) -> Result<serde_json::Value, AppError> {
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
    input: CreateQrCampaignInput,
) -> Result<serde_json::Value, AppError> {
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
async fn public_home(
    state: State<'_, AppState>,
    api_base_url: String,
) -> Result<PublicHomeData, AppError> {
    validate_api_base(&api_base_url)?;
    state.api.public_home(&api_base_url).await
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
    let profile = vault::load_fan(&state.app_data_dir, &pin)?;
    state.api.fan_dashboard(&profile).await?;
    *state.fan_session.write().await = Some(profile);
    *state.fan_pin.write().await = Some(Zeroizing::new(pin));
    fan_status(state).await
}

#[tauri::command]
async fn fan_lock(state: State<'_, AppState>) -> Result<FanSessionStatus, AppError> {
    *state.fan_session.write().await = None;
    *state.fan_pin.write().await = None;
    fan_status(state).await
}

#[tauri::command]
async fn fan_forget(state: State<'_, AppState>) -> Result<FanSessionStatus, AppError> {
    *state.fan_session.write().await = None;
    *state.fan_pin.write().await = None;
    vault::remove_fan(&state.app_data_dir)?;
    fan_status(state).await
}

#[tauri::command]
async fn fan_signup(
    state: State<'_, AppState>,
    mut input: FanSignupInput,
    pin: String,
) -> Result<FanAuthResult, AppError> {
    validate_fan_signup(&mut input, &pin)?;
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
        vault::save_fan(&state.app_data_dir, &pin, &profile)?;
        *state.fan_session.write().await = Some(profile);
        *state.fan_pin.write().await = Some(Zeroizing::new(pin));
    }
    Ok(result)
}

#[tauri::command]
async fn fan_confirm(
    state: State<'_, AppState>,
    mut input: FanConfirmationInput,
    pin: String,
) -> Result<FanAuthResult, AppError> {
    validate_fan_confirmation(&mut input, &pin)?;
    let (result, session_token) = state.api.fan_confirm(&input).await?;
    let profile = FanProfile {
        api_base_url: input.api_base_url,
        email: input.email,
        display_name: input.display_name,
        fan_session_token: session_token,
        pass_session_token: None,
        wallets: Vec::new(),
    };
    vault::save_fan(&state.app_data_dir, &pin, &profile)?;
    *state.fan_session.write().await = Some(profile);
    *state.fan_pin.write().await = Some(Zeroizing::new(pin));
    Ok(result)
}

#[tauri::command]
async fn fan_dashboard(state: State<'_, AppState>) -> Result<FanDashboardData, AppError> {
    let profile = fan_profile(&state).await?;
    state.api.fan_dashboard(&profile).await
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
    let mut profile = fan_profile(&state).await?;
    let (pass, pass_session_token) = state.api.claim_pass(&profile, &claim_token).await?;
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
    let mut profile = fan_profile(&state).await?;
    let mut wallet = state
        .api
        .ticket_wallet(&profile.api_base_url, &order_id, &checkout_token)
        .await?;
    attach_wallet_qrs(&mut wallet)?;
    profile.wallets.retain(|wallet| wallet.order_id != order_id);
    profile.wallets.push(WalletCredential {
        order_id,
        checkout_token,
    });
    persist_fan(&state, &profile).await?;
    *state.fan_session.write().await = Some(profile);
    Ok(wallet)
}

#[tauri::command]
async fn fan_wallets(state: State<'_, AppState>) -> Result<Vec<serde_json::Value>, AppError> {
    let profile = fan_profile(&state).await?;
    let mut result = Vec::with_capacity(profile.wallets.len());
    for wallet in &profile.wallets {
        let mut value = state
            .api
            .ticket_wallet(
                &profile.api_base_url,
                &wallet.order_id,
                &wallet.checkout_token,
            )
            .await?;
        attach_wallet_qrs(&mut value)?;
        result.push(value);
    }
    Ok(result)
}

#[tauri::command]
async fn fan_request_delivery(
    state: State<'_, AppState>,
    order_id: String,
) -> Result<serde_json::Value, AppError> {
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
    vault::save_fan(&state.app_data_dir, pin.as_str(), profile)
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
    if !valid_email(&input.email) || input.city_slug.is_empty() || input.policy_version.is_empty() {
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
    if !valid_email(&input.email) || input.token.len() < 24 {
        return Err(AppError::InvalidInput(
            "Nieprawidłowy e-mail lub token".into(),
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
    if parsed.scheme() != "https" && !cfg!(debug_assertions) {
        return Err(AppError::InvalidInput("API musi używać HTTPS".into()));
    }
    if parsed.username() != "" || parsed.password().is_some() {
        return Err(AppError::InvalidInput(
            "API URL nie może zawierać loginu".into(),
        ));
    }
    Ok(())
}

fn valid_email(value: &str) -> bool {
    value.len() <= 320
        && value
            .split_once('@')
            .is_some_and(|(local, domain)| !local.is_empty() && domain.contains('.'))
}

fn clean_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
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
                fan_session: RwLock::new(None),
                fan_pin: RwLock::new(None),
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
            dashboard,
            ticketing_overview,
            redeem_admission,
            redeem_coupon,
            issue_pass,
            revoke_pass,
            create_qr_campaign,
            revoke_qr_campaign,
            public_home,
            fan_status,
            fan_unlock,
            fan_lock,
            fan_forget,
            fan_signup,
            fan_confirm,
            fan_dashboard,
            fan_register_interest,
            fan_claim_pass,
            fan_admission_qr,
            fan_import_wallet,
            fan_wallets,
            fan_request_delivery,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Virya Mobile");
}
