mod beacon;
mod cache;
mod client;
mod fan;
mod http;
mod operator;
mod public;
mod retry;
mod site;
mod synesthesia;
mod ticketing;

use reqwest::{StatusCode, header::{ACCEPT, ORIGIN}};
use serde::Serialize;

use crate::AppError;

pub use client::CrowdRelayClient;

pub(crate) use beacon::BeaconPreferencesInput;
pub(crate) use site::SignalMerchBundleCatalog;
pub(crate) use ticketing::{TicketCheckoutInput, TicketCheckoutStart, TicketSaleOffer};

#[derive(Serialize)]
struct TenantStaffGateRequest<'a> {
    password: &'a str,
}

impl CrowdRelayClient {
    pub(crate) async fn verify_tenant_staff_access(&self, password: &str) -> Result<(), AppError> {
        let response = self
            .site_http
            .post(crate::tenant::STAFF_GATE_URL)
            .header(ACCEPT, "application/json")
            .header(ORIGIN, crate::tenant::STAFF_GATE_ORIGIN)
            .json(&TenantStaffGateRequest { password })
            .timeout(std::time::Duration::from_secs(12))
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => Ok(()),
            StatusCode::UNAUTHORIZED => Err(AppError::InvalidInput(
                crate::i18n::tr("native_invalid_staff_password").to_owned(),
            )),
            StatusCode::TOO_MANY_REQUESTS => Err(AppError::InvalidInput(
                crate::i18n::tr("native_staff_rate_limited").to_owned(),
            )),
            status @ StatusCode::SERVICE_UNAVAILABLE => Err(AppError::Remote {
                status: status.as_u16(),
                detail: crate::i18n::tr("native_staff_verification_unavailable").to_owned(),
            }),
            status => Err(AppError::Remote {
                status: status.as_u16(),
                detail: crate::i18n::tr("native_staff_verification_failed").to_owned(),
            }),
        }
    }
}
