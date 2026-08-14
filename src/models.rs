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

#[derive(Clone, Debug, Serialize)]
pub struct OperatorProfileInput {
    pub display_name: String,
    pub api_base_url: String,
    pub role: OperatorRole,
    pub bearer_token: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct SessionSummary {
    pub display_name: String,
    pub api_base_url: String,
    pub role: OperatorRole,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct SessionStatus {
    pub configured: bool,
    pub unlocked: bool,
    pub session: Option<SessionSummary>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct FanSessionStatus {
    pub configured: bool,
    pub unlocked: bool,
    pub session: Option<FanSummary>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct LauncherStatus {
    pub operator: SessionStatus,
    pub fan: FanSessionStatus,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FanSummary {
    pub email: String,
    pub display_name: Option<String>,
    pub wallet_count: usize,
    pub has_admission_pass: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct PublicHomeData {
    pub events: Vec<PublicEvent>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct FanHomeData {
    pub schema_version: u32,
    pub generated_at: String,
    pub profile: FanHomeProfile,
    pub next_event: Option<FanHomeEvent>,
    pub synesthesia: FanHomeSynesthesia,
    pub referral: FanHomeReferral,
    pub counts: FanHomeCounts,
    pub recommended_action: String,
    #[serde(default)]
    pub stale: bool,
}

impl FanHomeData {
    pub const SCHEMA_VERSION: u32 = 1;

    pub fn has_supported_schema(&self) -> bool {
        self.schema_version == Self::SCHEMA_VERSION
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct FanHomeProfile {
    pub display_name: Option<String>,
    pub primary_city: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FanHomeEvent {
    pub title: String,
    pub venue: Option<String>,
    pub city: Option<String>,
    pub starts_at: String,
    pub doors_at: Option<String>,
    pub ends_at: Option<String>,
    pub phase: String,
    pub ticket_url: Option<String>,
    pub interested: bool,
    pub has_pass: bool,
    pub has_paid_ticket: bool,
    pub ticket_sale_active: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct FanHomeSynesthesia {
    pub started: bool,
    pub completed: bool,
    pub rooms_completed: i16,
    pub client_total_elapsed_ms: Option<i64>,
    pub best_elapsed_ms: Option<i64>,
    pub completed_runs: i64,
    pub leaderboard_published: bool,
    pub leaderboard_rank: Option<i64>,
    pub linked_at: Option<String>,
    pub reward_entered: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct FanHomeReferral {
    pub qualified: i64,
    pub pending: i64,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct FanHomeCounts {
    pub event_interests: i64,
    pub active_passes: i64,
    pub paid_orders: i64,
    pub area_claims: i64,
}

#[derive(Clone, Debug, Deserialize)]
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

impl StaffEventDashboard {
    pub const SCHEMA_VERSION: i32 = 1;

    pub fn has_supported_schema(&self) -> bool {
        self.schema_version == Self::SCHEMA_VERSION
    }
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct MerchCatalog {
    #[serde(default)]
    pub products: Vec<MerchProduct>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct FanMerchBundleCatalog {
    #[serde(default)]
    pub bundles: Vec<FanMerchBundle>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FanMerchBundle {
    #[allow(dead_code)]
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub includes: Vec<String>,
    pub image_url: Option<String>,
    #[allow(dead_code)]
    pub secondary_image_url: Option<String>,
    pub product_url: String,
    pub currency: String,
    pub price_gross_minor: i64,
    pub original_price_gross_minor: i64,
    pub available: bool,
    pub availability: String,
    #[serde(default)]
    pub variants: Vec<FanMerchBundleVariant>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FanMerchBundleVariant {
    pub label: String,
    pub available: bool,
    #[allow(dead_code)]
    pub availability: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MerchProduct {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub currency: String,
    pub price_gross_minor: i64,
    pub active: bool,
    pub public: bool,
    #[serde(default)]
    pub variants: Vec<MerchVariant>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MerchVariant {
    #[allow(dead_code)]
    pub sku: String,
    pub label: String,
    pub active: bool,
    pub available: bool,
    pub availability: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TicketTypeOffer {
    #[allow(dead_code)]
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub price_gross_minor: i64,
    #[allow(dead_code)]
    pub capacity: Option<i32>,
    #[allow(dead_code)]
    pub sold: i32,
    #[allow(dead_code)]
    pub reserved: i32,
    pub available: i32,
    #[allow(dead_code)]
    pub sort_order: i32,
    pub active: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TicketSaleOffer {
    #[allow(dead_code)]
    pub event_id: String,
    #[allow(dead_code)]
    pub event_slug: String,
    #[allow(dead_code)]
    pub event_title: String,
    #[allow(dead_code)]
    pub event_status: String,
    #[allow(dead_code)]
    pub venue: Option<String>,
    #[allow(dead_code)]
    pub timezone: String,
    #[allow(dead_code)]
    pub starts_at: String,
    pub currency: String,
    #[allow(dead_code)]
    pub vat_rate_basis_points: i32,
    #[allow(dead_code)]
    pub capacity: i32,
    pub sold: i32,
    pub reserved: i32,
    pub available: i32,
    pub max_per_order: i32,
    #[allow(dead_code)]
    pub sales_open_at: String,
    #[allow(dead_code)]
    pub sales_close_at: String,
    pub active: bool,
    pub sales_state: String,
    #[serde(default)]
    pub ticket_types: Vec<TicketTypeOffer>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TicketCheckoutItemInput {
    pub ticket_type_slug: String,
    pub quantity: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TicketCheckoutInput {
    pub event_slug: String,
    pub buyer_name: Option<String>,
    pub items: Vec<TicketCheckoutItemInput>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TicketCheckoutStart {
    pub url: String,
    #[allow(dead_code)]
    pub order_id: String,
    pub order_reference: String,
    #[allow(dead_code)]
    pub expires_at: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct DashboardData {
    pub events: Vec<PublicEvent>,
    pub qr: Option<ConcertQrOverview>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct ConcertQrOverview {
    pub events: Vec<StaffEvent>,
    pub campaigns: Vec<QrCampaign>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct StaffEvent {
    pub slug: String,
    pub title: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct QrCampaign {
    pub id: String,
    pub event_title: String,
    pub label: String,
    pub max_checkins: Option<u32>,
    pub checkin_count: u64,
    pub active: bool,
    pub token: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct PublicEvent {
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub city: Option<EventCity>,
    pub venue: Option<String>,
    pub starts_at: String,
    pub ticket_url: Option<String>,
    pub image_url: Option<String>,
    #[serde(default)]
    pub image_thumbnail_url: Option<String>,
}

#[derive(Clone, Debug, Deserialize, PartialEq)]
pub struct EventCity {
    pub name: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TicketingOverview {
    pub sale: TicketSale,
    #[serde(default)]
    pub paid_tickets: i64,
    #[serde(default)]
    pub gross_sales_minor: i64,
    #[serde(default)]
    pub refunded_minor: i64,
    #[serde(default)]
    pub recent_orders: Vec<TicketOrder>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TicketSale {
    pub currency: String,
    #[serde(default)]
    pub reserved: i32,
    pub available: i32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TicketOrder {
    pub public_reference: String,
    pub buyer_email_masked: String,
    pub buyer_name: Option<String>,
    pub currency: String,
    pub amount_gross_minor: i64,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AdmissionRedemption {
    pub public_reference: String,
    pub holder_name: Option<String>,
    pub holder_email_masked: String,
    pub status: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CouponEnvelope {
    pub result: CouponRedemption,
}

#[derive(Clone, Debug, Deserialize)]
pub struct CouponRedemption {
    pub status: String,
    pub used_count: u32,
    pub max_uses: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct IssuedPass {
    pub public_reference: String,
    pub claim_token: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct CreateQrCampaignInput {
    pub event_slug: String,
    pub label: String,
    pub valid_from: String,
    pub valid_until: String,
    pub max_checkins: Option<u32>,
}

#[derive(Clone, Debug, Serialize)]
pub struct IssuePassInput {
    pub event_slug: String,
    pub pool_slug: String,
    pub fan_email: String,
    pub claim_expires_hours: u32,
}

#[derive(Clone, Debug, Serialize)]
pub struct RequestedCityInput {
    pub name: String,
    pub region: Option<String>,
    pub country_code: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct RequestedCityResult {
    pub city_slug: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct FanSignupInput {
    pub api_base_url: String,
    pub email: String,
    pub display_name: Option<String>,
    pub city_slug: String,
    pub locale: String,
    pub referral_code: Option<String>,
    pub policy_version: String,
    pub nearby_gigs_enabled: bool,
    pub nearby_radius_km: u16,
}

#[derive(Clone, Debug, Serialize)]
pub struct FanConfirmationInput {
    pub api_base_url: String,
    pub email: String,
    pub display_name: Option<String>,
    pub token: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FanAuthResult {
    pub session_created: bool,
    #[serde(default)]
    pub email_kind: Option<String>,
    #[serde(default)]
    pub email_queued: Option<bool>,
    #[serde(default)]
    pub retry_after_seconds: Option<u32>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct FanDashboardData {
    pub events: Vec<PublicEvent>,
    pub referral: ReferralProgress,
    #[serde(default)]
    pub interests: Vec<FanEventInterest>,
    pub admission_pass: Option<AdmissionPass>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FanEventInterest {
    pub event: PublicEvent,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct ReferralProgress {
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

#[derive(Clone, Debug, Deserialize)]
pub struct WeightedDrawEntry {
    #[serde(default)]
    pub slug: String,
    pub name: String,
    pub prize_kind: String,
    pub draw_at: String,
    pub total_entries: u32,
}

#[derive(Clone, Debug, Deserialize)]
pub struct MerchCoupon {
    pub code: String,
    pub discount_percent: u32,
    pub status: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct PhysicalRewardGrant {
    pub item_name: String,
    pub sku: String,
    pub status: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AdmissionPass {
    pub event_title: String,
    pub venue: Option<String>,
    pub starts_at: String,
    pub public_reference: String,
    pub status: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AdmissionQr {
    pub token: String,
    pub expires_at: String,
    pub qr_svg: Option<String>,
}

// These response DTOs mirror the complete AREA wire contract. The current UI
// intentionally renders only a subset; retained fields stay narrowly allowed.
#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AreaWallet {
    #[serde(default)]
    #[allow(dead_code)]
    pub token_balance: u32,
    #[serde(default)]
    pub reward_credits: u32,
    #[serde(default)]
    pub collection_size: u32,
    #[serde(default)]
    pub community: AreaCommunity,
    #[serde(default)]
    pub claims: Vec<AreaClaim>,
    #[serde(default)]
    #[allow(dead_code)]
    pub vouchers: Vec<AreaVoucher>,
    #[serde(default)]
    pub live_drops: Vec<AreaLiveDrop>,
    #[serde(default)]
    pub drops: Vec<AreaDrop>,
    #[serde(default)]
    #[allow(dead_code)]
    pub migration_required: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct AreaCommunity {
    #[serde(default)]
    #[allow(dead_code)]
    pub current: u32,
    #[serde(default)]
    #[allow(dead_code)]
    pub total: u32,
    #[serde(default)]
    pub percent: f64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AreaClaim {
    pub drop_id: String,
    #[allow(dead_code)]
    pub number: String,
    #[allow(dead_code)]
    pub city: String,
    #[allow(dead_code)]
    pub line: String,
    #[allow(dead_code)]
    pub track: String,
    #[allow(dead_code)]
    pub edition: String,
    #[allow(dead_code)]
    pub claimed_at: String,
    #[serde(default)]
    #[allow(dead_code)]
    pub distance_meters: u32,
    #[allow(dead_code)]
    pub edition_number: Option<u32>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AreaVoucher {
    #[allow(dead_code)]
    pub code: String,
    #[allow(dead_code)]
    pub tokens: u32,
    #[allow(dead_code)]
    pub status: String,
    #[allow(dead_code)]
    pub expires_at: u64,
    #[allow(dead_code)]
    pub free_product_label: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AreaDrop {
    pub id: String,
    pub number: String,
    pub city: String,
    pub region: String,
    #[allow(dead_code)]
    pub signal_city_slug: String,
    pub map_x: i16,
    pub map_y: i16,
    pub approximate_lat: f64,
    pub approximate_lng: f64,
    #[serde(default)]
    pub clue: AreaDropClue,
    #[serde(default)]
    pub active: bool,
    #[serde(default)]
    pub full: bool,
    #[serde(default)]
    pub claimed: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct AreaDropClue {
    #[serde(default)]
    pub en: String,
    #[serde(default)]
    pub pl: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct AreaLiveDrop {
    #[allow(dead_code)]
    pub id: String,
}
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AreaChallenge {
    pub challenge: String,
    #[allow(dead_code)]
    pub issued_at: u64,
    #[allow(dead_code)]
    pub expires_at: u64,
    pub min_samples: u32,
    pub max_samples: u32,
    pub min_duration_ms: u32,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AreaPositionSample {
    pub lat: f64,
    pub lng: f64,
    pub accuracy: f64,
    pub captured_at: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AreaClaimResult {
    #[allow(dead_code)]
    pub ok: bool,
    pub already_claimed: bool,
    pub collectible: Option<AreaCollectible>,
    pub reward_credits_awarded: u32,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AreaCollectible {
    #[allow(dead_code)]
    pub drop_id: String,
    pub number: String,
    pub city: String,
    #[allow(dead_code)]
    pub line: String,
    pub track: String,
    pub edition: String,
    #[allow(dead_code)]
    pub riddle: String,
}

// Operator responses keep diagnostics that are not shown on the compact screen yet.
pub use virya_signal_contracts::ops::*;

#[derive(Clone, Debug, Deserialize)]
pub struct OpsOutboxItem {
    pub id: String,
    pub event_type: String,
    #[allow(dead_code)]
    pub status: String,
    pub attempts: i32,
    pub max_attempts: i32,
    pub last_error_kind: Option<String>,
    #[allow(dead_code)]
    pub dead_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct OpsDeliveryItem {
    pub id: String,
    pub event_type: String,
    pub endpoint_name: String,
    pub endpoint_active: bool,
    #[allow(dead_code)]
    pub status: String,
    pub attempt_count: i32,
    pub max_attempts: i32,
    #[allow(dead_code)]
    pub last_response_status: Option<i16>,
    pub last_error_kind: Option<String>,
    #[allow(dead_code)]
    pub dead_at: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
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
    pub audience: AudienceSummary,
    #[serde(default)]
    pub ticket_revenue: Vec<AudienceRevenueSummary>,
    #[serde(default)]
    pub unavailable_sources: Vec<String>,
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

#[derive(Clone, Debug, Deserialize)]
pub struct OpsRetryResult {
    #[allow(dead_code)]
    pub operation_id: String,
    #[allow(dead_code)]
    pub target_type: String,
    #[allow(dead_code)]
    pub target_id: String,
    #[allow(dead_code)]
    pub status: String,
    pub replayed: bool,
}

// Show mode keeps the complete native response for stable IPC compatibility.
#[derive(Clone, Debug, Default, Deserialize)]
pub struct ShowModeStatus {
    pub prepared: bool,
    #[allow(dead_code)]
    pub event_slug: String,
    #[allow(dead_code)]
    pub event_title: Option<String>,
    #[allow(dead_code)]
    pub expires_at: Option<String>,
    pub eligible_passes: usize,
    pub pending: usize,
    #[allow(dead_code)]
    pub synced: usize,
    pub conflicts: usize,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ShowModeScanState {
    Pending,
    Synced,
    Conflict,
}

#[derive(Clone, Debug, Deserialize)]
pub struct ShowModeScanResult {
    #[allow(dead_code)]
    pub accepted: bool,
    pub duplicate: bool,
    pub public_reference: String,
    pub holder_name: Option<String>,
    pub holder_email_masked: String,
    #[allow(dead_code)]
    pub state: ShowModeScanState,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct ShowModeSyncResult {
    #[allow(dead_code)]
    pub attempted: usize,
    pub synced: usize,
    pub conflicts: usize,
    pub pending: usize,
}

#[derive(Clone, Debug, Deserialize)]
pub struct TicketWallet {
    pub order: WalletOrder,
    #[serde(default)]
    pub tickets: Vec<WalletTicket>,
    #[serde(default)]
    pub cached: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct WalletBatch {
    #[serde(default)]
    pub wallets: Vec<TicketWallet>,
    #[serde(default)]
    pub failed_count: usize,
    #[serde(default)]
    pub cached_count: usize,
}

#[derive(Clone, Debug, Deserialize)]
pub struct WalletOrder {
    pub order_id: String,
    pub public_reference: String,
    pub event_title: String,
    pub venue: Option<String>,
    pub starts_at: String,
    pub status: String,
}

#[derive(Clone, Debug, Deserialize)]
pub struct WalletTicket {
    pub ticket_type_name: String,
    pub public_reference: String,
    pub holder_name: Option<String>,
    pub holder_email_masked: String,
    pub qr_available: bool,
    pub qr_expires_at: String,
}
