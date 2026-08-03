#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used)]

trait OptionValueOrExt<T> {
    fn value_or(self, fallback: T) -> T;
}

impl<T> OptionValueOrExt<T> for Option<T> {
    #[allow(clippy::manual_unwrap_or)]
    fn value_or(self, fallback: T) -> T {
        match self {
            Some(value) => value,
            None => fallback,
        }
    }
}

mod api;
mod models;
mod vault;

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::Arc,
    time::{SystemTime, UNIX_EPOCH},
};

use api::CrowdRelayClient;
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine as _};
use futures_util::{stream, StreamExt};
use models::{
    AdmissionPass, AreaWallet, ConcertQrOverview, CreateQrCampaignInput, FanAuthResult,
    FanConfirmationInput, FanEventInterest, FanProfile, FanSessionStatus, FanSignupInput,
    IssuePassInput, OperatorOpsOverview, OperatorProfile, OpsRetryResult, PublicEvent,
    ReferralProgress, RequestedCityInput, RequestedCityResult, SessionStatus, ShowModeQueuedScan,
    ShowModeScanResult, ShowModeScanState, ShowModeSession, ShowModeStatus, ShowModeStore,
    ShowModeSyncResult, StaffPairingPayload, TicketWallet, TicketWalletApi, TicketingOverview,
    WalletBatch, WalletCredential, WalletTicket,
};
use qrcode::{render::svg, QrCode};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;
use thiserror::Error;
use time::{format_description::well_known::Rfc3339, OffsetDateTime};
use tokio::sync::{Mutex, RwLock};
use zeroize::Zeroizing;

pub struct AppState {
    session: RwLock<Option<Arc<OperatorProfile>>>,
    operator_pin: RwLock<Option<Zeroizing<String>>>,
    operator_vault_password: RwLock<Option<Zeroizing<Vec<u8>>>>,
    operator_mutation: Mutex<()>,
    show_mode_mutation: Mutex<()>,
    show_mode_store: RwLock<Option<ShowModeStore>>,
    fan_session: RwLock<Option<Arc<FanProfile>>>,
    fan_pin: RwLock<Option<Zeroizing<String>>>,
    fan_mutation: Mutex<()>,
    wallet_qr_tokens: RwLock<HashMap<String, HashMap<String, Zeroizing<String>>>>,
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
const SHOW_MODE_SYNC_CONCURRENCY: usize = 4;

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
async fn request_city(
    state: State<'_, AppState>,
    mut input: RequestedCityInput,
) -> Result<RequestedCityResult, AppError> {
    input.name = input.name.trim().to_owned();
    input.region = clean_optional(input.region.take());
    input.country_code = input.country_code.trim().to_ascii_uppercase();
    if input.name.chars().count() < 2
        || input.name.chars().count() > 120
        || input.name.chars().any(char::is_control)
        || input.country_code.len() != 2
        || !input
            .country_code
            .bytes()
            .all(|byte| byte.is_ascii_uppercase())
    {
        return Err(AppError::InvalidInput("Nieprawidłowa nazwa miasta".into()));
    }
    state
        .api
        .request_city("https://signal-api.virya.music/v1/", &input)
        .await
}

#[tauri::command]
async fn configure_from_pairing(
    state: State<'_, AppState>,
    pin: String,
    payload: String,
) -> Result<SessionStatus, AppError> {
    let pairing = parse_pairing_payload(&payload)?;
    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |value| value.as_secs());
    if pairing.version != 1 || pairing.expires_at < now || pairing.expires_at > now + 1800 {
        return Err(AppError::InvalidInput(
            "Kod parowania wygasł albo jest nieprawidłowy".into(),
        ));
    }
    configure(
        state,
        pin,
        OperatorProfile {
            display_name: pairing.display_name,
            api_base_url: pairing.api_base_url,
            role: pairing.role,
            bearer_token: pairing.bearer_token,
        },
    )
    .await
}

fn parse_pairing_payload(raw: &str) -> Result<StaffPairingPayload, AppError> {
    let raw = raw.trim();
    if raw.is_empty() || raw.len() > 8192 {
        return Err(AppError::InvalidInput("Nieprawidłowy kod parowania".into()));
    }
    if raw.starts_with('{') {
        return serde_json::from_str(raw).map_err(AppError::from);
    }
    let url = url::Url::parse(raw)?;
    if url.scheme() != "virya-signal" || url.host_str() != Some("pair") {
        return Err(AppError::InvalidInput("Nieprawidłowy kod parowania".into()));
    }
    let encoded = url
        .query_pairs()
        .find_map(|(key, value)| (key == "payload").then_some(value.into_owned()))
        .ok_or_else(|| AppError::InvalidInput("Kod parowania nie zawiera danych".into()))?;
    let mut padded = encoded;
    while padded.len() % 4 != 0 {
        padded.push('=');
    }
    let bytes = base64::engine::general_purpose::URL_SAFE
        .decode(padded)
        .map_err(|_| AppError::InvalidInput("Nieprawidłowy kod parowania".into()))?;
    serde_json::from_slice(&bytes).map_err(AppError::from)
}

