use std::time::Duration;

use reqwest::{
    header::{HeaderMap, ACCEPT, COOKIE, SET_COOKIE},
    Client, Method, RequestBuilder, Response, StatusCode,
};
use serde::{de::DeserializeOwned, Serialize};
use url::Url;
use uuid::Uuid;

use crate::{
    models::{
        CityListResponse, CitySignal, CreateQrCampaignInput, EventListResponse, FanAuthResult,
        FanConfirmationInput, FanProfile, FanSignupInput, IssuePassInput, OperatorProfile,
        OperatorRole, PublicEvent,
    },
    AppError,
};

const FAN_COOKIE: &str = "crowdrelay_fan";
const PASS_COOKIE: &str = "crowdrelay_pass_session";
const MAX_RESPONSE_BYTES: u64 = 2 * 1024 * 1024;
const MAX_TOKEN_BYTES: usize = 4096;

#[derive(Clone)]
pub struct CrowdRelayClient {
    http: Client,
}

impl CrowdRelayClient {
    pub fn new() -> Result<Self, AppError> {
        // Ring is materially smaller and faster to compile for Android than the
        // default AWS-LC provider. Installing it once also makes the TLS choice
        // explicit instead of depending on transitive feature defaults.
        let _ = rustls::crypto::ring::default_provider().install_default();
        let http = Client::builder()
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(5))
            .pool_idle_timeout(Duration::from_secs(60))
            .pool_max_idle_per_host(4)
            .tcp_keepalive(Duration::from_secs(30))
            .user_agent(concat!("crowdrelay-mobile/", env!("CARGO_PKG_VERSION")))
            .https_only(!cfg!(debug_assertions))
            .build()?;
        Ok(Self { http })
    }

    pub async fn validate(&self, profile: &OperatorProfile) -> Result<(), AppError> {
        let path = match profile.role {
            OperatorRole::Owner => "admin/event-qr/overview",
            OperatorRole::Staff => "staff/event-qr/overview",
        };
        let _: serde_json::Value = self
            .auth_json(profile, Method::GET, path, Option::<&()>::None)
            .await?;
        Ok(())
    }

    pub async fn operator_events(
        &self,
        profile: &OperatorProfile,
    ) -> Result<Vec<PublicEvent>, AppError> {
        let response: EventListResponse =
            self.public_json(profile, "public/events?limit=50").await?;
        Ok(response.events)
    }

    pub async fn operator_qr(
        &self,
        profile: &OperatorProfile,
    ) -> Result<serde_json::Value, AppError> {
        let qr_path = match profile.role {
            OperatorRole::Owner => "admin/event-qr/overview",
            OperatorRole::Staff => "staff/event-qr/overview",
        };
        self.auth_json::<serde_json::Value, ()>(profile, Method::GET, qr_path, None)
            .await
    }

    pub async fn public_events(&self, api_base_url: &str) -> Result<Vec<PublicEvent>, AppError> {
        let response: EventListResponse = self
            .public_json_base(api_base_url, "public/events?limit=50")
            .await?;
        Ok(response.events)
    }

    pub async fn public_cities(&self, api_base_url: &str) -> Result<Vec<CitySignal>, AppError> {
        let response: CityListResponse = self
            .public_json_base(api_base_url, "public/cities?limit=100")
            .await?;
        Ok(response.items)
    }

    pub async fn ticketing_overview(
        &self,
        profile: &OperatorProfile,
        event_slug: &str,
    ) -> Result<serde_json::Value, AppError> {
        let prefix = match profile.role {
            OperatorRole::Owner => "admin",
            OperatorRole::Staff => "staff",
        };
        self.auth_json(
            profile,
            Method::GET,
            &format!("{prefix}/events/{}/ticketing", segment(event_slug)?),
            Option::<&()>::None,
        )
        .await
    }

    pub async fn redeem_admission(
        &self,
        profile: &OperatorProfile,
        event_slug: &str,
        raw_code: &str,
    ) -> Result<serde_json::Value, AppError> {
        let token = normalize_scanned_code(raw_code)?;
        let body = if token.starts_with("v1.") || token.starts_with("t1.") {
            serde_json::json!({"event_slug": event_slug, "qr_token": token, "public_reference": null})
        } else {
            serde_json::json!({"event_slug": event_slug, "qr_token": null, "public_reference": token})
        };
        self.auth_json(profile, Method::POST, "staff/admission/redeem", Some(&body))
            .await
    }

    pub async fn redeem_coupon(
        &self,
        profile: &OperatorProfile,
        code: &str,
        order_reference: &str,
    ) -> Result<serde_json::Value, AppError> {
        let code = bounded_required(code, "kod kuponu", 128)?;
        let order_reference = bounded_required(order_reference, "numer sprzedaży", 200)?;
        let body = serde_json::json!({"code": code.to_ascii_uppercase(), "order_reference": order_reference});
        self.auth_json(profile, Method::POST, "staff/coupons/redeem", Some(&body))
            .await
    }

    pub async fn issue_pass(
        &self,
        profile: &OperatorProfile,
        input: &IssuePassInput,
    ) -> Result<serde_json::Value, AppError> {
        require_owner(profile)?;
        self.auth_json(profile, Method::POST, "admin/admission/passes", Some(input))
            .await
    }

    pub async fn revoke_pass(
        &self,
        profile: &OperatorProfile,
        reference: &str,
    ) -> Result<serde_json::Value, AppError> {
        require_owner(profile)?;
        self.auth_json(
            profile,
            Method::POST,
            &format!("admin/admission/passes/{}/revoke", segment(reference)?),
            Option::<&()>::None,
        )
        .await
    }

    pub async fn create_qr_campaign(
        &self,
        profile: &OperatorProfile,
        input: &CreateQrCampaignInput,
    ) -> Result<serde_json::Value, AppError> {
        self.auth_json(
            profile,
            Method::POST,
            "staff/event-qr/campaigns",
            Some(input),
        )
        .await
    }

    pub async fn revoke_qr_campaign(
        &self,
        profile: &OperatorProfile,
        campaign_id: &str,
    ) -> Result<serde_json::Value, AppError> {
        self.auth_json(
            profile,
            Method::POST,
            &format!("staff/event-qr/campaigns/{}/revoke", segment(campaign_id)?),
            Option::<&()>::None,
        )
        .await
    }

    pub async fn fan_signup(
        &self,
        input: &FanSignupInput,
    ) -> Result<(FanAuthResult, Option<String>), AppError> {
        let body = serde_json::json!({
            "email": input.email.trim(),
            "display_name": normalized_optional(&input.display_name),
            "city_slug": input.city_slug.trim(),
            "locale": input.locale.trim(),
            "referral_code": normalized_optional(&input.referral_code),
            "campaign_id": null,
            "consent": {
                "marketing": true,
                "policy_version": input.policy_version.trim(),
            }
        });
        let response = self
            .http
            .post(endpoint(&input.api_base_url, "fans")?)
            .header(ACCEPT, "application/json")
            .header("Idempotency-Key", Uuid::new_v4().to_string())
            .json(&body)
            .send()
            .await?;
        let token = response_cookie(response.headers(), FAN_COOKIE);
        let response_body: serde_json::Value = decode(response).await?;
        Ok((
            FanAuthResult {
                response: response_body,
                session_created: token.is_some(),
            },
            token,
        ))
    }

    pub async fn fan_confirm(
        &self,
        input: &FanConfirmationInput,
    ) -> Result<(FanAuthResult, String), AppError> {
        let response = self
            .http
            .post(endpoint(&input.api_base_url, "fans/confirm")?)
            .header(ACCEPT, "application/json")
            .header("Idempotency-Key", Uuid::new_v4().to_string())
            .json(&serde_json::json!({"token": input.token.trim()}))
            .send()
            .await?;
        let token =
            response_cookie(response.headers(), FAN_COOKIE).ok_or_else(|| AppError::Remote {
                status: response.status().as_u16(),
                detail: "Backend nie zwrócił sesji fana".into(),
            })?;
        let response_body: serde_json::Value = decode(response).await?;
        Ok((
            FanAuthResult {
                response: response_body,
                session_created: true,
            },
            token,
        ))
    }

    pub async fn fan_events(&self, profile: &FanProfile) -> Result<Vec<PublicEvent>, AppError> {
        let response: EventListResponse = self
            .public_json_base(&profile.api_base_url, "public/events?limit=50")
            .await?;
        Ok(response.events)
    }

    pub async fn fan_referral(&self, profile: &FanProfile) -> Result<serde_json::Value, AppError> {
        self.fan_json::<serde_json::Value, ()>(profile, Method::GET, "me/referral", None)
            .await
    }

    pub async fn fan_interests(&self, profile: &FanProfile) -> Result<serde_json::Value, AppError> {
        self.fan_json::<serde_json::Value, ()>(profile, Method::GET, "me/events?limit=50", None)
            .await
    }

    pub async fn fan_admission_pass(
        &self,
        profile: &FanProfile,
    ) -> Result<Option<serde_json::Value>, AppError> {
        match profile.pass_session_token.as_deref() {
            Some(token) => self
                .pass_json::<serde_json::Value, ()>(
                    &profile.api_base_url,
                    token,
                    Method::GET,
                    "me/pass",
                    None,
                )
                .await
                .map(Some),
            None => Ok(None),
        }
    }

    pub async fn register_interest(
        &self,
        profile: &FanProfile,
        event_slug: &str,
    ) -> Result<serde_json::Value, AppError> {
        let body = serde_json::json!({"campaign_id": null, "source": "mobile_app"});
        self.fan_json(
            profile,
            Method::POST,
            &format!("events/{}/interest", segment(event_slug)?),
            Some(&body),
        )
        .await
    }

    pub async fn claim_pass(
        &self,
        profile: &FanProfile,
        claim_token: &str,
    ) -> Result<(serde_json::Value, String), AppError> {
        let claim_token = bounded_required(claim_token, "token wejściówki", MAX_TOKEN_BYTES)?;
        let response = self
            .http
            .post(endpoint(&profile.api_base_url, "passes/claim")?)
            .header(ACCEPT, "application/json")
            .header("Idempotency-Key", Uuid::new_v4().to_string())
            .json(&serde_json::json!({"token": claim_token}))
            .send()
            .await?;
        let session_token =
            response_cookie(response.headers(), PASS_COOKIE).ok_or_else(|| AppError::Remote {
                status: response.status().as_u16(),
                detail: "Backend nie zwrócił sesji wejściówki".into(),
            })?;
        let body = decode(response).await?;
        Ok((body, session_token))
    }

    pub async fn admission_qr(&self, profile: &FanProfile) -> Result<serde_json::Value, AppError> {
        let token = profile
            .pass_session_token
            .as_deref()
            .ok_or_else(|| AppError::InvalidInput("Najpierw odbierz wejściówkę".into()))?;
        self.pass_json::<serde_json::Value, ()>(
            &profile.api_base_url,
            token,
            Method::GET,
            "me/pass/qr",
            None,
        )
        .await
    }

    pub async fn ticket_wallet(
        &self,
        api_base_url: &str,
        order_id: &str,
        checkout_token: &str,
    ) -> Result<serde_json::Value, AppError> {
        let order_id = uuid_segment(order_id)?;
        let checkout_token = bounded_required(checkout_token, "token zamówienia", MAX_TOKEN_BYTES)?;
        let response = self
            .http
            .get(endpoint(
                api_base_url,
                &format!("public/ticket-orders/{order_id}/wallet"),
            )?)
            .header(ACCEPT, "application/json")
            .bearer_auth(checkout_token)
            .send()
            .await?;
        decode(response).await
    }

    pub async fn request_ticket_delivery(
        &self,
        api_base_url: &str,
        order_id: &str,
        checkout_token: &str,
    ) -> Result<serde_json::Value, AppError> {
        let order_id = uuid_segment(order_id)?;
        let checkout_token = bounded_required(checkout_token, "token zamówienia", MAX_TOKEN_BYTES)?;
        let response = self
            .http
            .post(endpoint(
                api_base_url,
                &format!("public/ticket-orders/{order_id}/delivery-requests"),
            )?)
            .header(ACCEPT, "application/json")
            .bearer_auth(checkout_token)
            .header("Idempotency-Key", Uuid::new_v4().to_string())
            .send()
            .await?;
        decode(response).await
    }

    async fn public_json<T: DeserializeOwned>(
        &self,
        profile: &OperatorProfile,
        path: &str,
    ) -> Result<T, AppError> {
        self.public_json_base(&profile.api_base_url, path).await
    }

    async fn public_json_base<T: DeserializeOwned>(
        &self,
        api_base_url: &str,
        path: &str,
    ) -> Result<T, AppError> {
        let response = self
            .http
            .get(endpoint(api_base_url, path)?)
            .header(ACCEPT, "application/json")
            .send()
            .await?;
        decode(response).await
    }

    async fn auth_json<T, B>(
        &self,
        profile: &OperatorProfile,
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<T, AppError>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        let mut request = self
            .request(method, endpoint(&profile.api_base_url, path)?)
            .bearer_auth(profile.bearer_token.trim());
        if let Some(body) = body {
            request = request.json(body);
        }
        decode(request.send().await?).await
    }

    async fn fan_json<T, B>(
        &self,
        profile: &FanProfile,
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<T, AppError>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        let mut request = self
            .request(method, endpoint(&profile.api_base_url, path)?)
            .header(
                COOKIE,
                format!("{FAN_COOKIE}={}", profile.fan_session_token),
            );
        if let Some(body) = body {
            request = request.json(body);
        }
        decode(request.send().await?).await
    }

    async fn pass_json<T, B>(
        &self,
        api_base_url: &str,
        pass_session_token: &str,
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<T, AppError>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        let mut request = self
            .request(method, endpoint(api_base_url, path)?)
            .header(COOKIE, format!("{PASS_COOKIE}={pass_session_token}"));
        if let Some(body) = body {
            request = request.json(body);
        }
        decode(request.send().await?).await
    }

    fn request(&self, method: Method, url: Url) -> RequestBuilder {
        let needs_idempotency_key = !matches!(method, Method::GET | Method::HEAD | Method::OPTIONS);
        let request = self
            .http
            .request(method, url)
            .header(ACCEPT, "application/json");
        if needs_idempotency_key {
            request.header("Idempotency-Key", Uuid::new_v4().to_string())
        } else {
            request
        }
    }
}

