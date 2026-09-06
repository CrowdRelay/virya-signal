/// Explicit session lifecycle phase per domain.
///
/// The phase is *derived*, never stored. It used to live in its own
/// `RwLock<…Phase>` beside the session, and that second copy could disagree
/// with the first in two ways. On a cold start the lock initialised to
/// `Unconfigured` while `configured` was read from the vault on disk, so a
/// device with a real locked vault reported `configured: true, phase:
/// Unconfigured`. And because the phase and the credentials were two locks, a
/// status read could land between the two writes of an unlock and see one
/// without the other.
///
/// Both disappear if the phase is a function of the two facts the status
/// already reports rather than a third fact kept in step by hand. `resolve`
/// is that function, and `sessions_report_a_phase_derived_from_their_own_facts`
/// in `models/tests.rs` is its table.
///
/// Future variants (Unlocking, Refreshing, Expired, LoggingOut) are additional
/// *facts*, not additional copies: each would arrive with its own input to
/// `resolve` rather than as another lock to keep synchronised.
macro_rules! session_phase {
    ($name:ident) => {
        #[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Deserialize, Serialize)]
        #[serde(rename_all = "camelCase")]
        pub enum $name {
            #[default]
            Unconfigured,
            Locked,
            Active,
        }

        impl $name {
            /// `unlocked` implies credentials are in memory, which is only
            /// reachable from a persisted identity, so it decides on its own.
            /// Otherwise the vault on disk is what separates "never set up"
            /// from "set up and closed".
            #[must_use]
            pub const fn resolve(configured: bool, unlocked: bool) -> Self {
                match (configured, unlocked) {
                    (_, true) => Self::Active,
                    (true, false) => Self::Locked,
                    (false, false) => Self::Unconfigured,
                }
            }
        }
    };
}

session_phase!(OperatorSessionPhase);
session_phase!(FanSessionPhase);
session_phase!(BeaconSessionPhase);

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
    #[serde(default)]
    pub phase: OperatorSessionPhase,
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
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub redeemed_at: Option<String>,
    pub qr_token: Option<String>,
    pub qr_expires_at: String,
}

#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
pub struct WalletTicket {
    pub ticket_type_name: String,
    pub public_reference: String,
    pub holder_name: Option<String>,
    pub holder_email_masked: String,
    #[serde(default)]
    pub status: String,
    #[serde(default)]
    pub redeemed_at: Option<String>,
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
    /// How this device can be opened. A vault sealed without a PIN cannot be
    /// opened by one, and a PIN tried against it fails exactly like a wrong PIN
    /// — so the screen has to know which prompt is the true one rather than
    /// guessing from a failure.
    #[serde(default)]
    pub pin_unlock: bool,
    #[serde(default)]
    pub device_unlock: bool,
    /// Whether this device is capable of sealing a password at all. False on a
    /// build or a device with no usable keystore, where the PIN is the only
    /// offer worth making. True is a capability rather than a guarantee: the
    /// probe stops short of generating the real key, so an offer made on the
    /// strength of this can still fail at the first seal — which is why that
    /// failure removes the vault it was for instead of stranding it.
    #[serde(default)]
    pub device_unlock_supported: bool,
    #[serde(default)]
    pub phase: FanSessionPhase,
}

#[derive(Clone, Debug, Default, Serialize)]
pub struct LauncherStatus {
    pub operator: SessionStatus,
    pub fan: FanSessionStatus,
    pub beacon: BeaconSessionStatus,
}

// Shared verbatim with the WASM UI; see virya-signal-contracts::fan.
pub use virya_signal_contracts::fan::{FanHomeData, FanLocationState};
pub use virya_signal_contracts::push::{
    FanPushPreferences, FanPushPreferencesUpdate, FanPushStatus,
};
