use serde::{Deserialize, Deserializer, Serialize, de::Error as _};
use serde_json::{Map, Value};
use zeroize::Zeroize;

fn deserialize_string_or_bytes<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    normalize_compat_string(value).map_err(D::Error::custom)
}

fn deserialize_optional_string_or_bytes<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    value
        .map(normalize_compat_string)
        .transpose()
        .map_err(D::Error::custom)
}

fn deserialize_string_or_default<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    match value {
        Some(value) => normalize_compat_string(value).map_err(D::Error::custom),
        None => Ok(String::new()),
    }
}

fn deserialize_area_wallet_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    match value {
        Some(value) => {
            let normalized = normalize_compat_string(value).map_err(D::Error::custom)?;
            if normalized.trim().is_empty() {
                Ok(new_area_wallet_id())
            } else {
                Ok(normalized)
            }
        }
        None => Ok(new_area_wallet_id()),
    }
}

fn normalize_compat_string(value: Value) -> Result<String, String> {
    match value {
        Value::String(value) => Ok(value),
        Value::Array(values) => normalize_compat_array(values),
        Value::Object(values) => normalize_compat_object(values),
        Value::Number(value) => Ok(value.to_string()),
        Value::Bool(value) => Ok(value.to_string()),
        Value::Null => Err("expected text, received null".to_owned()),
    }
}

fn normalize_compat_array(values: Vec<Value>) -> Result<String, String> {
    if values.is_empty() {
        return Ok(String::new());
    }

    if values.iter().all(Value::is_number) {
        return decode_numeric_sequence(&values);
    }

    if values.iter().all(Value::is_string) {
        let mut output = String::new();
        for value in values {
            match value {
                Value::String(value) => output.push_str(&value),
                _ => return Err("text sequence contained a non-text element".to_owned()),
            }
        }
        return Ok(output);
    }

    if values.len() == 1 {
        let value = values.into_iter().next().ok_or_else(|| {
            "expected one compatibility value, but the sequence was empty".to_owned()
        })?;
        return normalize_compat_string(value);
    }

    let mut output = String::new();
    for value in values {
        output.push_str(&normalize_compat_string(value)?);
    }
    Ok(output)
}

fn normalize_compat_object(mut values: Map<String, Value>) -> Result<String, String> {
    const WRAPPER_KEYS: [&str; 12] = [
        "value",
        "data",
        "bytes",
        "buffer",
        "buf",
        "content",
        "string",
        "chars",
        "code_units",
        "String",
        "Bytes",
        "$value",
    ];

    for key in WRAPPER_KEYS {
        if let Some(value) = values.remove(key) {
            return normalize_compat_string(value);
        }
    }

    if values.keys().all(|key| key.parse::<usize>().is_ok()) {
        let mut indexed_values = Vec::with_capacity(values.len());
        for (key, value) in values {
            let index = key
                .parse::<usize>()
                .map_err(|_| format!("unsupported text object key `{key}`"))?;
            indexed_values.push((index, value));
        }
        indexed_values.sort_unstable_by_key(|(index, _)| *index);

        for (expected, (actual, _)) in indexed_values.iter().enumerate() {
            if *actual != expected {
                return Err(format!(
                    "text object byte indexes are not contiguous: expected {expected}, received {actual}"
                ));
            }
        }

        return normalize_compat_array(
            indexed_values.into_iter().map(|(_, value)| value).collect(),
        );
    }

    if values.len() == 1 {
        let value = values
            .into_iter()
            .next()
            .map(|(_, value)| value)
            .ok_or_else(|| "expected one compatibility object value".to_owned())?;
        return normalize_compat_string(value);
    }

    Err(format!(
        "unsupported text object with keys: {}",
        values.keys().cloned().collect::<Vec<_>>().join(", ")
    ))
}