async fn decode<T: DeserializeOwned>(response: Response) -> Result<T, AppError> {
    let status = response.status();
    if response
        .content_length()
        .is_some_and(|length| length > MAX_RESPONSE_BYTES)
    {
        return Err(AppError::Remote {
            status: status.as_u16(),
            detail: "Odpowiedź CrowdRelay jest zbyt duża".into(),
        });
    }
    let bytes = response.bytes().await?;
    if bytes.len() as u64 > MAX_RESPONSE_BYTES {
        return Err(AppError::Remote {
            status: status.as_u16(),
            detail: "Odpowiedź CrowdRelay jest zbyt duża".into(),
        });
    }
    if status.is_success() {
        return serde_json::from_slice(if bytes.is_empty() {
            b"null".as_slice()
        } else {
            bytes.as_ref()
        })
        .map_err(AppError::from);
    }
    let body = serde_json::from_slice(&bytes).unwrap_or_default();
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

fn endpoint(base: &str, path: &str) -> Result<Url, AppError> {
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

fn segment(value: &str) -> Result<String, AppError> {
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

fn uuid_segment(value: &str) -> Result<String, AppError> {
    Uuid::parse_str(value.trim())
        .map(|value| value.to_string())
        .map_err(|_| AppError::InvalidInput("Nieprawidłowy identyfikator zamówienia".into()))
}

fn normalize_scanned_code(value: &str) -> Result<String, AppError> {
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

fn response_cookie(headers: &HeaderMap, expected_name: &str) -> Option<String> {
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

fn bounded_required<'a>(value: &'a str, label: &str, max: usize) -> Result<&'a str, AppError> {
    let value = value.trim();
    if value.is_empty() || value.len() > max {
        Err(AppError::InvalidInput(format!("Nieprawidłowy {label}")))
    } else {
        Ok(value)
    }
}

fn normalized_optional(value: &Option<String>) -> Option<String> {
    value
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn require_owner(profile: &OperatorProfile) -> Result<(), AppError> {
    if profile.role == OperatorRole::Owner {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extracts_fragment_token() {
        assert_eq!(
            normalize_scanned_code("https://virya.music/win#token=v1.abc").unwrap(),
            "v1.abc"
        );
    }

    #[test]
    fn extracts_token_after_another_query_parameter() {
        assert_eq!(
            normalize_scanned_code("https://virya.music/win?source=app&token=t1.abc").unwrap(),
            "t1.abc"
        );
    }

    #[test]
    fn leaves_manual_reference() {
        assert_eq!(normalize_scanned_code(" VRY-ABCD ").unwrap(), "VRY-ABCD");
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
