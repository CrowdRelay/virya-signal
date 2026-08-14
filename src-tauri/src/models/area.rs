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

