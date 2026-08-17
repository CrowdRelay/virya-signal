use std::cell::{Cell, RefCell};

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
        CreateQrCampaignInput, DashboardData, EventCity, FanAuthResult, FanConfirmationInput,
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
    subscribers: Vec<(u64, RwSignal<u32>)>,
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
    static NEXT_RESUME_REFRESH_ID: Cell<u64> = const { Cell::new(1) };
}

fn unregister_resume_refresh(subscriber_id: u64) {
    RESUME_REFRESH_LISTENER.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(listener) = slot.as_mut() else {
            return;
        };
        listener.subscribers.retain(|(id, _)| *id != subscriber_id);
        if listener.subscribers.is_empty() {
            // Dropping the final subscriber removes the one global browser
            // listener as well. No wasm-bindgen closure is leaked.
            slot.take();
        }
    });
}

fn install_resume_refresh(status_refresh: RwSignal<u32>) {
    let subscriber_id = NEXT_RESUME_REFRESH_ID.with(|next| {
        let id = next.get();
        next.set(id.wrapping_add(1).max(1));
        id
    });

    let installed = RESUME_REFRESH_LISTENER.with(|slot| {
        let mut slot = slot.borrow_mut();
        if let Some(listener) = slot.as_mut() {
            listener.subscribers.push((subscriber_id, status_refresh));
            return true;
        }

        let global: JsValue = js_sys::global().into();
        let event = JsValue::from_str("virya:resume");
        let Ok(add_listener) =
            js_sys::Reflect::get(&global, &JsValue::from_str("addEventListener"))
        else {
            return false;
        };
        let Ok(add_listener) = add_listener.dyn_into::<js_sys::Function>() else {
            return false;
        };
        let Ok(remove_listener) =
            js_sys::Reflect::get(&global, &JsValue::from_str("removeEventListener"))
        else {
            return false;
        };
        let Ok(remove_listener) = remove_listener.dyn_into::<js_sys::Function>() else {
            return false;
        };

        let callback = Closure::<dyn FnMut(JsValue)>::new(move |_| {
            // Clone the lightweight signals before updating them. An update can
            // synchronously unmount a subscriber and run its cleanup, which
            // must be allowed to borrow the registry mutably.
            let subscribers = RESUME_REFRESH_LISTENER.with(|slot| {
                slot.borrow()
                    .as_ref()
                    .map(|listener| {
                        listener
                            .subscribers
                            .iter()
                            .map(|(_, signal)| *signal)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default()
            });
            for signal in subscribers {
                let _ = signal.try_update(|value| *value = value.wrapping_add(1));
            }
        });
        if add_listener
            .call2(&global, &event, callback.as_ref().unchecked_ref())
            .is_err()
        {
            return false;
        }

        *slot = Some(ResumeRefreshListener {
            global,
            event,
            remove_listener,
            callback,
            subscribers: vec![(subscriber_id, status_refresh)],
        });
        true
    });

    if installed {
        // Only the integer subscription id crosses the Leptos cleanup boundary,
        // so the cleanup closure remains Send + Sync even though the actual
        // wasm-bindgen callback stays thread-local.
        on_cleanup(move || unregister_resume_refresh(subscriber_id));
    }
}

fn spawn_lifecycle_task(future: impl std::future::Future<Output = ()> + 'static) {
    // Normal UI work is owner-scoped. Only native operations that must survive
    // Android pause/resume or a reactive Effect rerun use this explicit escape hatch.
    wasm_bindgen_futures::spawn_local(future);
}

fn finish_resumable_ui_task(
    busy: RwSignal<bool>,
    resume_pending: RwSignal<bool>,
    resume_refresh: RwSignal<u32>,
) {
    let pending = resume_pending.try_get_untracked() == Some(true);
    let _ = busy.try_set(false);
    if pending {
        let _ = resume_pending.try_set(false);
        let _ = resume_refresh.try_update(|value| *value = value.wrapping_add(1).max(1));
    }
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
    let launcher_initialized = RwSignal::new(false);
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
        let initial_load = !launcher_initialized.get_untracked();
        if initial_load {
            operator_status_loading.set(true);
            fan_status_loading.set(true);
            operator_status_failed.set(false);
            fan_status_failed.set(false);
        }
        spawn_local(async move {
            let result = bridge::launcher_status().await;
            let completed = latest_request_completed(&result);
            match result {
                Ok(Some(status)) => {
                    operator_status.set(status.operator);
                    operator_status_failed.set(false);
                    fan_status.set(status.fan);
                    fan_status_failed.set(false);
                    launcher_initialized.set(true);
                }
                Ok(None) => {}
                Err(message) => {
                    // A transient resume failure must never tear down an already
                    // authenticated FanApp/OperatorApp and destroy its scoped async
                    // owners. Only cold-start failure replaces the launcher surface.
                    if initial_load {
                        operator_status_failed.set(true);
                        fan_status_failed.set(true);
                    }
                    error.set(Some(message));
                }
            }
            if completed && initial_load {
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