fn decode_numeric_sequence(values: &[Value]) -> Result<String, String> {
    let code_units = values
        .iter()
        .map(value_as_u16)
        .collect::<Result<Vec<_>, _>>()?;

    if looks_like_utf16_le_bytes(&code_units) {
        return decode_utf16_bytes(&code_units, true);
    }
    if looks_like_utf16_be_bytes(&code_units) {
        return decode_utf16_bytes(&code_units, false);
    }

    if code_units.iter().all(|value| *value <= u8::MAX as u16) {
        let bytes = code_units
            .iter()
            .map(|value| *value as u8)
            .collect::<Vec<_>>();
        if let Ok(text) = String::from_utf8(bytes) {
            return Ok(text);
        }
    }

    String::from_utf16(&code_units)
        .map_err(|error| format!("invalid UTF-8/UTF-16 text sequence: {error}"))
}

fn value_as_u16(value: &Value) -> Result<u16, String> {
    if let Some(number) = value.as_u64() {
        return u16::try_from(number)
            .map_err(|_| format!("text code unit {number} exceeds the UTF-16 range"));
    }

    if let Some(number) = value.as_i64()
        && (-128..=-1).contains(&number)
    {
        return Ok((number as i8 as u8) as u16);
    }

    Err(format!(
        "expected a byte or UTF-16 code unit, received {value}"
    ))
}

fn looks_like_utf16_le_bytes(values: &[u16]) -> bool {
    values.len() >= 2
        && values.len().is_multiple_of(2)
        && values
            .iter()
            .enumerate()
            .all(|(index, value)| index % 2 == 0 || *value == 0)
}

fn looks_like_utf16_be_bytes(values: &[u16]) -> bool {
    values.len() >= 2
        && values.len().is_multiple_of(2)
        && values
            .iter()
            .enumerate()
            .all(|(index, value)| index % 2 != 0 || *value == 0)
}

fn decode_utf16_bytes(values: &[u16], little_endian: bool) -> Result<String, String> {
    let mut code_units = Vec::with_capacity(values.len() / 2);
    for pair in values.chunks_exact(2) {
        let first = pair[0] as u8;
        let second = pair[1] as u8;
        code_units.push(if little_endian {
            u16::from_le_bytes([first, second])
        } else {
            u16::from_be_bytes([first, second])
        });
    }
    String::from_utf16(&code_units)
        .map_err(|error| format!("invalid UTF-16 byte sequence: {error}"))
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OperatorRole {
    Owner,
    Staff,
}

mod staff_pairing;
pub use staff_pairing::{StaffPairingExchange, StaffPairingPayload};

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Zeroize)]
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
    #[serde(deserialize_with = "deserialize_string_or_bytes")]
    pub order_id: String,
    #[serde(deserialize_with = "deserialize_string_or_bytes")]
    pub checkout_token: String,
}

#[derive(Debug, Serialize)]
pub struct WalletBatch {
    pub wallets: Vec<TicketWallet>,
    pub failed_count: usize,
    pub cached_count: usize,
}

#[derive(Debug, Deserialize)]
pub struct TicketWalletApi {
    pub order: WalletOrder,
    #[serde(default)]
    pub tickets: Vec<WalletTicketApi>,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct TicketWallet {
    pub order: WalletOrder,
    pub tickets: Vec<WalletTicket>,
    #[serde(default)]
    pub cached: bool,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
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
pub struct WalletQrCredential {
    pub order_id: String,
    pub public_reference: String,
    pub token: String,
    pub expires_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize, Zeroize)]
#[zeroize(drop)]
pub struct FanProfile {
    #[serde(deserialize_with = "deserialize_string_or_bytes")]
    pub api_base_url: String,
    #[serde(
        default = "new_area_wallet_id",
        deserialize_with = "deserialize_area_wallet_id"
    )]
    pub area_wallet_id: String,
    #[serde(deserialize_with = "deserialize_string_or_bytes")]
    pub email: String,
    #[serde(default, deserialize_with = "deserialize_optional_string_or_bytes")]
    pub display_name: Option<String>,
    #[serde(deserialize_with = "deserialize_string_or_bytes")]
    pub fan_session_token: String,
    #[serde(default, deserialize_with = "deserialize_optional_string_or_bytes")]
    pub pass_session_token: Option<String>,
    #[serde(default)]
    pub wallets: Vec<WalletCredential>,
    #[serde(default)]
    #[zeroize(skip)]
    pub cached_wallets: Vec<TicketWallet>,
    #[serde(default)]
    pub cached_wallet_qr: Vec<WalletQrCredential>,
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

