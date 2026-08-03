use reqwest::{
    header::{HeaderMap, SET_COOKIE},
    Response, StatusCode,
};
use serde::de::DeserializeOwned;
use url::Url;
use uuid::Uuid;

use crate::{util::OptionValueOrExt, AppError};

pub(super) const MAX_RESPONSE_BYTES: u64 = 2 * 1024 * 1024;
pub(super) const MAX_TOKEN_BYTES: usize = 4096;

pub(super) async fn decode<T: DeserializeOwned>(mut response: Response) -> Result<T, AppError> {
    let status = response.status();
    let content_length = response.content_length();
    if content_length.is_some_and(|length| length > MAX_RESPONSE_BYTES) {
        return Err(AppError::Remote {
            status: status.as_u16(),
            detail: "Odpowiedź CrowdRelay jest zbyt duża".into(),
        });
    }
    let initial_capacity = content_length.value_or(0).min(MAX_RESPONSE_BYTES) as usize;
    let mut bytes = Vec::with_capacity(initial_capacity);
    while let Some(chunk) = response.chunk().await? {
        if bytes.len().saturating_add(chunk.len()) > MAX_RESPONSE_BYTES as usize {
            return Err(AppError::Remote {
                status: status.as_u16(),
                detail: "Odpowiedź CrowdRelay jest zbyt duża".into(),
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
    let detail = remote_detail(&body);
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
    "CrowdRelay odrzucił operację".into()
}

pub(super) fn endpoint(base: &str, path: &str) -> Result<Url, AppError> {
    let mut base = Url::parse(base.trim())?;
    let allowed_scheme =
        base.scheme() == "https" || (cfg!(debug_assertions) && base.scheme() == "http");
    if !allowed_scheme {
        return Err(AppError::InvalidInput(
            "Produkcyjny API URL musi używać HTTPS".into(),
        ));
    }
    if base.host_str().is_none()
        || base.username() != ""
        || base.password().is_some()
        || base.query().is_some()
        || base.fragment().is_some()
    {
        return Err(AppError::InvalidInput(
            "Nieprawidłowy bazowy URL API".into(),
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
        return Err(AppError::InvalidInput("Nieprawidłowy identyfikator".into()));
    }
    Ok(value.to_owned())
}

pub(super) fn uuid_segment(value: &str) -> Result<String, AppError> {
    Uuid::parse_str(value.trim())
        .map(|value| value.to_string())
        .map_err(|_| AppError::InvalidInput("Nieprawidłowy identyfikator zamówienia".into()))
}

pub(super) fn normalize_scanned_code(value: &str) -> Result<String, AppError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_TOKEN_BYTES {
        return Err(AppError::InvalidInput("Nieprawidłowy kod QR".into()));
    }
    if let Ok(url) = Url::parse(trimmed) {
        if let Some((_, token)) = url.query_pairs().find(|(name, _)| name == "token") {
            return bounded_required(token.as_ref(), "token QR", MAX_TOKEN_BYTES)
                .map(ToOwned::to_owned);
        }
        if let Some(fragment) = url.fragment() {
            if let Some((_, token)) =
                url::form_urlencoded::parse(fragment.as_bytes()).find(|(name, _)| name == "token")
            {
                return bounded_required(token.as_ref(), "token QR", MAX_TOKEN_BYTES)
                    .map(ToOwned::to_owned);
            }
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
        Err(AppError::InvalidInput(format!("Nieprawidłowy {label}")))
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