#[tauri::command]
async fn session_status(state: State<'_, AppState>) -> Result<SessionStatus, AppError> {
    let session = state.session.read().await;
    Ok(SessionStatus {
        configured: vault::exists(&state.app_data_dir),
        unlocked: session.is_some(),
        session: session.as_ref().map(|profile| profile.as_ref().into()),
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
    let vault_pin = pin.clone();
    run_blocking(move || vault::save(&app_data_dir, vault_pin.as_str(), &stored_profile)).await?;
    let password_dir = state.app_data_dir.clone();
    let password_pin = pin.clone();
    let vault_password =
        run_blocking(move || vault::operator_password(&password_dir, password_pin.as_str()))
            .await?;
    let _show_mutation = state.show_mode_mutation.lock().await;
    *state.session.write().await = Some(Arc::new(profile));
    *state.operator_pin.write().await = Some(pin);
    *state.operator_vault_password.write().await = Some(vault_password);
    *state.show_mode_store.write().await = None;
    drop(_show_mutation);
    drop(_mutation);
    session_status(state).await
}

#[tauri::command]
async fn unlock(state: State<'_, AppState>, pin: String) -> Result<SessionStatus, AppError> {
    let _mutation = state.operator_mutation.lock().await;
    validate_pin(&pin)?;
    let app_data_dir = state.app_data_dir.clone();
    let pin = Zeroizing::new(pin);
    let vault_pin = pin.clone();
    let profile = run_blocking(move || vault::load(&app_data_dir, vault_pin.as_str())).await?;
    let password_dir = state.app_data_dir.clone();
    let password_pin = pin.clone();
    let vault_password =
        run_blocking(move || vault::operator_password(&password_dir, password_pin.as_str()))
            .await?;
    let _show_mutation = state.show_mode_mutation.lock().await;
    *state.session.write().await = Some(Arc::new(profile));
    *state.operator_pin.write().await = Some(pin);
    *state.operator_vault_password.write().await = Some(vault_password);
    *state.show_mode_store.write().await = None;
    drop(_show_mutation);
    drop(_mutation);
    session_status(state).await
}

#[tauri::command]
async fn lock(state: State<'_, AppState>) -> Result<SessionStatus, AppError> {
    let _mutation = state.operator_mutation.lock().await;
    let _show_mutation = state.show_mode_mutation.lock().await;
    *state.session.write().await = None;
    *state.operator_pin.write().await = None;
    *state.operator_vault_password.write().await = None;
    *state.show_mode_store.write().await = None;
    drop(_show_mutation);
    drop(_mutation);
    session_status(state).await
}

#[tauri::command]
async fn forget_device(state: State<'_, AppState>) -> Result<SessionStatus, AppError> {
    let _mutation = state.operator_mutation.lock().await;
    let _show_mutation = state.show_mode_mutation.lock().await;
    *state.session.write().await = None;
    *state.operator_pin.write().await = None;
    *state.operator_vault_password.write().await = None;
    *state.show_mode_store.write().await = None;
    let app_data_dir = state.app_data_dir.clone();
    run_blocking(move || vault::remove(&app_data_dir)).await?;
    drop(_show_mutation);
    drop(_mutation);
    session_status(state).await
}

#[tauri::command]
async fn operator_events(state: State<'_, AppState>) -> Result<Vec<PublicEvent>, AppError> {
    let profile = operator_profile(&state).await?;
    state.api.operator_events(&profile).await
}

#[tauri::command]
async fn operator_qr(state: State<'_, AppState>) -> Result<ConcertQrOverview, AppError> {
    let profile = operator_profile(&state).await?;
    state.api.operator_qr(&profile).await
}

#[tauri::command]
async fn ticketing_overview(
    state: State<'_, AppState>,
    event_slug: String,
) -> Result<TicketingOverview, AppError> {
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
) -> Result<String, AppError> {
    validate_api_base(&api_base_url)?;
    let cities = state.api.public_cities(&api_base_url).await?;
    serde_json::to_string(&cities).map_err(AppError::from)
}

#[tauri::command]
async fn fan_status(state: State<'_, AppState>) -> Result<FanSessionStatus, AppError> {
    let session = state.fan_session.read().await;
    Ok(FanSessionStatus {
        configured: vault::fan_exists(&state.app_data_dir),
        unlocked: session.is_some(),
        session: session.as_ref().map(|profile| profile.as_ref().into()),
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
    *state.fan_session.write().await = Some(Arc::new(profile));
    *state.fan_pin.write().await = Some(pin_for_session);
    state.wallet_qr_tokens.write().await.clear();
    drop(_mutation);
    fan_status(state).await
}

#[tauri::command]
async fn fan_lock(state: State<'_, AppState>) -> Result<FanSessionStatus, AppError> {
    let _mutation = state.fan_mutation.lock().await;
    *state.fan_session.write().await = None;
    *state.fan_pin.write().await = None;
    state.wallet_qr_tokens.write().await.clear();
    drop(_mutation);
    fan_status(state).await
}

#[tauri::command]
async fn fan_forget(state: State<'_, AppState>) -> Result<FanSessionStatus, AppError> {
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
async fn operator_ops_overview(
    state: State<'_, AppState>,
) -> Result<OperatorOpsOverview, AppError> {
    let profile = operator_profile(&state).await?;
    state.api.operator_ops_overview(&profile).await
}

#[tauri::command]
async fn operator_retry(
    state: State<'_, AppState>,
    target_kind: String,
    target_id: String,
) -> Result<OpsRetryResult, AppError> {
    let profile = operator_profile(&state).await?;
    state
        .api
        .operator_retry(&profile, target_kind.trim(), target_id.trim())
        .await
}

fn unix_now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |value| value.as_secs())
}

fn parse_snapshot_timestamp(value: &str) -> Result<u64, AppError> {
    let timestamp = OffsetDateTime::parse(value, &Rfc3339)
        .map_err(|_| AppError::InvalidInput("Snapshot ma nieprawidłowy czas".into()))?
        .unix_timestamp();
    u64::try_from(timestamp)
        .map_err(|_| AppError::InvalidInput("Snapshot ma nieprawidłowy czas".into()))
}

fn hash_field(hasher: &mut Sha256, value: &str) {
    hasher.update(value.len().to_be_bytes());
    hasher.update(value.as_bytes());
}

fn show_mode_checksum(snapshot: &models::ShowModeSnapshot) -> String {
    let mut hasher = Sha256::new();
    hash_field(&mut hasher, "crowdrelay/show-mode/v1");
    hash_field(&mut hasher, &snapshot.schema_version.to_string());
    hash_field(&mut hasher, &snapshot.snapshot_id);
    hash_field(&mut hasher, &snapshot.event.slug);
    hash_field(&mut hasher, &snapshot.event.title);
    hash_field(&mut hasher, snapshot.event.venue.as_deref().value_or(""));
    hash_field(&mut hasher, &snapshot.event.starts_at);
    hash_field(&mut hasher, &snapshot.generated_at);
    hash_field(&mut hasher, &snapshot.expires_at);
    // Prepared snapshots are normalized by public_reference once, so every
    // checksum and scan can stream the same stable order without allocating.
    for pass in &snapshot.passes {
        hash_field(&mut hasher, &pass.public_reference);
        hash_field(&mut hasher, pass.holder_name.as_deref().value_or(""));
        hash_field(&mut hasher, &pass.holder_email_masked);
        hash_field(&mut hasher, pass.ticket_type_name.as_deref().value_or(""));
        hash_field(&mut hasher, if pass.offline_eligible { "1" } else { "0" });
        hash_field(&mut hasher, pass.qr_sha256.as_deref().value_or(""));
    }
    hex::encode(hasher.finalize())
}

fn snapshot_is_active(snapshot: &models::ShowModeSnapshot) -> bool {
    parse_snapshot_timestamp(&snapshot.expires_at)
        .is_ok_and(|expires_at| expires_at >= unix_now_secs())
}

fn parse_t1_reference(token: &str) -> Result<String, AppError> {
    let mut parts = token.trim().split('.');
    if parts.next() != Some("t1") {
        return Err(AppError::InvalidInput(
            "Tryb offline obsługuje wyłącznie trwałe bilety t1".into(),
        ));
    }
    let payload = parts
        .next()
        .ok_or_else(|| AppError::InvalidInput("Nieprawidłowy bilet QR".into()))?;
    let signature = parts
        .next()
        .ok_or_else(|| AppError::InvalidInput("Nieprawidłowy bilet QR".into()))?;
    if parts.next().is_some() || signature.len() != 64 {
        return Err(AppError::InvalidInput("Nieprawidłowy bilet QR".into()));
    }
    let bytes = URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|_| AppError::InvalidInput("Nieprawidłowy bilet QR".into()))?;
    if bytes.len() > 512 {
        return Err(AppError::InvalidInput("Nieprawidłowy bilet QR".into()));
    }
    #[derive(serde::Deserialize)]
    struct TicketReferenceClaims {
        #[serde(rename = "r")]
        public_reference: String,
    }
    let claims: TicketReferenceClaims = serde_json::from_slice(&bytes)?;
    let reference = claims.public_reference;
    if reference.is_empty()
        || reference.len() > 64
        || !reference
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err(AppError::InvalidInput("Nieprawidłowy bilet QR".into()));
    }
    Ok(reference)
}

async fn operator_vault_password(
    state: &State<'_, AppState>,
) -> Result<Zeroizing<Vec<u8>>, AppError> {
    state
        .operator_vault_password
        .read()
        .await
        .as_ref()
        .cloned()
        .ok_or(AppError::Locked)
}

async fn ensure_show_store_loaded(state: &State<'_, AppState>) -> Result<(), AppError> {
    if state.show_mode_store.read().await.is_some() {
        return Ok(());
    }
    let password = operator_vault_password(state).await?;
    let app_data_dir = state.app_data_dir.clone();
    let mut store =
        run_blocking(move || vault::load_show_mode_with_password(&app_data_dir, password.as_ref()))
            .await?;
    normalize_show_store(&mut store);
    let mut cached = state.show_mode_store.write().await;
    if cached.is_none() {
        *cached = Some(store);
    }
    Ok(())
}

async fn persist_show_store(state: &State<'_, AppState>) -> Result<(), AppError> {
    let payload = {
        let cached = state.show_mode_store.read().await;
        let store = cached.as_ref().ok_or(AppError::Locked)?;
        Zeroizing::new(serde_json::to_vec(store)?)
    };
    let password = operator_vault_password(state).await?;
    let app_data_dir = state.app_data_dir.clone();
    let result = run_blocking(move || {
        vault::save_show_mode_bytes_with_password(
            &app_data_dir,
            password.as_ref(),
            payload.as_ref(),
        )
    })
    .await;
    if result.is_err() {
        // A failed durable write must not leave a newer memory-only queue that
        // would disappear after a process restart. Reload the last disk state.
        *state.show_mode_store.write().await = None;
    }
    result
}

fn normalize_show_store(store: &mut ShowModeStore) {
    for session in store.sessions.values_mut() {
        session
            .snapshot
            .passes
            .sort_unstable_by(|left, right| left.public_reference.cmp(&right.public_reference));
        session
            .scans
            .sort_unstable_by(|left, right| left.public_reference.cmp(&right.public_reference));
    }
}

fn show_mode_status_for(event_slug: &str, store: &ShowModeStore) -> ShowModeStatus {
    let Some(session) = store.sessions.get(event_slug) else {
        return ShowModeStatus {
            event_slug: event_slug.to_owned(),
            ..ShowModeStatus::default()
        };
    };
    let (pending, synced, conflicts) = session.scans.iter().fold(
        (0_usize, 0_usize, 0_usize),
        |(pending, synced, conflicts), scan| match scan.state {
            ShowModeScanState::Pending => (pending + 1, synced, conflicts),
            ShowModeScanState::Synced => (pending, synced + 1, conflicts),
            ShowModeScanState::Conflict => (pending, synced, conflicts + 1),
        },
    );
    ShowModeStatus {
        prepared: snapshot_is_active(&session.snapshot),
        event_slug: event_slug.to_owned(),
        event_title: Some(session.snapshot.event.title.clone()),
        expires_at: Some(session.snapshot.expires_at.clone()),
        eligible_passes: session
            .snapshot
            .passes
            .iter()
            .filter(|pass| pass.offline_eligible && pass.qr_sha256.is_some())
            .count(),
        pending,
        synced,
        conflicts,
    }
}

#[tauri::command]
async fn show_mode_prepare(
    state: State<'_, AppState>,
    event_slug: String,
) -> Result<ShowModeStatus, AppError> {
    let _mutation = state.show_mode_mutation.lock().await;
    let profile = operator_profile(&state).await?;
    let event_slug = event_slug.trim();
    if event_slug.is_empty() || event_slug.len() > 128 {
        return Err(AppError::InvalidInput("Nieprawidłowy koncert".into()));
    }
    let mut snapshot = state.api.show_mode_snapshot(&profile, event_slug).await?;
    if snapshot.schema_version != 1 || snapshot.event.slug != event_slug {
        return Err(AppError::InvalidInput(
            "CrowdRelay zwrócił niezgodny snapshot koncertu".into(),
        ));
    }
    let generated_at = parse_snapshot_timestamp(&snapshot.generated_at)?;
    let expires_at = parse_snapshot_timestamp(&snapshot.expires_at)?;
    let now = unix_now_secs();
    if generated_at > now.saturating_add(300)
        || expires_at <= now
        || expires_at.saturating_sub(generated_at) > 72 * 60 * 60
    {
        return Err(AppError::InvalidInput(
            "Snapshot koncertu jest nieważny albo wygasł".into(),
        ));
    }
    if snapshot.passes.len() > 10_000 {
        return Err(AppError::InvalidInput(
            "Snapshot przekracza bezpieczny limit 10 000 wejść".into(),
        ));
    }
    snapshot
        .passes
        .sort_unstable_by(|left, right| left.public_reference.cmp(&right.public_reference));
    let checksum = show_mode_checksum(&snapshot);
    if snapshot.checksum_sha256.len() != 64
        || !snapshot.checksum_sha256.eq_ignore_ascii_case(&checksum)
    {
        return Err(AppError::InvalidInput(
            "Snapshot koncertu nie przeszedł kontroli integralności".into(),
        ));
    }
    ensure_show_store_loaded(&state).await?;
    let status = {
        let mut cached = state.show_mode_store.write().await;
        let store = cached.as_mut().ok_or(AppError::Locked)?;
        let mut previous_scans = store
            .sessions
            .remove(event_slug)
            .map_or_else(Vec::new, |session| session.scans);
        previous_scans
            .sort_unstable_by(|left, right| left.public_reference.cmp(&right.public_reference));
        store.sessions.insert(
            event_slug.to_owned(),
            ShowModeSession {
                snapshot,
                scans: previous_scans,
            },
        );
        show_mode_status_for(event_slug, store)
    };
    persist_show_store(&state).await?;
    Ok(status)
}

#[tauri::command]
async fn show_mode_status(
    state: State<'_, AppState>,
    event_slug: String,
) -> Result<ShowModeStatus, AppError> {
    let _mutation = state.show_mode_mutation.lock().await;
    ensure_show_store_loaded(&state).await?;
    let cached = state.show_mode_store.read().await;
    let store = cached.as_ref().ok_or(AppError::Locked)?;
    Ok(show_mode_status_for(event_slug.trim(), store))
}

#[tauri::command]
async fn show_mode_scan(
    state: State<'_, AppState>,
    event_slug: String,
    code: String,
) -> Result<ShowModeScanResult, AppError> {
    let _mutation = state.show_mode_mutation.lock().await;
    let event_slug = event_slug.trim();
    let token = code.trim();
    if token.len() > 4_096 {
        return Err(AppError::InvalidInput("Kod QR jest zbyt długi".into()));
    }
    let reference = parse_t1_reference(token)?;
    let token_hash = hex::encode(Sha256::digest(token.as_bytes()));
    ensure_show_store_loaded(&state).await?;
    let result = {
        let mut cached = state.show_mode_store.write().await;
        let store = cached.as_mut().ok_or(AppError::Locked)?;
        let session = store
            .sessions
            .get_mut(event_slug)
            .ok_or_else(|| AppError::Conflict("Najpierw przygotuj koncert offline".into()))?;
        if !snapshot_is_active(&session.snapshot) {
            return Err(AppError::Conflict(
                "Snapshot koncertu wygasł. Połącz się z siecią i pobierz nowy".into(),
            ));
        }
        let pass_index = session
            .snapshot
            .passes
            .binary_search_by(|pass| pass.public_reference.as_str().cmp(reference.as_str()))
            .map_err(|_| {
                AppError::Conflict("Bilet nie występuje w podpisanym snapshotcie".into())
            })?;
        let pass = &session.snapshot.passes[pass_index];
        if !pass.offline_eligible || pass.qr_sha256.as_deref() != Some(token_hash.as_str()) {
            return Err(AppError::Conflict(
                "Bilet nie występuje w podpisanym snapshotcie".into(),
            ));
        }
        match session
            .scans
            .binary_search_by(|scan| scan.public_reference.as_str().cmp(reference.as_str()))
        {
            Ok(index) => {
                let existing = &session.scans[index];
                return Ok(ShowModeScanResult {
                    accepted: existing.state != ShowModeScanState::Conflict,
                    duplicate: true,
                    public_reference: existing.public_reference.clone(),
                    holder_name: existing.holder_name.clone(),
                    holder_email_masked: existing.holder_email_masked.clone(),
                    state: existing.state.clone(),
                });
            }
            Err(insert_at) => {
                if session.scans.len() >= 10_000 {
                    return Err(AppError::Conflict(
                        "Lokalna kolejka skanów jest pełna".into(),
                    ));
                }
                let queued = ShowModeQueuedScan {
                    scan_id: uuid::Uuid::new_v4().to_string(),
                    public_reference: reference.clone(),
                    holder_name: pass.holder_name.clone(),
                    holder_email_masked: pass.holder_email_masked.clone(),
                    scanned_at_unix_secs: unix_now_secs(),
                    state: ShowModeScanState::Pending,
                    result_status: None,
                };
                let result = ShowModeScanResult {
                    accepted: true,
                    duplicate: false,
                    public_reference: reference,
                    holder_name: queued.holder_name.clone(),
                    holder_email_masked: queued.holder_email_masked.clone(),
                    state: queued.state.clone(),
                };
                session.scans.insert(insert_at, queued);
                result
            }
        }
    };
    persist_show_store(&state).await?;
    Ok(result)
}

#[tauri::command]
async fn show_mode_sync(
    state: State<'_, AppState>,
    event_slug: String,
) -> Result<ShowModeSyncResult, AppError> {
    let _mutation = state.show_mode_mutation.lock().await;
    let profile = operator_profile(&state).await?;
    let event_slug = event_slug.trim().to_owned();
    ensure_show_store_loaded(&state).await?;
    let pending = {
        let cached = state.show_mode_store.read().await;
        let store = cached.as_ref().ok_or(AppError::Locked)?;
        let session = store
            .sessions
            .get(&event_slug)
            .ok_or_else(|| AppError::Conflict("Brak przygotowanego koncertu".into()))?;
        session
            .scans
            .iter()
            .enumerate()
            .filter(|(_, scan)| scan.state == ShowModeScanState::Pending)
            .map(|(index, scan)| (index, scan.public_reference.clone()))
            .collect::<Vec<_>>()
    };
    let api = state.api.clone();
    let outcomes = stream::iter(pending.iter().cloned())
        .map(|(index, reference)| {
            let api = api.clone();
            let profile = Arc::clone(&profile);
            let event_slug = event_slug.clone();
            async move {
                let outcome = api
                    .redeem_admission(profile.as_ref(), &event_slug, &reference)
                    .await;
                (index, outcome)
            }
        })
        .buffer_unordered(SHOW_MODE_SYNC_CONCURRENCY)
        .collect::<Vec<_>>()
        .await;

    let mut result = ShowModeSyncResult {
        attempted: pending.len(),
        ..ShowModeSyncResult::default()
    };
    let mut unexpected = None;
    {
        let mut cached = state.show_mode_store.write().await;
        let store = cached.as_mut().ok_or(AppError::Locked)?;
        let session = store
            .sessions
            .get_mut(&event_slug)
            .ok_or_else(|| AppError::Conflict("Brak przygotowanego koncertu".into()))?;
        for (index, outcome) in outcomes {
            let Some(scan) = session.scans.get_mut(index) else {
                unexpected.get_or_insert(AppError::BackgroundTask);
                continue;
            };
            match outcome {
                Ok(value) => {
                    let status = value
                        .get("status")
                        .and_then(serde_json::Value::as_str)
                        .value_or("redeemed")
                        .to_owned();
                    scan.state = ShowModeScanState::Synced;
                    scan.result_status = Some(status);
                    result.synced += 1;
                }
                Err(
                    AppError::Conflict(_)
                    | AppError::NotFound
                    | AppError::Remote {
                        status: 404 | 409 | 422,
                        ..
                    },
                ) => {
                    scan.state = ShowModeScanState::Conflict;
                    scan.result_status = Some("conflict".into());
                    result.conflicts += 1;
                }
                Err(
                    AppError::Network(_)
                    | AppError::Unauthorized
                    | AppError::Remote {
                        status: 429 | 500..=599,
                        ..
                    },
                ) => {}
                Err(error) => {
                    unexpected.get_or_insert(error);
                }
            }
        }
        result.pending = session
            .scans
            .iter()
            .filter(|scan| scan.state == ShowModeScanState::Pending)
            .count();
    }
    persist_show_store(&state).await?;
    if let Some(error) = unexpected {
        return Err(error);
    }
    Ok(result)
}

#[tauri::command]
async fn show_mode_clear(
    state: State<'_, AppState>,
    event_slug: String,
) -> Result<ShowModeStatus, AppError> {
    let _mutation = state.show_mode_mutation.lock().await;
    let event_slug = event_slug.trim().to_owned();
    ensure_show_store_loaded(&state).await?;
    {
        let mut cached = state.show_mode_store.write().await;
        let store = cached.as_mut().ok_or(AppError::Locked)?;
        store.sessions.remove(&event_slug);
    }
    persist_show_store(&state).await?;
    Ok(ShowModeStatus {
        event_slug,
        ..ShowModeStatus::default()
    })
}

#[tauri::command]
async fn fan_area_wallet(state: State<'_, AppState>) -> Result<AreaWallet, AppError> {
    let profile = fan_profile(&state).await?;
    state.api.fan_area_wallet(&profile).await
}

#[tauri::command]
async fn fan_events(state: State<'_, AppState>) -> Result<Vec<PublicEvent>, AppError> {
    let profile = fan_profile(&state).await?;
    state.api.fan_events(&profile).await
}

#[tauri::command]
async fn fan_referral(state: State<'_, AppState>) -> Result<ReferralProgress, AppError> {
    let profile = fan_profile(&state).await?;
    state.api.fan_referral(&profile).await
}

#[tauri::command]
async fn fan_interests(state: State<'_, AppState>) -> Result<Vec<FanEventInterest>, AppError> {
    let profile = fan_profile(&state).await?;
    state.api.fan_interests(&profile).await
}

#[tauri::command]
async fn fan_admission_pass(state: State<'_, AppState>) -> Result<Option<AdmissionPass>, AppError> {
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
async fn fan_admission_qr(state: State<'_, AppState>) -> Result<serde_json::Value, AppError> {
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
async fn fan_import_wallet(
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
async fn fan_wallets(state: State<'_, AppState>) -> Result<WalletBatch, AppError> {
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
    if wallets.is_empty() {
        if let Some(error) = first_error {
            return Err(error);
        }
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
async fn render_wallet_qr(
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

async fn operator_profile(state: &State<'_, AppState>) -> Result<Arc<OperatorProfile>, AppError> {
    state.session.read().await.clone().ok_or(AppError::Locked)
}

async fn fan_profile(state: &State<'_, AppState>) -> Result<Arc<FanProfile>, AppError> {
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
        || !(25..=500).contains(&input.nearby_radius_km)
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
    std::panic::set_hook(Box::new(|panic_info| {
        eprintln!("[virya:native-panic] {panic_info}");
    }));
    let runtime_result = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            #[cfg(mobile)]
            app.handle().plugin(tauri_plugin_barcode_scanner::init())?;
            let app_data_dir = app.path().app_local_data_dir()?;
            let api = CrowdRelayClient::new(app_data_dir.join("public-cache-v1.json"))?;
            app.manage(AppState {
                session: RwLock::new(None),
                operator_pin: RwLock::new(None),
                operator_vault_password: RwLock::new(None),
                operator_mutation: Mutex::new(()),
                show_mode_mutation: Mutex::new(()),
                show_mode_store: RwLock::new(None),
                fan_session: RwLock::new(None),
                fan_pin: RwLock::new(None),
                fan_mutation: Mutex::new(()),
                wallet_qr_tokens: RwLock::new(HashMap::new()),
                api,
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
            operator_ops_overview,
            operator_retry,
            show_mode_prepare,
            show_mode_status,
            show_mode_scan,
            show_mode_sync,
            show_mode_clear,
            ticketing_overview,
            redeem_admission,
            redeem_coupon,
            issue_pass,
            revoke_pass,
            create_qr_campaign,
            revoke_qr_campaign,
            public_events,
            public_cities,
            request_city,
            configure_from_pairing,
            fan_status,
            fan_unlock,
            fan_lock,
            fan_forget,
            fan_signup,
            fan_confirm,
            fan_events,
            fan_area_wallet,
            fan_referral,
            fan_interests,
            fan_admission_pass,
            fan_register_interest,
            fan_claim_pass,
            fan_admission_qr,
            fan_import_wallet,
            fan_wallets,
            render_wallet_qr,
            fan_request_delivery,
        ])
        .run(tauri::generate_context!());
    if let Err(error) = runtime_result {
        eprintln!("[virya:runtime] application terminated: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn email_validation_rejects_malformed_and_whitespace() {
        assert!(valid_email("fan@example.com"));
        assert!(!valid_email("fan @example.com"));
        assert!(!valid_email("fan@example"));
        assert!(!valid_email("fan@@example.com"));
        assert!(!valid_email("@example.com"));
    }

    #[test]
    fn api_base_rejects_credentials_query_and_fragment() {
        assert!(validate_api_base("https://signal-api.virya.music/v1/").is_ok());
        assert!(validate_api_base("https://user@example.com/v1/").is_err());
        assert!(validate_api_base("https://example.com/v1/?token=secret").is_err());
        assert!(validate_api_base("https://example.com/v1/#fragment").is_err());
    }

    #[test]
    fn pin_limits_use_character_count() {
        assert!(validate_pin("123456").is_ok());
        assert!(validate_pin("ążźćńó").is_ok());
        assert!(validate_pin("12345").is_err());
        assert!(validate_pin(&"x".repeat(129)).is_err());
    }

    #[test]
    fn qr_render_is_bounded_and_produces_svg() {
        let svg = test_value(render_qr("v1.test-token"));
        assert!(svg.starts_with("<svg"));
        assert!(render_qr("").is_err());
        assert!(render_qr(&"x".repeat(MAX_SECRET_BYTES + 1)).is_err());
    }

    #[test]
    fn optional_text_is_trimmed() {
        assert_eq!(
            clean_optional(Some("  Virya  ".into())),
            Some("Virya".into())
        );
        assert_eq!(clean_optional(Some("   ".into())), None);
        assert_eq!(clean_optional(None), None);
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

    fn sample_show_snapshot() -> models::ShowModeSnapshot {
        models::ShowModeSnapshot {
            schema_version: 1,
            snapshot_id: "snapshot-1".into(),
            event: models::ShowModeEvent {
                slug: "virya-live".into(),
                title: "Virya Live".into(),
                venue: Some("Club".into()),
                starts_at: "2026-08-02T18:00:00Z".into(),
            },
            generated_at: "2026-08-02T12:00:00Z".into(),
            expires_at: "2099-08-02T23:00:00Z".into(),
            checksum_sha256: String::new(),
            passes: vec![models::ShowModePass {
                public_reference: "VRY-TICKET-1".into(),
                holder_name: Some("Fan".into()),
                holder_email_masked: "f***@example.com".into(),
                ticket_type_name: Some("Regular".into()),
                offline_eligible: true,
                qr_sha256: Some("ab".repeat(32)),
            }],
        }
    }

    #[test]
    fn durable_t1_parser_extracts_only_bounded_public_reference() {
        let claims = serde_json::json!({"r": "VRY-TICKET-1"});
        let payload = URL_SAFE_NO_PAD.encode(test_value(serde_json::to_vec(&claims)));
        let token = format!("t1.{payload}.{}", "a".repeat(64));
        assert_eq!(test_value(parse_t1_reference(&token)), "VRY-TICKET-1");
        assert!(parse_t1_reference("v1.not-durable").is_err());
        assert!(parse_t1_reference(&format!("t1.{payload}.short")).is_err());
    }

    #[test]
    fn show_snapshot_checksum_is_deterministic_and_content_sensitive() {
        let snapshot = sample_show_snapshot();
        let checksum = show_mode_checksum(&snapshot);
        assert_eq!(checksum, show_mode_checksum(&snapshot));
        let mut changed = snapshot;
        changed.passes[0].holder_name = Some("Other".into());
        assert_ne!(checksum, show_mode_checksum(&changed));
    }

    #[test]
    fn show_store_normalization_sorts_for_binary_search_and_counts_states() {
        let mut snapshot = sample_show_snapshot();
        let mut other = snapshot.passes[0].clone();
        other.public_reference = "VRY-TICKET-0".into();
        let duplicate = snapshot.passes[0].clone();
        snapshot.passes.insert(0, duplicate);
        snapshot.passes[0].public_reference = "VRY-TICKET-2".into();
        snapshot.passes.push(other);
        let mut store = ShowModeStore::default();
        store.sessions.insert(
            "virya-live".into(),
            ShowModeSession {
                snapshot,
                scans: vec![
                    ShowModeQueuedScan {
                        scan_id: "2".into(),
                        public_reference: "VRY-TICKET-2".into(),
                        holder_name: None,
                        holder_email_masked: "—".into(),
                        scanned_at_unix_secs: 2,
                        state: ShowModeScanState::Conflict,
                        result_status: None,
                    },
                    ShowModeQueuedScan {
                        scan_id: "1".into(),
                        public_reference: "VRY-TICKET-1".into(),
                        holder_name: None,
                        holder_email_masked: "—".into(),
                        scanned_at_unix_secs: 1,
                        state: ShowModeScanState::Pending,
                        result_status: None,
                    },
                ],
            },
        );
        normalize_show_store(&mut store);
        let session = &store.sessions["virya-live"];
        assert!(session
            .snapshot
            .passes
            .windows(2)
            .all(|w| w[0].public_reference <= w[1].public_reference));
        assert!(session
            .scans
            .windows(2)
            .all(|w| w[0].public_reference <= w[1].public_reference));
        let status = show_mode_status_for("virya-live", &store);
        assert_eq!((status.pending, status.synced, status.conflicts), (1, 0, 1));
    }
}
