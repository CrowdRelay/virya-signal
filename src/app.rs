use crate::i18n::{self, Language, tr};
use crate::util::{OptionValueOrElseExt, OptionValueOrExt};

mod area;
mod formatters;
mod types;

use area::AreaGameScreen;
use formatters::{
    day, event_location, event_time_location, human_time, local_to_rfc3339, money, month, optional,
};
use leptos::prelude::*;
use types::*;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};
use wasm_bindgen_futures::spawn_local;

use crate::{
    bridge,
    models::{
        AdmissionPass, AdmissionQr, AdmissionRedemption, AreaWallet, AutopilotActionPayload,
        AutopilotChiefOfStaff, AutopilotMutation, AutopilotPolicySummary, CouponEnvelope,
        CreateQrCampaignInput, DashboardData, FanAuthResult, FanConfirmationInput,
        FanDashboardData, FanEventInterest, FanHomeData, FanMerchBundleCatalog, FanPushStatus,
        FanSessionStatus, FanSignupInput, IssuePassInput, IssuedPass, MerchCatalog,
        OperatorAutopilotOverview, OperatorOpsOverview, OperatorProfileInput, OperatorRole,
        OperatorSignalOverview, OpsDeliveryItem, OpsOutboxItem, OpsRetryResult, PublicEvent,
        PublicHomeData, QrCampaign, ReferralProgress, RequestedCityInput, RequestedCityResult,
        SessionStatus, ShowModeScanResult, ShowModeStatus, ShowModeSyncResult, StaffEventDashboard,
        TicketCheckoutInput, TicketCheckoutItemInput, TicketCheckoutStart, TicketSaleOffer,
        TicketWallet, TicketingOverview, WalletBatch, WalletTicket,
    },
};

fn install_resume_refresh(status_refresh: RwSignal<u32>) {
    let global: JsValue = js_sys::global().into();
    let Ok(listener) = js_sys::Reflect::get(&global, &JsValue::from_str("addEventListener")) else {
        return;
    };
    let Ok(listener) = listener.dyn_into::<js_sys::Function>() else {
        return;
    };
    let callback = Closure::<dyn FnMut(JsValue)>::new(move |_| {
        status_refresh.update(|value| *value = value.wrapping_add(1));
    });
    if listener
        .call2(
            &global,
            &JsValue::from_str("virya:resume"),
            callback.as_ref().unchecked_ref(),
        )
        .is_err()
    {
        return;
    }

    // Note: We intentionally leak the closure here because wasm-bindgen closures
    // are not Send + Sync, so they can't be stored in Leptos signals or cleaned up
    // with on_cleanup. This is a known limitation and acceptable for this use case.
    std::mem::forget(callback);
}

#[component]
pub fn App() -> impl IntoView {
    let mode = RwSignal::new(RootMode::Fan);
    let operator_status = RwSignal::new(SessionStatus::default());
    let fan_status = RwSignal::new(FanSessionStatus::default());
    let operator_status_loading = RwSignal::new(true);
    let fan_status_loading = RwSignal::new(true);
    let operator_status_failed = RwSignal::new(false);
    let fan_status_failed = RwSignal::new(false);
    let status_refresh = RwSignal::new(0_u32);
    let push_target = RwSignal::new(None::<String>);
    let error = RwSignal::new(None::<String>);
    install_resume_refresh(status_refresh);

    Effect::new(move |_| {
        status_refresh.get();
        if !bridge::native_available() {
            return;
        }
        spawn_local(async move {
            match bridge::invoke::<Option<String>, _>("fan_push_take_target", &EmptyArgs {}).await {
                Ok(Some(target)) => {
                    push_target.set(Some(target));
                    mode.set(RootMode::Fan);
                }
                Ok(None) => {}
                Err(message) => error.set(Some(message)),
            }
        });
    });

    Effect::new(move |_| {
        status_refresh.get();
        operator_status_loading.set(true);
        fan_status_loading.set(true);
        operator_status_failed.set(false);
        fan_status_failed.set(false);
        spawn_local(async move {
            let result = bridge::launcher_status().await;
            match result {
                Ok(Some(status)) => {
                    operator_status.set(status.operator);
                    operator_status_failed.set(false);
                    fan_status.set(status.fan);
                    fan_status_failed.set(false);
                }
                Ok(None) => {
                    operator_status_failed.set(true);
                    fan_status_failed.set(true);
                }
                Err(message) => {
                    operator_status_failed.set(true);
                    fan_status_failed.set(true);
                    error.set(Some(message));
                }
            }
            operator_status_loading.set(false);
            fan_status_loading.set(false);
        });
    });

    view! {
        <main class="app-shell">
            {move || match mode.get() {
                RootMode::Fan => view! {
                    <FanPortal
                        mode=mode
                        status=fan_status
                        status_loading=fan_status_loading
                        status_failed=fan_status_failed
                        status_refresh=status_refresh
                        push_target=push_target
                        error=error
                    />
                }.into_any(),
                RootMode::StaffGate => view! {
                    <StaffGate mode=mode error=error />
                }.into_any(),
                RootMode::Team => view! {
                    <OperatorPortal
                        mode=mode
                        status=operator_status
                        status_loading=operator_status_loading
                        status_failed=operator_status_failed
                        status_refresh=status_refresh
                        error=error
                    />
                }.into_any(),
            }}
            <Toast error=error />
        </main>
    }
}

// These sections intentionally compile into this module through `include!`.
// It keeps the existing component visibility and contracts unchanged while
// making the operator, fan and refresh code independently reviewable.
include!("app/operator.rs");
include!("app/scanner.rs");
include!("app/fan_home.rs");
include!("app/fan.rs");
include!("app/support.rs");