#[derive(Clone, Debug, Default, Serialize)]
pub struct LauncherStatus {
    pub operator: SessionStatus,
    pub fan: FanSessionStatus,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FanHomeProfile {
    pub display_name: Option<String>,
    pub locale: Option<String>,
    pub primary_city: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct FanHomeEvent {
    pub slug: String,
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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FanHomeSynesthesia {
    pub started: bool,
    pub completed: bool,
    pub rooms_completed: i16,
    pub client_total_elapsed_ms: Option<i64>,
    pub linked_at: Option<String>,
    pub reward_entered: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FanHomeReferral {
    pub qualified: i64,
    pub pending: i64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct FanHomeCounts {
    pub event_interests: i64,
    pub active_passes: i64,
    pub paid_orders: i64,
    pub area_claims: i64,
}

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
    pub drops: Vec<AreaDrop>,
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
    #[serde(default)]
    pub distance_meters: u32,
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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AreaDropClue {
    #[serde(default)]
    pub en: String,
    #[serde(default)]
    pub pl: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AreaLiveDrop {
    pub id: String,
}
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AreaChallenge {
    pub challenge: String,
    pub issued_at: u64,
    pub expires_at: u64,
    pub min_samples: u32,
    pub max_samples: u32,
    pub min_duration_ms: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AreaPositionSample {
    pub lat: f64,
    pub lng: f64,
    pub accuracy: f64,
    pub captured_at: u64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AreaClaimResult {
    pub ok: bool,
    pub already_claimed: bool,
    pub collectible: Option<AreaCollectible>,
    pub reward_credits_awarded: u32,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AreaCollectible {
    pub drop_id: String,
    pub number: String,
    pub city: String,
    pub line: String,
    pub track: String,
    pub edition: String,
    #[allow(dead_code)]
    pub riddle: String,
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
pub struct DatabaseRuntimeSummary {
    #[serde(default)]
    pub server_version_num: i32,
    #[serde(default)]
    pub io_method: Option<String>,
    #[serde(default)]
    pub io_workers: Option<i32>,
    #[serde(default)]
    pub io_max_concurrency: Option<i32>,
    #[serde(default)]
    pub effective_io_concurrency: Option<i32>,
    #[serde(default)]
    pub maintenance_io_concurrency: Option<i32>,
    #[serde(default)]
    pub io_combine_limit_bytes: Option<i64>,
    #[serde(default)]
    pub io_max_combine_limit_bytes: Option<i64>,
    #[serde(default)]
    pub async_io_active: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AreaRuntimeSummary {
    #[serde(default)]
    pub credits_total: i64,
    #[serde(default)]
    pub vouchers_issued: i64,
    #[serde(default)]
    pub stale_voucher_reservations: i64,
    #[serde(default)]
    pub ticket_rewards_issued: i64,
    #[serde(default)]
    pub stale_ticket_reward_reservations: i64,
    #[serde(default)]
    pub legacy_imported_players: i64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OpsSummary {
    #[serde(default)]
    pub outbox: QueueSummary,
    #[serde(default)]
    pub deliveries: QueueSummary,
    #[serde(default)]
    pub http: HttpRequestSummary,
    #[serde(default)]
    pub database: DatabaseRuntimeSummary,
    #[serde(default)]
    pub area: AreaRuntimeSummary,
    #[serde(default)]
    pub schema_version: u32,
    #[serde(default)]
    pub release: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct HttpRequestSummary {
    #[serde(default)]
    pub requests: u64,
    #[serde(default)]
    pub errors_4xx: u64,
    #[serde(default)]
    pub errors_5xx: u64,
    #[serde(default)]
    pub average_ms: u64,
    #[serde(default)]
    pub p50_ms: u64,
    #[serde(default)]
    pub p95_ms: u64,
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

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AutopilotPolicySummary {
    pub context: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub autonomy_level: String,
    #[serde(default)]
    pub minimum_confidence: u16,
    #[serde(default)]
    pub max_actions_24h: u32,
    #[serde(default)]
    pub version: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExperimentAllocation {
    pub variant_id: String,
    pub allocation_basis_points: u16,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AutopilotActionPayload {
    ChangeTicketPrice {
        ticket_type_id: String,
        from_minor: i64,
        to_minor: i64,
    },
    ChangeTicketCapacity {
        ticket_type_id: String,
        from_capacity: u32,
        to_capacity: u32,
        guardrail_version: i64,
    },
    RequestFanLifecycleMessage {
        fan_id: String,
        template_key: String,
    },
    RequestMerchReorder {
        variant_id: String,
        quantity: u32,
    },
    ChangeMerchPrice {
        product_id: String,
        from_minor: i64,
        to_minor: i64,
        economics_version: i64,
    },
    RequestBookingOutreach {
        city_id: String,
        target_id: String,
        target_version: i64,
        target_name: String,
        score: u16,
        phase: String,
    },
    RequestAudienceCampaign {
        event_id: String,
        phase: String,
        template_key: String,
    },
    RequestMerchBundle {
        product_a: String,
        product_b: String,
        bundle_price_minor: i64,
        affinity_basis_points: u16,
    },
    RequestOutreach {
        opportunity_id: String,
        target_id: String,
        target_version: i64,
        target_name: String,
        phase: String,
        template_key: String,
    },
    RequestContentArtifact {
        source_id: String,
        source_version: i64,
        artifact: String,
        template_key: String,
    },
    AdjustExperiment {
        experiment_id: String,
        expected_version: i64,
        winner_variant_id: String,
        allocations: Vec<ExperimentAllocation>,
        complete: bool,
    },
    CompleteShowTask {
        event_id: String,
        task: String,
    },
    EscalateShowTask {
        event_id: String,
        task: String,
    },
    RequestPromotionBudgetChange {
        campaign_id: String,
        from_minor: i64,
        to_minor: i64,
        roas_basis_points: u32,
    },
    ExecuteReleaseMilestone {
        release_id: String,
        title: String,
        release_at: String,
        milestone: String,
    },
    ApplyLiveOpportunity {
        opportunity_id: String,
        opportunity_kind: String,
        score: u16,
    },
    PrepareFundingPackage {
        opportunity_id: String,
    },
    SubmitFundingApplication {
        opportunity_id: String,
    },
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PendingAutopilotAction {
    pub id: String,
    pub context: String,
    pub action_kind: String,
    pub subject_kind: String,
    pub subject_id: String,
    pub payload: AutopilotActionPayload,
    pub created_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RecentAutopilotDecision {
    pub id: String,
    pub context: String,
    pub decision_kind: String,
    pub confidence: u16,
    pub disposition: String,
    pub reason: String,
    pub evaluated_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RecentAutopilotAction {
    pub id: String,
    pub context: String,
    pub action_kind: String,
    pub subject_kind: String,
    pub subject_id: String,
    pub status: String,
    pub attempt_count: u32,
    pub created_at: String,
    pub finished_at: Option<String>,
    pub last_error_kind: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RecentAutopilotEffect {
    pub measurement_id: String,
    pub action_id: String,
    pub context: String,
    pub measurement_kind: String,
    pub assessment: String,
    pub delta_basis_points: i32,
    pub baseline_value: f64,
    pub observed_value: f64,
    pub observed_at: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OperatorAutopilotOverview {
    #[serde(default)]
    pub runtime_enabled: bool,
    #[serde(default)]
    pub policies: Vec<AutopilotPolicySummary>,
    #[serde(default)]
    pub needs_you: Vec<PendingAutopilotAction>,
    #[serde(default)]
    pub recent_decisions: Vec<RecentAutopilotDecision>,
    #[serde(default)]
    pub recent_actions: Vec<RecentAutopilotAction>,
    #[serde(default)]
    pub recent_effects: Vec<RecentAutopilotEffect>,
    #[serde(default)]
    pub queued_actions: i64,
    #[serde(default)]
    pub processing_actions: i64,
    #[serde(default)]
    pub succeeded_24h: i64,
    #[serde(default)]
    pub failed_24h: i64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ChiefOfStaffOpportunity {
    pub context: String,
    pub decision_kind: String,
    pub subject_kind: String,
    pub subject_id: String,
    pub confidence: u16,
    pub reason: String,
    pub needs_approval: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ChiefOfStaffShowTask {
    pub event_id: String,
    pub event_title: String,
    pub task_key: String,
    pub status: String,
    pub starts_at: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AutopilotChiefOfStaff {
    #[serde(default)]
    pub executed_24h: i64,
    #[serde(default)]
    pub failed_24h: i64,
    #[serde(default)]
    pub needs_you: i64,
    #[serde(default)]
    pub estimated_minutes_saved_24h: i64,
    #[serde(default)]
    pub measured_improved_7d: i64,
    #[serde(default)]
    pub measured_neutral_7d: i64,
    #[serde(default)]
    pub measured_worsened_7d: i64,
    #[serde(default)]
    pub top_opportunities: Vec<ChiefOfStaffOpportunity>,
    #[serde(default)]
    pub show_tasks: Vec<ChiefOfStaffShowTask>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AutopilotMutation {
    pub operation_id: String,
    pub target_id: String,
    pub status: String,
    #[serde(default)]
    pub replayed: bool,
}

#[derive(Clone, Debug, Serialize)]
pub struct AutopilotAuthorityRequest {
    pub enabled: bool,
    pub autonomy_level: String,
    pub minimum_confidence_basis_points: u16,
    pub max_actions_24h: u32,
    pub expected_version: i64,
}

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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EcosystemMeta {
    pub api_version: String,
    pub schema_version: u32,
    pub release: String,
    pub build_timestamp: Option<String>,
    pub minimum_postgres_server_version_num: i32,
    pub capabilities: std::collections::BTreeMap<String, bool>,
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
    #[serde(default)]
    pub email_kind: Option<String>,
    #[serde(default)]
    pub email_queued: Option<bool>,
    #[serde(default)]
    pub retry_after_seconds: Option<u32>,
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

    #[test]
    fn referral_payload_accepts_legacy_byte_sequences_for_text_fields() {
        let referral: ReferralProgress = test_value(serde_json::from_value(serde_json::json!({
            "referral_code": b"VIRYA".to_vec(),
            "qualified_referrals": 2,
            "pending_referrals": 1,
            "draw_entries": [{
                "name": b"Backstage".to_vec(),
                "prize_kind": b"pass".to_vec(),
                "draw_at": b"2026-08-02T20:00:00Z".to_vec(),
                "total_entries": 4
            }],
            "coupons": [{
                "code": b"VIRYA10".to_vec(),
                "discount_percent": 10,
                "status": b"active".to_vec()
            }],
            "physical_rewards": [{
                "item_name": b"Album".to_vec(),
                "sku": b"ALBUM-01".to_vec(),
                "status": b"granted".to_vec()
            }]
        })));
        assert_eq!(referral.referral_code, "VIRYA");
        assert_eq!(referral.draw_entries[0].name, "Backstage");
        assert_eq!(referral.coupons[0].code, "VIRYA10");
        assert_eq!(referral.physical_rewards[0].sku, "ALBUM-01");
    }
}

#[cfg(test)]
mod fan_profile_compat_tests {
    use super::*;

    #[test]
    fn legacy_byte_sequences_are_migrated_to_strings() {
        let payload = serde_json::json!({
            "api_base_url": b"https://signal-api.virya.music/v1/".to_vec(),
            "email": b"fan@example.com".to_vec(),
            "display_name": b"Fan".to_vec(),
            "fan_session_token": b"session-token".to_vec(),
            "pass_session_token": null,
            "wallets": [{
                "order_id": b"01234567-89ab-cdef-0123-456789abcdef".to_vec(),
                "checkout_token": b"checkout-token".to_vec()
            }]
        });
        let profile: FanProfile = match serde_json::from_value(payload) {
            Ok(profile) => profile,
            Err(error) => panic!("legacy profile migration failed: {error}"),
        };
        assert_eq!(profile.email, "fan@example.com");
        assert_eq!(profile.fan_session_token, "session-token");
        assert_eq!(profile.wallets[0].checkout_token, "checkout-token");
        assert!(!profile.area_wallet_id.is_empty());
    }
}

#[cfg(test)]
mod compat_string_shape_tests {
    use super::*;

    fn normalized(value: Value) -> String {
        match normalize_compat_string(value) {
            Ok(value) => value,
            Err(error) => panic!("compatibility normalization failed: {error}"),
        }
    }

    #[test]
    fn accepts_node_buffer_objects() {
        assert_eq!(
            normalized(serde_json::json!({
                "type": "Buffer",
                "data": [86, 73, 82, 89, 65]
            })),
            "VIRYA"
        );
    }

    #[test]
    fn accepts_indexed_byte_objects() {
        assert_eq!(
            normalized(serde_json::json!({
                "0": 86,
                "1": 73,
                "2": 82,
                "3": 89,
                "4": 65
            })),
            "VIRYA"
        );
    }

    #[test]
    fn accepts_character_sequences() {
        assert_eq!(
            normalized(serde_json::json!(["V", "I", "R", "Y", "A"])),
            "VIRYA"
        );
    }

    #[test]
    fn accepts_signed_utf8_byte_sequences() {
        assert_eq!(normalized(serde_json::json!([-59, -68])), "ż");
    }

    #[test]
    fn accepts_utf16_code_units() {
        assert_eq!(
            normalized(serde_json::json!([86, 105, 114, 121, 97, 32, 281])),
            "Virya ę"
        );
    }

    #[test]
    fn accepts_wrapped_compatibility_values() {
        assert_eq!(
            normalized(serde_json::json!({
                "value": {"bytes": [86, 73, 82, 89, 65]}
            })),
            "VIRYA"
        );
    }

    #[test]
    fn null_area_wallet_id_is_regenerated() {
        let payload = serde_json::json!({
            "api_base_url": "https://signal-api.virya.music/v1/",
            "area_wallet_id": null,
            "email": "fan@example.com",
            "display_name": null,
            "fan_session_token": "session-token",
            "pass_session_token": null,
            "wallets": []
        });
        let profile: FanProfile = match serde_json::from_value(payload) {
            Ok(profile) => profile,
            Err(error) => panic!("null wallet id migration failed: {error}"),
        };
        assert!(!profile.area_wallet_id.is_empty());
    }

    #[test]
    fn null_referral_code_uses_empty_compatibility_value() {
        let payload = serde_json::json!({
            "referral_code": null,
            "qualified_referrals": 0,
            "pending_referrals": 0,
            "draw_entries": [],
            "coupons": [],
            "physical_rewards": []
        });
        let referral: ReferralProgress = match serde_json::from_value(payload) {
            Ok(referral) => referral,
            Err(error) => panic!("null referral migration failed: {error}"),
        };
        assert!(referral.referral_code.is_empty());
    }
}
