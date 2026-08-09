use serde::Serialize;

use crate::models::{
    CreateQrCampaignInput, FanConfirmationInput, FanSignupInput, IssuePassInput,
    OperatorProfileInput, RequestedCityInput, TicketCheckoutInput,
};

pub(super) const API_BASE: &str = "https://signal-api.virya.music/v1/";
pub(super) const POLICY_VERSION: &str = "2026-07";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum RootMode {
    Fan,
    StaffGate,
    Team,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum OperatorTab {
    Home,
    Signal,
    Scan,
    Tickets,
    Discounts,
    Campaigns,
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
pub(super) struct PinArgs<'a> {
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
