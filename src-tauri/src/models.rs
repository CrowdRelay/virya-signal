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

#[derive(Clone, Debug, Serialize)]
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

#[derive(Clone, Debug, Default, Serialize)]
pub struct SessionStatus {
    pub configured: bool,
    pub unlocked: bool,
    pub session: Option<SessionSummary>,
}

#[derive(Clone, Debug, Deserialize, Serialize, Zeroize)]
#[zeroize(drop)]
pub struct WalletCredential {
    pub order_id: String,
    pub checkout_token: String,
}

#[derive(Debug, Serialize)]
pub struct WalletBatch {
    pub wallets: Vec<TicketWallet>,
    pub failed_count: usize,
}

#[derive(Debug, Deserialize)]
pub struct TicketWalletApi {
    pub order: WalletOrder,
    #[serde(default)]
    pub tickets: Vec<WalletTicketApi>,
}

#[derive(Debug, Serialize)]
pub struct TicketWallet {
    pub order: WalletOrder,
    pub tickets: Vec<WalletTicket>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct WalletOrder {
    pub order_id: String,
    pub public_reference: String,
    pub event_title: String,
    pub venue: Option<String>,
    pub starts_at: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct WalletTicketApi {
    pub ticket_type_name: String,
    pub public_reference: String,
    pub holder_name: Option<String>,
    pub holder_email_masked: String,
    pub qr_token: Option<String>,
    pub qr_expires_at: String,
}

#[derive(Debug, Serialize)]
pub struct WalletTicket {
    pub ticket_type_name: String,
    pub public_reference: String,
    pub holder_name: Option<String>,
    pub holder_email_masked: String,
    pub qr_available: bool,
    pub qr_expires_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Zeroize)]
#[zeroize(drop)]
pub struct FanProfile {
    pub api_base_url: String,
    #[serde(default = "new_area_wallet_id")]
    pub area_wallet_id: String,
    pub email: String,
    pub display_name: Option<String>,
    pub fan_session_token: String,
    pub pass_session_token: Option<String>,
    pub wallets: Vec<WalletCredential>,
}

fn new_area_wallet_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

#[derive(Clone, Debug, Serialize)]
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

#[derive(Clone, Debug, Default, Serialize)]
pub struct FanSessionStatus {
    pub configured: bool,
    pub unlocked: bool,
    pub session: Option<FanSummary>,
}

#[derive(Clone, Debug, Deserialize)]
pub struct EventListResponse {
    pub events: Vec<PublicEvent>,
}

#[derive(Clone, Debug, Deserialize)]
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
    pub slug: String,
    pub title: String,
    pub description: Option<String>,
    pub city: Option<EventCity>,
    pub venue: Option<String>,
    pub starts_at: String,
    pub ticket_url: Option<String>,
    pub image_url: Option<String>,
    #[serde(default, alias = "thumbnail_url", alias = "image_mobile_url")]
    pub image_thumbnail_url: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct EventCity {
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
    pub name: String,
    pub prize_kind: String,
    pub draw_at: String,
    pub total_entries: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct MerchCoupon {
    pub code: String,
    pub discount_percent: u32,
    pub status: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PhysicalRewardGrant {
    pub item_name: String,
    pub sku: String,
    pub status: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FanEventInterest {
    pub event: PublicEvent,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AdmissionPass {
    pub event_title: String,
    pub venue: Option<String>,
    pub starts_at: String,
    pub public_reference: String,
    pub status: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AreaWallet {
    #[serde(default)]
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
    pub vouchers: Vec<AreaVoucher>,
    #[serde(default)]
    pub live_drops: Vec<AreaLiveDrop>,
    #[serde(default)]
    pub migration_required: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AreaCommunity {
    #[serde(default)]
    pub current: u32,
    #[serde(default)]
    pub total: u32,
    #[serde(default)]
    pub percent: f64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AreaClaim {
    pub drop_id: String,
    pub number: String,
    pub city: String,
    pub line: String,
    pub track: String,
    pub edition: String,
    pub claimed_at: String,
    pub edition_number: Option<u32>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AreaVoucher {
    pub code: String,
    pub tokens: u32,
    pub status: String,
    pub expires_at: u64,
    pub free_product_label: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AreaLiveDrop {
    pub id: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct QueueSummary {
    #[serde(default)]
    pub pending: i64,
    #[serde(default)]
    pub processing: i64,
    #[serde(default)]
    pub delivered_24h: i64,
    #[serde(default)]
    pub dead: i64,
    #[serde(default)]
    pub oldest_pending_seconds: i64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OpsSummary {
    #[serde(default)]
    pub outbox: QueueSummary,
    #[serde(default)]
    pub deliveries: QueueSummary,
}

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

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct OpsRetryResult {
    pub operation_id: String,
    pub target_type: String,
    pub target_id: String,
    pub status: String,
    pub replayed: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ShowModeSnapshot {
    pub schema_version: u32,
    pub snapshot_id: String,
    pub event: ShowModeEvent,
    pub generated_at: String,
    pub expires_at: String,
    pub checksum_sha256: String,
    #[serde(default)]
    pub passes: Vec<ShowModePass>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ShowModeEvent {
    pub slug: String,
    pub title: String,
    pub venue: Option<String>,
    pub starts_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ShowModePass {
    pub public_reference: String,
    pub holder_name: Option<String>,
    pub holder_email_masked: String,
    pub ticket_type_name: Option<String>,
    pub offline_eligible: bool,
    pub qr_sha256: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ShowModeScanState {
    Pending,
    Synced,
    Conflict,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ShowModeQueuedScan {
    pub scan_id: String,
    pub public_reference: String,
    pub holder_name: Option<String>,
    pub holder_email_masked: String,
    pub scanned_at_unix_secs: u64,
    pub state: ShowModeScanState,
    pub result_status: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ShowModeStore {
    #[serde(default)]
    pub sessions: std::collections::HashMap<String, ShowModeSession>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ShowModeSession {
    pub snapshot: ShowModeSnapshot,
    #[serde(default)]
    pub scans: Vec<ShowModeQueuedScan>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ShowModeStatus {
    pub prepared: bool,
    pub event_slug: String,
    pub event_title: Option<String>,
    pub expires_at: Option<String>,
    pub eligible_passes: usize,
    pub pending: usize,
    pub synced: usize,
    pub conflicts: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ShowModeScanResult {
    pub accepted: bool,
    pub duplicate: bool,
    pub public_reference: String,
    pub holder_name: Option<String>,
    pub holder_email_masked: String,
    pub state: ShowModeScanState,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ShowModeSyncResult {
    pub attempted: usize,
    pub synced: usize,
    pub conflicts: usize,
    pub pending: usize,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RequestedCityInput {
    pub name: String,
    pub region: Option<String>,
    pub country_code: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RequestedCityResult {
    pub city_slug: String,
    pub display_name: String,
    pub status: String,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StaffPairingPayload {
    pub version: u8,
    pub api_base_url: String,
    pub display_name: String,
    pub role: OperatorRole,
    pub bearer_token: String,
    pub expires_at: u64,
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
    pub nearby_gigs_enabled: bool,
    pub nearby_radius_km: u16,
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

#[cfg(test)]
mod tests {
    use super::*;

    fn test_value<T, E>(result: Result<T, E>) -> T
    where
        E: std::fmt::Debug,
    {
        match result {
            Ok(value) => value,
            Err(error) => panic!("test setup failed: {error:?}"),
        }
    }

    fn test_some<T>(value: Option<T>) -> T {
        match value {
            Some(value) => value,
            None => panic!("test setup expected Some value"),
        }
    }

    #[test]
    fn public_event_ignores_backend_fields_outside_the_webview_contract() {
        let event: PublicEvent = test_value(serde_json::from_value(serde_json::json!({
            "id": "backend-only",
            "slug": "virya-live",
            "title": "Virya Live",
            "description": "Concert",
            "city": {"id": "backend-only", "name": "Wrocław"},
            "venue": "Club",
            "starts_at": "2026-08-01T20:00:00Z",
            "ticket_url": "https://virya.music/tickets",
            "image_url": null,
            "large_backend_object": {"unused": true}
        })));
        assert_eq!(event.slug, "virya-live");
        assert_eq!(test_some(event.city).name, "Wrocław");
    }

    #[test]
    fn ticketing_overview_keeps_only_rendered_fields() {
        let overview: TicketingOverview = test_value(serde_json::from_value(serde_json::json!({
            "sale": {
                "currency": "PLN",
                "reserved": 3,
                "available": 97,
                "ticket_types": [{"id": "unused"}]
            },
            "paid_tickets": 42,
            "gross_sales_minor": 123400,
            "refunded_minor": 0,
            "recent_orders": [{
                "public_reference": "VRY-ORDER",
                "buyer_email_masked": "f***@example.com",
                "buyer_name": "Fan",
                "currency": "PLN",
                "amount_gross_minor": 9900,
                "tickets": [{"private": "unused"}]
            }]
        })));
        assert_eq!(overview.sale.available, 97);
        assert_eq!(overview.recent_orders.len(), 1);
    }

    #[test]
    fn referral_payload_accepts_extra_reward_metadata() {
        let referral: ReferralProgress = test_value(serde_json::from_value(serde_json::json!({
            "referral_code": "VIRYA",
            "qualified_referrals": 2,
            "pending_referrals": 1,
            "draw_entries": [{
                "name": "Backstage",
                "prize_kind": "pass",
                "draw_at": "2026-08-02T20:00:00Z",
                "total_entries": 4,
                "max_entries": 99
            }],
            "coupons": [],
            "physical_rewards": []
        })));
        assert_eq!(referral.draw_entries[0].total_entries, 4);
    }
}
