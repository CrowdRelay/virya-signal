#[derive(Clone, Debug, Default, Deserialize)]
pub struct BeaconSummary {
    pub display_name: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct BeaconSessionStatus {
    pub configured: bool,
    pub unlocked: bool,
    pub session: Option<BeaconSummary>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BeaconPreferences {
    pub radius_km: i32,
    #[serde(default)]
    pub topics: Vec<String>,
    pub nearby_gigs_enabled: bool,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BeaconNearbyEvent {
    pub id: String,
    pub title: String,
    pub venue: Option<String>,
    pub city: String,
    pub starts_at: String,
    pub distance_km: i32,
    pub engagement_status: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BeaconHomeData {
    pub preferences: BeaconPreferences,
    #[serde(default)]
    pub nearby_events: Vec<BeaconNearbyEvent>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BeaconPressRoomEvent {
    pub title: String,
    pub city: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BeaconPressAsset {
    pub asset_kind: String,
    pub label_pl: String,
    pub label_en: String,
    pub url: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BeaconPressRoomData {
    pub event: Option<BeaconPressRoomEvent>,
    #[serde(default)]
    pub assets: Vec<BeaconPressAsset>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BeaconPressRequestItem {
    pub event_title: Option<String>,
    pub request_kind: String,
    pub details: Option<String>,
    pub status: String,
    pub resolution_note: Option<String>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BeaconPressRequestsData {
    #[serde(default)]
    pub requests: Vec<BeaconPressRequestItem>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BeaconReleaseCampaign {
    pub campaign_id: String,
    pub title: String,
    pub product_name: String,
    pub variant_label: String,
    pub recipient_status: String,
    pub claim_deadline: String,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BeaconReleasesData {
    #[serde(default)]
    pub campaigns: Vec<BeaconReleaseCampaign>,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct BeaconEngagementResult {}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct BeaconMutationResult {}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignalNewsFeed {
    #[serde(default)]
    pub items: Vec<SignalNewsItem>,
}

#[derive(Clone, Debug, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SignalNewsItem {
    pub tag: SignalLocalizedText,
    pub title: SignalLocalizedText,
    pub summary: SignalLocalizedText,
    pub url: SignalLocalizedText,
}

#[derive(Clone, Debug, Default, Deserialize)]
pub struct SignalLocalizedText {
    pub pl: String,
    pub en: String,
}
