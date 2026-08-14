pub use virya_signal_contracts::ops::*;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OpsOutboxItem {
    pub id: String,
    pub event_type: String,
    pub status: String,
    pub attempts: i32,
    pub max_attempts: i32,
    pub last_error_kind: Option<String>,
    pub dead_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OpsDeliveryItem {
    pub id: String,
    pub event_type: String,
    pub endpoint_name: String,
    pub endpoint_active: bool,
    pub status: String,
    pub attempt_count: i32,
    pub max_attempts: i32,
    pub last_response_status: Option<i16>,
    pub last_error_kind: Option<String>,
    pub dead_at: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OperatorOpsOverview {
    #[serde(default)]
    pub summary: OpsSummary,
    #[serde(default)]
    pub dead_deliveries: Vec<OpsDeliveryItem>,
    #[serde(default)]
    pub dead_outbox: Vec<OpsOutboxItem>,
    #[serde(default)]
    pub unavailable_sources: Vec<String>,
}

pub use virya_signal_contracts::autopilot::*;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OperatorSignalOverview {
    #[serde(default)]
    pub generated_at: String,
    #[serde(default)]
    pub summary: SignalFanSummary,
    #[serde(default)]
    pub activity: SignalActivitySummary,
    #[serde(default)]
    pub top_cities: Vec<SignalCitySummary>,
    #[serde(default)]
    pub audience: AudienceSummary,
    #[serde(default)]
    pub ticket_revenue: Vec<AudienceRevenueSummary>,
    #[serde(default)]
    pub unavailable_sources: Vec<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AudienceSummary {
    #[serde(default)]
    pub active_fans: i64,
    #[serde(default)]
    pub marketing_consented_fans: i64,
    #[serde(default)]
    pub ticket_buyers: i64,
    #[serde(default)]
    pub attendees: i64,
    #[serde(default)]
    pub synesthesia_participants: i64,
    #[serde(default)]
    pub qualified_referrals: i64,
    #[serde(default)]
    pub paid_ticket_orders: i64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AudienceRevenueSummary {
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub paid_orders: i64,
    #[serde(default)]
    pub gross_paid_minor: i64,
    #[serde(default)]
    pub refunded_minor: i64,
    #[serde(default)]
    pub after_refunds_minor: i64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SignalFanSummary {
    #[serde(default)]
    pub total_fans: i64,
    #[serde(default)]
    pub active_fans: i64,
    #[serde(default)]
    pub pending_fans: i64,
    #[serde(default)]
    pub unsubscribed_fans: i64,
    #[serde(default)]
    pub suppressed_fans: i64,
    #[serde(default)]
    pub marketing_opted_in: i64,
    #[serde(default)]
    pub nearby_enabled: i64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SignalActivitySummary {
    #[serde(default)]
    pub new_fans_7d: i64,
    #[serde(default)]
    pub new_fans_30d: i64,
    #[serde(default)]
    pub referral_attributions_total: i64,
    #[serde(default)]
    pub referral_attributions_30d: i64,
    #[serde(default)]
    pub event_interests_total: i64,
    #[serde(default)]
    pub event_interests_30d: i64,
    #[serde(default)]
    pub nearby_notifications_30d: i64,
    #[serde(default)]
    pub pending_city_requests: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SignalCitySummary {
    pub name: String,
    pub country_code: String,
    pub active_fans: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OpsRetryResult {
    pub operation_id: String,
    pub target_type: String,
    pub target_id: String,
    pub status: String,
    pub replayed: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
