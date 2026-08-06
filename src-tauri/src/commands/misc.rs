//! Small commands that don't belong to the operator/fan/show-mode domains.

use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;
use zeroize::Zeroizing;

use crate::{
    AppError, AppState, i18n,
    models::{
        FanSessionStatus, LauncherStatus, RequestedCityInput, RequestedCityResult, SessionStatus,
    },
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
    state: State<'_, AppState>,
    locale: String,
) -> Result<LauncherStatus, AppError> {
    i18n::set_language(&locale);
    let operator_session = state.session.read().await;
    let fan_session = state.fan_session.read().await;
    Ok(LauncherStatus {
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
        },
    })
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

#[tauri::command]
pub(crate) async fn submit_anonymous_feedback(
    state: State<'_, AppState>,
    category: String,
    message: String,
) -> Result<(), AppError> {
    state
        .api
        .submit_anonymous_feedback(&category, &message)
        .await
}
