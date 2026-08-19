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

/// Fan preference payloads are generated from CrowdRelay OpenAPI and
/// re-exported here so the existing normalized IPC module path stays stable.
pub use crate::fan::{FanPushPreferences, FanPushPreferencesUpdate};

#[cfg(test)]
mod tests {
    use super::FanPushPreferencesUpdate;
    use serde_json::json;

    #[test]
    fn generated_update_has_exact_canonical_fields() {
        let value = FanPushPreferencesUpdate {
            shows: true,
            releases: false,
            community: true,
            merch: false,
            quiet_hours_enabled: true,
            quiet_start: "22:30".to_owned(),
            quiet_end: "07:45".to_owned(),
        };
        let wire = serde_json::to_value(&value).expect("serialize push preferences");
        let object = wire.as_object().expect("object");
        assert_eq!(object.len(), 7);
        for key in [
            "shows",
            "releases",
            "community",
            "merch",
            "quietHoursEnabled",
            "quietStart",
            "quietEnd",
        ] {
            assert!(object.contains_key(key), "missing {key}");
        }
        assert!(!object.contains_key("quietTimezone"));
    }

    #[test]
    fn generated_update_rejects_timezone_authority_from_client() {
        let result = serde_json::from_value::<FanPushPreferencesUpdate>(json!({
            "shows": true,
            "releases": true,
            "community": true,
            "merch": true,
            "quietHoursEnabled": false,
            "quietStart": "22:00",
            "quietEnd": "08:00",
            "quietTimezone": "America/New_York"
        }));
        assert!(result.is_err());
    }
}
