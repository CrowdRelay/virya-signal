use std::cell::{Cell, RefCell};

use crate::i18n::{self, Language, tr};
use crate::util::{OptionValueOrElseExt, OptionValueOrExt};

mod area;
mod formatters;
mod types;

use crate::util::spawn_local;
use area::AreaGameScreen;
use formatters::{
    day, event_location, event_time_location, human_time, local_to_rfc3339, money, month, optional,
    plural_key,
};
use leptos::prelude::*;
use types::*;
use wasm_bindgen::{JsCast, JsValue, closure::Closure};

use crate::{
    bridge,
    models::{
        AdmissionPass, AdmissionQr, AdmissionRedemption, AreaWallet, AutopilotActionPayload,
        AutopilotChiefOfStaff, AutopilotMutation, AutopilotPolicySummary, BeaconEngagementResult,
        BeaconHomeData, BeaconMutationResult, BeaconPressRequestsData, BeaconPressRoomData,
        BeaconReleasesData, BeaconSessionStatus, CouponEnvelope, CreateQrCampaignInput,
        DashboardData, EventCity, FanAuthResult, FanConfirmationInput, FanDashboardData,
        FanEventInterest, FanHomeData, FanMerchBundleCatalog, FanPushPreferences,
        FanPushPreferencesUpdate, FanPushStatus, FanSessionStatus, FanSignupInput, FanTarget,
        IssuePassInput, IssuedPass, MerchCatalog, OperatorAutopilotOverview, OperatorOpsOverview,
        OperatorProfileInput, OperatorRole, OperatorSignalOverview, OpsDeliveryItem, OpsOutboxItem,
        OpsRetryResult, PublicEvent, PublicHomeData, QrCampaign, ReferralProgress,
        RequestedCityInput, RequestedCityResult, SessionStatus, ShowChecklist, ShowModeScanResult,
        ShowModeStatus, ShowModeSyncResult, SignalNewsFeed, StaffEventDashboard,
        TicketCheckoutInput, TicketCheckoutItemInput, TicketCheckoutStart, TicketSaleOffer,
        TicketWallet, TicketingOverview, WalletBatch, WalletTicket,
    },
};

struct ResumeRefreshListener {
    global: JsValue,
    events: Vec<JsValue>,
    remove_listener: js_sys::Function,
    callback: Closure<dyn FnMut(JsValue)>,
    // The flag is the subscriber's appetite for bare window focus. Explicit
    // virya:resume always reaches everyone.
    subscribers: Vec<(u64, RwSignal<u32>, bool)>,
}

impl Drop for ResumeRefreshListener {
    fn drop(&mut self) {
        for event in &self.events {
            let _ = self.remove_listener.call2(
                &self.global,
                event,
                self.callback.as_ref().unchecked_ref(),
            );
        }
    }
}

thread_local! {
    static RESUME_REFRESH_LISTENER: RefCell<Option<ResumeRefreshListener>> = const { RefCell::new(None) };
    static NEXT_RESUME_REFRESH_ID: Cell<u64> = const { Cell::new(1) };
}

struct LanguageRefreshListener {
    global: JsValue,
    event: JsValue,
    remove_listener: js_sys::Function,
    callback: Closure<dyn FnMut(JsValue)>,
}

impl Drop for LanguageRefreshListener {
    fn drop(&mut self) {
        let _ = self.remove_listener.call2(
            &self.global,
            &self.event,
            self.callback.as_ref().unchecked_ref(),
        );
    }
}

thread_local! {
    static LANGUAGE_REFRESH_LISTENER: RefCell<Option<LanguageRefreshListener>> = const { RefCell::new(None) };
}

/// Send a shell's content column back to the top.
///
/// Keep-alive tab pages are siblings inside a single `.content`, so the shell
/// has one scroll offset for every tab: scroll deep into Merch, switch to
/// Shows, and Shows opens mid-page. The fan shell tried to correct this with
/// `window().scroll_to_with_x_and_y(0, 0)`, and the operator shell claimed the
/// arrangement preserved scroll per panel. Neither held — `.authenticated` is
/// `height: 100dvh; overflow: hidden`, so the window has nothing to scroll and
/// that call moved nothing, while per-panel preservation would need per-panel
/// scroll boxes. Ask the column that actually scrolls.
fn reset_content_scroll(content: NodeRef<leptos::html::Div>) {
    if let Some(column) = content.get_untracked() {
        column.set_scroll_top(0);
    }
}

