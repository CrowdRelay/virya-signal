#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StaffEventDashboard {
    pub schema_version: i32,
    pub slug: String,
    pub title: String,
    pub venue: Option<String>,
    pub starts_at: String,
    pub interested_fans: i64,
    pub paid_orders: i64,
    pub paid_tickets: i64,
    pub passes_issued: i64,
    pub passes_claimed: i64,
    pub passes_redeemed: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EventListResponse {
    pub events: Vec<PublicEvent>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct MerchCatalog {
    #[serde(default)]
    pub products: Vec<MerchProduct>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MerchProduct {
    #[serde(deserialize_with = "deserialize_string_or_default")]
    pub slug: String,
    #[serde(deserialize_with = "deserialize_string_or_default")]
    pub name: String,
    #[serde(default, deserialize_with = "deserialize_optional_string_or_bytes")]
    pub description: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string_or_bytes")]
    pub image_url: Option<String>,
    #[serde(deserialize_with = "deserialize_string_or_default")]
    pub currency: String,
    pub price_gross_minor: i64,
    pub active: bool,
    pub public: bool,
    #[serde(default)]
    pub variants: Vec<MerchVariant>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MerchVariant {
    #[serde(deserialize_with = "deserialize_string_or_default")]
    pub sku: String,
    #[serde(deserialize_with = "deserialize_string_or_default")]
    pub label: String,
    pub active: bool,
    pub available: bool,
    #[serde(deserialize_with = "deserialize_string_or_default")]
    pub availability: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CityListResponse {
    pub items: Vec<CitySignal>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CitySignal {
    #[serde(deserialize_with = "deserialize_string_or_default")]
    pub slug: String,
    #[serde(deserialize_with = "deserialize_string_or_default")]
    pub name: String,
    #[serde(deserialize_with = "deserialize_string_or_default")]
    pub country_code: String,
    pub fan_count: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PublicEvent {
    #[serde(deserialize_with = "deserialize_string_or_default")]
    pub slug: String,
    #[serde(deserialize_with = "deserialize_string_or_default")]
    pub title: String,
    #[serde(default, deserialize_with = "deserialize_optional_string_or_bytes")]
    pub description: Option<String>,
    pub city: Option<EventCity>,
    #[serde(default, deserialize_with = "deserialize_optional_string_or_bytes")]
    pub venue: Option<String>,
    #[serde(deserialize_with = "deserialize_string_or_default")]
    pub starts_at: String,
    #[serde(default, deserialize_with = "deserialize_optional_string_or_bytes")]
    pub ticket_url: Option<String>,
    #[serde(default, deserialize_with = "deserialize_optional_string_or_bytes")]
    pub image_url: Option<String>,
    #[serde(
        default,
        alias = "thumbnail_url",
        alias = "image_mobile_url",
        deserialize_with = "deserialize_optional_string_or_bytes"
    )]
    pub image_thumbnail_url: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EventCity {
    #[serde(deserialize_with = "deserialize_string_or_default")]
    pub name: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TicketingOverview {
    pub sale: TicketSaleSummary,
    #[serde(default)]
    pub paid_tickets: i64,
    #[serde(default)]
    pub gross_sales_minor: i64,
    #[serde(default)]
    pub refunded_minor: i64,
    #[serde(default)]
    pub recent_orders: Vec<TicketOrderSummary>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TicketSaleSummary {
    pub currency: String,
    #[serde(default)]
    pub reserved: i32,
    pub available: i32,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct TicketOrderSummary {
    pub public_reference: String,
    pub buyer_email_masked: String,
    pub buyer_name: Option<String>,
    pub currency: String,
    pub amount_gross_minor: i64,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct ConcertQrOverview {
    #[serde(default)]
    pub events: Vec<StaffEventSummary>,
    #[serde(default)]
    pub campaigns: Vec<QrCampaignSummary>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct StaffEventSummary {
    pub slug: String,
    pub title: String,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct QrCampaignSummary {
    pub id: String,
    pub event_title: String,
    pub label: String,
    pub max_checkins: Option<u32>,
    pub checkin_count: u64,
    pub active: bool,
    pub token: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ReferralProgress {
    #[serde(default, deserialize_with = "deserialize_string_or_default")]
    pub referral_code: String,
    #[serde(default)]
    pub qualified_referrals: u32,
    #[serde(default)]
    pub pending_referrals: u32,
    #[serde(default)]
    pub draw_entries: Vec<WeightedDrawEntry>,
    #[serde(default)]
    pub coupons: Vec<MerchCoupon>,
    #[serde(default)]
    pub physical_rewards: Vec<PhysicalRewardGrant>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WeightedDrawEntry {
    #[serde(default, deserialize_with = "deserialize_string_or_bytes")]
    pub slug: String,
    #[serde(deserialize_with = "deserialize_string_or_bytes")]
    pub name: String,
    #[serde(deserialize_with = "deserialize_string_or_bytes")]
    pub prize_kind: String,
    #[serde(deserialize_with = "deserialize_string_or_bytes")]
    pub draw_at: String,
    pub total_entries: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MerchCoupon {
    #[serde(deserialize_with = "deserialize_string_or_bytes")]
    pub code: String,
    pub discount_percent: u32,
    #[serde(deserialize_with = "deserialize_string_or_bytes")]
    pub status: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PhysicalRewardGrant {
    #[serde(deserialize_with = "deserialize_string_or_bytes")]
    pub item_name: String,
    #[serde(deserialize_with = "deserialize_string_or_bytes")]
    pub sku: String,
    #[serde(deserialize_with = "deserialize_string_or_bytes")]
    pub status: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FanEventInterest {
    pub event: PublicEvent,
}
