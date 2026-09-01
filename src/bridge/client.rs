
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct RuntimeTranslations {
    native_bridge_unavailable: &'static str,
    operation_timeout: &'static str,
    camera_module_unavailable: &'static str,
    camera_denied: &'static str,
    location_module_unavailable: &'static str,
    location_denied: &'static str,
    location_unavailable: &'static str,
    scanner_label: &'static str,
    scanner_title: &'static str,
    scanner_hint: &'static str,
    scanner_cancel: &'static str,
    scanner_closing: &'static str,
    scanner_unavailable: &'static str,
    unknown_error: &'static str,
    report_type: &'static str,
    report_time: &'static str,
    report_operation: &'static str,
    report_path: &'static str,
    report_error: &'static str,
    diagnostics: &'static str,
    previous_failure: &'static str,
    current_failure: &'static str,
    report_help: &'static str,
    copy_report: &'static str,
    restart: &'static str,
    close: &'static str,
    report_copied: &'static str,
    copy_manually: &'static str,
    interrupted: &'static str,
    unclean_shutdown: &'static str,
}

const DEFAULT_IPC_TIMEOUT_MS: u32 = 30_000;
const MIN_IPC_TIMEOUT_MS: u32 = 2_000;

pub fn native_available() -> bool {
    native_bridge_available_js()
}

pub async fn invoke<T, A>(command: &str, args: &A) -> Result<T, String>
where
    T: DeserializeOwned,
    A: Serialize + ?Sized,
{
    invoke_timeout(command, args, DEFAULT_IPC_TIMEOUT_MS).await
}

pub async fn invoke_timeout<T, A>(command: &str, args: &A, timeout_ms: u32) -> Result<T, String>
where
    T: DeserializeOwned,
    A: Serialize + ?Sized,
{
    let timeout_ms = timeout_ms.max(MIN_IPC_TIMEOUT_MS);
    let args = serde_wasm_bindgen::to_value(args).map_err(|error| error.to_string())?;
    let value = invoke_js(command, args, timeout_ms)
        .await
        .map_err(js_error)?;
    serde_wasm_bindgen::from_value(value).map_err(decode_error)
}

/// Fetches the launcher-time status for both operator and fan sessions in a
/// single native round-trip, cutting startup IPC latency roughly in half.
pub async fn launcher_status() -> Result<Option<crate::models::LauncherStatus>, String> {
    #[derive(Serialize)]
    struct LauncherStatusArgs {
        locale: &'static str,
    }

    invoke_latest::<crate::models::LauncherStatus, _>(
        "launcher_status",
        &LauncherStatusArgs {
            locale: i18n::current().code(),
        },
        10_000,
        "launcher:status",
    )
    .await
}

/// Runs a read request in a named UI scope. Starting a newer request in the
/// same scope makes the older result disappear, preventing stale state writes.
pub async fn invoke_latest<T, A>(
    command: &str,
    args: &A,
    timeout_ms: u32,
    scope: &str,
) -> Result<Option<T>, String>
where
    T: DeserializeOwned,
    A: Serialize + ?Sized,
{
    let timeout_ms = timeout_ms.max(MIN_IPC_TIMEOUT_MS);
    let args = serde_wasm_bindgen::to_value(args).map_err(|error| error.to_string())?;
    let value = invoke_latest_js(command, args, timeout_ms, scope)
        .await
        .map_err(js_error)?;
    if value.is_undefined() {
        Ok(None)
    } else {
        serde_wasm_bindgen::from_value(value)
            .map(Some)
            .map_err(decode_error)
    }
}

pub fn invalidate_latest(prefix: &str) {
    invalidate_latest_js(prefix);
}

pub fn referral_code_from_location() -> Option<String> {
    let value = referral_code_from_location_js();
    let value = value.trim();
    (!value.is_empty()).then(|| value.to_owned())
}

pub fn fan_tab_state() -> String {
    read_fan_tab_js()
}

pub fn set_fan_tab_state(value: &str) {
    write_fan_tab_js(value);
}

pub fn root_mode_state() -> String {
    read_root_mode_js()
}

pub fn set_root_mode_state(value: &str) {
    write_root_mode_js(value);
}

async fn promise_string(promise: js_sys::Promise) -> Result<String, String> {
    wasm_bindgen_futures::JsFuture::from(promise)
        .await
        .map_err(js_error)?
        .as_string()
        .ok_or_else(|| i18n::tr("server_response_has_an_unexpected_format").to_owned())
}

pub async fn copy_text(text: &str) -> Result<(), String> {
    let promise = copy_text_js(text).map_err(js_error)?;
    promise_string(promise).await.map(|_| ())
}

