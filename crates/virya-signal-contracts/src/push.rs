use serde::{Deserialize, Serialize};

/// Normalized native push state exposed over Tauri IPC.
#[derive(Clone, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FanPushStatus {
    pub supported: bool,
    pub backend_enabled: bool,
    pub enabled: bool,
    pub permission: String,
    pub transport: Option<String>,
    pub detail: Option<String>,
}
