use reqwest::{
    Response, StatusCode,
    header::{HeaderMap, SET_COOKIE},
};
use serde::de::DeserializeOwned;
use url::Url;
use uuid::Uuid;

use crate::{AppError, util::OptionValueOrExt};

pub(super) const MAX_RESPONSE_BYTES: u64 = 2 * 1024 * 1024;
pub(super) const MAX_TOKEN_BYTES: usize = 4096;

pub(super) async fn decode<T: DeserializeOwned>(response: Response) -> Result<T, AppError> {
    decode_with_error_mapper(response, |_| None).await
}

pub(super) async fn decode_with_error_mapper<T: DeserializeOwned>(
    mut response: Response,
    error_mapper: fn(&serde_json::Value) -> Option<String>,
) -> Result<T, AppError> {
    let status = response.status();
    let request_id = response
        .headers()
        .get("x-request-id")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.chars().take(24).collect::<String>());
    let release = response
        .headers()
        .get("x-crowdrelay-release")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.chars().take(40).collect::<String>());
    let server_timing = response
        .headers()
        .get("server-timing")
        .and_then(|value| value.to_str().ok())
        .map(|value| value.chars().take(96).collect::<String>());
    let content_length = response.content_length();
    if content_length.is_some_and(|length| length > MAX_RESPONSE_BYTES) {
        return Err(AppError::Remote {
            status: status.as_u16(),
            detail: crate::i18n::tr("native_response_too_large").into(),
        });
    }
    let initial_capacity = content_length.value_or(0).min(MAX_RESPONSE_BYTES) as usize;
    let mut bytes = Vec::with_capacity(initial_capacity);
    while let Some(chunk) = response.chunk().await? {
        if bytes.len().saturating_add(chunk.len()) > MAX_RESPONSE_BYTES as usize {
            return Err(AppError::Remote {
                status: status.as_u16(),
                detail: crate::i18n::tr("native_response_too_large").into(),
            });
        }
        bytes.extend_from_slice(&chunk);
    }
    if status.is_success() {
        return serde_json::from_slice(if bytes.is_empty() {
            b"null".as_slice()
        } else {
            &bytes
        })
        .map_err(AppError::from);
    }
    let body = match serde_json::from_slice(&bytes) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("[virya:http] error response was not valid JSON: {error}");
            serde_json::Value::Null
        }
    };
    let mut detail = error_mapper(&body).unwrap_or_else(|| remote_detail(&body));
    if let Some(ref request_id) = request_id {
        detail.push_str(&format!(" · ref {request_id}"));
    }
    eprintln!(
        "[virya:http] status={} ref={} release={} timing={}",
        status.as_u16(),
        request_id.as_deref().unwrap_or("-"),
        release.as_deref().unwrap_or("-"),
        server_timing.as_deref().unwrap_or("-"),
    );
    match status {
        StatusCode::UNAUTHORIZED => Err(AppError::Unauthorized),
        StatusCode::FORBIDDEN => Err(AppError::Forbidden),
        StatusCode::CONFLICT => Err(AppError::Conflict(detail)),
        StatusCode::NOT_FOUND => Err(AppError::NotFound),
        StatusCode::UNPROCESSABLE_ENTITY | StatusCode::BAD_REQUEST => {
            Err(AppError::InvalidInput(detail))
        }
        _ => Err(AppError::Remote {
            status: status.as_u16(),
            detail,
        }),
    }
}

fn remote_detail(body: &serde_json::Value) -> String {
    for key in ["detail", "message", "error"] {
        if let Some(value) = body.get(key) {
            if let Some(message) = value.as_str() {
                return message.chars().take(500).collect();
            }
            if let Some(items) = value.as_array() {
                let messages = items
                    .iter()
                    .filter_map(|item| item.get("msg").and_then(serde_json::Value::as_str))
                    .take(3)
                    .collect::<Vec<_>>()
                    .join(", ");
                if !messages.is_empty() {
                    return messages.chars().take(500).collect();
                }
            }
        }
    }
    crate::i18n::tr("native_operation_rejected").into()
}

