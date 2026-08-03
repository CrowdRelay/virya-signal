//! Small commands that don't belong to the operator/fan/show-mode domains.

use tauri::{AppHandle, State};
use tauri_plugin_opener::OpenerExt;

use crate::{
    AppError, AppState,
    models::{RequestedCityInput, RequestedCityResult},
    validation::clean_optional,
};

#[tauri::command]
pub(crate) fn open_external_url(app: AppHandle, url: String) -> Result<(), AppError> {
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
        return Err(AppError::InvalidInput("Nieprawidłowa nazwa miasta".into()));
    }
    state
        .api
        .request_city("https://signal-api.virya.music/v1/", &input)
        .await
}