pub async fn share_text(title: &str, text: &str, url: &str) -> Result<String, String> {
    let promise = share_text_js(title, text, url).map_err(js_error)?;
    promise_string(promise).await
}

/// Tactile confirmation for key interactions. On Android WebView this drives
/// the native Vibrator service via `navigator.vibrate()`; on desktop it
/// silently no-ops. Fire-and-forget — never awaited on a UI path.
pub fn haptic(kind: &str) {
    haptic_js(kind);
}

pub async fn invoke_unit<A>(command: &str, args: &A) -> Result<(), String>
where
    A: Serialize + ?Sized,
{
    let args = serde_wasm_bindgen::to_value(args).map_err(|error| error.to_string())?;
    invoke_js(command, args, DEFAULT_IPC_TIMEOUT_MS)
        .await
        .map_err(js_error)?;
    Ok(())
}

pub async fn scan_qr() -> Result<Option<String>, String> {
    const CANCELLED: &str = "__VIRYA_SCAN_CANCELLED__";
    // Debug-only deterministic scanner input for the Android black-box suite.
    // Production artifacts never set this value and release builds compile the
    // branch out entirely, so real users always cross the native camera path.
    #[cfg(debug_assertions)]
    if let Some(value) = option_env!("VIRYA_SIGNAL_E2E_QR_PAYLOAD") {
        let value = value.trim();
        if !value.is_empty() {
            return Ok(Some(value.to_owned()));
        }
    }
    let value = scan_qr_js().await.map_err(js_error)?;
    let value = value
        .as_string()
        .ok_or_else(|| i18n::tr("scanner_returned_no_code").to_owned())?;
    if value == CANCELLED {
        return Ok(None);
    }
    let value = value.trim();
    if value.is_empty() {
        Err(i18n::tr("scanner_returned_no_code").to_owned())
    } else {
        Ok(Some(value.to_owned()))
    }
}

pub async fn scan_and_confirm_fan() -> Result<Option<crate::models::FanSessionStatus>, String> {
    #[cfg(debug_assertions)]
    #[derive(Serialize)]
    struct ScannedTokenArgs<'a> {
        token: &'a str,
    }

    #[cfg(debug_assertions)]
    if let Some(value) = option_env!("VIRYA_SIGNAL_E2E_QR_PAYLOAD") {
        let value = value.trim();
        if !value.is_empty() {
            return invoke::<crate::models::FanSessionStatus, _>(
                "fan_confirm_scanned",
                &ScannedTokenArgs { token: value },
            )
            .await
            .map(Some);
        }
    }

    let value = scan_and_confirm_fan_js().await.map_err(js_error)?;
    if value.is_null() || value.is_undefined() {
        return Ok(None);
    }
    serde_wasm_bindgen::from_value(value)
        .map(Some)
        .map_err(decode_error)
}

pub async fn scan_and_confirm_beacon() -> Result<Option<crate::models::BeaconSessionStatus>, String> {
    #[cfg(debug_assertions)]
    #[derive(Serialize)]
    struct ScannedTokenArgs<'a> {
        token: &'a str,
    }

    #[cfg(debug_assertions)]
    if let Some(value) = option_env!("VIRYA_SIGNAL_E2E_QR_PAYLOAD") {
        let value = value.trim();
        if !value.is_empty() {
            return invoke::<crate::models::BeaconSessionStatus, _>(
                "beacon_confirm_scanned",
                &ScannedTokenArgs { token: value },
            )
            .await
            .map(Some);
        }
    }

    let value = scan_and_confirm_beacon_js().await.map_err(js_error)?;
    if value.is_null() || value.is_undefined() {
        return Ok(None);
    }
    serde_wasm_bindgen::from_value(value)
        .map(Some)
        .map_err(decode_error)
}

pub async fn current_position() -> Result<crate::models::AreaPositionSample, String> {
    let value = current_position_js().await.map_err(js_error)?;
    let value: RawPosition = serde_wasm_bindgen::from_value(value).map_err(decode_error)?;
    validate_position(&value)?;
    Ok(crate::models::AreaPositionSample {
        lat: value.lat,
        lng: value.lng,
        accuracy: value.accuracy,
        captured_at: value.captured_at,
    })
}

