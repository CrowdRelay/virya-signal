use std::time::Duration;

use reqwest::{
    header::{HeaderMap, COOKIE, SET_COOKIE},
    Client, Method, Response, StatusCode,
};
use serde::{de::DeserializeOwned, Serialize};
use url::Url;
use uuid::Uuid;

use crate::{
    models::{
        CityListResponse, CreateQrCampaignInput, DashboardData, EventListResponse, FanAuthResult,
        FanConfirmationInput, FanDashboardData, FanProfile, FanSignupInput, IssuePassInput,
        OperatorProfile, OperatorRole, PublicHomeData,
    },
    AppError,
};

const FAN_COOKIE: &str = "crowdrelay_fan";
const PASS_COOKIE: &str = "crowdrelay_pass_session";

#[derive(Clone)]
pub struct CrowdRelayClient {
    http: Client,
}

impl CrowdRelayClient {
    pub fn new() -> Result<Self, AppError> {
        let http = Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("virya-mobile/0.2")
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

    pub async fn dashboard(&self, profile: &OperatorProfile) -> Result<DashboardData, AppError> {
        let public: EventListResponse = self.public_json(profile, "public/events?limit=50").await?;
        let qr_path = match profile.role {
            OperatorRole::Owner => "admin/event-qr/overview",
            OperatorRole::Staff => "staff/event-qr/overview",
        };
        let qr = self
            .auth_json::<serde_json::Value, ()>(profile, Method::GET, qr_path, None)
            .await
            .ok();
        Ok(DashboardData {
            events: public.events,
            qr,
        })
    }

    pub async fn public_home(&self, api_base_url: &str) -> Result<PublicHomeData, AppError> {
        let events: EventListResponse = self
            .public_json_base(api_base_url, "public/events?limit=50")
            .await?;
        let cities: CityListResponse = self
            .public_json_base(api_base_url, "public/cities?limit=100")
            .await?;
        Ok(PublicHomeData {
            events: events.events,
            cities: cities.items,
        })
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
        let token = normalize_scanned_code(raw_code);
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
        let body = serde_json::json!({"code": code.trim().to_ascii_uppercase(), "order_reference": order_reference.trim()});
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
            .header("Idempotency-Key", Uuid::new_v4().to_string())
            .json(&serde_json::json!({"token": input.token.trim()}))
            .send()
            .await?;
        let token = response_cookie(response.headers(), FAN_COOKIE)
            .ok_or_else(|| AppError::Remote {
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

    pub async fn fan_dashboard(
        &self,
        profile: &FanProfile,
    ) -> Result<FanDashboardData, AppError> {
        let events: EventListResponse = self
            .public_json_base(&profile.api_base_url, "public/events?limit=50")
            .await?;
        let referral = self
            .fan_json::<serde_json::Value, ()>(profile, Method::GET, "me/referral", None)
            .await?;
        let interests = self
            .fan_json::<serde_json::Value, ()>(profile, Method::GET, "me/events?limit=50", None)
            .await?;
        let admission_pass = match profile.pass_session_token.as_deref() {
            Some(token) => self
                .pass_json::<serde_json::Value, ()>(
                    &profile.api_base_url,
                    token,
                    Method::GET,
                    "me/pass",
                    None,
                )
                .await
                .ok(),
            None => None,
        };
        Ok(FanDashboardData {
            events: events.events,
            referral,
            interests,
            admission_pass,
        })
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
        let response = self
            .http
            .post(endpoint(&profile.api_base_url, "passes/claim")?)
            .header("Idempotency-Key", Uuid::new_v4().to_string())
            .json(&serde_json::json!({"token": claim_token.trim()}))
            .send()
            .await?;
        let session_token = response_cookie(response.headers(), PASS_COOKIE)
            .ok_or_else(|| AppError::Remote {
                status: response.status().as_u16(),
                detail: "Backend nie zwrócił sesji wejściówki".into(),
            })?;
        let body = decode(response).await?;
        Ok((body, session_token))
    }

    pub async fn admission_qr(
        &self,
        profile: &FanProfile,
    ) -> Result<serde_json::Value, AppError> {
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
        let response = self
            .http
            .get(endpoint(
                api_base_url,
                &format!("public/ticket-orders/{order_id}/wallet"),
            )?)
            .bearer_auth(checkout_token.trim())
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
        let response = self
            .http
            .post(endpoint(
                api_base_url,
                &format!("public/ticket-orders/{order_id}/delivery-requests"),
            )?)
            .bearer_auth(checkout_token.trim())
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
        let response = self.http.get(endpoint(api_base_url, path)?).send().await?;
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
            .http
            .request(method, endpoint(&profile.api_base_url, path)?)
            .bearer_auth(profile.bearer_token.trim())
            .header("Idempotency-Key", Uuid::new_v4().to_string());
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
            .http
            .request(method, endpoint(&profile.api_base_url, path)?)
            .header(COOKIE, format!("{FAN_COOKIE}={}", profile.fan_session_token))
            .header("Idempotency-Key", Uuid::new_v4().to_string());
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
            .http
            .request(method, endpoint(api_base_url, path)?)
            .header(COOKIE, format!("{PASS_COOKIE}={pass_session_token}"))
            .header("Idempotency-Key", Uuid::new_v4().to_string());
        if let Some(body) = body {
            request = request.json(body);
        }
        decode(request.send().await?).await
    }
}

async fn decode<T: DeserializeOwned>(response: Response) -> Result<T, AppError> {
    let status = response.status();
    if status.is_success() {
        return Ok(response.json().await?);
    }
    let body: serde_json::Value = response.json().await.unwrap_or_default();
    let detail = body
        .get("detail")
        .and_then(|value| value.as_str())
        .unwrap_or("CrowdRelay odrzucił operację");
    match status {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(AppError::Unauthorized),
        StatusCode::CONFLICT => Err(AppError::Conflict(detail.to_owned())),
        StatusCode::NOT_FOUND => Err(AppError::InvalidInput("Nie znaleziono danych".into())),
        StatusCode::UNPROCESSABLE_ENTITY | StatusCode::BAD_REQUEST => {
            Err(AppError::InvalidInput(detail.to_owned()))
        }
        _ => Err(AppError::Remote {
            status: status.as_u16(),
            detail: detail.to_owned(),
        }),
    }
}

fn endpoint(base: &str, path: &str) -> Result<Url, AppError> {
    let mut base = Url::parse(base.trim())?;
    if base.scheme() != "https" && !cfg!(debug_assertions) {
        return Err(AppError::InvalidInput(
            "Produkcyjny API URL musi używać HTTPS".into(),
        ));
    }
    if base.username() != "" || base.password().is_some() {
        return Err(AppError::InvalidInput("API URL nie może zawierać danych logowania".into()));
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

fn normalize_scanned_code(value: &str) -> String {
    let trimmed = value.trim();
    for marker in ["#token=", "?token="] {
        if let Some((_, token)) = trimmed.split_once(marker) {
            return token.split('&').next().unwrap_or(token).trim().to_owned();
        }
    }
    trimmed.to_owned()
}

fn response_cookie(headers: &HeaderMap, expected_name: &str) -> Option<String> {
    headers
        .get_all(SET_COOKIE)
        .iter()
        .filter_map(|header| header.to_str().ok())
        .filter_map(|header| header.split(';').next())
        .filter_map(|pair| pair.trim().split_once('='))
        .find_map(|(name, value)| (name == expected_name && !value.is_empty()).then(|| value.to_owned()))
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
            normalize_scanned_code("https://virya.music/win#token=v1.abc"),
            "v1.abc"
        );
    }

    #[test]
    fn leaves_manual_reference() {
        assert_eq!(normalize_scanned_code(" VRY-ABCD "), "VRY-ABCD");
    }

    #[test]
    fn rejects_non_uuid_order_id() {
        assert!(uuid_segment("order-1").is_err());
    }
}
