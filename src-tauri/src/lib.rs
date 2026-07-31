#![forbid(unsafe_code)]

mod api;
mod models;
mod vault;

use std::path::PathBuf;

use api::CrowdRelayClient;
use models::{CreateQrCampaignInput, DashboardData, IssuePassInput, OperatorProfile, SessionStatus};
use tauri::{Manager, State};
use thiserror::Error;
use tokio::sync::RwLock;

pub struct AppState {
    session: RwLock<Option<OperatorProfile>>,
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
    #[error("Nieznany błąd aplikacji")]
    Unknown,
}

impl serde::Serialize for AppError {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error> where S: serde::Serializer {
        serializer.serialize_str(&self.to_string())
    }
}

#[tauri::command]
async fn session_status(state: State<'_, AppState>) -> Result<SessionStatus, AppError> {
    let session = state.session.read().await;
    Ok(SessionStatus { configured: vault::exists(&state.app_data_dir), unlocked: session.is_some(), session: session.as_ref().map(Into::into) })
}

#[tauri::command]
async fn configure(state: State<'_, AppState>, pin: String, mut profile: OperatorProfile) -> Result<SessionStatus, AppError> {
    validate_profile(&mut profile)?;
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
    let profile = profile(&state).await?;
    state.api.dashboard(&profile).await
}

#[tauri::command]
async fn ticketing_overview(state: State<'_, AppState>, event_slug: String) -> Result<serde_json::Value, AppError> {
    let profile = profile(&state).await?;
    state.api.ticketing_overview(&profile, &event_slug).await
}

#[tauri::command]
async fn redeem_admission(state: State<'_, AppState>, event_slug: String, code: String) -> Result<serde_json::Value, AppError> {
    let profile = profile(&state).await?;
    state.api.redeem_admission(&profile, &event_slug, &code).await
}

#[tauri::command]
async fn redeem_coupon(state: State<'_, AppState>, code: String, order_reference: String) -> Result<serde_json::Value, AppError> {
    let profile = profile(&state).await?;
    state.api.redeem_coupon(&profile, &code, &order_reference).await
}

#[tauri::command]
async fn issue_pass(state: State<'_, AppState>, input: IssuePassInput) -> Result<serde_json::Value, AppError> {
    let profile = profile(&state).await?;
    state.api.issue_pass(&profile, &input).await
}

#[tauri::command]
async fn revoke_pass(state: State<'_, AppState>, public_reference: String) -> Result<serde_json::Value, AppError> {
    let profile = profile(&state).await?;
    state.api.revoke_pass(&profile, &public_reference).await
}

#[tauri::command]
async fn create_qr_campaign(state: State<'_, AppState>, input: CreateQrCampaignInput) -> Result<serde_json::Value, AppError> {
    let profile = profile(&state).await?;
    state.api.create_qr_campaign(&profile, &input).await
}

#[tauri::command]
async fn revoke_qr_campaign(state: State<'_, AppState>, campaign_id: String) -> Result<serde_json::Value, AppError> {
    let profile = profile(&state).await?;
    state.api.revoke_qr_campaign(&profile, &campaign_id).await
}

async fn profile(state: &State<'_, AppState>) -> Result<OperatorProfile, AppError> {
    state.session.read().await.clone().ok_or(AppError::Locked)
}

fn validate_profile(profile: &mut OperatorProfile) -> Result<(), AppError> {
    profile.display_name = profile.display_name.trim().to_owned();
    profile.api_base_url = profile.api_base_url.trim().to_owned();
    profile.bearer_token = profile.bearer_token.trim().to_owned();
    if profile.display_name.is_empty() || profile.display_name.chars().count() > 80 { return Err(AppError::InvalidInput("Nieprawidłowa nazwa urządzenia".into())); }
    if profile.bearer_token.len() < 24 || profile.bearer_token.len() > 512 { return Err(AppError::InvalidInput("Nieprawidłowy token urządzenia".into())); }
    let parsed = url::Url::parse(&profile.api_base_url)?;
    if parsed.scheme() != "https" && !cfg!(debug_assertions) { return Err(AppError::InvalidInput("API musi używać HTTPS".into())); }
    Ok(())
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            #[cfg(mobile)]
            app.handle().plugin(tauri_plugin_barcode_scanner::init())?;
            let app_data_dir = app.path().app_local_data_dir()?;
            app.manage(AppState { session: RwLock::new(None), api: CrowdRelayClient::new()?, app_data_dir });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            session_status, configure, unlock, lock, forget_device, dashboard,
            ticketing_overview, redeem_admission, redeem_coupon, issue_pass, revoke_pass,
            create_qr_campaign, revoke_qr_campaign,
        ])
        .run(tauri::generate_context!())
        .expect("failed to run Virya Control");
}
