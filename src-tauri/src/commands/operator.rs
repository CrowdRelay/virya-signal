//! Operator (owner/staff) session lifecycle and every CrowdRelay command that
//! requires an authenticated operator bearer token.

use std::sync::Arc;

use tauri::State;
use zeroize::Zeroizing;

use crate::{
    models::{
        ConcertQrOverview, CreateQrCampaignInput, IssuePassInput, OperatorOpsOverview,
        OperatorProfile, OperatorSignalOverview, OpsRetryResult, PublicEvent, SessionStatus,
        TicketingOverview,
    },
    session::{operator_profile, run_blocking},
    validation::{
        validate_api_base, validate_campaign, validate_issue_pass, validate_operator_profile,
        validate_pin,
    },
    vault, AppError, AppState,
};

#[tauri::command]
pub(crate) async fn session_status(state: State<'_, AppState>) -> Result<SessionStatus, AppError> {
    let session = state.session.read().await;
    Ok(SessionStatus {
        configured: vault::exists(&state.app_data_dir),
        unlocked: session.is_some(),
        session: session.as_ref().map(|profile| profile.as_ref().into()),
    })
}

#[tauri::command]
pub(crate) async fn configure(
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
pub(crate) async fn unlock(
    state: State<'_, AppState>,
    pin: String,
) -> Result<SessionStatus, AppError> {
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
pub(crate) async fn lock(state: State<'_, AppState>) -> Result<SessionStatus, AppError> {
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
pub(crate) async fn forget_device(state: State<'_, AppState>) -> Result<SessionStatus, AppError> {
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
pub(crate) async fn operator_events(
    state: State<'_, AppState>,
) -> Result<Vec<PublicEvent>, AppError> {
    let profile = operator_profile(&state).await?;
    state.api.operator_events(&profile).await
}

#[tauri::command]
pub(crate) async fn operator_qr(state: State<'_, AppState>) -> Result<ConcertQrOverview, AppError> {
    let profile = operator_profile(&state).await?;
    state.api.operator_qr(&profile).await
}

#[tauri::command]
pub(crate) async fn ticketing_overview(
    state: State<'_, AppState>,
    event_slug: String,
) -> Result<TicketingOverview, AppError> {
    let profile = operator_profile(&state).await?;
    state.api.ticketing_overview(&profile, &event_slug).await
}

#[tauri::command]
pub(crate) async fn redeem_admission(
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
pub(crate) async fn redeem_coupon(
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
pub(crate) async fn issue_pass(
    state: State<'_, AppState>,
    mut input: IssuePassInput,
) -> Result<serde_json::Value, AppError> {
    validate_issue_pass(&mut input)?;
    let profile = operator_profile(&state).await?;
    state.api.issue_pass(&profile, &input).await
}

#[tauri::command]
pub(crate) async fn revoke_pass(
    state: State<'_, AppState>,
    public_reference: String,
) -> Result<serde_json::Value, AppError> {
    let profile = operator_profile(&state).await?;
    state.api.revoke_pass(&profile, &public_reference).await
}

#[tauri::command]
pub(crate) async fn create_qr_campaign(
    state: State<'_, AppState>,
    mut input: CreateQrCampaignInput,
) -> Result<serde_json::Value, AppError> {
    validate_campaign(&mut input)?;
    let profile = operator_profile(&state).await?;
    state.api.create_qr_campaign(&profile, &input).await
}

#[tauri::command]
pub(crate) async fn revoke_qr_campaign(
    state: State<'_, AppState>,
    campaign_id: String,
) -> Result<serde_json::Value, AppError> {
    let profile = operator_profile(&state).await?;
    state.api.revoke_qr_campaign(&profile, &campaign_id).await
}

#[tauri::command]
pub(crate) async fn public_events(
    state: State<'_, AppState>,
    api_base_url: String,
) -> Result<Vec<PublicEvent>, AppError> {
    validate_api_base(&api_base_url)?;
    state.api.public_events(&api_base_url).await
}

#[tauri::command]
pub(crate) async fn public_cities(
    state: State<'_, AppState>,
    api_base_url: String,
) -> Result<String, AppError> {
    validate_api_base(&api_base_url)?;
    let cities = state.api.public_cities(&api_base_url).await?;
    serde_json::to_string(&cities).map_err(AppError::from)
}

#[tauri::command]
pub(crate) async fn operator_signal_overview(
    state: State<'_, AppState>,
) -> Result<OperatorSignalOverview, AppError> {
    let profile = operator_profile(&state).await?;
    state.api.operator_signal_overview(&profile).await
}

#[tauri::command]
pub(crate) async fn operator_ops_overview(
    state: State<'_, AppState>,
) -> Result<OperatorOpsOverview, AppError> {
    let profile = operator_profile(&state).await?;
    state.api.operator_ops_overview(&profile).await
}

#[tauri::command]
pub(crate) async fn operator_retry(
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
