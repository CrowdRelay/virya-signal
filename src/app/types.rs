use serde::Serialize;

use crate::models::{
    CreateQrCampaignInput, FanConfirmationInput, FanPushPreferencesUpdate, FanSignupInput,
    IssuePassInput, OperatorProfileInput, RequestedCityInput, TicketCheckoutInput,
};

const PRODUCTION_API_BASE: &str = "https://signal-api.virya.music/v1/";
#[cfg(debug_assertions)]
pub(super) const API_BASE: &str = match option_env!("VIRYA_SIGNAL_E2E_API_BASE") {
    Some(value) if value.is_empty() => PRODUCTION_API_BASE,
    Some(value) => value,
    None => PRODUCTION_API_BASE,
};
#[cfg(not(debug_assertions))]
pub(super) const API_BASE: &str = PRODUCTION_API_BASE;
pub(super) const POLICY_VERSION: &str = "2026-07";
/// Pre-provisioned demo fan confirmation token for Google Play reviewers.
/// Single-use, 30-day TTL. Bound to a pending demo fan in the Virya workspace.
pub(super) const DEMO_FAN_TOKEN: &str =
    "d2cc8fda6a69eb6aeedb42e6b0aedcdbed88c376b7f00de5df2e3f0affd45e40";
pub(super) const DEMO_FAN_PIN: &str = "2580";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RootMode {
    Fan,
    Latarnik,
    StaffGate,
    Team,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum BeaconTab {
    Briefing,
    Radar,
    Press,
    Access,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum OperatorTab {
    Home,
    Signal,
    Scan,
    Tickets,
    Discounts,
    Campaigns,
    Checklist,
    Settings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FanTab {
    Signal,
    Events,
    Merch,
    Game,
    Wallet,
    Profile,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum FanAccessMode {
    Signup,
    Confirm,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct FanLoadingState {
    pub home: bool,
    pub events: bool,
    pub referral: bool,
    pub interests: bool,
    pub merch: bool,
    pub admission_pass: bool,
    pub wallets: bool,
    pub area: bool,
}

#[derive(Clone, Copy, Debug, Default)]
pub(super) struct FanLoadedState {
    pub home: bool,
    pub referral: bool,
    pub events: bool,
    pub interests: bool,
    pub merch: bool,
    pub admission_pass: bool,
    pub wallets: bool,
    pub area: bool,
}

/// Which refresh generation each Latarnik section was last loaded at. Entering
/// a tab that is already current must not refetch it; an explicit refresh bumps
/// the generation and every section becomes claimable again.
#[derive(Clone, Copy, Debug, Default)]
pub(super) struct BeaconLoadedState {
    pub home: u32,
    pub news: u32,
    pub requests: u32,
    pub releases: u32,
}

#[derive(Clone, Copy, Debug)]
pub(super) struct OperatorLoadingState {
    pub events: bool,
    pub qr: bool,
}

impl OperatorLoadingState {
    pub const fn all() -> Self {
        Self {
            events: true,
            qr: true,
        }
    }
}

impl FanLoadingState {
    pub const fn all() -> Self {
        Self {
            home: true,
            events: true,
            referral: true,
            interests: true,
            merch: true,
            admission_pass: true,
            wallets: true,
            area: true,
        }
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct EmptyArgs {}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FanPushPreferencesArgs<'a> {
    pub preferences: &'a FanPushPreferencesUpdate,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PinArgs<'a> {
    pub pin: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FanConfirmLinkArgs<'a> {
    pub api_base_url: &'a str,
    pub pin: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct StaffGateArgs<'a> {
    pub password: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ConfigureArgs<'a> {
    pub pin: &'a str,
    pub profile: &'a OperatorProfileInput,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct EventArgs<'a> {
    pub event_slug: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ChecklistUpdateArgs<'a> {
    pub event_slug: &'a str,
    pub item_key: &'a str,
    pub status: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct TicketCheckoutArgs<'a> {
    pub input: &'a TicketCheckoutInput,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RedeemArgs<'a> {
    pub event_slug: &'a str,
    pub code: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CouponArgs<'a> {
    pub code: &'a str,
    pub order_reference: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct IssueArgs<'a> {
    pub input: &'a IssuePassInput,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ReferenceArgs<'a> {
    pub public_reference: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CampaignArgs<'a> {
    pub input: &'a CreateQrCampaignInput,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct CampaignIdArgs<'a> {
    pub campaign_id: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RetryArgs<'a> {
    pub target_kind: &'a str,
    pub target_id: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FanSignupArgs<'a> {
    pub input: &'a FanSignupInput,
    pub pin: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FanConfirmArgs<'a> {
    pub input: &'a FanConfirmationInput,
    pub pin: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FanPrepareConfirmationArgs<'a> {
    pub api_base_url: &'a str,
    pub pin: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct FanAccessArgs<'a> {
    pub api_base_url: &'a str,
    pub email: &'a str,
    pub locale: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ClaimArgs<'a> {
    pub claim_token: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct ImportWalletArgs<'a> {
    pub order_id: &'a str,
    pub checkout_token: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct OrderArgs<'a> {
    pub order_id: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct WalletQrArgs<'a> {
    pub order_id: &'a str,
    pub public_reference: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct PairingArgs<'a> {
    pub pin: &'a str,
    pub payload: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct RequestedCityArgs<'a> {
    pub input: &'a RequestedCityInput,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct UrlArgs<'a> {
    pub url: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AnonymousFeedbackArgs<'a> {
    pub category: &'a str,
    pub message: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AreaDropArgs<'a> {
    pub drop_id: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AreaClaimArgs<'a> {
    pub drop_id: &'a str,
    pub challenge: &'a str,
    pub samples: &'a [crate::models::AreaPositionSample],
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AutopilotAuthorityArgs<'a> {
    pub context: &'a str,
    pub enabled: bool,
    pub autonomy_level: &'a str,
    pub minimum_confidence_basis_points: u16,
    pub max_actions_24h: u32,
    pub expected_version: i64,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AutopilotActionArgs<'a> {
    pub action_id: &'a str,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct AutopilotAssignArgs<'a> {
    pub action_id: &'a str,
    pub member_key: &'a str,
}
