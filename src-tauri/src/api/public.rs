use reqwest::header::{ACCEPT, CONTENT_TYPE, COOKIE};
use uuid::Uuid;

use crate::{
    AppError,
    models::{
        AreaChallenge, AreaClaimResult, AreaPositionSample, AreaWallet, FanProfile,
        RequestedCityInput, RequestedCityResult, TicketWalletApi,
    },
};

use super::{
    client::{FAN_COOKIE, WALLET_REQUEST_TIMEOUT},
    http::{
        MAX_TOKEN_BYTES, bounded_required, decode, decode_with_error_mapper, endpoint, segment,
        uuid_segment,
    },
};

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct AreaChallengeRequest<'a> {
    drop_id: &'a str,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct AreaClaimRequest<'a> {
    drop_id: &'a str,
    challenge: &'a str,
    samples: &'a [AreaPositionSample],
}

fn fan_cookie(profile: &FanProfile) -> Result<String, AppError> {
    let token = profile.fan_session_token.trim();
    if token.len() != 64 || !token.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err(AppError::Unauthorized);
    }
    let normalized = token.to_ascii_lowercase();
    Ok(format!("{FAN_COOKIE}={normalized}"))
}

fn area_error_detail(body: &serde_json::Value) -> Option<String> {
    let key = match body.get("code")?.as_str()? {
        "INVALID_REQUEST" => "native_area_claim_invalid",
        "DROP_INACTIVE" => "native_area_drop_inactive",
        "CHALLENGE_INVALID" => "native_area_challenge_invalid",
        "RATE_LIMITED" => "native_area_rate_limited",
        "NOT_ENOUGH_SAMPLES" => "native_area_not_enough_samples",
        "LOW_ACCURACY" => "native_area_low_accuracy",
        "OUTSIDE_ZONE" => "native_area_outside_zone",
        "DROP_FULL" => "native_area_drop_full",
        "CLAIM_CONFLICT" => "native_area_claim_conflict",
        "AUTH_REQUIRED" => "native_fan_login_required",
        "TEMPORARY" => "native_area_temporary",
        _ => return None,
    };
    Some(crate::i18n::tr(key).to_owned())
}

impl super::CrowdRelayClient {
    pub async fn fan_area_wallet(&self, profile: &FanProfile) -> Result<AreaWallet, AppError> {
        let cookie = fan_cookie(profile)?;
        let response = self
            .http
            .get(endpoint(&profile.api_base_url, "me/area")?)
            .header(ACCEPT, "application/json")
            .header(COOKIE, cookie)
            .timeout(WALLET_REQUEST_TIMEOUT)
            .send()
            .await?;
        decode_with_error_mapper(response, area_error_detail).await
    }

    pub async fn fan_area_challenge(
        &self,
        profile: &FanProfile,
        drop_id: &str,
    ) -> Result<AreaChallenge, AppError> {
        let drop_id = segment(drop_id)?;
        let cookie = fan_cookie(profile)?;
        let response = self
            .http
            .post(endpoint(&profile.api_base_url, "me/area/challenge")?)
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .header(COOKIE, cookie)
            .header("Idempotency-Key", Uuid::new_v4().to_string())
            .json(&AreaChallengeRequest { drop_id: &drop_id })
            .timeout(WALLET_REQUEST_TIMEOUT)
            .send()
            .await?;
        decode_with_error_mapper(response, area_error_detail).await
    }

    pub async fn fan_area_claim(
        &self,
        profile: &FanProfile,
        drop_id: &str,
        challenge: &str,
        samples: &[AreaPositionSample],
    ) -> Result<AreaClaimResult, AppError> {
        let drop_id = segment(drop_id)?;
        let cookie = fan_cookie(profile)?;
        let response = self
            .http
            .post(endpoint(&profile.api_base_url, "me/area/claim")?)
            .header(ACCEPT, "application/json")
            .header(CONTENT_TYPE, "application/json")
            .header(COOKIE, cookie)
            .header("Idempotency-Key", Uuid::new_v4().to_string())
            .json(&AreaClaimRequest {
                drop_id: &drop_id,
                challenge,
                samples,
            })
            .timeout(WALLET_REQUEST_TIMEOUT)
            .send()
            .await?;
        decode_with_error_mapper(response, area_error_detail).await
    }

    pub async fn request_city(
        &self,
        api_base_url: &str,
        input: &RequestedCityInput,
    ) -> Result<RequestedCityResult, AppError> {
        let response = self
            .http
            .post(endpoint(api_base_url, "public/cities/requests")?)
            .header(ACCEPT, "application/json")
            .header("Idempotency-Key", Uuid::new_v4().to_string())
            .json(input)
            .send()
            .await?;
        decode(response).await
    }

    pub async fn ticket_wallet(
        &self,
        api_base_url: &str,
        order_id: &str,
        checkout_token: &str,
    ) -> Result<TicketWalletApi, AppError> {
        let order_id = uuid_segment(order_id)?;
        let checkout_token = bounded_required(
            checkout_token,
            crate::i18n::tr("native_order_token_label"),
            MAX_TOKEN_BYTES,
        )?;
        let response = self
            .http
            .get(endpoint(
                api_base_url,
                &format!("public/ticket-orders/{order_id}/wallet"),
            )?)
            .header(ACCEPT, "application/json")
            .bearer_auth(checkout_token)
            .timeout(WALLET_REQUEST_TIMEOUT)
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
        let checkout_token = bounded_required(
            checkout_token,
            crate::i18n::tr("native_order_token_label"),
            MAX_TOKEN_BYTES,
        )?;
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
}
