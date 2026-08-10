use serde::Deserialize;

use super::OperatorRole;

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StaffPairingPayload {
    pub version: u8,
    pub api_base_url: String,
    pub display_name: String,
    pub role: OperatorRole,
    pub pairing_code: String,
    pub expires_at: u64,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct StaffPairingExchange {
    pub version: u8,
    pub display_name: String,
    pub role: OperatorRole,
    pub bearer_token: String,
    pub session_id: String,
    pub expires_at: u64,
}