fn install_language_refresh(language_refresh: RwSignal<u32>) {
    LANGUAGE_REFRESH_LISTENER.with(|slot| {
        if slot.borrow().is_some() {
            return;
        }

        let global: JsValue = js_sys::global().into();
        let Ok(add_listener) =
            js_sys::Reflect::get(&global, &JsValue::from_str("addEventListener"))
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
        let event = JsValue::from_str("virya:language-change");
        let callback = Closure::<dyn FnMut(JsValue)>::new(move |_| {
            let _ = language_refresh.try_update(|value| {
                *value = value.wrapping_add(1).max(1);
            });
        });
        if add_listener
            .call2(&global, &event, callback.as_ref().unchecked_ref())
            .is_err()
        {
            return;
        }
        *slot.borrow_mut() = Some(LanguageRefreshListener {
            global,
            event,
            remove_listener,
            callback,
        });
    });
}

fn unregister_resume_refresh(subscriber_id: u64) {
    RESUME_REFRESH_LISTENER.with(|slot| {
        let mut slot = slot.borrow_mut();
        let Some(listener) = slot.as_mut() else {
            return;
        };
        listener
            .subscribers
            .retain(|(id, _, _)| *id != subscriber_id);
        if listener.subscribers.is_empty() {
            // Dropping the final subscriber removes the one global browser
            // listener as well. No wasm-bindgen closure is leaked.
            slot.take();
        }
    });
}

/// Root-level subscription. Adds bare window focus on top of the explicit
/// resume event, because warm App Links and notification intents arrive through
/// `onNewIntent` without recreating the WebView and dispatch nothing of ours.
fn install_root_resume_refresh(status_refresh: RwSignal<u32>) {
    install_resume_subscriber(status_refresh, true);
}

/// Subscription for a mounted panel. Focus alone is not evidence that anything
/// changed, and these subscribers answer it with real IPC and network work, so
/// they wake only on the deliberate resume signal.
fn install_resume_refresh(status_refresh: RwSignal<u32>) {
    install_resume_subscriber(status_refresh, false);
}

