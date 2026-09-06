#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used)]

mod api;
mod commands;
mod crash;
mod device_unlock;
mod error;
mod feedback_queue;
mod i18n;
mod models;
mod push_plugin;
mod session;
mod util;
mod validation;
mod vault;

pub use error::AppError;

use std::{
    collections::HashMap,
    path::PathBuf,
    sync::{Arc, atomic::AtomicU64},
};

use api::CrowdRelayClient;
use commands::{
    beacon::{
        beacon_clear_pending_confirmation, beacon_clear_pending_invite, beacon_confirm_scanned,
        beacon_coverage, beacon_engagement, beacon_exchange_invite, beacon_exchange_pending,
        beacon_home, beacon_leave, beacon_lock, beacon_logout, beacon_news,
        beacon_preferences_update, beacon_prepare_invite, beacon_press_request_create,
        beacon_press_requests, beacon_press_room, beacon_push_disable, beacon_push_enable,
        beacon_push_open_settings, beacon_push_sync, beacon_release_confirm,
        beacon_release_decline, beacon_releases, beacon_status, beacon_take_app_link,
        beacon_unlock,
    },
    fan::{
        fan_admission_pass, fan_admission_qr, fan_area_challenge, fan_area_claim, fan_area_wallet,
        fan_cached_events, fan_cached_home, fan_cached_merch_catalog, fan_cached_sections,
        fan_cached_wallets, fan_claim_pass, fan_clear_pending_confirm_link,
        fan_clear_pending_confirmation, fan_confirm, fan_confirm_link, fan_confirm_scanned,
        fan_delete_account, fan_device_unlock, fan_disable_device_unlock, fan_enable_device_unlock,
        fan_events, fan_forget, fan_home, fan_import_wallet, fan_interests, fan_lock,
        fan_merch_bundles, fan_merch_catalog, fan_prepare_confirmation, fan_push_disable,
        fan_push_enable, fan_push_open_settings, fan_push_preferences, fan_push_status,
        fan_push_sync, fan_push_take_target, fan_push_update_preferences, fan_referral,
        fan_register_interest, fan_request_access, fan_request_delivery, fan_set_location,
        fan_signup, fan_start_ticket_checkout, fan_status, fan_take_confirm_link, fan_ticket_sale,
        fan_unlock, fan_unpublish_synesthesia_leaderboard, fan_wallets, render_wallet_qr,
    },
    misc::{
        launcher_status, open_external_url, request_city, submit_anonymous_feedback,
        verify_staff_access,
    },
    operator::{
        configure, create_qr_campaign, forget_device, issue_pass, lock, operator_cached_sections,
        operator_events, operator_push_disable, operator_push_enable, operator_push_open_settings,
        operator_push_sync, operator_qr, operator_show_checklist, operator_signal_overview,
        operator_update_show_checklist, public_cities, public_events, redeem_admission,
        redeem_coupon, revoke_pass, revoke_qr_campaign, session_status, staff_event_dashboard,
        ticketing_overview, unlock,
    },
    pairing::configure_from_pairing,
    show_mode::{
        show_mode_clear, show_mode_close, show_mode_prepare, show_mode_scan, show_mode_status,
        show_mode_sync,
    },
    synesthesia::{fan_link_pending_synesthesia, fan_take_synesthesia_app_link},
};
use models::{BeaconProfile, FanProfile, OperatorProfile, ShowModeStore};
use tauri::Manager;
use tokio::sync::{Mutex, RwLock};
use zeroize::Zeroizing;

struct PendingFanConfirmation {
    api_base_url: String,
    /// `None` when the fan chose to let this device hold the vault password.
    /// The scanned-QR exchange reads this to know which credential the
    /// confirmation was started with.
    pin: Option<Zeroizing<String>>,
}

struct PendingBeaconConfirmation {
    api_base_url: String,
    pin: Zeroizing<String>,
    radius_km: i32,
    locale: String,
    topics: Vec<String>,
}

