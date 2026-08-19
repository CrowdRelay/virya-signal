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

/// Fan-controlled notification categories. Missing server state maps to all
/// categories enabled and quiet hours disabled for backward compatibility.
#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FanPushPreferences {
    pub shows: bool,
    pub releases: bool,
    pub community: bool,
    pub merch: bool,
    pub quiet_hours_enabled: bool,
    pub quiet_start: String,
    pub quiet_end: String,
    pub quiet_timezone: String,
}

impl Default for FanPushPreferences {
    fn default() -> Self {
        Self {
            shows: true,
            releases: true,
            community: true,
            merch: true,
            quiet_hours_enabled: false,
            quiet_start: "22:00".to_owned(),
            quiet_end: "08:00".to_owned(),
            quiet_timezone: "Europe/Warsaw".to_owned(),
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct FanPushPreferencesUpdate {
    pub shows: bool,
    pub releases: bool,
    pub community: bool,
    pub merch: bool,
    pub quiet_hours_enabled: bool,
    pub quiet_start: String,
    pub quiet_end: String,
}
