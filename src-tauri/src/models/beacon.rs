#[derive(Clone, Debug, Deserialize, Serialize, Zeroize)]
#[zeroize(drop)]
pub struct BeaconProfile {
    pub api_base_url: String,
    pub beacon_id: String,
    pub display_name: String,
    pub beacon_kind: String,
    pub bearer_token: String,
    pub session_id: String,
    pub client_kind: String,
    pub expires_at: String,
    #[serde(default)]
    pub push_enabled: bool,
    #[serde(default)]
    pub push_last_sync_ok: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct BeaconSummary {
    pub beacon_id: String,
    pub display_name: String,
    pub beacon_kind: String,
    pub expires_at: String,
}

impl From<&BeaconProfile> for BeaconSummary {
    fn from(value: &BeaconProfile) -> Self {
        Self {
            beacon_id: value.beacon_id.clone(),
            display_name: value.display_name.clone(),
            beacon_kind: value.beacon_kind.clone(),
            expires_at: value.expires_at.clone(),
        }
    }
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct BeaconSessionStatus {
    pub configured: bool,
    pub unlocked: bool,
    pub session: Option<BeaconSummary>,
    #[serde(default)]
    pub phase: BeaconSessionPhase,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BeaconPreferences {
    pub radius_km: i32,
    pub locale: String,
    #[serde(default)]
    pub topics: Vec<String>,
    pub nearby_gigs_enabled: bool,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BeaconExchangeApi {
    pub beacon_id: String,
    pub display_name: String,
    pub beacon_kind: String,
    pub bearer_token: String,
    pub session_id: String,
    pub client_kind: String,
    pub expires_at: String,
    // Server-canonical preferences echoed by exchange. The session surface
    // re-reads them from `beacon/me`, so keep the wire contract without
    // pretending the field is consumed here.
    #[allow(dead_code)]
    pub preferences: BeaconPreferences,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BeaconNearbyEvent {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub venue: Option<String>,
    pub city: String,
    pub starts_at: String,
    pub ticket_url: Option<String>,
    pub distance_km: i32,
    pub engagement_status: Option<String>,
    pub help_kind: Option<String>,
    pub last_notified_at: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BeaconPressLinks {
    pub home_url: String,
    pub epk_url: String,
    pub gallery_url: String,
    pub rider_url: String,
    pub spotify_url: String,
    pub youtube_url: String,
    pub contact_email: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BeaconHomeData {
    pub beacon_id: String,
    pub display_name: String,
    pub beacon_kind: String,
    pub city: Option<String>,
    pub preferences: BeaconPreferences,
    #[serde(default)]
    pub nearby_events: Vec<BeaconNearbyEvent>,
    pub press_room: BeaconPressLinks,
    pub open_press_requests: i64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BeaconPressRoomEvent {
    pub id: String,
    pub slug: String,
    pub title: String,
    pub venue: Option<String>,
    pub city: Option<String>,
    pub starts_at: String,
    pub doors_at: Option<String>,
    pub ticket_url: Option<String>,
    pub description: Option<String>,
    pub image_url: Option<String>,
    pub listen_url: Option<String>,
    pub trailer_url: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BeaconPressAsset {
    pub id: String,
    pub event_id: Option<String>,
    pub asset_key: String,
    pub asset_kind: String,
    pub label_pl: String,
    pub label_en: String,
    pub url: String,
    pub sort_order: i32,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BeaconPressRoomData {
    pub event_id: Option<String>,
    pub event: Option<BeaconPressRoomEvent>,
    #[serde(default)]
    pub assets: Vec<BeaconPressAsset>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BeaconPressRequestItem {
    pub id: String,
    pub event_id: Option<String>,
    pub event_title: Option<String>,
    pub request_kind: String,
    pub details: Option<String>,
    pub status: String,
    pub resolution_note: Option<String>,
    pub created_at: String,
    pub resolved_at: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BeaconPressRequestsData {
    #[serde(default)]
    pub requests: Vec<BeaconPressRequestItem>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BeaconReleaseCampaign {
    pub campaign_id: String,
    pub slug: String,
    pub title: String,
    pub product_name: String,
    pub variant_label: String,
    pub status: String,
    pub recipient_status: String,
    pub claim_deadline: String,
    pub recipient_name: Option<String>,
    pub recipient_phone: Option<String>,
    pub parcel_locker_code: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BeaconReleasesData {
    #[serde(default)]
    pub campaigns: Vec<BeaconReleaseCampaign>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BeaconEngagementResult {
    pub event_id: String,
    pub status: String,
    pub help_kind: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BeaconMutationResult {
    pub status: Option<String>,
    pub message: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignalNewsFeed {
    pub version: u8,
    #[serde(default)]
    pub items: Vec<SignalNewsItem>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SignalNewsItem {
    pub slug: String,
    pub published_at: String,
    pub tag: SignalLocalizedText,
    pub title: SignalLocalizedText,
    pub summary: SignalLocalizedText,
    pub image_url: String,
    pub url: SignalLocalizedText,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct SignalLocalizedText {
    pub pl: String,
    pub en: String,
}
