use reqwest::header::ACCEPT;
use uuid::Uuid;

use crate::{
    models::{AreaWallet, FanProfile, RequestedCityInput, RequestedCityResult, TicketWalletApi},
    AppError,
};

use super::{
    client::WALLET_REQUEST_TIMEOUT,
    http::{bounded_required, decode, endpoint, uuid_segment, MAX_TOKEN_BYTES},
};

const AREA_COOKIE: &str = "virya-area-wallet";
const AREA_WALLET_URL: &str = "https://virya.music/api/area/wallet";

impl super::CrowdRelayClient {
    pub async fn fan_area_wallet(&self, profile: &FanProfile) -> Result<AreaWallet, AppError> {
        let wallet_id = uuid::Uuid::parse_str(profile.area_wallet_id.trim()).map_err(|_| {
            AppError::InvalidInput("Nieprawidłowy identyfikator portfela AREA".into())
        })?;
        let response = self
            .http
            .get(url::Url::parse(AREA_WALLET_URL)?)
            .header(ACCEPT, "application/json")
            .header(
                reqwest::header::COOKIE,
                format!("{AREA_COOKIE}={wallet_id}"),
            )
            .timeout(WALLET_REQUEST_TIMEOUT)
            .send()
            .await?;
        decode(response).await
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
        let checkout_token = bounded_required(checkout_token, "token zamówienia", MAX_TOKEN_BYTES)?;
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
}