pub async fn collect_location_samples(
    min_samples: u32,
    max_samples: u32,
    min_duration_ms: u32,
) -> Result<Vec<crate::models::AreaPositionSample>, String> {
    let value = collect_location_samples_js(min_samples, max_samples, min_duration_ms)
        .await
        .map_err(js_error)?;
    let values: Vec<RawPosition> = serde_wasm_bindgen::from_value(value).map_err(decode_error)?;
    Ok(values
        .into_iter()
        .filter(|value| validate_position(value).is_ok())
        .map(|value| crate::models::AreaPositionSample {
            lat: value.lat,
            lng: value.lng,
            accuracy: value.accuracy,
            captured_at: value.captured_at,
        })
        .collect())
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct RawPosition {
    lat: f64,
    lng: f64,
    accuracy: f64,
    captured_at: u64,
}

fn validate_position(value: &RawPosition) -> Result<(), String> {
    if !value.lat.is_finite() || !(-90.0..=90.0).contains(&value.lat) {
        return Err(i18n::tr("invalid_location_sample").to_owned());
    }
    if !value.lng.is_finite() || !(-180.0..=180.0).contains(&value.lng) {
        return Err(i18n::tr("invalid_location_sample").to_owned());
    }
    if !value.accuracy.is_finite() || value.accuracy < 0.0 {
        return Err(i18n::tr("invalid_location_sample").to_owned());
    }
    Ok(())
}

pub fn install_runtime_guards() {
    let translations = RuntimeTranslations {
        native_bridge_unavailable: i18n::tr("native_app_bridge_is_unavailable"),
        operation_timeout: i18n::tr("operation_command_timed_out"),
        camera_module_unavailable: i18n::tr("camera_permission_module_is_unavailable_in_this"),
        camera_denied: i18n::tr("camera_access_is_denied_enable_camera_for"),
        location_module_unavailable: i18n::tr("location_module_is_unavailable_in_this_app"),
        location_denied: i18n::tr("location_access_is_denied_enable_location_for"),
        location_unavailable: i18n::tr("could_not_read_a_fresh_location_move"),
        scanner_label: i18n::tr("qr_code_scanner"),
        scanner_title: i18n::tr("scan_qr_code_2"),
        scanner_hint: i18n::tr("place_the_code_inside_the_frame"),
        scanner_cancel: i18n::tr("back_cancel_scanning"),
        scanner_closing: i18n::tr("closing"),
        scanner_unavailable: i18n::tr("scanner_is_available_only_in_the_ios"),
        unknown_error: i18n::tr("unknown_application_error"),
        report_type: i18n::tr("type"),
        report_time: i18n::tr("time"),
        report_operation: i18n::tr("operation"),
        report_path: i18n::tr("path"),
        report_error: i18n::tr("bug_label"),
        diagnostics: i18n::tr("virya_signal_diagnostics"),
        previous_failure: i18n::tr("previous_launch_ended_with_an_error"),
        current_failure: i18n::tr("app_caught_an_error"),
        report_help: i18n::tr("we_do_not_hide_failures_copy_the"),
        copy_report: i18n::tr("copy_report"),
        restart: i18n::tr("restart_app"),
        close: i18n::tr("close"),
        report_copied: i18n::tr("report_copied"),
        copy_manually: i18n::tr("press_and_hold_the_report_text_and"),
        interrupted: i18n::tr("previous_launch_interrupted_operation_command"),
        unclean_shutdown: i18n::tr("previous_launch_ended_without_a_clean_shutdown"),
    };
    if let Ok(value) = serde_wasm_bindgen::to_value(&translations) {
        set_runtime_translations_js(value);
    }
    install_runtime_guards_js();
}

fn decode_error(error: serde_wasm_bindgen::Error) -> String {
    let raw = error.to_string();
    if raw.len() > 200 {
        i18n::tr("server_response_has_an_unexpected_format").to_owned()
    } else {
        i18n::format("response_decoding_error_raw", &[raw])
    }
}

fn js_error(value: JsValue) -> String {
    // Tauri commands serialize `AppError` as `{"kind": "...", "message": "..."}`.
    // Embed both in the error string as `kind\x1fmessage` so the UI can
    // classify by kind without substring-matching the translated message.
    // Errors without a `kind` field (JS bridge errors, timeouts) get no prefix.
    if !value.is_null()
        && !value.is_undefined()
        && let (Ok(kind_val), Ok(msg_val)) = (
            js_sys::Reflect::get(&value, &JsValue::from_str("kind")),
            js_sys::Reflect::get(&value, &JsValue::from_str("message")),
        )
        && let (Some(kind), Some(message)) = (kind_val.as_string(), msg_val.as_string())
        && !kind.is_empty()
        && !message.is_empty()
    {
        return format!("{kind}\x1f{message}");
    }
    value
        .as_string()
        .or_else(|| {
            js_sys::Reflect::get(&value, &JsValue::from_str("message"))
                .ok()
                .and_then(|message| message.as_string())
        })
        .or_else(|| {
            js_sys::JSON::stringify(&value)
                .ok()
                .and_then(|v| v.as_string())
        })
        .value_or_else(|| i18n::tr("unknown_application_error").to_owned())
}
