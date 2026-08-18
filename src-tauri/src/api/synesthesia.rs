use reqwest::Method;
use serde::Deserialize;

use crate::{AppError, models::FanProfile};

#[derive(Debug, Deserialize)]
struct SynesthesiaLinkResponse {
    linked: bool,
}

#[derive(Debug, Deserialize)]
struct SynesthesiaLeaderboardUnpublishResponse {
    published: bool,
}

impl super::CrowdRelayClient {
    pub async fn fan_link_synesthesia_handoff(
        &self,
        profile: &FanProfile,
        handoff_code: &str,
    ) -> Result<bool, AppError> {
        self.require_capability(&profile.api_base_url, "synesthesia_runs_v1")
            .await?;
        self.require_capability(&profile.api_base_url, "synesthesia_leaderboard_v1")
            .await?;
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
    pub async fn fan_unpublish_synesthesia_leaderboard(
        &self,
        profile: &FanProfile,
    ) -> Result<bool, AppError> {
        self.require_capability(
            &profile.api_base_url,
            "synesthesia_leaderboard_unpublish_v1",
        )
        .await?;
        let response: SynesthesiaLeaderboardUnpublishResponse = self
            .fan_json::<SynesthesiaLeaderboardUnpublishResponse, ()>(
                profile,
                Method::DELETE,
                "me/synesthesia/leaderboard",
                None,
            )
            .await?;
        if response.published {
            return Err(AppError::Conflict(
                "synesthesia_leaderboard_unpublish_not_confirmed".to_owned(),
            ));
        }
        self.invalidate_fan_home(profile).await;
        Ok(true)
    }
}
