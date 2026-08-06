//! Native-side input validation and normalization. Every `#[tauri::command]`
//! that accepts webview input runs its payload through one of these
//! functions before it reaches `CrowdRelayClient` or the vault, so untrusted
//! strings never leak past this boundary un-trimmed or unbounded.

use zeroize::Zeroizing;

use crate::{
    AppError, MAX_SECRET_BYTES,
    models::{
        CreateQrCampaignInput, FanConfirmationInput, FanSignupInput, IssuePassInput,
        OperatorProfile,
    },
};

pub(crate) fn validate_operator_profile(profile: &mut OperatorProfile) -> Result<(), AppError> {
    profile.display_name = profile.display_name.trim().to_owned();
    profile.api_base_url = profile.api_base_url.trim().to_owned();
    profile.bearer_token = profile.bearer_token.trim().to_owned();
    if profile.display_name.is_empty() || profile.display_name.chars().count() > 80 {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_invalid_device_name").into(),
        ));
    }
    if profile.bearer_token.len() < 24 || profile.bearer_token.len() > 512 {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_invalid_device_token").into(),
        ));
    }
    validate_api_base(&profile.api_base_url)
}

pub(crate) fn validate_fan_signup(input: &mut FanSignupInput, pin: &str) -> Result<(), AppError> {
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
            crate::i18n::tr("native_complete_fan_data").into(),
        ));
    }
    Ok(())
}

fn normalized_fan_confirmation_token(value: &str) -> Option<String> {
    fn exact_hex(value: &str) -> Option<String> {
        let value = value.trim();
        (value.len() == 64 && value.bytes().all(|byte| byte.is_ascii_hexdigit()))
            .then(|| value.to_ascii_lowercase())
    }

    let value = value.trim();
    if let Some(token) = exact_hex(value) {
        return Some(token);
    }
    for prefix in ["token=", "#token="] {
        if let Some(token) = value.strip_prefix(prefix).and_then(exact_hex) {
            return Some(token);
        }
    }
    let url = url::Url::parse(value).ok()?;
    if let Some((_, token)) = url.query_pairs().find(|(name, _)| name == "token")
        && let Some(token) = exact_hex(token.as_ref())
    {
        return Some(token);
    }
    if let Some(fragment) = url.fragment()
        && let Some((_, token)) =
            url::form_urlencoded::parse(fragment.as_bytes()).find(|(name, _)| name == "token")
        && let Some(token) = exact_hex(token.as_ref())
    {
        return Some(token);
    }
    None
}

pub(crate) fn validate_fan_confirmation(
    input: &mut FanConfirmationInput,
    pin: &str,
) -> Result<(), AppError> {
    validate_pin(pin)?;
    input.api_base_url = input.api_base_url.trim().to_owned();
    input.email = input.email.trim().to_ascii_lowercase();
    input.token = normalized_fan_confirmation_token(&input.token)
        .ok_or_else(|| AppError::InvalidInput(crate::i18n::tr("native_paste_valid_code").into()))?;
    input.display_name = clean_optional(input.display_name.take());
    validate_api_base(&input.api_base_url)?;
    if !valid_email(&input.email)
        || input
            .display_name
            .as_ref()
            .is_some_and(|value| value.chars().count() > 100)
    {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_invalid_email_or_token").into(),
        ));
    }
    Ok(())
}

pub(crate) fn validate_issue_pass(input: &mut IssuePassInput) -> Result<(), AppError> {
    input.event_slug = input.event_slug.trim().to_owned();
    input.pool_slug = input.pool_slug.trim().to_owned();
    input.fan_email = input.fan_email.trim().to_ascii_lowercase();
    if !valid_slug(&input.event_slug)
        || !valid_slug(&input.pool_slug)
        || !valid_email(&input.fan_email)
        || !(1..=720).contains(&input.claim_expires_hours)
    {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_invalid_pass_data").into(),
        ));
    }
    Ok(())
}

pub(crate) fn validate_campaign(input: &mut CreateQrCampaignInput) -> Result<(), AppError> {
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
            crate::i18n::tr("native_invalid_qr_campaign_data").into(),
        ));
    }
    Ok(())
}

/// New operator PINs are intentionally short, numeric and usable at the gate.
/// `validate_pin` remains broader for unlocking profiles created by older builds.
pub(crate) fn validate_new_operator_pin(pin: &str) -> Result<(), AppError> {
    let length = pin.len();
    if (4..=6).contains(&length) && pin.bytes().all(|byte| byte.is_ascii_digit()) {
        Ok(())
    } else {
        Err(AppError::InvalidInput(
            crate::i18n::tr("native_operator_pin_4_6").into(),
        ))
    }
}

pub(crate) fn validate_pin(pin: &str) -> Result<(), AppError> {
    if (4..=128).contains(&pin.chars().count()) {
        Ok(())
    } else {
        Err(AppError::InvalidInput(
            crate::i18n::tr("native_pin_min_4").into(),
        ))
    }
}

pub(crate) fn validate_api_base(value: &str) -> Result<(), AppError> {
    let parsed = url::Url::parse(value.trim())?;
    let allowed_scheme =
        parsed.scheme() == "https" || (cfg!(debug_assertions) && parsed.scheme() == "http");
    if !allowed_scheme {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_api_must_use_https").into(),
        ));
    }
    if parsed.host_str().is_none()
        || parsed.username() != ""
        || parsed.password().is_some()
        || parsed.query().is_some()
        || parsed.fragment().is_some()
    {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_invalid_api_base_url").into(),
        ));
    }
    Ok(())
}

pub(crate) fn valid_email(value: &str) -> bool {
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

pub(crate) fn clean_optional(value: Option<String>) -> Option<String> {
    value
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
}

pub(crate) fn valid_slug(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 200
        && value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
}

pub(crate) fn bounded_secret(value: String, label: &str) -> Result<Zeroizing<String>, AppError> {
    let value = Zeroizing::new(value.trim().to_owned());
    if value.is_empty() || value.len() > MAX_SECRET_BYTES {
        Err(AppError::InvalidInput(crate::i18n::replace(
            "native_invalid_label",
            &[("label", label.to_owned())],
        )))
    } else {
        Ok(value)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn new_operator_pin_accepts_four_to_six_ascii_digits() {
        assert!(validate_new_operator_pin("1234").is_ok());
        assert!(validate_new_operator_pin("123456").is_ok());
        assert!(validate_new_operator_pin("123").is_err());
        assert!(validate_new_operator_pin("1234567").is_err());
        assert!(validate_new_operator_pin("12a4").is_err());
        assert!(validate_new_operator_pin("１２３４").is_err());
    }

    #[test]
    fn legacy_pin_limits_use_character_count() {
        assert!(validate_pin("1234").is_ok());
        assert!(validate_pin("123456").is_ok());
        assert!(validate_pin("ążźćńó").is_ok());
        assert!(validate_pin("123").is_err());
        assert!(validate_pin(&"x".repeat(129)).is_err());
    }

    #[test]
    fn confirmation_accepts_raw_token_and_urls() {
        let token = "A".repeat(64);
        assert_eq!(
            normalized_fan_confirmation_token(&token),
            Some("a".repeat(64))
        );
        assert_eq!(
            normalized_fan_confirmation_token(&format!(
                "https://virya.music/signal/confirm#token={token}"
            )),
            Some("a".repeat(64))
        );
        assert_eq!(
            normalized_fan_confirmation_token(&format!(
                "virya-signal://fan/confirm?source=mail&token={token}"
            )),
            Some("a".repeat(64))
        );
        assert_eq!(normalized_fan_confirmation_token("not-a-token"), None);
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
}
