//! Fan-facing session lifecycle: signup/confirmation, event interests,
//! referral progress, admission passes and the ticket wallet (including
//! locally-rendered QR codes, whose raw tokens never leave the native
//! process once fetched).

use std::{
    collections::HashMap,
    path::Path,
    sync::{Arc, atomic::Ordering},
};

use futures_util::{StreamExt, stream};
use qrcode::{QrCode, render::svg};
use tauri::{AppHandle, Manager, State};
use tokio::sync::MutexGuard;
use zeroize::Zeroizing;

use crate::{
    AppError, AppState, MAX_SECRET_BYTES, PendingFanConfirmation,
    api::{
        FanPushConfigApi, SignalMerchBundleCatalog, TicketCheckoutInput, TicketCheckoutStart,
        TicketSaleOffer,
    },
    models::{
        AdmissionPass, AreaChallenge, AreaClaimResult, AreaPositionSample, FanAuthResult,
        FanConfirmationInput, FanEventInterest, FanHomeData, FanLocationState, FanProfile,
        FanPushPreferences, FanPushPreferencesUpdate, FanPushStatus, FanSessionPhase,
        FanSessionStatus, FanSignupInput, MerchCatalogResult, PublicEventsResult, ReferralProgress,
        TicketWallet, TicketWalletApi, WalletBatch, WalletCredential, WalletQrCredential,
        WalletTicket,
    },
    session::{fan_profile, persist_fan, run_blocking},
    validation::{
        bounded_secret, validate_api_base, validate_fan_confirmation,
        validate_fan_confirmation_token_only, validate_fan_signup, validate_pin,
    },
    vault,
};

const MAX_WALLETS: usize = 24;
const WALLET_FETCH_CONCURRENCY: usize = 8;
const NATIVE_PUSH_INSTALLATION_FILE: &str = "push-installation-id-v1";

include!("fan/push.rs");
include!("fan/session_commerce.rs");
include!("fan/sections.rs");
include!("fan/device_unlock.rs");
include!("fan/signup.rs");
include!("fan/wallet.rs");
include!("fan/tests.rs");
