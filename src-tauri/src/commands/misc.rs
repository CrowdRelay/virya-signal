//! Small commands that don't belong to the operator/fan/show-mode domains.

use tauri::{AppHandle, Manager, State};
use tauri_plugin_opener::OpenerExt;
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::{
    AppError, AppState, feedback_queue, i18n,
    models::{
        BeaconSessionStatus, FanSessionStatus, LauncherStatus, RequestedCityInput,
        RequestedCityResult, SessionStatus,
    },
    session::run_blocking,
    validation::clean_optional,
    vault,
};

#[tauri::command]
pub(crate) fn open_external_url(app: AppHandle, url: String) -> Result<(), AppError> {
    let url = url.trim();
    if url.len() > 2048 {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_link_too_long").into(),
        ));
    }
    let parsed = url::Url::parse(url)?;
    if parsed.scheme() != "https"
        || parsed.host_str().is_none()
        || parsed.username() != ""
        || parsed.password().is_some()
    {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_https_links_only").into(),
        ));
    }
    app.opener()
        .open_url(parsed.as_str(), None::<&str>)
        .map_err(|error| {
            AppError::InvalidInput(crate::i18n::replace(
                "native_open_link_failed",
                &[("error", error.to_string())],
            ))
        })
}

#[tauri::command]
pub(crate) async fn request_city(
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
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_city_name_invalid").into(),
        ));
    }
    state
        .api
        .request_city("https://signal-api.virya.music/v1/", &input)
        .await
}

#[tauri::command]
pub(crate) async fn launcher_status(
    app: AppHandle,
    state: State<'_, AppState>,
    locale: String,
) -> Result<LauncherStatus, AppError> {
    i18n::set_language(&locale);
    let operator_session = state.session.read().await;
    let fan_session = state.fan_session.read().await;
    let beacon_session = state.beacon_session.read().await;
    let fan_unlock_mode = crate::device_unlock::effective_mode(&state).await;
    let status = LauncherStatus {
        operator: SessionStatus {
            configured: vault::exists(&state.app_data_dir),
            unlocked: operator_session.is_some(),
            session: operator_session
                .as_ref()
                .map(|profile| profile.as_ref().into()),
        },
        fan: FanSessionStatus {
            configured: vault::fan_exists(&state.app_data_dir),
            unlocked: fan_session.is_some(),
            session: fan_session.as_ref().map(|profile| profile.as_ref().into()),
            pin_unlock: fan_unlock_mode.pin,
            device_unlock: fan_unlock_mode.device,
            device_unlock_supported: state.device_unlock_supported,
        },
        beacon: BeaconSessionStatus {
            configured: vault::beacon_exists(&state.app_data_dir),
            unlocked: beacon_session.is_some(),
            session: beacon_session
                .as_ref()
                .map(|profile| profile.as_ref().into()),
        },
    };
    drop(operator_session);
    drop(fan_session);
    drop(beacon_session);

    // Anonymous feedback is maintenance work. A queued retry can involve disk
    // I/O plus up to three network requests and must never delay first render.
    tauri::async_runtime::spawn(async move {
        let state = app.state::<AppState>();
        flush_feedback_outbox(&state).await;
    });

    Ok(status)
}

#[tauri::command]
pub(crate) async fn verify_staff_access(
    state: State<'_, AppState>,
    password: String,
) -> Result<(), AppError> {
    if password.is_empty() || password.len() > 256 {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_enter_valid_staff_password").to_owned(),
        ));
    }
    let password = Zeroizing::new(password);
    state.api.verify_staff_access(password.as_str()).await
}

/// Delivery outcome of one anonymous submission.
///
/// `"queued"` is not `"sent"`. A retryable failure parks the submission in the
/// on-disk outbox and a later launch flushes it, which is worth doing — but the
/// server has not accepted anything yet, and telling the fan it has is the app
/// inventing an outcome CrowdRelay never gave it.
const FEEDBACK_SENT: &str = "sent";
const FEEDBACK_QUEUED: &str = "queued";

#[tauri::command]
pub(crate) async fn submit_anonymous_feedback(
    state: State<'_, AppState>,
    category: String,
    message: String,
) -> Result<String, AppError> {
    let submission_id = Uuid::new_v4().to_string();
    match state
        .api
        .submit_anonymous_feedback(&submission_id, &category, &message)
        .await
    {
        Ok(()) => Ok(FEEDBACK_SENT.to_owned()),
        Err(error) if feedback_retryable(&error) => {
            let _queue = state.feedback_queue_mutation.lock().await;
            let dir = state.app_data_dir.clone();
            let queued = feedback_queue::QueuedFeedback {
                submission_id,
                category,
                message,
                queued_at_unix: time::OffsetDateTime::now_utc().unix_timestamp(),
            };
            run_blocking(move || feedback_queue::enqueue(&dir, queued)).await?;
            Ok(FEEDBACK_QUEUED.to_owned())
        }
        Err(error) => Err(error),
    }
}

fn feedback_retryable(error: &AppError) -> bool {
    matches!(error, AppError::Network(_))
        || matches!(error, AppError::Remote { status, .. } if *status == 429 || *status >= 500)
}

async fn flush_feedback_outbox(state: &State<'_, AppState>) {
    let _queue = state.feedback_queue_mutation.lock().await;
    let dir = state.app_data_dir.clone();
    let mut queued = match run_blocking(move || feedback_queue::load(&dir)).await {
        Ok(values) => values,
        Err(_) => return,
    };
    if queued.is_empty() {
        return;
    }
    let mut delivered = 0usize;
    for item in queued.iter().take(3) {
        match state
            .api
            .submit_anonymous_feedback(&item.submission_id, &item.category, &item.message)
            .await
        {
            Ok(()) => delivered += 1,
            Err(error) if feedback_retryable(&error) => break,
            Err(_) => delivered += 1,
        }
    }
    if delivered == 0 {
        return;
    }
    queued.drain(0..delivered.min(queued.len()));
    let dir = state.app_data_dir.clone();
    let _ = run_blocking(move || feedback_queue::save(&dir, &queued)).await;
}
