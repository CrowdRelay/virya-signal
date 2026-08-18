//! Cross-cutting session-state accessors shared by the operator, fan and
//! show-mode command modules. Kept separate from `AppState` itself (defined
//! in `lib.rs`) so each command module only needs to import the handful of
//! helpers it actually uses.

use std::sync::Arc;

use tauri::State;
use zeroize::Zeroizing;

use crate::{
    AppError, AppState,
    models::{BeaconProfile, FanProfile, OperatorProfile, OperatorSignalOverview},
    vault,
};

/// Runs a CPU/IO-bound closure on the blocking thread pool and converts a
/// panicked or cancelled task into a plain `AppError` instead of propagating
/// a panic (see `docs/ARCHITECTURE.md` and the crate-level panic strategy in
/// the workspace `Cargo.toml`, which relies on this boundary to keep a single
/// failed command from taking down the whole process).
pub(crate) async fn run_blocking<T, F>(task: F) -> Result<T, AppError>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, AppError> + Send + 'static,
{
    tokio::task::spawn_blocking(task)
        .await
        .map_err(|_| AppError::BackgroundTask)?
}

pub(crate) async fn operator_profile(
    state: &State<'_, AppState>,
) -> Result<Arc<OperatorProfile>, AppError> {
    state.session.read().await.clone().ok_or(AppError::Locked)
}

pub(crate) async fn fan_profile(state: &State<'_, AppState>) -> Result<Arc<FanProfile>, AppError> {
    state
        .fan_session
        .read()
        .await
        .clone()
        .ok_or(AppError::Locked)
}

pub(crate) async fn beacon_profile(
    state: &State<'_, AppState>,
) -> Result<Arc<BeaconProfile>, AppError> {
    state
        .beacon_session
        .read()
        .await
        .clone()
        .ok_or(AppError::Locked)
}

pub(crate) async fn operator_vault_password(
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

pub(crate) async fn persist_fan(
    state: &State<'_, AppState>,
    profile: &FanProfile,
) -> Result<(), AppError> {
    let pin = state.fan_pin.read().await.clone().ok_or(AppError::Locked)?;
    let app_data_dir = state.app_data_dir.clone();
    let profile = profile.clone();
    run_blocking(move || vault::save_fan(&app_data_dir, pin.as_str(), &profile)).await
}

pub(crate) async fn persist_beacon(
    state: &State<'_, AppState>,
    profile: &BeaconProfile,
) -> Result<(), AppError> {
    let pin = state
        .beacon_pin
        .read()
        .await
        .clone()
        .ok_or(AppError::Locked)?;
    let app_data_dir = state.app_data_dir.clone();
    let profile = profile.clone();
    run_blocking(move || vault::save_beacon(&app_data_dir, pin.as_str(), &profile)).await
}

pub(crate) async fn persist_operator_signal_cache(
    state: &State<'_, AppState>,
    overview: &OperatorSignalOverview,
) -> Result<(), AppError> {
    let password = operator_vault_password(state).await?;
    let app_data_dir = state.app_data_dir.clone();
    let overview = overview.clone();
    run_blocking(move || {
        vault::save_operator_signal_cache_with_password(&app_data_dir, password.as_ref(), &overview)
    })
    .await
}

pub(crate) async fn load_operator_signal_cache(
    state: &State<'_, AppState>,
) -> Result<Option<OperatorSignalOverview>, AppError> {
    let password = operator_vault_password(state).await?;
    let app_data_dir = state.app_data_dir.clone();
    let overview = run_blocking(move || {
        vault::load_operator_signal_cache_with_password(&app_data_dir, password.as_ref())
    })
    .await?;
    Ok((!overview.generated_at.trim().is_empty()).then_some(overview))
}
