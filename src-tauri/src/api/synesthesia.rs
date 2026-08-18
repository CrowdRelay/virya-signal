use reqwest::Method;
use serde::Deserialize;

use crate::{AppError, models::FanProfile};

#[derive(Debug, Deserialize)]
struct SynesthesiaLinkResponse {
    linked: bool,
}

impl super::CrowdRelayClient {
    pub async fn fan_link_synesthesia_handoff(
        &self,
        profile: &FanProfile,
        handoff_code: &str,
    ) -> Result<bool, AppError> {
        let handoff_code = handoff_code.trim();
        if handoff_code.len() != 64 || !handoff_code.bytes().all(|byte| byte.is_ascii_hexdigit()) {
            return Err(AppError::InvalidInput(
                "invalid_synesthesia_handoff".to_owned(),
            ));
        }
        let body = serde_json::json!({"handoff_code": handoff_code.to_ascii_lowercase()});
        let response: SynesthesiaLinkResponse = self
            .fan_json(profile, Method::POST, "me/synesthesia/link", Some(&body))
            .await?;
        Ok(response.linked)
    }
}
