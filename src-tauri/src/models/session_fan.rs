#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq, Zeroize)]
#[zeroize(drop)]
pub struct OperatorProfile {
    pub display_name: String,
    pub api_base_url: String,
    #[zeroize(skip)]
    pub role: OperatorRole,
    pub bearer_token: String,
    #[serde(default)]
    #[zeroize(skip)]
    pub session_expires_at: Option<u64>,
}

#[derive(Clone, Debug, Serialize)]
pub struct SessionSummary {
    pub display_name: String,
    pub api_base_url: String,
    pub role: OperatorRole,
    pub session_expires_at: Option<u64>,
}

impl From<&OperatorProfile> for SessionSummary {
    fn from(value: &OperatorProfile) -> Self {
        Self {
            display_name: value.display_name.clone(),
            api_base_url: value.api_base_url.clone(),
            role: value.role.clone(),
            session_expires_at: value.session_expires_at,
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
    #[serde(default)]
    pub push_enabled: bool,
    #[serde(default)]
    pub push_last_sync_ok: bool,
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
    pub best_elapsed_ms: Option<i64>,
    pub completed_runs: i64,
    pub leaderboard_published: bool,
    pub leaderboard_rank: Option<i64>,
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
