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
    pub git_sha: Option<String>,
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

