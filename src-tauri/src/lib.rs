#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used)]

mod api;
mod commands;
mod crash;
mod error;
mod i18n;
mod models;
mod session;
mod util;
mod validation;
mod vault;

pub use error::AppError;

use std::{collections::HashMap, path::PathBuf, sync::Arc};

use api::CrowdRelayClient;
use commands::{
    fan::{
        fan_admission_pass, fan_admission_qr, fan_area_challenge, fan_area_claim, fan_area_wallet,
        fan_claim_pass, fan_confirm, fan_events, fan_forget, fan_import_wallet, fan_interests,
        fan_lock, fan_merch_bundles, fan_merch_catalog, fan_referral, fan_register_interest,
        fan_request_access, fan_request_delivery, fan_signup, fan_start_ticket_checkout,
        fan_status, fan_ticket_sale, fan_unlock, fan_wallets, render_wallet_qr,
    },
    misc::{
        launcher_status, open_external_url, request_city, submit_anonymous_feedback,
        verify_staff_access,
    },
    operator::{
        configure, create_qr_campaign, forget_device, issue_pass, lock, operator_events,
        operator_ops_overview, operator_qr, operator_retry, operator_signal_overview,
        public_cities, public_events, redeem_admission, redeem_coupon, revoke_pass,
        revoke_qr_campaign, session_status, ticketing_overview, unlock,
    },
    pairing::configure_from_pairing,
    show_mode::{
        show_mode_clear, show_mode_prepare, show_mode_scan, show_mode_status, show_mode_sync,
    },
};
use models::{FanProfile, OperatorProfile, ShowModeStore};
use tauri::Manager;
use tokio::sync::{Mutex, RwLock};
use zeroize::Zeroizing;

/// Native session/application state, shared across every command via
/// Tauri's `State` extractor. Field access helpers live in `session.rs`;
/// commands themselves live under `commands/`.
pub struct AppState {
    session: RwLock<Option<Arc<OperatorProfile>>>,
    operator_pin: RwLock<Option<Zeroizing<String>>>,
    operator_vault_password: RwLock<Option<Zeroizing<Vec<u8>>>>,
    operator_mutation: Mutex<()>,
    show_mode_mutation: Mutex<()>,
    show_mode_store: RwLock<Option<ShowModeStore>>,
    fan_session: RwLock<Option<Arc<FanProfile>>>,
    fan_pin: RwLock<Option<Zeroizing<String>>>,
    fan_mutation: Mutex<()>,
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
            {
                let mut plugin_errors = Vec::new();
                if let Err(error) = app.handle().plugin(tauri_plugin_barcode_scanner::init()) {
                    plugin_errors.push(format!("barcode-scanner: {error}"));
                }
                if let Err(error) = app.handle().plugin(tauri_plugin_geolocation::init()) {
                    plugin_errors.push(format!("geolocation: {error}"));
                }
                if !plugin_errors.is_empty() {
                    let report = format!(
                        "mobile plugin initialization degraded: {}",
                        plugin_errors.join("; ")
                    );
                    eprintln!("[virya:mobile-plugin] {report}");
                    crash::write_native_crash_report(&report);
                }
            }

            let api = CrowdRelayClient::new(app_data_dir.join("public-cache-v1.json"))?;
            app.manage(AppState {
                session: RwLock::new(None),
                operator_pin: RwLock::new(None),
                operator_vault_password: RwLock::new(None),
                operator_mutation: Mutex::new(()),
                show_mode_mutation: Mutex::new(()),
                show_mode_store: RwLock::new(None),
                fan_session: RwLock::new(None),
                fan_pin: RwLock::new(None),
                fan_mutation: Mutex::new(()),
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
            operator_signal_overview,
            operator_ops_overview,
            operator_retry,
            show_mode_prepare,
            show_mode_status,
            show_mode_scan,
            show_mode_sync,
            show_mode_clear,
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
            fan_unlock,
            fan_lock,
            fan_forget,
            fan_signup,
            fan_request_access,
            fan_confirm,
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
            submit_anonymous_feedback,
        ])
        .run(tauri::generate_context!());
    if let Err(error) = runtime_result {
        let report = format!("tauri runtime terminated: {error}");
        eprintln!("[virya:runtime] {report}");
        crash::write_native_crash_report(&report);
    }
}
