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

/// Returns the current unix timestamp in seconds, or 0 if the clock is before
/// the epoch (which would only make every expiry check pass trivially).
fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Runs a CPU/IO-bound closure on the blocking thread pool and converts a
/// panicked or cancelled task into a plain `AppError` instead of propagating
/// a panic (see the crate-level panic strategy in
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
    let session = state.session.read().await.clone();
    let profile = session.as_ref().ok_or(AppError::Locked)?;
    // The client has the expiry and should act on it: an expired staff session
    // keeps making requests until CrowdRelay 401s otherwise. The backend is
    // still authoritative, so this is a client-side optimization that avoids
    // sending a bearer the server has already invalidated. Lock the session
    // (clear credentials) so the operator is returned to the unlock screen
    // instead of receiving opaque 401s on every subsequent command.
    if let Some(expires_at) = profile.session_expires_at
        && expires_at <= unix_now()
    {
        drop(session);
        *state.session.write().await = None;
        *state.operator_pin.write().await = None;
        *state.operator_vault_password.write().await = None;
        *state.show_mode_store.write().await = None;
        *state.operator_sections_cache.write().await = None;
        return Err(AppError::Locked);
    }
    Ok(profile.clone())
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
    let app_data_dir = state.app_data_dir.clone();
    let profile = profile.clone();
    // Reuse the password derived at unlock. Falling back to the PIN re-runs
    // Argon2, which costs more than the snapshot write itself.
    if let Some(password) = state.fan_vault_password.read().await.as_ref().cloned() {
        run_blocking(move || {
            vault::save_fan_with_password(&app_data_dir, password.as_ref(), &profile)
        })
        .await?;
        return Ok(());
    }
    let pin = state.fan_pin.read().await.clone().ok_or(AppError::Locked)?;
    run_blocking(move || vault::save_fan(&app_data_dir, pin.as_str(), &profile)).await?;
    Ok(())
}

pub(crate) async fn persist_beacon(
    state: &State<'_, AppState>,
    profile: &BeaconProfile,
) -> Result<(), AppError> {
    let app_data_dir = state.app_data_dir.clone();
    let profile = profile.clone();
    // Reuse the password derived at unlock. Falling back to the PIN re-runs
    // Argon2, which costs more than the snapshot write itself.
    if let Some(password) = state.beacon_vault_password.read().await.as_ref().cloned() {
        return run_blocking(move || {
            vault::save_beacon_with_password(&app_data_dir, password.as_ref(), &profile)
        })
        .await;
    }
    let pin = state
        .beacon_pin
        .read()
        .await
        .clone()
        .ok_or(AppError::Locked)?;
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
