use reqwest::{Method, header::ACCEPT};
use serde::Deserialize;
use uuid::Uuid;

use crate::{
    AppError,
    models::{
        AdmissionPass, FanAuthResult, FanConfirmationInput, FanEventInterest, FanProfile,
        FanSignupInput, PublicEvent, ReferralProgress,
    },
};

use super::{
    client::FAN_COOKIE,
    http::{
        MAX_TOKEN_BYTES, bounded_required, decode, endpoint, normalized_optional, response_cookie,
        segment,
    },
};

#[derive(Deserialize)]
struct FanSignupApiResponse {
    #[serde(default)]
    email_kind: Option<String>,
    #[serde(default)]
    email_queued: Option<bool>,
    #[serde(default)]
    retry_after_seconds: Option<u32>,
}

#[derive(Deserialize)]
struct FanConfirmationApiResponse {
    fan_session_token: Option<String>,
}

impl super::CrowdRelayClient {
    pub async fn fan_events(&self, profile: &FanProfile) -> Result<Vec<PublicEvent>, AppError> {
        self.public_events(&profile.api_base_url).await
    }

    pub async fn fan_referral(&self, profile: &FanProfile) -> Result<ReferralProgress, AppError> {
        self.fan_json::<ReferralProgress, ()>(profile, Method::GET, "me/referral", None)
            .await
    }

    pub async fn fan_interests(
        &self,
        profile: &FanProfile,
    ) -> Result<Vec<FanEventInterest>, AppError> {
        self.fan_json::<Vec<FanEventInterest>, ()>(profile, Method::GET, "me/events?limit=50", None)
            .await
    }

    pub async fn fan_admission_pass(
        &self,
        profile: &FanProfile,
    ) -> Result<Option<AdmissionPass>, AppError> {
        match profile.pass_session_token.as_deref() {
            Some(token) => self
                .pass_json::<AdmissionPass, ()>(
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
    ) -> Result<(AdmissionPass, String), AppError> {
        let claim_token = bounded_required(claim_token, "token wejściówki", MAX_TOKEN_BYTES)?;
        let response = self
            .http
            .post(endpoint(&profile.api_base_url, "passes/claim")?)
            .header(ACCEPT, "application/json")
            .header("Idempotency-Key", Uuid::new_v4().to_string())
            .json(&serde_json::json!({"token": claim_token}))
            .send()
            .await?;
        let session_token = response_cookie(response.headers(), super::client::PASS_COOKIE)
            .ok_or_else(|| AppError::Remote {
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
            },
            "nearby_gigs": {
                "enabled": input.nearby_gigs_enabled,
                "radius_km": input.nearby_radius_km,
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
        let body: FanSignupApiResponse = decode(response).await?;
        Ok((
            FanAuthResult {
                session_created: token.is_some(),
                email_kind: body.email_kind,
                email_queued: body.email_queued,
                retry_after_seconds: body.retry_after_seconds,
            },
            token,
        ))
    }

    pub async fn fan_request_access(
        &self,
        api_base_url: &str,
        email: &str,
        locale: &str,
    ) -> Result<serde_json::Value, AppError> {
        let body = serde_json::json!({
            "email": email,
            "locale": locale,
        });
        let response = self
            .http
            .post(endpoint(api_base_url, "fans/access")?)
            .header(ACCEPT, "application/json")
            .header("Idempotency-Key", Uuid::new_v4().to_string())
            .json(&body)
            .send()
            .await?;
        decode(response).await
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
        let cookie_token = response_cookie(response.headers(), FAN_COOKIE);
        let body: FanConfirmationApiResponse = match decode(response).await {
            Ok(body) => body,
            Err(AppError::Conflict(_)) => {
                return Err(AppError::Conflict(
                    "Ten kod został już wykorzystany. Wróć do ZACZYNAM i poproś o nową wiadomość."
                        .into(),
                ));
            }
            Err(AppError::NotFound) => {
                return Err(AppError::InvalidInput(
                    "Kod jest nieprawidłowy albo wygasł. Poproś o nową wiadomość.".into(),
                ));
            }
            Err(error) => return Err(error),
        };
        let session_token = body
            .fan_session_token
            .or(cookie_token)
            .filter(|value| !value.is_empty() && value.len() <= MAX_TOKEN_BYTES)
            .ok_or_else(|| AppError::Remote {
                status: 200,
                detail: "Backend potwierdził kod, ale nie zwrócił sesji fana".into(),
            })?;
        Ok((
            FanAuthResult {
                session_created: true,
                email_kind: None,
                email_queued: None,
                retry_after_seconds: None,
            },
            session_token,
        ))
    }
}
