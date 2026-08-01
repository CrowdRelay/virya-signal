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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FanSessionStatus {
    pub configured: bool,
    pub unlocked: bool,
    pub session: Option<FanSummary>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FanSummary {
    pub api_base_url: String,
    pub email: String,
    pub display_name: Option<String>,
    pub wallet_count: usize,
    pub has_admission_pass: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct PublicHomeData {
    pub events: Vec<PublicEvent>,
    pub cities: Vec<CitySignal>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CitySignal {
    pub slug: String,
    pub name: String,
    pub country_code: String,
    pub fan_count: u64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
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
    pub listen_url: Option<String>,
    pub image_url: Option<String>,
    pub trailer_url: Option<String>,
    pub external_event_url: Option<String>,
    pub updated_at: String,
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
pub struct TicketingOverview {
    pub sale: TicketSale,
    #[serde(default)]
    pub reserved_orders: i64,
    #[serde(default)]
    pub checkout_created_orders: i64,
    #[serde(default)]
    pub reserved_tickets: i64,
    #[serde(default)]
    pub paid_orders: i64,
    #[serde(default)]
    pub paid_tickets: i64,
    #[serde(default)]
    pub gross_sales_minor: i64,
    #[serde(default)]
    pub refunded_minor: i64,
    #[serde(default)]
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
    #[serde(default)]
    pub sold: i32,
    #[serde(default)]
    pub reserved: i32,
    pub available: i32,
    pub max_per_order: i32,
    pub sales_open_at: String,
    pub sales_close_at: String,
    pub active: bool,
    pub sales_state: String,
    #[serde(default)]
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
    #[serde(default)]
    pub sold: i32,
    #[serde(default)]
    pub reserved: i32,
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
    #[serde(default)]
    pub amount_refunded_minor: i64,
    pub paid_at: Option<String>,
    pub refunded_at: Option<String>,
    #[serde(default)]
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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FanDashboardData {
    pub events: Vec<PublicEvent>,
    pub referral: ReferralProgress,
    #[serde(default)]
    pub interests: Vec<FanEventInterest>,
    pub admission_pass: Option<AdmissionPass>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FanEventInterest {
    pub event: PublicEvent,
    pub interested_at: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ReferralProgress {
    pub referral_code: String,
    #[serde(default)]
    pub qualified_referrals: u32,
    #[serde(default)]
    pub pending_referrals: u32,
    pub next_reward_threshold: Option<u32>,
    #[serde(default)]
    pub draw_entries: Vec<WeightedDrawEntry>,
    #[serde(default)]
    pub coupons: Vec<MerchCoupon>,
    #[serde(default)]
    pub physical_rewards: Vec<PhysicalRewardGrant>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WeightedDrawEntry {
    pub draw_id: String,
    pub slug: String,
    pub name: String,
    pub prize_kind: String,
    pub closes_at: String,
    pub draw_at: String,
    pub qualified_referrals: u32,
    pub base_entries: u32,
    pub referral_entries: u32,
    pub concert_checkins: u32,
    pub checkin_entries: u32,
    pub total_entries: u32,
    pub max_entries: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MerchCoupon {
    pub id: String,
    pub reward_grant_id: String,
    pub reward_rule_id: String,
    pub code: String,
    pub discount_percent: u32,
    pub max_uses: u32,
    pub used_count: u32,
    pub status: String,
    pub expires_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PhysicalRewardGrant {
    pub reward_grant_id: String,
    pub reward_rule_id: String,
    pub item_name: String,
    pub sku: String,
    pub status: String,
    pub granted_at: String,
    pub expires_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AdmissionPass {
    pub pass_id: String,
    pub session_id: Option<String>,
    pub event_id: String,
    pub event_slug: String,
    pub event_title: String,
    pub venue: Option<String>,
    pub starts_at: String,
    pub holder_name: Option<String>,
    pub holder_email_masked: String,
    pub public_reference: String,
    pub status: String,
    pub session_expires_at: String,
    pub redeemed_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AdmissionQr {
    pub token: String,
    pub expires_at: String,
    pub qr_svg: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TicketWallet {
    pub order: WalletOrder,
    #[serde(default)]
    pub tickets: Vec<WalletTicket>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WalletOrder {
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
    #[serde(default)]
    pub amount_refunded_minor: i64,
    pub paid_at: Option<String>,
    pub refunded_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct WalletTicket {
    pub pass_id: String,
    pub order_item_id: String,
    pub ticket_type_slug: String,
    pub ticket_type_name: String,
    pub sequence: i32,
    pub public_reference: String,
    pub status: String,
    pub holder_name: Option<String>,
    pub holder_email_masked: String,
    pub redeemed_at: Option<String>,
    pub qr_token: Option<String>,
    pub qr_not_before: String,
    pub qr_expires_at: String,
    pub qr_svg: Option<String>,
}
