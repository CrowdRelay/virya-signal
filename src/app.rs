use std::cell::RefCell;

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
use leptos::task::spawn_local_scoped_with_cancellation as spawn_local;
use types::*;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};

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
        SessionStatus, ShowChecklist, ShowModeScanResult, ShowModeStatus, ShowModeSyncResult,
        StaffEventDashboard, TicketCheckoutInput, TicketCheckoutItemInput, TicketCheckoutStart,
        TicketSaleOffer, TicketWallet, TicketingOverview, WalletBatch, WalletTicket,
    },
};

struct ResumeRefreshListener {
    global: JsValue,
    event: JsValue,
    remove_listener: js_sys::Function,
    callback: Closure<dyn FnMut(JsValue)>,
}

impl Drop for ResumeRefreshListener {
    fn drop(&mut self) {
        let _ = self.remove_listener.call2(
            &self.global,
            &self.event,
            self.callback.as_ref().unchecked_ref(),
        );
    }
}

thread_local! {
    static RESUME_REFRESH_LISTENER: RefCell<Option<ResumeRefreshListener>> = const { RefCell::new(None) };
}

fn install_resume_refresh(status_refresh: RwSignal<u32>) {
    let global: JsValue = js_sys::global().into();
    let event = JsValue::from_str("virya:resume");
    let Ok(add_listener) = js_sys::Reflect::get(&global, &JsValue::from_str("addEventListener"))
    else {
        return;
    };
    let Ok(add_listener) = add_listener.dyn_into::<js_sys::Function>() else {
        return;
    };
    let Ok(remove_listener) =
        js_sys::Reflect::get(&global, &JsValue::from_str("removeEventListener"))
    else {
        return;
    };
    let Ok(remove_listener) = remove_listener.dyn_into::<js_sys::Function>() else {
        return;
    };
    let callback = Closure::<dyn FnMut(JsValue)>::new(move |_| {
        let _ = status_refresh.try_update(|value| *value = value.wrapping_add(1));
    });
    if add_listener
        .call2(&global, &event, callback.as_ref().unchecked_ref())
        .is_err()
    {
        return;
    }

    RESUME_REFRESH_LISTENER.with(|slot| {
        *slot.borrow_mut() = Some(ResumeRefreshListener {
            global,
            event,
            remove_listener,
            callback,
        });
    });
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
    let operator_push_target = RwSignal::new(None::<String>);
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
                    if target.starts_with("/staff/") {
                        operator_push_target.set(Some(target));
                        mode.set(RootMode::StaffGate);
                    } else {
                        push_target.set(Some(target));
                        mode.set(RootMode::Fan);
                    }
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
            let completed = latest_request_completed(&result);
            match result {
                Ok(Some(status)) => {
                    operator_status.set(status.operator);
                    operator_status_failed.set(false);
                    fan_status.set(status.fan);
                    fan_status_failed.set(false);
                }
                Ok(None) => {}
                Err(message) => {
                    operator_status_failed.set(true);
                    fan_status_failed.set(true);
                    error.set(Some(message));
                }
            }
            if completed {
                operator_status_loading.set(false);
                fan_status_loading.set(false);
            }
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
                        push_target=operator_push_target
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
