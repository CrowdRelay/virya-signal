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

#[derive(Clone, Debug, Deserialize, Serialize, Zeroize)]
pub struct WalletCredential {
    pub order_id: String,
    pub checkout_token: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Zeroize)]
#[zeroize(drop)]
pub struct FanProfile {
    pub api_base_url: String,
    pub email: String,
    pub display_name: Option<String>,
    pub fan_session_token: String,
    pub pass_session_token: Option<String>,
    pub wallets: Vec<WalletCredential>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FanSummary {
    pub api_base_url: String,
    pub email: String,
    pub display_name: Option<String>,
    pub wallet_count: usize,
    pub has_admission_pass: bool,
}

impl From<&FanProfile> for FanSummary {
    fn from(value: &FanProfile) -> Self {
        Self {
            api_base_url: value.api_base_url.clone(),
            email: value.email.clone(),
            display_name: value.display_name.clone(),
            wallet_count: value.wallets.len(),
            has_admission_pass: value.pass_session_token.is_some(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FanSessionStatus {
    pub configured: bool,
    pub unlocked: bool,
    pub session: Option<FanSummary>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EventListResponse {
    pub events: Vec<PublicEvent>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CityListResponse {
    pub items: Vec<CitySignal>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CitySignal {
    pub slug: String,
    pub name: String,
    pub country_code: String,
    pub fan_count: u64,
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
pub struct PublicHomeData {
    pub events: Vec<PublicEvent>,
    pub cities: Vec<CitySignal>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DashboardData {
    pub events: Vec<PublicEvent>,
    pub qr: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FanDashboardData {
    pub events: Vec<PublicEvent>,
    pub referral: serde_json::Value,
    pub interests: serde_json::Value,
    pub admission_pass: Option<serde_json::Value>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FanSignupInput {
    pub api_base_url: String,
    pub email: String,
    pub display_name: Option<String>,
    pub city_slug: String,
    pub locale: String,
    pub referral_code: Option<String>,
    pub policy_version: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FanConfirmationInput {
    pub api_base_url: String,
    pub email: String,
    pub display_name: Option<String>,
    pub token: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FanAuthResult {
    pub response: serde_json::Value,
    pub session_created: bool,
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