fn install_resume_subscriber(status_refresh: RwSignal<u32>, wants_focus: bool) {
    let subscriber_id = NEXT_RESUME_REFRESH_ID.with(|next| {
        let id = next.get();
        next.set(id.wrapping_add(1).max(1));
        id
    });

    let installed = RESUME_REFRESH_LISTENER.with(|slot| {
        let mut slot = slot.borrow_mut();
        if let Some(listener) = slot.as_mut() {
            listener
                .subscribers
                .push((subscriber_id, status_refresh, wants_focus));
            return true;
        }

        let global: JsValue = js_sys::global().into();
        // The explicit virya:resume event closes camera/settings races. Window
        // focus additionally covers warm Android App Links and notification
        // intents, whose onNewIntent callback does not recreate the WebView.
        let events = vec![
            JsValue::from_str("virya:resume"),
            JsValue::from_str("focus"),
        ];
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

        let callback = Closure::<dyn FnMut(JsValue)>::new(move |event: JsValue| {
            // An unreadable event type falls back to waking everyone, so a
            // genuine resume can never be filtered away by accident.
            let focus_only = js_sys::Reflect::get(&event, &JsValue::from_str("type"))
                .ok()
                .and_then(|value| value.as_string())
                .is_some_and(|value| value != "virya:resume");
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
                            .filter(|(_, _, wants_focus)| !focus_only || *wants_focus)
                            .map(|(_, signal, _)| *signal)
                            .collect::<Vec<_>>()
                    })
                    .unwrap_or_default()
            });
            for signal in subscribers {
                let _ = signal.try_update(|value| *value = value.wrapping_add(1));
            }
        });
        for (registered, event) in events.iter().enumerate() {
            if add_listener
                .call2(&global, event, callback.as_ref().unchecked_ref())
                .is_err()
            {
                for previous in &events[..registered] {
                    let _ =
                        remove_listener.call2(&global, previous, callback.as_ref().unchecked_ref());
                }
                return false;
            }
        }

        *slot = Some(ResumeRefreshListener {
            global,
            events,
            remove_listener,
            callback,
            subscribers: vec![(subscriber_id, status_refresh, wants_focus)],
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

pub(crate) fn spawn_lifecycle_task(future: impl std::future::Future<Output = ()> + 'static) {
    // Normal UI work is owner-scoped. Only native operations that must survive
    // Android pause/resume or a reactive Effect rerun use this explicit escape
    // hatch — plus the pre-mount bootstrap in `main`, which runs before any
    // reactive owner or executor exists at all.
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

fn persisted_root_mode() -> RootMode {
    match bridge::root_mode_state().as_str() {
        "latarnik" => RootMode::Latarnik,
        "team" => RootMode::Team,
        _ => RootMode::Fan,
    }
}

fn persist_root_mode(mode: RootMode) {
    match mode {
        RootMode::Latarnik => bridge::set_root_mode_state("latarnik"),
        RootMode::Fan => bridge::set_root_mode_state("fan"),
        // Team is persisted only for the current WebView session on the JS
        // side. This survives the i18n reload without making staff access a
        // durable localStorage bypass after a real app restart.
        RootMode::Team => bridge::set_root_mode_state("team"),
        RootMode::StaffGate => {}
    }
}

#[component]
pub fn App() -> impl IntoView {
    let mode = RwSignal::new(persisted_root_mode());
    let operator_status = RwSignal::new(SessionStatus::default());
    let operator_dashboard = RwSignal::new(None::<DashboardData>);
    let operator_tab = RwSignal::new(OperatorTab::Home);
    let fan_status = RwSignal::new(FanSessionStatus::default());
    let beacon_status = RwSignal::new(BeaconSessionStatus::default());
    let beacon_pending_link = RwSignal::new(false);
    let synesthesia_pending_link = RwSignal::new(false);
    let synesthesia_link_busy = RwSignal::new(false);
    let operator_status_loading = RwSignal::new(true);
    let fan_status_loading = RwSignal::new(true);
    let beacon_status_loading = RwSignal::new(true);
    let operator_status_failed = RwSignal::new(false);
    let fan_status_failed = RwSignal::new(false);
    let beacon_status_failed = RwSignal::new(false);
    let status_refresh = RwSignal::new(0_u32);
    let launcher_initialized = RwSignal::new(false);
    let push_target = RwSignal::new(None::<String>);
    let operator_push_target = RwSignal::new(None::<String>);
    let beacon_push_target = RwSignal::new(None::<String>);
    let error = RwSignal::new(None::<String>);
    let language_refresh = RwSignal::new(0_u32);
    install_root_resume_refresh(status_refresh);
    install_language_refresh(language_refresh);

    Effect::new(move |_| {
        persist_root_mode(mode.get());
    });

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
                    } else if target.starts_with("/latarnik") {
                        beacon_push_target.set(Some(target));
                        mode.set(RootMode::Latarnik);
                    } else {
                        push_target.set(Some(target));
                        mode.set(RootMode::Fan);
                    }
                }
                Ok(None) => {}
                Err(message) => set_error_debounced(error, message),
            }
        });
    });

    Effect::new(move |_| {
        status_refresh.get();
        if !bridge::native_available() {
            return;
        }
        spawn_local(async move {
            match bridge::invoke::<bool, _>("beacon_take_app_link", &EmptyArgs {}).await {
                // The native side keeps the capability until it is exchanged or
                // cleared, so this reports `true` on every later tick as well.
                // Only the transition is an event; repeating the ceremony would
                // relock a working session and pin the user to this mode.
                Ok(true) if !beacon_pending_link.get_untracked() => {
                    // A fresh invitation is a new trust ceremony. If an old
                    // Beacon session is still unlocked in memory (for example
                    // after the relationship was paused/re-invited remotely),
                    // force the access surface before showing the pending link.
                    // The previous Stronghold vault remains intact until the
                    // new exchange commits, so cancelling can still unlock it.
                    match bridge::invoke::<BeaconSessionStatus, _>("beacon_lock", &EmptyArgs {})
                        .await
                    {
                        Ok(value) => beacon_status.set(value),
                        Err(message) => {
                            set_error_debounced(error, message);
                            return;
                        }
                    }
                    beacon_pending_link.set(true);
                    mode.set(RootMode::Latarnik);
                }
                Ok(_) => {}
                Err(message) => set_error_debounced(error, message),
            }
        });
    });

    // Synesthesia completion App Links have their own native channel. The
    // capability stays native while the fan is locked and is consumed only
    // after the existing fan session is unlocked. QR/scanner state is unrelated.
    Effect::new(move |_| {
        status_refresh.get();
        if !bridge::native_available() {
            return;
        }
        spawn_local(async move {
            match bridge::invoke::<bool, _>("fan_take_synesthesia_app_link", &EmptyArgs {}).await {
                Ok(true) => {
                    synesthesia_pending_link.set(true);
                    mode.set(RootMode::Fan);
                }
                Ok(false) => {}
                Err(message) => set_error_debounced(error, message),
            }
        });
    });

    Effect::new(move |_| {
        status_refresh.get();
        let pending = synesthesia_pending_link.get();
        let unlocked = fan_status.get().unlocked;
        if !pending || !unlocked || synesthesia_link_busy.get_untracked() {
            return;
        }
        synesthesia_link_busy.set(true);
        spawn_lifecycle_task(async move {
            match bridge::invoke_timeout::<String, _>(
                "fan_link_pending_synesthesia",
                &EmptyArgs {},
                15_000,
            )
            .await
            {
                Ok(outcome) if outcome == "linked" => {
                    let _ = synesthesia_pending_link.try_set(false);
                    status_refresh.update(|value| *value = value.wrapping_add(1));
                    let _ =
                        error.try_set(Some(tr("synesthesia_result_saved_in_signal").to_owned()));
                }
                Ok(outcome) if outcome == "expired" => {
                    let _ = synesthesia_pending_link.try_set(false);
                    status_refresh.update(|value| *value = value.wrapping_add(1));
                    let _ = error.try_set(Some(tr("synesthesia_handoff_expired_retry").to_owned()));
                }
                Ok(_) => {
                    let _ = synesthesia_pending_link.try_set(false);
                }
                Err(message) => {
                    // Retryable failures keep the native capability. A later
                    // resume/status refresh can retry without minting a new one.
                    let _ = error.try_set(Some(message));
                }
            }
            let _ = synesthesia_link_busy.try_set(false);
        });
    });

    Effect::new(move |_| {
        status_refresh.get();
        let initial_load = !launcher_initialized.get_untracked();
        if initial_load {
            operator_status_loading.set(true);
            fan_status_loading.set(true);
            beacon_status_loading.set(true);
            operator_status_failed.set(false);
            fan_status_failed.set(false);
            beacon_status_failed.set(false);
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
                    beacon_status.set(status.beacon);
                    beacon_status_failed.set(false);
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
                        beacon_status_failed.set(true);
                    }
                    set_error_debounced(error, message);
                }
            }
            if completed && initial_load {
                operator_status_loading.set(false);
                fan_status_loading.set(false);
                beacon_status_loading.set(false);
            }
        });
    });

    // Fans and beacons never see error toasts. Clear the signal on entry
    // to those modes so a stale error from a staff session doesn't surface
    // later. Staff/owner modes still get the Toast for diagnosis.
    Effect::new(move |_| {
        if !matches!(mode.get(), RootMode::Team | RootMode::StaffGate) {
            error.set(None);
        }
    });

    view! {
        <main class="app-shell">
            {move || {
                // Translation strings are intentionally external to the WASM and
                // evaluated while a component subtree is built. Rebuild only the
                // active portal after a language change instead of reloading the
                // WebView, which would destroy transient Team-mode state on Android.
                language_refresh.get();
                match mode.get() {
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
                RootMode::Latarnik => view! {
                    <BeaconPortal
                        mode=mode
                        status=beacon_status
                        status_loading=beacon_status_loading
                        status_failed=beacon_status_failed
                        status_refresh=status_refresh
                        pending_link=beacon_pending_link
                        push_target=beacon_push_target
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
                        dashboard=operator_dashboard
                        tab=operator_tab
                        status_loading=operator_status_loading
                        status_failed=operator_status_failed
                        status_refresh=status_refresh
                        push_target=operator_push_target
                        error=error
                    />
                }.into_any(),
                }
            }}
            // Error toasts are for staff/owner only. Fans and beacons never
            // see them — their data refreshes are already silent (keep last
            // good snapshot on failure), and a red popup on a fan's face
            // after a successful pin login is the exact experience we must
            // avoid. Staff sees errors so they can fix them later.
            <Show when=move || matches!(mode.get(), RootMode::Team | RootMode::StaffGate)>
                <Toast error=error />
            </Show>
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
include!("app/beacon.rs");
include!("app/support.rs");
