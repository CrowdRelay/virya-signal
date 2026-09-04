// The operator Signal snapshot: aggregate fan, activity and retention counts.
// Textually included by `models.rs`, like `models/beacon.rs`, so an inner doc
// comment is not legal here. Split out because that file is at the modularity
// limit and these are one screen's read model, used nowhere else.

#[derive(Clone, Debug, Default, Deserialize)]
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
    pub retention_loop: SignalRetentionLoop,
    #[serde(default)]
    pub audience: AudienceSummary,
    #[serde(default)]
    pub ticket_revenue: Vec<AudienceRevenueSummary>,
    #[serde(default)]
    pub unavailable_sources: Vec<String>,
}

/// Every stage between a fan naming a city and a push reaching their device.
/// The operator screen showed only the moderation queue, which is the one stage
/// that does not block delivery -- a city can be approved and still reach
/// nobody because it has no coordinates.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct SignalRetentionLoop {
    #[serde(default)]
    pub cities_awaiting_coordinates: i64,
    #[serde(default)]
    pub cities_resolved: i64,
    #[serde(default)]
    pub fans_with_coordinates: i64,
    #[serde(default)]
    pub nearby_eligible_fans: i64,
    #[serde(default)]
    #[allow(dead_code)]
    pub notifications_created: i64,
    #[serde(default)]
    pub pushes_queued: i64,
    #[serde(default)]
    pub pushes_sent: i64,
    #[serde(default)]
    pub pushes_delivered: i64,
    #[serde(default)]
    #[allow(dead_code)]
    pub pushes_failed: i64,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct AudienceSummary {
    #[serde(default)]
    #[allow(dead_code)]
    pub active_fans: i64,
    #[serde(default)]
    #[allow(dead_code)]
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
    #[allow(dead_code)]
    pub paid_ticket_orders: i64,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct AudienceRevenueSummary {
    #[serde(default)]
    pub currency: String,
    #[serde(default)]
    pub paid_orders: i64,
    #[serde(default)]
    #[allow(dead_code)]
    pub gross_paid_minor: i64,
    #[serde(default)]
    #[allow(dead_code)]
    pub refunded_minor: i64,
    #[serde(default)]
    pub after_refunds_minor: i64,
}

#[derive(Clone, Debug, Default, Deserialize)]
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

#[derive(Clone, Debug, Default, Deserialize)]
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

#[derive(Clone, Debug, Deserialize)]
pub struct SignalCitySummary {
    pub name: String,
    pub country_code: String,
    pub active_fans: i64,
}
