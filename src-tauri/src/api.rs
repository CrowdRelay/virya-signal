use std::time::Duration;

use reqwest::{Client, Method, Response, StatusCode};
use serde::{de::DeserializeOwned, Serialize};
use url::Url;
use uuid::Uuid;

use crate::{AppError, models::{CreateQrCampaignInput, DashboardData, EventListResponse, IssuePassInput, OperatorProfile, OperatorRole}};

#[derive(Clone)]
pub struct CrowdRelayClient { http: Client }

impl CrowdRelayClient {
    pub fn new() -> Result<Self, AppError> {
        let http = Client::builder().timeout(Duration::from_secs(10)).user_agent("virya-control/0.1").build()?;
        Ok(Self { http })
    }

    pub async fn validate(&self, profile: &OperatorProfile) -> Result<(), AppError> {
        let path = match profile.role { OperatorRole::Owner => "admin/event-qr/overview", OperatorRole::Staff => "staff/event-qr/overview" };
        let _: serde_json::Value = self.auth_json(profile, Method::GET, path, Option::<&()>::None).await?;
        Ok(())
    }

    pub async fn dashboard(&self, profile: &OperatorProfile) -> Result<DashboardData, AppError> {
        let public: EventListResponse = self.public_json(profile, "public/events?limit=50").await?;
        let qr_path = match profile.role { OperatorRole::Owner => "admin/event-qr/overview", OperatorRole::Staff => "staff/event-qr/overview" };
        let qr = self.auth_json::<serde_json::Value, ()>(profile, Method::GET, qr_path, None).await.ok();
        Ok(DashboardData { events: public.events, qr })
    }

    pub async fn ticketing_overview(&self, profile: &OperatorProfile, event_slug: &str) -> Result<serde_json::Value, AppError> {
        let prefix = match profile.role { OperatorRole::Owner => "admin", OperatorRole::Staff => "staff" };
        self.auth_json(profile, Method::GET, &format!("{prefix}/events/{}/ticketing", segment(event_slug)?), Option::<&()>::None).await
    }

    pub async fn redeem_admission(&self, profile: &OperatorProfile, event_slug: &str, raw_code: &str) -> Result<serde_json::Value, AppError> {
        let token = normalize_scanned_code(raw_code);
        let body = if token.starts_with("v1.") || token.starts_with("t1.") {
            serde_json::json!({"event_slug": event_slug, "qr_token": token, "public_reference": null})
        } else {
            serde_json::json!({"event_slug": event_slug, "qr_token": null, "public_reference": token})
        };
        self.auth_json(profile, Method::POST, "staff/admission/redeem", Some(&body)).await
    }

    pub async fn redeem_coupon(&self, profile: &OperatorProfile, code: &str, order_reference: &str) -> Result<serde_json::Value, AppError> {
        let body = serde_json::json!({"code": code.trim().to_ascii_uppercase(), "order_reference": order_reference.trim()});
        self.auth_json(profile, Method::POST, "staff/coupons/redeem", Some(&body)).await
    }

    pub async fn issue_pass(&self, profile: &OperatorProfile, input: &IssuePassInput) -> Result<serde_json::Value, AppError> {
        require_owner(profile)?;
        self.auth_json(profile, Method::POST, "admin/admission/passes", Some(input)).await
    }

    pub async fn revoke_pass(&self, profile: &OperatorProfile, reference: &str) -> Result<serde_json::Value, AppError> {
        require_owner(profile)?;
        self.auth_json(profile, Method::POST, &format!("admin/admission/passes/{}/revoke", segment(reference)?), Option::<&()>::None).await
    }

    pub async fn create_qr_campaign(&self, profile: &OperatorProfile, input: &CreateQrCampaignInput) -> Result<serde_json::Value, AppError> {
        self.auth_json(profile, Method::POST, "staff/event-qr/campaigns", Some(input)).await
    }

    pub async fn revoke_qr_campaign(&self, profile: &OperatorProfile, campaign_id: &str) -> Result<serde_json::Value, AppError> {
        self.auth_json(profile, Method::POST, &format!("staff/event-qr/campaigns/{}/revoke", segment(campaign_id)?), Option::<&()>::None).await
    }

    async fn public_json<T: DeserializeOwned>(&self, profile: &OperatorProfile, path: &str) -> Result<T, AppError> {
        let response = self.http.get(endpoint(&profile.api_base_url, path)?).send().await?;
        decode(response).await
    }

    async fn auth_json<T, B>(&self, profile: &OperatorProfile, method: Method, path: &str, body: Option<&B>) -> Result<T, AppError>
    where T: DeserializeOwned, B: Serialize + ?Sized {
        let mut request = self.http.request(method, endpoint(&profile.api_base_url, path)?)
            .bearer_auth(profile.bearer_token.trim())
            .header("Idempotency-Key", Uuid::new_v4().to_string());
        if let Some(body) = body { request = request.json(body); }
        decode(request.send().await?).await
    }
}

async fn decode<T: DeserializeOwned>(response: Response) -> Result<T, AppError> {
    let status = response.status();
    if status.is_success() { return Ok(response.json().await?); }
    let body: serde_json::Value = response.json().await.unwrap_or_default();
    let detail = body.get("detail").and_then(|v| v.as_str()).unwrap_or("CrowdRelay odrzucił operację");
    match status {
        StatusCode::UNAUTHORIZED | StatusCode::FORBIDDEN => Err(AppError::Unauthorized),
        StatusCode::CONFLICT => Err(AppError::Conflict(detail.to_owned())),
        StatusCode::UNPROCESSABLE_ENTITY | StatusCode::BAD_REQUEST => Err(AppError::InvalidInput(detail.to_owned())),
        _ => Err(AppError::Remote { status: status.as_u16(), detail: detail.to_owned() }),
    }
}

fn endpoint(base: &str, path: &str) -> Result<Url, AppError> {
    let mut base = Url::parse(base.trim())?;
    if base.scheme() != "https" && !cfg!(debug_assertions) { return Err(AppError::InvalidInput("Produkcyjny API URL musi używać HTTPS".into())); }
    if !base.path().ends_with('/') { base.set_path(&format!("{}/", base.path())); }
    base.join(path).map_err(AppError::from)
}

fn segment(value: &str) -> Result<String, AppError> {
    let value = value.trim();
    if value.is_empty() || value.len() > 200 || !value.chars().all(|c| c.is_ascii_alphanumeric() || matches!(c, '-' | '_')) {
        return Err(AppError::InvalidInput("Nieprawidłowy identyfikator".into()));
    }
    Ok(value.to_owned())
}

fn normalize_scanned_code(value: &str) -> String {
    let trimmed = value.trim();
    for marker in ["#token=", "?token="] {
        if let Some((_, token)) = trimmed.split_once(marker) { return token.split('&').next().unwrap_or(token).trim().to_owned(); }
    }
    trimmed.to_owned()
}

fn require_owner(profile: &OperatorProfile) -> Result<(), AppError> {
    if profile.role == OperatorRole::Owner { Ok(()) } else { Err(AppError::Forbidden) }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test] fn extracts_fragment_token() { assert_eq!(normalize_scanned_code("https://virya.music/win#token=v1.abc"), "v1.abc"); }
    #[test] fn leaves_manual_reference() { assert_eq!(normalize_scanned_code(" VRY-ABCD "), "VRY-ABCD"); }
}