pub(super) fn endpoint(base: &str, path: &str) -> Result<Url, AppError> {
    let mut base = Url::parse(base.trim())?;
    let allowed_scheme =
        base.scheme() == "https" || (cfg!(debug_assertions) && base.scheme() == "http");
    if !allowed_scheme {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_production_api_https").into(),
        ));
    }
    if base.host_str().is_none()
        || base.username() != ""
        || base.password().is_some()
        || base.query().is_some()
        || base.fragment().is_some()
    {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_invalid_api_base_url").into(),
        ));
    }
    if !base.path().ends_with('/') {
        base.set_path(&format!("{}/", base.path()));
    }
    base.join(path).map_err(AppError::from)
}

pub(super) fn segment(value: &str) -> Result<String, AppError> {
    let value = value.trim();
    if value.is_empty()
        || value.len() > 200
        || !value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_invalid_identifier").into(),
        ));
    }
    Ok(value.to_owned())
}

pub(super) fn uuid_segment(value: &str) -> Result<String, AppError> {
    Uuid::parse_str(value.trim())
        .map(|value| value.to_string())
        .map_err(|_| AppError::InvalidInput(crate::i18n::tr("native_invalid_order_id").into()))
}

pub(super) fn normalize_scanned_code(value: &str) -> Result<String, AppError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_TOKEN_BYTES {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_invalid_qr_code").into(),
        ));
    }
    if let Ok(url) = Url::parse(trimmed) {
        if let Some((_, token)) = url.query_pairs().find(|(name, _)| name == "token") {
            return bounded_required(
                token.as_ref(),
                crate::i18n::tr("native_qr_token_label"),
                MAX_TOKEN_BYTES,
            )
            .map(ToOwned::to_owned);
        }
        if let Some(fragment) = url.fragment()
            && let Some((_, token)) =
                url::form_urlencoded::parse(fragment.as_bytes()).find(|(name, _)| name == "token")
        {
            return bounded_required(
                token.as_ref(),
                crate::i18n::tr("native_qr_token_label"),
                MAX_TOKEN_BYTES,
            )
            .map(ToOwned::to_owned);
        }
    }
    Ok(trimmed.to_owned())
}

pub(super) fn response_cookie(headers: &HeaderMap, expected_name: &str) -> Option<String> {
    headers
        .get_all(SET_COOKIE)
        .iter()
        .filter_map(|header| header.to_str().ok())
        .filter_map(|header| header.split(';').next())
        .filter_map(|pair| pair.trim().split_once('='))
        .find_map(|(name, value)| {
            (name == expected_name && !value.is_empty() && value.len() <= MAX_TOKEN_BYTES)
                .then(|| value.to_owned())
        })
}

pub(super) fn bounded_required<'a>(
    value: &'a str,
    label: &str,
    max: usize,
) -> Result<&'a str, AppError> {
    let value = value.trim();
    if value.is_empty() || value.len() > max {
        Err(AppError::InvalidInput(crate::i18n::replace(
            "native_invalid_label",
            &[("label", label.to_owned())],
        )))
    } else {
        Ok(value)
    }
}

pub(super) fn normalized_optional(value: &Option<String>) -> Option<String> {
    value
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

pub(super) fn require_owner(profile: &crate::models::OperatorProfile) -> Result<(), AppError> {
    if profile.role == crate::models::OperatorRole::Owner {
        Ok(())
    } else {
        Err(AppError::Forbidden)
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
    fn extracts_fragment_token() {
        assert_eq!(
            test_value(normalize_scanned_code(
                "https://virya.music/win#token=v1.abc"
            )),
            "v1.abc"
        );
    }

    #[test]
    fn extracts_token_after_another_query_parameter() {
        assert_eq!(
            test_value(normalize_scanned_code(
                "https://virya.music/win?source=app&token=t1.abc"
            )),
            "t1.abc"
        );
    }

    #[test]
    fn leaves_manual_reference() {
        assert_eq!(test_value(normalize_scanned_code(" VRY-ABCD ")), "VRY-ABCD");
    }

    #[test]
    fn rejects_non_uuid_order_id() {
        assert!(uuid_segment("order-1").is_err());
    }

    #[test]
    fn rejects_credentialed_or_fragmented_api_base() {
        assert!(endpoint("https://user@example.com/v1", "events").is_err());
        assert!(endpoint("https://example.com/v1#fragment", "events").is_err());
    }
}