/// Native session/application state, shared across every command via
/// Tauri's `State` extractor. Field access helpers live in `session.rs`;
/// commands themselves live under `commands/`.
pub struct AppState {
    session: RwLock<Option<Arc<OperatorProfile>>>,
    operator_pin: RwLock<Option<Zeroizing<String>>>,
    operator_vault_password: RwLock<Option<Zeroizing<Vec<u8>>>>,
    operator_mutation: Mutex<()>,
    operator_push_mutation: Mutex<()>,
    /// Monotonic counter incremented by `lock`/`forget_device`. Session-
    /// establishing commands (`unlock`/`configure`) capture it before their
    /// blocking work and re-verify before publishing, so a lock that fires
    /// during Argon2 cannot be silently undone by the completing unlock.
    operator_session_epoch: AtomicU64,
    /// In-memory mirror of the encrypted operator-panel snapshot, for the same
    /// reason as `fan_sections_cache`: six panels refresh independently and
    /// none of them should have to decrypt the other five back first.
    operator_sections_cache: RwLock<Option<vault::OperatorSectionsCacheSnapshot>>,
    operator_sections_cache_mutation: Mutex<()>,
    show_mode_mutation: Mutex<()>,
    show_mode_store: RwLock<Option<ShowModeStore>>,
    fan_session: RwLock<Option<Arc<FanProfile>>>,
    fan_pin: RwLock<Option<Zeroizing<String>>>,
    fan_vault_password: RwLock<Option<Zeroizing<Vec<u8>>>>,
    /// The effective unlock mode, resolved once and then held.
    ///
    /// `fan_status` runs at the end of nearly every fan command and
    /// `launcher_status` is polled on resume, so reading the record from disk
    /// there put a file read, a JSON parse and a `stat` on a path that is
    /// otherwise pure memory. The value only changes when this process writes
    /// it, which is where the cache is dropped.
    fan_unlock_mode: RwLock<Option<device_unlock::UnlockMode>>,
    pending_fan_confirmation: Mutex<Option<PendingFanConfirmation>>,
    pending_fan_confirm_token: Mutex<Option<Zeroizing<String>>>,
    pending_synesthesia_handoff: Mutex<Option<Zeroizing<String>>>,
    fan_mutation: Mutex<()>,
    /// Same role as `operator_session_epoch` for the fan identity. `fan_lock`
    /// already takes `fan_mutation`, but the epoch makes the guard explicit
    /// and covers the background push reconciliation path.
    fan_session_epoch: AtomicU64,
    /// In-memory mirror of the encrypted dashboard-fragment snapshot. Each
    /// section refreshes on its own schedule, so the mirror lets a single
    /// section update the record without decrypting the vault to read the
    /// other three back first.
    fan_sections_cache: RwLock<Option<vault::FanSectionsCacheSnapshot>>,
    fan_sections_cache_mutation: Mutex<()>,
    beacon_session: RwLock<Option<Arc<BeaconProfile>>>,
    beacon_pin: RwLock<Option<Zeroizing<String>>>,
    beacon_vault_password: RwLock<Option<Zeroizing<Vec<u8>>>>,
    beacon_mutation: Mutex<()>,
    /// Same role as `operator_session_epoch` for the beacon identity.
    /// `beacon_lock` deliberately skips `beacon_mutation`, so this is the
    /// primary guard against an in-flight `beacon_unlock` republishing the
    /// session after lock cleared it.
    beacon_session_epoch: AtomicU64,
    pending_beacon_confirmation: Mutex<Option<PendingBeaconConfirmation>>,
    pending_beacon_link: Mutex<Option<Zeroizing<String>>>,
    native_push_available: bool,
    /// Whether this device can seal the fan vault password with a hardware
    /// key. Probed once at setup: the answer cannot change while the process
    /// runs, and probing it per status read would put a keystore call on the
    /// path of every launcher poll.
    device_unlock_supported: bool,
    feedback_queue_mutation: Mutex<()>,
    wallet_qr_tokens: RwLock<HashMap<String, HashMap<String, Zeroizing<String>>>>,
    api: CrowdRelayClient,
    app_data_dir: PathBuf,
}

