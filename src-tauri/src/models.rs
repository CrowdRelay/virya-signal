use serde::{Deserialize, Serialize};
use zeroize::Zeroize;

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OperatorRole {
    Owner,
    Staff,
}

#[derive(Clone, Debug, Deserialize, Serialize, Zeroize)]
#[zeroize(drop)]
pub struct OperatorProfile {
    pub display_name: String,
    pub api_base_url: String,
    #[zeroize(skip)]
    pub role: OperatorRole,
    pub bearer_token: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SessionSummary {
    pub display_name: String,
    pub api_base_url: String,
    pub role: OperatorRole,
}

impl From<&OperatorProfile> for SessionSummary {
    fn from(value: &OperatorProfile) -> Self {
        Self {
            display_name: value.display_name.clone(),
            api_base_url: value.api_base_url.clone(),
            role: value.role.clone(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SessionStatus {
    pub configured: bool,
    pub unlocked: bool,
    pub session: Option<SessionSummary>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EventListResponse {
    pub events: Vec<PublicEvent>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PublicEvent {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub city: Option<serde_json::Value>,
    pub venue: Option<String>,
    pub venue_address: Option<String>,
    pub timezone: String,
    pub starts_at: String,
    pub doors_at: Option<String>,
    pub ends_at: Option<String>,
    pub ticket_url: Option<String>,
    pub listen_url: Option<String>,
    pub image_url: Option<String>,
    pub trailer_url: Option<String>,
    pub external_event_url: Option<String>,
    pub updated_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DashboardData {
    pub events: Vec<PublicEvent>,
    pub qr: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateQrCampaignInput {
    pub event_slug: String,
    pub label: String,
    pub valid_from: String,
    pub valid_until: String,
    pub max_checkins: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IssuePassInput {
    pub event_slug: String,
    pub pool_slug: String,
    pub fan_email: String,
    pub claim_expires_hours: u32,
}
