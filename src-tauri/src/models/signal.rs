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
