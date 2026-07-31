use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OperatorRole {
    Owner,
    Staff,
}

impl OperatorRole {
    pub fn label(&self) -> &'static str {
        match self {
            Self::Owner => "Owner",
            Self::Staff => "Staff",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OperatorProfileInput {
    pub display_name: String,
    pub api_base_url: String,
    pub role: OperatorRole,
    pub bearer_token: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct SessionSummary {
    pub display_name: String,
    pub api_base_url: String,
    pub role: OperatorRole,
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

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct PublicEvent {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub city: Option<EventCity>,
    pub venue: Option<String>,
    pub venue_address: Option<String>,
    pub timezone: String,
    pub starts_at: String,
    pub doors_at: Option<String>,
    pub ends_at: Option<String>,
    pub ticket_url: Option<String>,
    pub image_url: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
pub struct EventCity {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub country_code: String,
    pub region: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct DashboardData {
    pub events: Vec<PublicEvent>,
    pub qr: Option<ConcertQrOverview>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ConcertQrOverview {
    pub events: Vec<StaffEvent>,
    pub campaigns: Vec<QrCampaign>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct StaffEvent {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub venue: Option<String>,
    pub starts_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct QrCampaign {
    pub id: String,
    pub event_id: String,
    pub event_slug: String,
    pub event_title: String,
    pub venue: Option<String>,
    pub starts_at: String,
    pub label: String,
    pub valid_from: String,
    pub valid_until: String,
    pub max_checkins: Option<u32>,
    pub checkin_count: u64,
    pub active: bool,
    pub revoked_at: Option<String>,
    pub created_at: String,
    pub token: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TicketingOverview {
    pub sale: TicketSale,
    pub reserved_orders: i64,
    pub paid_orders: i64,
    pub paid_tickets: i64,
    pub gross_sales_minor: i64,
    pub refunded_minor: i64,
    pub recent_orders: Vec<TicketOrder>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TicketSale {
    pub event_id: String,
    pub event_slug: String,
    pub event_title: String,
    pub event_status: String,
    pub venue: Option<String>,
    pub timezone: String,
    pub starts_at: String,
    pub currency: String,
    pub vat_rate_basis_points: i32,
    pub capacity: i32,
    pub available: i32,
    pub max_per_order: i32,
    pub sales_open_at: String,
    pub sales_close_at: String,
    pub active: bool,
    pub sales_state: String,
    pub ticket_types: Vec<TicketType>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TicketType {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub price_gross_minor: i64,
    pub capacity: Option<i32>,
    pub available: i32,
    pub sort_order: i32,
    pub active: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TicketOrder {
    pub order_id: String,
    pub public_reference: String,
    pub event_slug: String,
    pub event_title: String,
    pub venue: Option<String>,
    pub timezone: String,
    pub starts_at: String,
    pub status: String,
    pub buyer_email_masked: String,
    pub buyer_name: Option<String>,
    pub currency: String,
    pub amount_gross_minor: i64,
    pub amount_refunded_minor: i64,
    pub paid_at: Option<String>,
    pub refunded_at: Option<String>,
    pub tickets: Vec<IssuedTicket>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IssuedTicket {
    pub pass_id: String,
    pub public_reference: String,
    pub status: String,
    pub holder_name: Option<String>,
    pub holder_email_masked: String,
    pub redeemed_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AdmissionRedemption {
    pub pass_id: String,
    pub event_id: String,
    pub public_reference: String,
    pub holder_name: Option<String>,
    pub holder_email_masked: String,
    pub status: String,
    pub redeemed_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CouponEnvelope {
    pub result: CouponRedemption,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CouponRedemption {
    pub coupon_id: String,
    pub reward_grant_id: String,
    pub status: String,
    pub used_count: u32,
    pub max_uses: u32,
    pub redeemed_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct IssuedPass {
    pub pass_id: String,
    pub event_id: String,
    pub fan_id: String,
    pub public_reference: String,
    pub claim_token: String,
    pub claim_expires_at: String,
    pub created: bool,
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