/// Shared bounds referenced by more than one command module. Kept here at
/// the crate root instead of in `validation.rs` because they also gate
/// non-input-validation code paths (QR rendering, wallet storage).
pub(crate) const MAX_SECRET_BYTES: usize = 4096;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    std::panic::set_hook(Box::new(|panic_info| {
        let report = format!("native panic: {panic_info}");
        eprintln!("[virya:native-panic] {report}");
        crash::write_native_crash_report(&report);
    }));
    let runtime_result = tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            // Establish the crash-report destination before any optional mobile
            // plugin is initialized. A plugin initialization failure used to
            // abort setup before the reporter knew where to persist evidence.
            let app_data_dir = app.path().app_local_data_dir()?;
            let crash_report_path = app_data_dir.join(crash::NATIVE_CRASH_REPORT_FILE);
            let _ = crash::NATIVE_CRASH_REPORT_PATH.set(crash_report_path);

            #[cfg(mobile)]
            let native_push_available = {
                let mut plugin_errors = Vec::new();
                if let Err(error) = app.handle().plugin(tauri_plugin_barcode_scanner::init()) {
                    plugin_errors.push(format!("barcode-scanner: {error}"));
                }
                if let Err(error) = app.handle().plugin(tauri_plugin_geolocation::init()) {
                    plugin_errors.push(format!("geolocation: {error}"));
                }
                #[cfg(target_os = "android")]
                let push_available = match app.handle().plugin(push_plugin::init()) {
                    Ok(()) => true,
                    Err(error) => {
                        plugin_errors.push(format!("signal-push: {error}"));
                        false
                    }
                };
                #[cfg(not(target_os = "android"))]
                let push_available = false;
                if !plugin_errors.is_empty() {
                    let report = format!(
                        "mobile plugin initialization degraded: {}",
                        plugin_errors.join("; ")
                    );
                    eprintln!("[virya:mobile-plugin] {report}");
                    crash::write_native_crash_report(&report);
                }
                push_available
            };

            #[cfg(not(mobile))]
            let native_push_available = false;

            // Generating the key is the only honest test of support: a device
            // can advertise a keystore and still refuse AES-GCM key
            // generation, and finding that out at unlock is finding it out too
            // late.
            #[cfg(target_os = "android")]
            let device_unlock_supported =
                native_push_available && push_plugin::device_secret_supported(app.handle());
            #[cfg(not(target_os = "android"))]
            let device_unlock_supported = false;
            let api = CrowdRelayClient::new(app_data_dir.join("public-cache-v1.json"))?;
            app.manage(AppState {
                session: RwLock::new(None),
                operator_pin: RwLock::new(None),
                operator_vault_password: RwLock::new(None),
                operator_mutation: Mutex::new(()),
                operator_push_mutation: Mutex::new(()),
                operator_session_epoch: AtomicU64::new(0),
                operator_sections_cache: RwLock::new(None),
                operator_sections_cache_mutation: Mutex::new(()),
                show_mode_mutation: Mutex::new(()),
                show_mode_store: RwLock::new(None),
                fan_session: RwLock::new(None),
                fan_pin: RwLock::new(None),
                fan_vault_password: RwLock::new(None),
                fan_unlock_mode: RwLock::new(None),
                pending_fan_confirmation: Mutex::new(None),
                pending_fan_confirm_token: Mutex::new(None),
                pending_synesthesia_handoff: Mutex::new(None),
                fan_sections_cache: RwLock::new(None),
                fan_sections_cache_mutation: Mutex::new(()),
                fan_mutation: Mutex::new(()),
                fan_session_epoch: AtomicU64::new(0),
                beacon_session: RwLock::new(None),
                beacon_pin: RwLock::new(None),
                beacon_vault_password: RwLock::new(None),
                beacon_mutation: Mutex::new(()),
                beacon_session_epoch: AtomicU64::new(0),
                pending_beacon_confirmation: Mutex::new(None),
                pending_beacon_link: Mutex::new(None),
                native_push_available,
                device_unlock_supported,
                feedback_queue_mutation: Mutex::new(()),
                wallet_qr_tokens: RwLock::new(HashMap::new()),
                api,
                app_data_dir,
            });
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            open_external_url,
            crash::native_crash_report,
            crash::acknowledge_native_crash,
            session_status,
            launcher_status,
            verify_staff_access,
            configure,
            unlock,
            lock,
            forget_device,
            operator_events,
            operator_qr,
            operator_show_checklist,
            operator_update_show_checklist,
            operator_push_sync,
            operator_push_enable,
            operator_push_open_settings,
            operator_push_disable,
            operator_signal_overview,
            show_mode_prepare,
            show_mode_status,
            show_mode_scan,
            show_mode_sync,
            show_mode_clear,
            show_mode_close,
            staff_event_dashboard,
            ticketing_overview,
            redeem_admission,
            redeem_coupon,
            issue_pass,
            revoke_pass,
            create_qr_campaign,
            revoke_qr_campaign,
            public_events,
            public_cities,
            request_city,
            configure_from_pairing,
            fan_status,
            fan_take_synesthesia_app_link,
            fan_take_confirm_link,
            fan_confirm_link,
            fan_device_unlock,
            fan_enable_device_unlock,
            fan_disable_device_unlock,
            fan_link_pending_synesthesia,
            fan_unlock,
            fan_lock,
            fan_forget,
            fan_delete_account,
            fan_unpublish_synesthesia_leaderboard,
            fan_signup,
            fan_request_access,
            fan_prepare_confirmation,
            fan_clear_pending_confirmation,
            fan_clear_pending_confirm_link,
            fan_confirm,
            fan_confirm_scanned,
            fan_home,
            fan_cached_home,
            fan_events,
            fan_merch_catalog,
            fan_merch_bundles,
            fan_ticket_sale,
            fan_start_ticket_checkout,
            fan_area_wallet,
            fan_area_challenge,
            fan_area_claim,
            fan_referral,
            fan_interests,
            fan_admission_pass,
            fan_register_interest,
            fan_claim_pass,
            fan_admission_qr,
            fan_import_wallet,
            fan_wallets,
            render_wallet_qr,
            fan_request_delivery,
            fan_push_sync,
            fan_push_status,
            fan_push_enable,
            fan_push_disable,
            fan_push_open_settings,
            fan_cached_events,
            fan_cached_merch_catalog,
            fan_cached_sections,
            fan_cached_wallets,
            operator_cached_sections,
            fan_push_preferences,
            fan_push_update_preferences,
            fan_set_location,
            fan_push_take_target,
            beacon_status,
            beacon_take_app_link,
            beacon_prepare_invite,
            beacon_clear_pending_confirmation,
            beacon_clear_pending_invite,
            beacon_confirm_scanned,
            beacon_exchange_pending,
            beacon_news,
            beacon_unlock,
            beacon_lock,
            beacon_exchange_invite,
            beacon_home,
            beacon_preferences_update,
            beacon_press_room,
            beacon_press_requests,
            beacon_press_request_create,
            beacon_engagement,
            beacon_coverage,
            beacon_releases,
            beacon_release_confirm,
            beacon_release_decline,
            beacon_logout,
            beacon_leave,
            beacon_push_sync,
            beacon_push_enable,
            beacon_push_disable,
            beacon_push_open_settings,
            submit_anonymous_feedback,
        ])
        .run(tauri::generate_context!());
    if let Err(error) = runtime_result {
        let report = format!("tauri runtime terminated: {error}");
        eprintln!("[virya:runtime] {report}");
        crash::write_native_crash_report(&report);
    }
}
