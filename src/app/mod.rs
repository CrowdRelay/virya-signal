use crate::i18n::{self, Language, tr};
use crate::util::{OptionValueOrElseExt, OptionValueOrExt};

mod formatters;
mod types;

use formatters::{
    day, event_location, event_time_location, human_time, local_to_rfc3339, money, month, optional,
};
use leptos::prelude::*;
use types::*;
use wasm_bindgen_futures::spawn_local;

use crate::{
    bridge,
    models::{
        AdmissionPass, AdmissionQr, AdmissionRedemption, AreaWallet, CouponEnvelope,
        CreateQrCampaignInput, DashboardData, FanAuthResult, FanConfirmationInput,
        FanDashboardData, FanEventInterest, FanMerchBundleCatalog, FanSessionStatus,
        FanSignupInput, IssuePassInput, IssuedPass, MerchCatalog, OperatorOpsOverview,
        OperatorProfileInput, OperatorRole, OperatorSignalOverview, OpsDeliveryItem, OpsOutboxItem,
        OpsRetryResult, PublicEvent, PublicHomeData, QrCampaign, ReferralProgress,
        RequestedCityInput, RequestedCityResult, SessionStatus, ShowModeScanResult, ShowModeStatus,
        ShowModeSyncResult, TicketCheckoutInput, TicketCheckoutItemInput, TicketCheckoutStart,
        TicketSaleOffer, TicketWallet, TicketingOverview, WalletBatch, WalletTicket,
    },
};

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
    let error = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        status_refresh.get();
        operator_status_loading.set(true);
        fan_status_loading.set(true);
        operator_status_failed.set(false);
        fan_status_failed.set(false);
        spawn_local(async move {
            match bridge::launcher_status().await {
                Ok(status) => {
                    operator_status.set(status.operator);
                    operator_status_failed.set(false);
                    fan_status.set(status.fan);
                    fan_status_failed.set(false);
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

#[component]
fn BackButton(mode: RwSignal<RootMode>) -> impl IntoView {
    view! { <button class="back-button" on:click=move |_| mode.set(RootMode::Fan)>{tr("sygna")}</button> }
}

#[component]
fn StaffEntryButton(mode: RwSignal<RootMode>) -> impl IntoView {
    view! {
        <button
            type="button"
            class="staff-entry-button"
            on:click=move |_| mode.set(RootMode::StaffGate)
        >
            <span aria-hidden="true">"⌁"</span>
            {tr("jestes_w_staffie")}
        </button>
    }
}

#[component]
fn StaffGate(mode: RwSignal<RootMode>, error: RwSignal<Option<String>>) -> impl IntoView {
    let password = RwSignal::new(String::new());
    let busy = RwSignal::new(false);

    let submit = move |_| {
        if busy.get_untracked() {
            return;
        }
        let current_password = password.get_untracked();
        if current_password.is_empty() {
            error.set(Some(
                tr("podaj_haso_staff_uzywane_w_panelu_virya").to_owned(),
            ));
            return;
        }
        busy.set(true);
        spawn_local(async move {
            let result = bridge::invoke_unit(
                "verify_staff_access",
                &StaffGateArgs {
                    password: &current_password,
                },
            )
            .await;
            password.set(String::new());
            busy.set(false);
            match result {
                Ok(()) => mode.set(RootMode::Team),
                Err(message) => error.set(Some(message)),
            }
        });
    };

    view! {
        <section class="access-screen staff-gate-screen">
            <button
                class="back-button"
                disabled=move || busy.get()
                on:click=move |_| mode.set(RootMode::Fan)
            >
                {tr("sygna")}
            </button>
            <header class="hero compact staff-gate-hero">
                <p class="eyebrow">"VIRYA / STAFF"</p>
                <h1>{tr("strefa")}<em>{tr("zespou")}</em></h1>
                <p>{tr("dostep_do_bramki_sprzedazy_i_obsugi_koncertu_jest")}</p>
            </header>
            <div class="access-card staff-gate-card">
                <div class="staff-gate-lock" aria-hidden="true">"⌁"</div>
                <div>
                    <p class="eyebrow">{tr("weryfikacja_staff")}</p>
                    <h2>{tr("haso_panelu_virya")}</h2>
                    <p>{tr("uzyj_tego_samego_hasa_co_w_qr_bramce")}</p>
                </div>
                <label>
                    {tr("haso_staff")}
                    <input
                        type="password"
                        autocomplete="current-password"
                        maxlength="256"
                        prop:value=move || password.get()
                        on:input=move |event| password.set(event_target_value(&event))
                    />
                </label>
                <button
                    class="primary"
                    disabled=move || busy.get() || password.get().is_empty()
                    on:click=submit
                >
                    {move || if busy.get() { tr("sprawdzam") } else { tr("otworz_strefe_staff") }}
                </button>
                <p class="staff-gate-note">{tr("haso_jest_sprawdzane_po_stronie_virya_music_nie")}</p>
            </div>
        </section>
    }
}

#[component]
fn OperatorPortal(
    mode: RwSignal<RootMode>,
    status: RwSignal<SessionStatus>,
    status_loading: RwSignal<bool>,
    status_failed: RwSignal<bool>,
    status_refresh: RwSignal<u32>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let dashboard = RwSignal::new(None::<DashboardData>);
    let tab = RwSignal::new(OperatorTab::Home);

    view! {
        {move || if status_failed.get() {
            view! { <StatusFailure mode=mode status_refresh=status_refresh label=tr("nie_udao_sie_odczytac_sejfu_staff") show_back=true /> }.into_any()
        } else if status_loading.get() {
            view! { <AccessLoader mode=mode label=tr("sprawdzam_bezpieczny_sejf") show_back=true /> }.into_any()
        } else if status.get().unlocked {
            view! { <OperatorApp mode=mode status=status dashboard=dashboard tab=tab error=error /> }.into_any()
        } else {
            view! { <OperatorAccess mode=mode status=status error=error /> }.into_any()
        }}
    }
}

fn new_operator_pin_is_valid(pin: &str) -> bool {
    (4..=6).contains(&pin.len()) && pin.bytes().all(|byte| byte.is_ascii_digit())
}

fn normalize_new_operator_pin(value: String) -> String {
    value.chars().filter(char::is_ascii_digit).take(6).collect()
}

#[component]
fn OperatorAccess(
    mode: RwSignal<RootMode>,
    status: RwSignal<SessionStatus>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let pin = RwSignal::new(String::new());
    let pairing = RwSignal::new(String::new());
    let advanced = RwSignal::new(false);
    let name = RwSignal::new(tr("virya_staff").to_owned());
    let token = RwSignal::new(String::new());
    let api = RwSignal::new(API_BASE.to_owned());
    let role = RwSignal::new(OperatorRole::Staff);
    let busy = RwSignal::new(false);

    let unlock = move |_| {
        let current_pin = pin.get();
        if current_pin.chars().count() < 4 {
            error.set(Some(tr("pin_musi_miec_co_najmniej_4_znaki").to_owned()));
            return;
        }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<SessionStatus, _>("unlock", &PinArgs { pin: &current_pin }).await
            {
                Ok(value) => {
                    pin.set(String::new());
                    status.set(value);
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let pair = move |payload: String| {
        let current_pin = pin.get();
        if !new_operator_pin_is_valid(&current_pin) || payload.trim().is_empty() {
            error.set(Some(
                tr("podaj_46_cyfrowy_pin_i_zeskanuj_albo_wklej").to_owned(),
            ));
            return;
        }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<SessionStatus, _>(
                "configure_from_pairing",
                &PairingArgs {
                    pin: &current_pin,
                    payload: &payload,
                },
            )
            .await
            {
                Ok(value) => {
                    pin.set(String::new());
                    pairing.set(String::new());
                    status.set(value);
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let scan_pairing = move |_| {
        spawn_local(async move {
            match bridge::scan_qr().await {
                Ok(Some(value)) => {
                    pairing.set(value);
                    if new_operator_pin_is_valid(&pin.get()) {
                        pair(pairing.get());
                    }
                }
                Ok(None) => {}
                Err(message) => error.set(Some(message)),
            }
        });
    };
    let submit_pairing = move |_| pair(pairing.get());

    let configure_manual = move |_| {
        let current_pin = pin.get();
        let profile = OperatorProfileInput {
            display_name: name.get(),
            api_base_url: api.get(),
            role: role.get(),
            bearer_token: token.get(),
        };
        if !new_operator_pin_is_valid(&current_pin) || profile.bearer_token.trim().len() < 24 {
            error.set(Some(
                tr("podaj_46_cyfrowy_pin_i_poprawny_token_urzadzenia").to_owned(),
            ));
            return;
        }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<SessionStatus, _>(
                "configure",
                &ConfigureArgs {
                    pin: &current_pin,
                    profile: &profile,
                },
            )
            .await
            {
                Ok(value) => {
                    pin.set(String::new());
                    token.set(String::new());
                    status.set(value);
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    view! {
        <section class="access-screen">
            <BackButton mode=mode />
            <header class="hero compact">
                <p class="eyebrow">{tr("virya_control")}</p>
                <h1>{tr("sparuj")}<em>{tr("urzadzenie")}</em></h1>
                <p>{tr("bez_przepisywania_api_roli_i_dugiego_sekretu")}</p>
            </header>
            <div class="access-card">
                <Show when=move || status.get().configured fallback=move || view! {
                    <div class="pairing-flow">
                        <Show when=move || pairing.get().trim().is_empty() fallback=move || view! {
                            <div class="pairing-scanned">
                                <span class="pairing-ok">"✓"</span>
                                <strong>{tr("kod_zeskanowany")}</strong>
                                <small>{tr("wpisz_pin_ponizej_i_kliknij_sparuj")}</small>
                            </div>
                        }>
                            <button class="pairing-scan primary" on:click=scan_pairing disabled=move || busy.get()>
                                <span class="pairing-qr">"▦"</span>
                                <strong>{move || if busy.get() { tr("acze") } else { tr("zeskanuj_kod_qr") }}</strong>
                                <small>{tr("kod_pokazany_w_panelu_virya")}</small>
                            </button>
                        </Show>
                        <div class="pairing-divider"><span></span><small>{tr("albo")}</small><span></span></div>
                        <label>{tr("kod_parowania")}<textarea rows="2" placeholder="virya-signal://pair?…" prop:value=move || pairing.get() on:input=move |e| pairing.set(event_target_value(&e))></textarea></label>
                        <label class="pin-field">
                            <span class="pin-field-label">{tr("utworz_pin_do_odblokowania")}</span>
                            <small id="operator-new-pin-help">{tr("wpisz_46_cyfr_np_2580_to_pin_tylko")}</small>
                            <input type="password" autocomplete="new-password" inputmode="numeric" pattern="[0-9]*" maxlength="6" placeholder=tr("np_2580") aria-describedby="operator-new-pin-help" prop:value=move || pin.get() on:input=move |e| pin.set(normalize_new_operator_pin(event_target_value(&e))) />
                        </label>
                        <button class="primary" on:click=submit_pairing disabled=move || busy.get() || pairing.get().trim().is_empty() || !new_operator_pin_is_valid(&pin.get())>{tr("sparuj_2")}</button>
                        <button class="text-button" type="button" on:click=move |_| advanced.update(|v| *v = !*v)>
                            {move || if advanced.get() { tr("ukryj_ustawienia_reczne") } else { tr("ustawienia_zaawansowane") }}
                        </button>
                        <Show when=move || advanced.get()>
                            <div class="advanced-config">
                                <label>{tr("nazwa_urzadzenia_osoby")}<input prop:value=move || name.get() on:input=move |e| name.set(event_target_value(&e)) /></label>
                                <label>"API CrowdRelay"<input prop:value=move || api.get() on:input=move |e| api.set(event_target_value(&e)) /></label>
                                <div class="segmented">
                                    <button class:active=move || role.get() == OperatorRole::Owner on:click=move |_| role.set(OperatorRole::Owner)>{tr("owner")}</button>
                                    <button class:active=move || role.get() == OperatorRole::Staff on:click=move |_| role.set(OperatorRole::Staff)>{tr("staff")}</button>
                                </div>
                                <label>{tr("token_urzadzenia")}<textarea rows="3" prop:value=move || token.get() on:input=move |e| token.set(event_target_value(&e))></textarea></label>
                                <button class="ghost" on:click=configure_manual disabled=move || busy.get() || !new_operator_pin_is_valid(&pin.get())>{tr("zapisz_recznie")}</button>
                            </div>
                        </Show>
                    </div>
                }>
                    <div class="form-grid">
                        <p class="lock-copy">{tr("profil_operatora_jest_zaszyfrowany_lokalnie")}</p>
                        <label class="pin-field">
                            <span class="pin-field-label">{tr("pin_do_odblokowania_aplikacji")}</span>
                            <small id="operator-unlock-pin-help">{tr("wpisz_pin_ustawiony_podczas_parowania_tego_urzadzenia")}</small>
                            <input type="password" autocomplete="current-password" inputmode="numeric" placeholder=tr("twoj_pin") aria-describedby="operator-unlock-pin-help" prop:value=move || pin.get() on:input=move |e| pin.set(event_target_value(&e)) />
                        </label>
                        <button class="primary" disabled=move || busy.get() || pin.get().chars().count() < 4 on:click=unlock>{tr("odblokuj")}</button>
                    </div>
                </Show>
            </div>
        </section>
    }
}

#[component]
fn OperatorApp(
    mode: RwSignal<RootMode>,
    status: RwSignal<SessionStatus>,
    dashboard: RwSignal<Option<DashboardData>>,
    tab: RwSignal<OperatorTab>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let loading = RwSignal::new(OperatorLoadingState::all());

    let signal_overview = RwSignal::new(None::<OperatorSignalOverview>);
    let signal_loading = RwSignal::new(false);
    let signal_requested = RwSignal::new(false);
    let menu_open = RwSignal::new(false);
    Effect::new(move |_| {
        if status.get().unlocked && dashboard.get().is_none() {
            dashboard.set(Some(DashboardData::default()));
            refresh_operator_parts(dashboard, loading, error);
        }
    });

    let role = move || {
        status
            .get()
            .session
            .map(|s| s.role)
            .value_or(OperatorRole::Staff)
    };

    let owner = Signal::derive(move || {
        status
            .get()
            .session
            .is_some_and(|session| session.role == OperatorRole::Owner)
    });

    Effect::new(move |_| {
        let should_load = status.get().unlocked
            && owner.get()
            && tab.get() == OperatorTab::Signal
            && !signal_requested.get()
            && !signal_loading.get();
        if should_load {
            signal_requested.set(true);
            refresh_operator_signal(signal_overview, signal_loading, error);
        }
    });

    let close = move |_| {
        bridge::invalidate_latest("operator:");
        spawn_local(async move {
            match bridge::invoke::<SessionStatus, _>("lock", &EmptyArgs {}).await {
                Ok(value) => {
                    dashboard.set(None);
                    loading.set(OperatorLoadingState::all());
                    signal_overview.set(None);
                    signal_loading.set(false);
                    signal_requested.set(false);
                    status.set(value);
                    mode.set(RootMode::Fan);
                }
                Err(message) => error.set(Some(message)),
            }
        });
    };

    view! {
        <section class="authenticated">
            <header class="topbar">
                <div on:dblclick=move |_| refresh_operator_parts(dashboard, loading, error) style="cursor:pointer"><p class="eyebrow">{tr("virya_control")}</p><strong>{move || status.get().session.map(|s| s.display_name).value_or_else(Default::default)}</strong></div>
                <div class="topbar-actions">
                    <span class="role-pill">{move || role().label()}</span>
                    <button class="menu-trigger" aria-label=tr("otworz_menu") aria-expanded=move || menu_open.get() on:click=move |_| menu_open.update(|v| *v = !*v)><i></i><i></i><i></i></button>
                    <button aria-label=tr("zamknij_i_zablokuj_panel") on:click=close>"×"</button>
                </div>
            </header>
            <Show when=move || menu_open.get()>
                <div class="overflow-backdrop" on:click=move |_| menu_open.set(false)></div>
                <nav class="overflow-menu">
                    <button class:active=move || tab.get() == OperatorTab::Discounts on:click=move |_| { tab.set(OperatorTab::Discounts); menu_open.set(false); }><span>"%"</span>{tr("znizki")}</button>
                    <button class:active=move || tab.get() == OperatorTab::Campaigns on:click=move |_| { tab.set(OperatorTab::Campaigns); menu_open.set(false); }><span>"◫"</span>{tr("kody_qr")}</button>
                    <button class:active=move || tab.get() == OperatorTab::Settings on:click=move |_| { tab.set(OperatorTab::Settings); menu_open.set(false); }><span>"⚙"</span>{tr("ustawienia")}</button>
                </nav>
            </Show>
            <div class="content">
                {move || match tab.get() {
                    OperatorTab::Home => view! { <OperatorHome dashboard=dashboard loading=loading /> }.into_any(),
                    OperatorTab::Signal => view! { <OperatorSignal overview=signal_overview loading=signal_loading owner=owner error=error /> }.into_any(),
                    OperatorTab::Scan => view! { <Scanner dashboard=dashboard loading=loading error=error /> }.into_any(),
                    OperatorTab::Tickets => view! { <Tickets dashboard=dashboard loading=loading error=error owner=owner /> }.into_any(),
                    OperatorTab::Discounts => view! { <Discounts error=error /> }.into_any(),
                    OperatorTab::Campaigns => view! { <Campaigns dashboard=dashboard loading=loading error=error /> }.into_any(),
                    OperatorTab::Settings => view! { <OperatorSettings status=status dashboard=dashboard loading=loading error=error /> }.into_any(),
                }}
            </div>
            <nav class="bottom-nav four primary-four">
                <NavButton tab=tab own=OperatorTab::Home icon="home" label=tr("start") />
                <NavButton tab=tab own=OperatorTab::Signal icon="signal" label=tr("sygna_2") />
                <NavButton tab=tab own=OperatorTab::Scan icon="scan" label=tr("skan") />
                <NavButton tab=tab own=OperatorTab::Tickets icon="ticket" label=tr("bilety") />
            </nav>
        </section>
    }
}

#[component]
fn NavButton<T>(tab: RwSignal<T>, own: T, icon: &'static str, label: &'static str) -> impl IntoView
where
    T: Copy + PartialEq + Send + Sync + 'static,
{
    view! { <button class:active=move || tab.get() == own on:click=move |_| tab.set(own)><NavGlyph icon=icon/><small>{label}</small></button> }
}

#[component]
fn NavGlyph(icon: &'static str) -> impl IntoView {
    match icon {
        "signal" => view! {
            <svg class="nav-icon" viewBox="0 0 24 24" aria-hidden="true" focusable="false">
                <path d="M6 15V9M12 19V5M18 16V8"/>
            </svg>
        }
        .into_any(),
        "events" => view! {
            <svg class="nav-icon" viewBox="0 0 24 24" aria-hidden="true" focusable="false">
                <rect x="3" y="5" width="18" height="16" rx="2"/>
                <path d="M8 3v4M16 3v4M3 10h18M8 14h3M13 14h3M8 17h3"/>
            </svg>
        }
        .into_any(),
        "scan" => view! {
            <svg class="nav-icon" viewBox="0 0 24 24" aria-hidden="true" focusable="false">
                <path d="M8 4H4v4M16 4h4v4M20 16v4h-4M8 20H4v-4M7 12h10"/>
            </svg>
        }
        .into_any(),
        "shop" => view! {
            <svg class="nav-icon" viewBox="0 0 24 24" aria-hidden="true" focusable="false">
                <path d="M5 8h14l-1 12H6L5 8Z"/>
                <path d="M9 9V6a3 3 0 0 1 6 0v3"/>
            </svg>
        }
        .into_any(),
        "ticket" => view! {
            <svg class="nav-icon" viewBox="0 0 24 24" aria-hidden="true" focusable="false">
                <path d="M4 6h16v4a2 2 0 0 0 0 4v4H4v-4a2 2 0 0 0 0-4V6Z"/>
                <path d="M9 9v6"/>
            </svg>
        }
        .into_any(),
        _ => view! {
            <svg class="nav-icon" viewBox="0 0 24 24" aria-hidden="true" focusable="false">
                <path d="M4 13 12 5l8 8M7 11v9h10v-9"/>
            </svg>
        }
        .into_any(),
    }
}

#[component]
fn OperatorHome(
    dashboard: RwSignal<Option<DashboardData>>,
    loading: RwSignal<OperatorLoadingState>,
) -> impl IntoView {
    view! {
        <section class="screen">
            <header class="screen-title"><p class="eyebrow">LIVE OPERATIONS</p><h2>{tr("dzisiaj_pod_kontrola")}</h2></header>
            <Show when=move || !loading.get().events fallback=move || view! { <Skeleton /> }>
            {move || dashboard.with(|state| state.as_ref().map(|data| {
                let next = data.events.first().cloned();
                let event_count = data.events.len();
                let events = data.events.iter().take(8).cloned().collect::<Vec<_>>();
                let active = data.qr.as_ref().map(|q| q.campaigns.iter().filter(|c| c.active).count()).value_or(0);
                let checkins = data.qr.as_ref().map(|q| q.campaigns.iter().map(|c| c.checkin_count).sum::<u64>()).value_or(0);
                let qr_loading = loading.get().qr;
                view! {
                    {next.map(|event| {
                        let location = event_location(&event);
                        let time = human_time(&event.starts_at);
                        let title = event.title;
                        view! {
                            <article class="hero-card"><p class="eyebrow">{tr("nastepny_koncert")}</p><h3>{title}</h3><p>{location}</p><time>{time}</time></article>
                        }
                    })}
                    <div class="stats-grid"><Metric value=event_count.to_string() label=tr("koncerty_2")/><Metric value=if qr_loading { "…".to_owned() } else { active.to_string() } label=tr("aktywne_qr")/><Metric value=if qr_loading { "…".to_owned() } else { checkins.to_string() } label=tr("check_iny")/></div>
                    <div class="section-head"><h3>{tr("nadchodzace")}</h3><span>{event_count}</span></div>
                    {if events.is_empty() {
                        view! { <div class="empty-state"><strong>{tr("brak_nadchodzacych_koncertow")}</strong><p>{tr("kiedy_pojawi_sie_nowe_wydarzenie_zobaczysz_je_tutaj")}</p></div> }.into_any()
                    } else {
                        view! { <div class="card-list">{events.into_iter().map(|event| view! { <EventCard event=event /> }).collect_view()}</div> }.into_any()
                    }}
                }
            }.into_any())).value_or_else(|| view! { <Skeleton /> }.into_any())}
            </Show>
        </section>
    }
}

#[component]
fn Metric(value: String, label: &'static str) -> impl IntoView {
    view! { <article class="metric"><strong>{value}</strong><span>{label}</span></article> }
}

#[component]
fn EventCard(event: PublicEvent) -> impl IntoView {
    let event_day = day(&event.starts_at);
    let event_month = month(&event.starts_at);
    let location = event_location(&event);
    let title = event.title;
    view! {
        <article class="event-card">
            <div class="date-block"><strong>{event_day}</strong><span>{event_month}</span></div>
            <div><h4>{title}</h4><p>{location}</p></div><span class="chevron">">"</span>
        </article>
    }
}

#[component]
fn OperatorSignal(
    overview: RwSignal<Option<OperatorSignalOverview>>,
    loading: RwSignal<bool>,
    owner: Signal<bool>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let refresh = move |_| {
        if owner.get_untracked() {
            refresh_operator_signal(overview, loading, error);
        }
    };

    view! {
        <section class="screen signal-admin-screen">
            <header class="screen-title">
                <p class="eyebrow">{tr("virya_signal")}</p>
                <h2>{tr("spoecznosc_i_wzrost")}</h2>
                <p class="screen-copy">{tr("zbiorczy_obraz_sygnau_bez_danych_osobowych_fanow")}</p>
            </header>
            <Show
                when=move || owner.get()
                fallback=move || view! {
                    <div class="empty-state">
                        <strong>{tr("widok_tylko_dla_ownera")}</strong>
                        <p>{tr("statystyki_zgod_wzrostu_i_miast_sa_dostepne_wyacznie")}</p>
                    </div>
                }
            >
                <div class="signal-admin-toolbar">
                    <p>{tr("dane_sa_agregowane_w_crowdrelay_i_nie_zawieraja")}</p>
                    <button class="text-button" on:click=refresh disabled=move || loading.get()>
                        {move || if loading.get() { tr("odswiezam") } else { tr("odswiez") }}
                    </button>
                </div>
                <Show when=move || !loading.get() fallback=move || view! { <Skeleton rows=4 /> }>
                    {move || {
                        overview
                            .get()
                            .map(|data| view! { <SignalOverviewContent data=data /> }.into_any())
                            .value_or_else(|| {
                                view! {
                                    <div class="empty-state">
                                        <strong>{tr("brak_snapshotu_sygnau")}</strong>
                                        <p>{tr("odswiez_dane_jezeli_backend_jest_jeszcze_w_trakcie")}</p>
                                    </div>
                                }
                                .into_any()
                            })
                    }}
                </Show>
            </Show>
        </section>
    }
}

#[component]
fn SignalOverviewContent(data: OperatorSignalOverview) -> impl IntoView {
    let summary = data.summary;
    let activity = data.activity;
    let confirmation_base = summary.active_fans.saturating_add(summary.pending_fans);
    let confirmation_rate = if confirmation_base > 0 {
        format!(
            "{:.0}%",
            (summary.active_fans.max(0) as f64 * 100.0) / confirmation_base as f64
        )
    } else {
        "—".to_owned()
    };
    let generated_at = human_time(&data.generated_at);
    let unavailable = data.unavailable_sources;
    let degraded_view = if unavailable.is_empty() {
        None
    } else {
        Some(view! {
            <p class="security-note warning">
                {i18n::format("snapshot_czesciowy_niedostepne_zroda_value", &[unavailable.join(", ").to_string()])}
            </p>
        })
    };
    let city_count = data.top_cities.len();
    let city_cards = data
        .top_cities
        .into_iter()
        .map(|city| {
            view! {
                <article class="signal-city-card">
                    <div>
                        <strong>{city.name}</strong>
                        <small>{city.country_code}</small>
                    </div>
                    <span>{i18n::format("value_aktywnych", &[city.active_fans.max(0).to_string()])}</span>
                </article>
            }
        })
        .collect_view();
    let cities_view = if city_count == 0 {
        view! {
            <div class="empty-state compact">
                <strong>{tr("brak_agregatu_miast")}</strong>
                <p>{tr("sygna_nie_ma_jeszcze_potwierdzonych_danych_miejskich_albo")}</p>
            </div>
        }
        .into_any()
    } else {
        view! { <div class="signal-city-list">{city_cards}</div> }.into_any()
    };

    view! {
        <div class="signal-admin-content">
            <div class="stats-grid">
                <Metric value=summary.active_fans.max(0).to_string() label=tr("aktywni")/>
                <Metric value=summary.marketing_opted_in.max(0).to_string() label=tr("zgody_marketingowe")/>
                <Metric value=activity.new_fans_30d.max(0).to_string() label=tr("nowi_30_dni")/>
            </div>
            <article class="signal-health-card">
                <div>
                    <p class="eyebrow">{tr("zdrowie_bazy")}</p>
                    <strong>{confirmation_rate}</strong>
                    <span>{tr("potwierdzonych_sposrod_aktywnych_i_oczekujacych")}</span>
                </div>
                <dl>
                    <div><dt>{tr("wszyscy")}</dt><dd>{summary.total_fans.max(0)}</dd></div>
                    <div><dt>{tr("oczekujacy")}</dt><dd>{summary.pending_fans.max(0)}</dd></div>
                    <div><dt>{tr("wypisani")}</dt><dd>{summary.unsubscribed_fans.max(0)}</dd></div>
                    <div><dt>{tr("wyciszeni")}</dt><dd>{summary.suppressed_fans.max(0)}</dd></div>
                    <div><dt>{tr("powiadomienia_w_poblizu")}</dt><dd>{summary.nearby_enabled.max(0)}</dd></div>
                </dl>
            </article>
            <div class="section-head"><h3>{tr("aktywnosc")}</h3><span>{tr("30_dni_caosc")}</span></div>
            <div class="signal-activity-grid">
                <article><strong>{activity.new_fans_7d.max(0)}</strong><span>{tr("nowi_7_dni")}</span></article>
                <article><strong>{format!("{} / {}", activity.referral_attributions_30d.max(0), activity.referral_attributions_total.max(0))}</strong><span>{tr("polecenia")}</span></article>
                <article><strong>{format!("{} / {}", activity.event_interests_30d.max(0), activity.event_interests_total.max(0))}</strong><span>{tr("zainteresowania_koncertami")}</span></article>
                <article><strong>{activity.nearby_notifications_30d.max(0)}</strong><span>{tr("powiadomienia_nearby")}</span></article>
                <article><strong>{activity.pending_city_requests.max(0)}</strong><span>{tr("miasta_do_moderacji")}</span></article>
            </div>
            {degraded_view}
            <div class="section-head"><h3>{tr("najsilniejsze_miasta")}</h3><span>{city_count}</span></div>
            {cities_view}
            <p class="security-note">{i18n::format("snapshot_generated_at_dane_wyacznie_zagregowane", std::slice::from_ref(&generated_at))}</p>
        </div>
    }
}

#[component]
fn Scanner(
    dashboard: RwSignal<Option<DashboardData>>,
    loading: RwSignal<OperatorLoadingState>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let event_slug = RwSignal::new(String::new());
    let manual = RwSignal::new(String::new());
    let result = RwSignal::new(None::<AdmissionRedemption>);
    let busy = RwSignal::new(false);
    let offline = RwSignal::new(false);
    let show_mode = RwSignal::new(ShowModeStatus::default());
    let show_message = RwSignal::new(String::new());

    let refresh_show_status = move |slug: String| {
        if slug.is_empty() {
            show_mode.set(ShowModeStatus::default());
            offline.set(false);
            return;
        }
        spawn_local(async move {
            match bridge::invoke::<ShowModeStatus, _>(
                "show_mode_status",
                &EventArgs { event_slug: &slug },
            )
            .await
            {
                Ok(value) => {
                    if !value.prepared {
                        offline.set(false);
                    }
                    show_mode.set(value);
                }
                Err(message) => error.set(Some(message)),
            }
        });
    };

    let select_event = move |event| {
        let slug = event_target_value(&event);
        event_slug.set(slug.clone());
        result.set(None);
        show_message.set(String::new());
        refresh_show_status(slug);
    };

    let redeem_code = move |code: String| {
        let slug = event_slug.get_untracked();
        if slug.is_empty() {
            error.set(Some(tr("najpierw_wybierz_koncert").to_owned()));
            return;
        }
        busy.set(true);
        let use_offline = offline.get_untracked();
        spawn_local(async move {
            if use_offline {
                match bridge::invoke::<ShowModeScanResult, _>(
                    "show_mode_scan",
                    &RedeemArgs {
                        event_slug: &slug,
                        code: &code,
                    },
                )
                .await
                {
                    Ok(value) => {
                        let status = if value.duplicate {
                            "offline_duplicate"
                        } else {
                            "offline_queued"
                        };
                        result.set(Some(AdmissionRedemption {
                            public_reference: value.public_reference,
                            holder_name: value.holder_name,
                            holder_email_masked: value.holder_email_masked,
                            status: status.to_owned(),
                        }));
                        refresh_show_status(slug.clone());
                    }
                    Err(message) => error.set(Some(message)),
                }
            } else {
                match bridge::invoke::<AdmissionRedemption, _>(
                    "redeem_admission",
                    &RedeemArgs {
                        event_slug: &slug,
                        code: &code,
                    },
                )
                .await
                {
                    Ok(value) => result.set(Some(value)),
                    Err(message) => error.set(Some(message)),
                }
            }
            busy.set(false);
        });
    };

    let scan = move |_| {
        spawn_local(async move {
            match bridge::scan_qr().await {
                Ok(Some(code)) => redeem_code(code),
                Ok(None) => {}
                Err(message) => error.set(Some(message)),
            }
        });
    };
    let manual_submit = move |_| redeem_code(manual.get().trim().to_owned());

    let prepare = move |_| {
        let slug = event_slug.get();
        if slug.is_empty() {
            error.set(Some(tr("wybierz_koncert").to_owned()));
            return;
        }
        busy.set(true);
        show_message.set(String::new());
        spawn_local(async move {
            match bridge::invoke::<ShowModeStatus, _>(
                "show_mode_prepare",
                &EventArgs { event_slug: &slug },
            )
            .await
            {
                Ok(value) => {
                    show_message.set(i18n::format(
                        "snapshot_gotowy_value_trwaych_biletow",
                        &[value.eligible_passes.to_string()],
                    ));
                    offline.set(true);
                    show_mode.set(value);
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let sync = move |_| {
        let slug = event_slug.get();
        if slug.is_empty() {
            return;
        }
        busy.set(true);
        show_message.set(String::new());
        spawn_local(async move {
            match bridge::invoke::<ShowModeSyncResult, _>(
                "show_mode_sync",
                &EventArgs { event_slug: &slug },
            )
            .await
            {
                Ok(value) => {
                    show_message.set(i18n::format(
                        "sync_value_zapisane_value_konfliktow_value_nadal_czeka",
                        &[
                            value.synced.to_string(),
                            value.conflicts.to_string(),
                            value.pending.to_string(),
                        ],
                    ));
                    refresh_show_status(slug.clone());
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let clear = move |_| {
        let slug = event_slug.get();
        if slug.is_empty() {
            return;
        }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<ShowModeStatus, _>(
                "show_mode_clear",
                &EventArgs { event_slug: &slug },
            )
            .await
            {
                Ok(value) => {
                    show_mode.set(value);
                    offline.set(false);
                    show_message.set(tr("dane_koncertu_usuniete_z_urzadzenia").to_owned());
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    view! {
        <section class="screen">
            <header class="screen-title"><p class="eyebrow">GATE MODE</p><h2>{tr("skanuj_wejscie")}</h2></header>
            <label class="select-label">{tr("koncert")}<select disabled=move || loading.get().events prop:value=move || event_slug.get() on:change=select_event><option value="">{move || if loading.get().events { tr("aduje_koncerty") } else { tr("wybierz_wydarzenie") }}</option><For each=move || operator_events(dashboard) key=|event| event.slug.clone() children=move |event| view! { <option value=event.slug.clone()>{event.title}</option> } /></select></label>
            <article class:show-mode-active=move || offline.get() class="show-mode-card">
                <div class="section-head"><div><p class="eyebrow">OFFLINE SHOW MODE</p><h3>{move || if offline.get() { tr("bramka_pracuje_lokalnie") } else { tr("odpornosc_na_brak_lte") }}</h3></div><button type="button" class:active=move || offline.get() disabled=move || !show_mode.get().prepared || busy.get() on:click=move |_| offline.update(|value| *value = !*value)>{move || if offline.get() { tr("offline_on") } else { tr("offline_off") }}</button></div>
                <p>{move || if show_mode.get().prepared { i18n::format("value_biletow_value_oczekuje_value_konfliktow", &[show_mode.get().eligible_passes.to_string(), show_mode.get().pending.to_string(), show_mode.get().conflicts.to_string()]) } else { tr("pobierz_bezpieczny_snapshot_przed_otwarciem_bram").to_owned() }}</p>
                <div class="show-mode-actions"><button type="button" on:click=prepare disabled=move || busy.get() || event_slug.get().is_empty()>{tr("przygotuj_offline")}</button><button type="button" on:click=sync disabled=move || busy.get() || !show_mode.get().prepared>{tr("synchronizuj")}</button><button type="button" class="danger ghost" on:click=clear disabled=move || busy.get() || !show_mode.get().prepared>{tr("wyczysc")}</button></div>
                <Show when=move || !show_message.get().is_empty()><small>{move || show_message.get()}</small></Show>
            </article>
            <button class="scanner-button" on:click=scan disabled=move || busy.get()><span class="scanner-frame"></span><strong>{move || if busy.get() { tr("weryfikuje") } else if offline.get() { tr("skanuj_lokalnie") } else { tr("uruchom_aparat") }}</strong><small>{move || if offline.get() { tr("wyacznie_trway_qr_biletu_t1") } else { tr("qr_biletu_lub_wejsciowki") }}</small></button>
            <Show when=move || !offline.get()><div class="manual-row"><input placeholder=tr("kod_qr_lub_numer_wejsciowki") prop:value=move || manual.get() on:input=move |e| manual.set(event_target_value(&e))/><button on:click=manual_submit disabled=move || busy.get()>{tr("sprawdz")}</button></div></Show>
            {move || result.get().map(|entry| {
                let success = matches!(
                    entry.status.as_str(),
                    "redeemed" | "already_redeemed" | "offline_queued" | "offline_duplicate"
                );
                view! { <article class:scan-success=success class:scan-warning=!success class="scan-result"><strong>{entry.status.to_uppercase()}</strong><span>{entry.public_reference}</span><p>{entry.holder_name.value_or(entry.holder_email_masked)}</p></article> }
            })}
        </section>
    }
}

#[component]
fn Tickets(
    dashboard: RwSignal<Option<DashboardData>>,
    loading: RwSignal<OperatorLoadingState>,
    error: RwSignal<Option<String>>,
    owner: Signal<bool>,
) -> impl IntoView {
    let event_slug = RwSignal::new(String::new());
    let overview = RwSignal::new(None::<TicketingOverview>);
    let busy = RwSignal::new(false);
    let fan_email = RwSignal::new(String::new());
    let pool_slug = RwSignal::new("tickets".to_owned());
    let revoke_ref = RwSignal::new(String::new());
    let issued = RwSignal::new(None::<IssuedPass>);

    let load = move |_| {
        let slug = event_slug.get();
        if slug.is_empty() {
            error.set(Some(tr("wybierz_koncert").to_owned()));
            return;
        }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<TicketingOverview, _>(
                "ticketing_overview",
                &EventArgs { event_slug: &slug },
            )
            .await
            {
                Ok(value) => overview.set(Some(value)),
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let issue = move |_| {
        let input = IssuePassInput {
            event_slug: event_slug.get(),
            pool_slug: pool_slug.get().trim().to_owned(),
            fan_email: fan_email.get().trim().to_owned(),
            claim_expires_hours: 72,
        };
        if input.event_slug.is_empty() || input.fan_email.trim().is_empty() {
            error.set(Some(tr("wybierz_koncert_i_podaj_e_mail_fana").to_owned()));
            return;
        }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<IssuedPass, _>("issue_pass", &IssueArgs { input: &input }).await
            {
                Ok(value) => issued.set(Some(value)),
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let revoke = move |_| {
        let reference = revoke_ref.get().trim().to_owned();
        if reference.is_empty() {
            error.set(Some(tr("podaj_public_reference_wejsciowki").to_owned()));
            return;
        }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke_unit(
                "revoke_pass",
                &ReferenceArgs {
                    public_reference: &reference,
                },
            )
            .await
            {
                Ok(_) => error.set(Some(tr("wejsciowka_zostaa_uniewazniona").to_owned())),
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    view! {
        <section class="screen">
            <header class="screen-title"><p class="eyebrow">TICKETING</p><h2>{tr("bilety_i_wejsciowki")}</h2></header>
            <div class="toolbar"><select disabled=move || loading.get().events prop:value=move || event_slug.get() on:change=move |e| event_slug.set(event_target_value(&e))><option value="">{move || if loading.get().events { tr("aduje_koncerty") } else { tr("wybierz_koncert_2") }}</option>{move || operator_events(dashboard).into_iter().map(|event| view! { <option value=event.slug.clone()>{event.title}</option> }).collect_view()}</select><button on:click=load disabled=move || busy.get() || loading.get().events>{tr("odswiez")}</button></div>
            {move || overview.get().map(|data| view! {
                <div class="stats-grid wide"><Metric value=data.paid_tickets.to_string() label=tr("sprzedane")/><Metric value=data.sale.reserved.to_string() label=tr("w_trakcie")/><Metric value=data.sale.available.to_string() label=tr("dostepne")/></div>
                <div class="revenue-card"><p>{tr("obrot_brutto")}</p><strong>{money(data.gross_sales_minor, &data.sale.currency)}</strong><span>{format!("zwroty: {}", money(data.refunded_minor, &data.sale.currency))}</span></div>
                <div class="section-head"><h3>{tr("ostatnie_zamowienia")}</h3><span>{data.recent_orders.len()}</span></div>
                <div class="card-list">{data.recent_orders.into_iter().map(|order| view! { <article class="order-card"><div><strong>{order.public_reference}</strong><p>{order.buyer_name.value_or(order.buyer_email_masked)}</p></div><span>{money(order.amount_gross_minor, &order.currency)}</span></article> }).collect_view()}</div>
            })}
            <Show when=move || owner.get()><div class="admin-box"><p class="eyebrow">OWNER ONLY</p><h3>{tr("reczna_wejsciowka")}</h3><p class="inline-note">{tr("numer_wejsciowki_to_bezpieczny_publiczny_identyfikator_np_vry")}</p><input placeholder="fan@email.com" prop:value=move || fan_email.get() on:input=move |e| fan_email.set(event_target_value(&e))/><input placeholder="pool slug" prop:value=move || pool_slug.get() on:input=move |e| pool_slug.set(event_target_value(&e))/><button class="primary" on:click=issue disabled=move || busy.get()>{tr("wydaj_wejsciowke")}</button>{move || issued.get().map(|pass| view! { <div class="token-box"><strong>{pass.public_reference}</strong><p>Token roszczenia: {pass.claim_token}</p></div> })}<hr/><input placeholder=tr("numer_wejsciowki_np_vry") prop:value=move || revoke_ref.get() on:input=move |e| revoke_ref.set(event_target_value(&e))/><button class="danger" on:click=revoke disabled=move || busy.get()>{tr("uniewaznij")}</button></div></Show>
        </section>
    }
}

#[component]
fn Discounts(error: RwSignal<Option<String>>) -> impl IntoView {
    let code = RwSignal::new(String::new());
    let order = RwSignal::new(String::new());
    let result = RwSignal::new(None::<CouponEnvelope>);
    let busy = RwSignal::new(false);
    let redeem = move |_| {
        let c = code.get().trim().to_owned();
        let o = order.get().trim().to_owned();
        if c.is_empty() || o.is_empty() {
            error.set(Some(tr("podaj_kod_i_numer_sprzedazy").to_owned()));
            return;
        }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<CouponEnvelope, _>(
                "redeem_coupon",
                &CouponArgs {
                    code: &c,
                    order_reference: &o,
                },
            )
            .await
            {
                Ok(value) => result.set(Some(value)),
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };
    view! {
        <section class="screen"><header class="screen-title"><p class="eyebrow">MERCH DESK</p><h2>{tr("realizuj_znizke")}</h2></header><div class="coupon-visual"><span>%</span><div><strong>{tr("virya_signal")}</strong><p>{tr("kupon_fanowski_kontrolowane_uzycie")}</p></div></div><div class="form-grid panel"><label>{tr("kod_znizkowy")}<input autocapitalize="characters" placeholder="VIRYA-…" prop:value=move || code.get() on:input=move |e| code.set(event_target_value(&e))/></label><label>{tr("numer_sprzedazy")}<input placeholder="MERCH-WRO-001" prop:value=move || order.get() on:input=move |e| order.set(event_target_value(&e))/></label><button class="primary" on:click=redeem disabled=move || busy.get()>{tr("zrealizuj_kupon")}</button></div>{move || result.get().map(|envelope| view! { <article class="scan-result scan-success"><strong>{tr("kupon_zrealizowany")}</strong><span>{envelope.result.status}</span><p>{i18n::format("uzycie_value_value", &[envelope.result.used_count.to_string(), envelope.result.max_uses.to_string()])}</p></article> })}</section>
    }
}

#[component]
fn Campaigns(
    dashboard: RwSignal<Option<DashboardData>>,
    loading: RwSignal<OperatorLoadingState>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let event_slug = RwSignal::new(String::new());
    let label = RwSignal::new(tr("wejscie_gowne").to_owned());
    let valid_from = RwSignal::new(String::new());
    let valid_until = RwSignal::new(String::new());
    let max_checkins = RwSignal::new(String::new());
    let busy = RwSignal::new(false);

    let create = move |_| {
        let Some(from) = local_to_rfc3339(&valid_from.get()) else {
            error.set(Some(tr("podaj_poprawny_poczatek_waznosci").to_owned()));
            return;
        };
        let Some(until) = local_to_rfc3339(&valid_until.get()) else {
            error.set(Some(tr("podaj_poprawny_koniec_waznosci").to_owned()));
            return;
        };
        let max_value = max_checkins.get();
        let max = if max_value.trim().is_empty() {
            None
        } else {
            match max_value.trim().parse::<u32>() {
                Ok(value) if value > 0 => Some(value),
                _ => {
                    error.set(Some(tr("limit_musi_byc_dodatnia_liczba").to_owned()));
                    return;
                }
            }
        };
        if until <= from {
            error.set(Some(
                tr("koniec_kampanii_musi_byc_pozniej_niz_poczatek").to_owned(),
            ));
            return;
        }
        let input = CreateQrCampaignInput {
            event_slug: event_slug.get(),
            label: label.get().trim().to_owned(),
            valid_from: from,
            valid_until: until,
            max_checkins: max,
        };
        if input.event_slug.is_empty() || input.label.is_empty() {
            error.set(Some(tr("wybierz_koncert_i_nazwij_kampanie").to_owned()));
            return;
        }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<QrCampaign, _>(
                "create_qr_campaign",
                &CampaignArgs { input: &input },
            )
            .await
            {
                Ok(_) => {
                    error.set(Some(tr("kampania_qr_utworzona").to_owned()));
                    refresh_operator_qr(dashboard, loading, error);
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    view! {
        <section class="screen"><header class="screen-title"><p class="eyebrow">CONCERT SIGNAL</p><h2>{tr("kampanie_qr")}</h2></header><div class="form-grid panel"><label>{tr("koncert")}<select disabled=move || loading.get().qr prop:value=move || event_slug.get() on:change=move |e| event_slug.set(event_target_value(&e))><option value="">{move || if loading.get().qr { tr("aduje_kampanie") } else { tr("wybierz_koncert_2") }}</option><For each=move || operator_qr_events(dashboard) key=|event| event.slug.clone() children=move |event| view! { <option value=event.slug.clone()>{event.title}</option> } /></select></label><label>{tr("nazwa_punktu_kampanii")}<input prop:value=move || label.get() on:input=move |e| label.set(event_target_value(&e))/></label><div class="two-cols"><label>{tr("wazna_od")}<input type="datetime-local" prop:value=move || valid_from.get() on:input=move |e| valid_from.set(event_target_value(&e))/></label><label>{tr("wazna_do")}<input type="datetime-local" prop:value=move || valid_until.get() on:input=move |e| valid_until.set(event_target_value(&e))/></label></div><label>{tr("limit_check_inow_opcjonalnie")}<input inputmode="numeric" prop:value=move || max_checkins.get() on:input=move |e| max_checkins.set(event_target_value(&e))/></label><button class="primary" on:click=create disabled=move || busy.get() || loading.get().qr>{tr("utworz_kampanie")}</button></div><div class="section-head"><h3>{tr("aktywne_i_historyczne")}</h3></div><Show when=move || !loading.get().qr fallback=move || view! { <Skeleton /> }><div class="card-list"><For each=move || operator_campaigns(dashboard) key=|campaign| campaign.id.clone() children=move |campaign| view! { <CampaignCard campaign=campaign dashboard=dashboard loading=loading error=error /> } /></div></Show></section>
    }
}

#[component]
fn CampaignCard(
    campaign: QrCampaign,
    dashboard: RwSignal<Option<DashboardData>>,
    loading: RwSignal<OperatorLoadingState>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let id = campaign.id.clone();
    let active = campaign.active;
    let revoke = move |_| {
        let campaign_id = id.clone();
        spawn_local(async move {
            match bridge::invoke_unit(
                "revoke_qr_campaign",
                &CampaignIdArgs {
                    campaign_id: &campaign_id,
                },
            )
            .await
            {
                Ok(_) => {
                    error.set(Some(tr("kampania_zostaa_wyaczona").to_owned()));
                    refresh_operator_qr(dashboard, loading, error);
                }
                Err(message) => error.set(Some(message)),
            }
        });
    };
    let revoke_button = if active {
        Some(view! {
            <button class="danger ghost" on:click=revoke>
                {tr("wyacz_kampanie")}
            </button>
        })
    } else {
        None
    };
    view! {
        <article class="campaign-card"><div class="campaign-head"><div><strong>{campaign.label}</strong><p>{campaign.event_title}</p></div><span class:online=active class:offline=!active>{if active { tr("active") } else { tr("closed") }}</span></div><div class="campaign-stats"><span>{i18n::format("value_check_inow", &[campaign.checkin_count.to_string()])}</span><span>{campaign.max_checkins.map(|v| i18n::format("limit_value", &[v.to_string()])).value_or_else(|| tr("bez_limitu").to_owned())}</span></div>{campaign.token.map(|token| view! { <code>{token}</code> })}{revoke_button}</article>
    }
}

#[component]
fn LanguageSwitch() -> impl IntoView {
    let selected = i18n::current();
    view! {
        <article class="language-setting">
            <div>
                <strong>{tr("jezyk_aplikacji")}</strong>
                <p>{tr("zmiana_jezyka_przeaduje_interfejs_dane_i_sesja_pozostana")}</p>
            </div>
            <div class="language-switch" role="group" aria-label=tr("jezyk")>
                <button type="button" class:active=selected == Language::Pl on:click=move |_| i18n::select(Language::Pl)>"PL"</button>
                <button type="button" class:active=selected == Language::En on:click=move |_| i18n::select(Language::En)>"EN"</button>
            </div>
        </article>
    }
}

#[component]
fn OperatorSettings(
    status: RwSignal<SessionStatus>,
    dashboard: RwSignal<Option<DashboardData>>,
    loading: RwSignal<OperatorLoadingState>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let ops = RwSignal::new(None::<OperatorOpsOverview>);
    let ops_loading = RwSignal::new(false);
    let owner = Signal::derive(move || {
        status
            .get()
            .session
            .is_some_and(|session| session.role == OperatorRole::Owner)
    });
    Effect::new(move |_| {
        if owner.get() && ops.get().is_none() && !ops_loading.get() {
            refresh_operator_ops(ops, ops_loading, error);
        }
    });
    let refresh = move |_| {
        refresh_operator_parts(dashboard, loading, error);
        if owner.get_untracked() {
            refresh_operator_ops(ops, ops_loading, error);
        }
    };
    let lock = move |_| {
        spawn_local(async move {
            match bridge::invoke::<SessionStatus, _>("lock", &EmptyArgs {}).await {
                Ok(value) => {
                    dashboard.set(None);
                    loading.set(OperatorLoadingState::all());
                    status.set(value);
                }
                Err(message) => error.set(Some(message)),
            }
        })
    };
    let forget = move |_| {
        spawn_local(async move {
            match bridge::invoke::<SessionStatus, _>("forget_device", &EmptyArgs {}).await {
                Ok(value) => {
                    dashboard.set(None);
                    loading.set(OperatorLoadingState::all());
                    status.set(value);
                }
                Err(message) => error.set(Some(message)),
            }
        })
    };
    view! {
        <section class="screen">
            <header class="screen-title"><p class="eyebrow">{tr("device")}</p><h2>{tr("ustawienia")}</h2></header>
            <div class="settings-list">
                <LanguageSwitch />
                <article><div><strong>{tr("poaczenie")}</strong><p>{move || status.get().session.map(|s| s.api_base_url).value_or_else(Default::default)}</p></div><span class:online=move || !loading.get().events && !loading.get().qr>{move || if loading.get().events || loading.get().qr { tr("acze_2") } else { tr("online") }}</span></article>
                <article><div><strong>{tr("uprawnienia")}</strong><p>{move || status.get().session.map(|s| s.role.label().to_owned()).value_or_else(Default::default)}</p></div></article>
                <button on:click=refresh disabled=move || loading.get().events || loading.get().qr>{tr("odswiez_wszystkie_dane")}</button>
                <button on:click=lock>{tr("zablokuj_panel")}</button>
                <button class="danger ghost" on:click=forget>{tr("usun_profil_operatora")}</button>
            </div>
            <Show when=move || owner.get()><OpsPanel overview=ops loading=ops_loading error=error /></Show>
            <AnonymousFeedback error=error />
            <p class="security-note">{tr("token_operatora_przechowuje_zaszyfrowany_sejf_stronghold_warstwa_webview")}</p>
        </section>
    }
}

#[component]
fn OpsPanel(
    overview: RwSignal<Option<OperatorOpsOverview>>,
    loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let refresh = move |_| refresh_operator_ops(overview, loading, error);
    view! {
        <section class="ops-panel"><div class="section-head"><div><p class="eyebrow">CONTROL PLANE</p><h3>{tr("kolejki_i_dostawy")}</h3></div><button class="text-button" on:click=refresh disabled=move || loading.get()>{tr("odswiez_2")}</button></div>
            <Show when=move || !loading.get() fallback=move || view! { <Skeleton rows=2 /> }>
                {move || overview.get().map(|data| {
                    let summary = data.summary;
                    let deliveries = data.dead_deliveries;
                    let outbox = data.dead_outbox;
                    let unavailable_sources = data.unavailable_sources;
                    let healthy = deliveries.is_empty() && outbox.is_empty() && unavailable_sources.is_empty();
                    let degraded_view = (!unavailable_sources.is_empty()).then(|| view! {
                        <p class="security-note warning">{i18n::format("cockpit_dziaa_czesciowo_niedostepne_value", &[unavailable_sources.join(", ").to_string()])}</p>
                    });
                    let deliveries_view = (!deliveries.is_empty()).then(|| view! {
                        <div class="section-head"><h3>{tr("martwe_dostawy")}</h3></div>
                        <div class="ops-list"><For each=move || deliveries.clone() key=|item| item.id.clone() children=move |item| view! { <OpsDeliveryCard item=item overview=overview loading=loading error=error /> } /></div>
                    });
                    let outbox_view = (!outbox.is_empty()).then(|| view! {
                        <div class="section-head"><h3>{tr("martwy_outbox")}</h3></div>
                        <div class="ops-list"><For each=move || outbox.clone() key=|item| item.id.clone() children=move |item| view! { <OpsOutboxCard item=item overview=overview loading=loading error=error /> } /></div>
                    });
                    let healthy_view = healthy.then(|| view! { <p class="ops-healthy">{tr("brak_martwych_wpisow_tor_dostaw_jest_czysty")}</p> });
                    view! {
                        <div class="ops-metrics"><Metric value=summary.outbox.pending.to_string() label="outbox pending"/><Metric value=summary.outbox.dead.to_string() label="outbox dead"/><Metric value=summary.deliveries.pending.to_string() label="delivery pending"/><Metric value=summary.deliveries.dead.to_string() label="delivery dead"/></div>
                        {degraded_view}
                        {deliveries_view}
                        {outbox_view}
                        {healthy_view}
                    }
                })}
            </Show>
        </section>
    }
}

#[component]
fn OpsDeliveryCard(
    item: OpsDeliveryItem,
    overview: RwSignal<Option<OperatorOpsOverview>>,
    loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let target_id = item.id.clone();
    let endpoint_active = item.endpoint_active;
    let retry =
        move |_| retry_operator_item("deliveries", target_id.clone(), overview, loading, error);
    view! { <article class="ops-item"><div><strong>{item.event_type}</strong><p>{i18n::format("value_proba_value_value", &[item.endpoint_name.to_string(), item.attempt_count.to_string(), item.max_attempts.to_string()])}</p><small>{item.last_error_kind.value_or_else(|| tr("brak_kodu_bedu").to_owned())}</small></div><button class="danger ghost" on:click=retry disabled=move || loading.get() || !endpoint_active>{tr("retry")}</button></article> }
}

#[component]
fn OpsOutboxCard(
    item: OpsOutboxItem,
    overview: RwSignal<Option<OperatorOpsOverview>>,
    loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let target_id = item.id.clone();
    let retry = move |_| retry_operator_item("outbox", target_id.clone(), overview, loading, error);
    view! { <article class="ops-item"><div><strong>{item.event_type}</strong><p>{i18n::format("proba_value_value", &[item.attempts.to_string(), item.max_attempts.to_string()])}</p><small>{item.last_error_kind.value_or_else(|| tr("brak_kodu_bedu").to_owned())}</small></div><button class="danger ghost" on:click=retry disabled=move || loading.get()>{tr("retry")}</button></article> }
}

fn retry_operator_item(
    target_kind: &'static str,
    target_id: String,
    overview: RwSignal<Option<OperatorOpsOverview>>,
    loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) {
    if loading.get_untracked() {
        return;
    }
    loading.set(true);
    spawn_local(async move {
        match bridge::invoke::<OpsRetryResult, _>(
            "operator_retry",
            &RetryArgs {
                target_kind,
                target_id: &target_id,
            },
        )
        .await
        {
            Ok(result) => {
                error.set(Some(if result.replayed {
                    tr("retry_by_juz_wczesniej_przyjety").to_owned()
                } else {
                    tr("wpis_wroci_do_kolejki").to_owned()
                }));
                refresh_operator_ops(overview, loading, error);
            }
            Err(message) => {
                loading.set(false);
                error.set(Some(message));
            }
        }
    });
}

#[component]
fn FanPortal(
    mode: RwSignal<RootMode>,
    status: RwSignal<FanSessionStatus>,
    status_loading: RwSignal<bool>,
    status_failed: RwSignal<bool>,
    status_refresh: RwSignal<u32>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    // Entering the fan zone must be a local-only transition. City data is
    // fetched only when the user explicitly asks for the canonical list.
    let public = RwSignal::new(Some(PublicHomeData::default()));

    view! {
        <Show when=move || !status.get().unlocked>
            <StaffEntryButton mode=mode />
        </Show>
        {move || if status_failed.get() {
            view! { <StatusFailure mode=mode status_refresh=status_refresh label=tr("nie_udao_sie_odczytac_profilu_fana") show_back=false /> }.into_any()
        } else if status_loading.get() {
            view! { <AccessLoader mode=mode label=tr("sprawdzam_twoj_sygna") show_back=false /> }.into_any()
        } else if status.get().unlocked {
            view! { <FanApp mode=mode status=status public=public error=error /> }.into_any()
        } else {
            view! { <FanAccess status=status error=error /> }.into_any()
        }}
    }
}

#[component]
fn StatusFailure(
    mode: RwSignal<RootMode>,
    status_refresh: RwSignal<u32>,
    label: &'static str,
    show_back: bool,
) -> impl IntoView {
    view! {
        <section class="access-screen status-failure">
            <Show when=move || show_back>
                <BackButton mode=mode />
            </Show>
            <div class="access-card">
                <p class="eyebrow">{label}</p>
                <h2>{tr("profil_pozostaje_nietkniety")}</h2>
                <p>{tr("aplikacja_nie_przejdzie_do_rejestracji_ani_parowania_dopoki")}</p>
                <button
                    class="primary"
                    on:click=move |_| status_refresh.update(|value| *value = value.wrapping_add(1))
                >
                    {tr("sprobuj_ponownie")}
                </button>
            </div>
        </section>
    }
}

#[component]
fn AccessLoader(mode: RwSignal<RootMode>, label: &'static str, show_back: bool) -> impl IntoView {
    view! {
        <section class="access-screen status-loader">
            <Show when=move || show_back>
                <BackButton mode=mode />
            </Show>
            <div class="access-card">
                <p class="eyebrow">{label}</p>
                <Skeleton rows=2 />
            </div>
        </section>
    }
}

// VIRYA SIGNAL FAN CONFIRM UX V1
fn submit_fan_confirmation(
    email: RwSignal<String>,
    name: RwSignal<String>,
    token: RwSignal<String>,
    pin: RwSignal<String>,
    busy: RwSignal<bool>,
    status: RwSignal<FanSessionStatus>,
    error: RwSignal<Option<String>>,
) {
    if busy.get_untracked() {
        return;
    }
    let input_email = email.get_untracked().trim().to_owned();
    let current_token = token.get_untracked().trim().to_owned();
    let current_pin = pin.get_untracked();
    if input_email.is_empty() {
        error.set(Some(
            tr("podaj_e_mail_uzyty_przy_zapisie_do_sygnau").to_owned(),
        ));
        return;
    }
    if current_token.is_empty() {
        error.set(Some(tr("wklej_kod_cay_link_albo_zeskanuj_qr_z").to_owned()));
        return;
    }
    if current_pin.chars().count() < 4 {
        error.set(Some(
            tr("ustaw_lokalny_pin_majacy_co_najmniej_4_znaki").to_owned(),
        ));
        return;
    }
    let input = FanConfirmationInput {
        api_base_url: API_BASE.to_owned(),
        email: input_email,
        display_name: optional(name.get_untracked().trim().to_owned()),
        token: current_token,
    };
    busy.set(true);
    spawn_local(async move {
        match bridge::invoke::<FanAuthResult, _>(
            "fan_confirm",
            &FanConfirmArgs {
                input: &input,
                pin: &current_pin,
            },
        )
        .await
        {
            Ok(_) => {
                pin.set(String::new());
                token.set(String::new());
                refresh_fan_status(status, error);
            }
            Err(message) => error.set(Some(message)),
        }
        busy.set(false);
    });
}

#[component]
fn FanAccess(status: RwSignal<FanSessionStatus>, error: RwSignal<Option<String>>) -> impl IntoView {
    let access_mode = RwSignal::new(FanAccessMode::Signup);
    let email = RwSignal::new(String::new());
    let name = RwSignal::new(String::new());
    // City onboarding is deliberately local-first. The remote canonical list is
    // not placed in Leptos reactive state: this removes a repeatedly crashing
    // Android WebView/WASM path while preserving full signup functionality.
    let custom_city_name = RwSignal::new(String::new());
    let custom_region = RwSignal::new(String::new());
    let referral = RwSignal::new(String::new());
    let token = RwSignal::new(String::new());
    let pin = RwSignal::new(String::new());
    let consent = RwSignal::new(false);
    let nearby_enabled = RwSignal::new(true);
    let radius_km = RwSignal::new(150_u16);
    let busy = RwSignal::new(false);
    let recovery_open = RwSignal::new(false);

    let unlock = move |_| {
        let current_pin = pin.get();
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<FanSessionStatus, _>(
                "fan_unlock",
                &PinArgs { pin: &current_pin },
            )
            .await
            {
                Ok(value) => {
                    pin.set(String::new());
                    status.set(value);
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let signup = move |_| {
        if !consent.get() {
            error.set(Some(
                tr("zgoda_marketingowa_jest_wymagana_do_doaczenia_do_sygnau").to_owned(),
            ));
            return;
        }
        let current_pin = pin.get();
        let requested = RequestedCityInput {
            name: custom_city_name.get().trim().to_owned(),
            region: optional(custom_region.get().trim().to_owned()),
            country_code: "PL".to_owned(),
        };
        let input_email = email.get().trim().to_owned();
        let input_name = optional(name.get().trim().to_owned());
        let input_referral = optional(referral.get().trim().to_owned());
        let nearby = nearby_enabled.get();
        let radius = radius_km.get();
        busy.set(true);
        spawn_local(async move {
            let city_slug = match bridge::invoke::<RequestedCityResult, _>(
                "request_city",
                &RequestedCityArgs { input: &requested },
            )
            .await
            {
                Ok(value) => value.city_slug,
                Err(message) => {
                    error.set(Some(i18n::format(
                        "nie_udao_sie_zapisac_miasta_message",
                        std::slice::from_ref(&message),
                    )));
                    busy.set(false);
                    return;
                }
            };
            if city_slug.trim().is_empty() {
                error.set(Some(tr("wybierz_miasto_albo_wpisz_wasne").to_owned()));
                busy.set(false);
                return;
            }
            let input = FanSignupInput {
                api_base_url: API_BASE.to_owned(),
                email: input_email,
                display_name: input_name,
                city_slug,
                locale: i18n::current().code().to_owned(),
                referral_code: input_referral,
                policy_version: POLICY_VERSION.to_owned(),
                nearby_gigs_enabled: nearby,
                nearby_radius_km: radius,
            };
            match bridge::invoke::<FanAuthResult, _>(
                "fan_signup",
                &FanSignupArgs {
                    input: &input,
                    pin: &current_pin,
                },
            )
            .await
            {
                Ok(result) => {
                    if result.session_created {
                        pin.set(String::new());
                        refresh_fan_status(status, error);
                    } else {
                        access_mode.set(FanAccessMode::Confirm);
                        let message = match result.email_queued {
                            Some(true)
                                if result.email_kind.as_deref() == Some("session_recovery") =>
                            {
                                { tr("wysalismy_bezpieczny_link_dostepu_zeskanuj_qr_albo_wklej") }
                                    .to_owned()
                            }
                            Some(true) => {
                                { tr("wysalismy_kod_potwierdzajacy_zeskanuj_qr_albo_wklej_kod") }
                                    .to_owned()
                            }
                            Some(false) => {
                                let minutes = result
                                    .retry_after_seconds
                                    .map(|seconds| seconds.saturating_add(59) / 60)
                                    .unwrap_or(15)
                                    .max(1);
                                i18n::format(
                                    "nowa_wiadomosc_nie_zostaa_wysana_bo_poprzedni_kod",
                                    &[minutes.to_string()],
                                )
                            }
                            None => {
                                { tr("zgoszenie_zostao_przyjete_sprawdz_skrzynke_i_spam_jesli") }
                                    .to_owned()
                            }
                        };
                        error.set(Some(message));
                    }
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let confirm = move |_| {
        submit_fan_confirmation(email, name, token, pin, busy, status, error);
    };

    let request_access = move |_| {
        if busy.get_untracked() {
            return;
        }
        let current_email = email.get_untracked().trim().to_owned();
        if current_email.is_empty() {
            error.set(Some(tr("podaj_e_mail_uzyty_w_virya_signal").to_owned()));
            return;
        }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke_unit(
                "fan_request_access",
                &FanAccessArgs {
                    api_base_url: API_BASE,
                    email: &current_email,
                    locale: i18n::current().code(),
                },
            )
            .await
            {
                Ok(()) => {
                    access_mode.set(FanAccessMode::Confirm);
                    error.set(Some(
                        tr("jesli_ten_e_mail_jest_zapisany_w_virya").to_owned(),
                    ));
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let scan_confirmation = move |_| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        spawn_local(async move {
            match bridge::scan_qr().await {
                Ok(Some(value)) => {
                    token.set(value);
                    busy.set(false);
                    if !email.get_untracked().trim().is_empty()
                        && pin.get_untracked().chars().count() >= 4
                    {
                        submit_fan_confirmation(email, name, token, pin, busy, status, error);
                    } else {
                        error.set(Some(
                            tr("qr_zeskanowany_uzupenij_e_mail_i_lokalny_pin").to_owned(),
                        ));
                    }
                    return;
                }
                Ok(None) => {}
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    view! {
        <section class="fan-access">
            <header class="fan-access-hero">
                <p class="eyebrow">{tr("virya_signal")}</p>
                <h1>{tr("koncerty_bilety")}<br/><em>{tr("i_nagrody")}</em></h1>
                <p class="hero-subtitle">{tr("doacz_w_3_krokach")}</p>
                <ol class="signal-steps" aria-label=tr("jak_doaczyc")>
                    <li><span class="step-num">"1"</span>{tr("podaj_e_mail_i_miasto")}</li>
                    <li><span class="step-num">"2"</span>{tr("potwierdz_kod_z_wiadomosci")}</li>
                    <li><span class="step-num">"3"</span>{tr("odkrywaj_koncerty_blisko_ciebie")}</li>
                </ol>
                <div class="signal-purpose-grid" aria-label=tr("co_daje_virya_signal")>
                    <span><b aria-hidden="true">"⌁"</b>{tr("koncerty_blisko_ciebie")}</span>
                    <span><b aria-hidden="true">"▣"</b>{tr("bilety_i_qr_w_telefonie")}</span>
                    <span><b aria-hidden="true">"✦"</b>{tr("nagrody_za_proste_akcje")}</span>
                </div>
            </header>
            <Show when=move || status.get().configured fallback=move || view! {
                <div class="access-card fan-card">
                    <div class="segmented">
                        <button class:active=move || access_mode.get() == FanAccessMode::Signup on:click=move |_| access_mode.set(FanAccessMode::Signup)>{tr("zaczynam")}</button>
                        <button class:active=move || access_mode.get() == FanAccessMode::Confirm on:click=move |_| access_mode.set(FanAccessMode::Confirm)>{tr("mam_kod")}</button>
                    </div>
                    <div class="form-grid fan-form">
                        <label>{tr("e_mail")}<input type="email" autocomplete="email" prop:value=move || email.get() on:input=move |e| email.set(event_target_value(&e))/></label>
                        <label>{tr("imie_nazwa_opcjonalnie")}<input prop:value=move || name.get() on:input=move |e| name.set(event_target_value(&e))/></label>
                        <Show when=move || access_mode.get() == FanAccessMode::Signup fallback=move || view! {
                            <>
                                <p class="confirm-hint"><strong>{tr("najszybciej_zeskanuj_qr_z_maila")}</strong><br/>{tr("mozesz_tez_wkleic_cay_link_albo_64_znakowy")}</p>
                                <label>{tr("link_lub_kod_z_e_maila")}<textarea rows="3" autocomplete="one-time-code" spellcheck="false" autocapitalize="none" placeholder=tr("wklej_link_kod_albo_uzyj_qr") prop:value=move || token.get() on:input=move |e| token.set(event_target_value(&e))></textarea></label>
                                <div class="confirmation-actions single">
                                    <button type="button" class="confirmation-action primary-scan" disabled=move || busy.get() on:click=scan_confirmation><span aria-hidden="true">"▦"</span><strong>{tr("skanuj_qr")}</strong><small>{tr("albo_przytrzymaj_pole_wyzej_i_wybierz_wklej")}</small></button>
                                </div>
                                <label>{tr("lokalny_pin")}<input type="password" autocomplete="new-password" inputmode="numeric" prop:value=move || pin.get() on:input=move |e| pin.set(event_target_value(&e))/></label>
                                <p class="confirmation-note">{tr("pin_szyfruje_profil_tylko_na_tym_urzadzeniu_nie")}</p>
                                <button class="primary" disabled=move || busy.get() || email.get().trim().is_empty() || token.get().trim().is_empty() || pin.get().chars().count() < 4 on:click=confirm>{tr("potwierdz_i_wejdz")}</button>
                                <button type="button" class="text-button" disabled=move || busy.get() || email.get().trim().is_empty() on:click=request_access>{tr("mam_juz_konto_wyslij_link_logowania")}</button>
                                <p class="confirmation-resend">{tr("nie_ma_wiadomosci_sprawdz_spam_po_15_minutach")}</p>
                            </>
                        }>
                            <>
                                <div class="custom-city-fields city-stable-entry">
                                    <label>{tr("miejscowosc")}<input placeholder=tr("np_bielawa") prop:value=move || custom_city_name.get() on:input=move |e| custom_city_name.set(event_target_value(&e))/></label>
                                    <label>{tr("wojewodztwo_region_opcjonalnie")}<input placeholder=tr("dolnoslaskie") prop:value=move || custom_region.get() on:input=move |e| custom_region.set(event_target_value(&e))/></label>
                                    <p class="inline-note">{tr("wpisz_miejscowosc_recznie_dopasujemy_ja_do_mapy_sygnau")}</p>
                                </div>
                                <div class="nearby-pref">
                                    <label class="check-label"><input type="checkbox" prop:checked=move || nearby_enabled.get() on:change=move |e| nearby_enabled.set(event_target_checked(&e))/><span>{tr("powiadamiaj_mnie_o_koncertach_w_poblizu")}</span></label>
                                    <Show when=move || nearby_enabled.get()>
                                        <div class="radius-picker">
                                            <button type="button" class:active=move || radius_km.get()==50 on:click=move |_| radius_km.set(50)>"50 km"</button>
                                            <button type="button" class:active=move || radius_km.get()==100 on:click=move |_| radius_km.set(100)>"100 km"</button>
                                            <button type="button" class:active=move || radius_km.get()==150 on:click=move |_| radius_km.set(150)>"150 km"</button>
                                            <button type="button" class:active=move || radius_km.get()==250 on:click=move |_| radius_km.set(250)>"250 km"</button>
                                        </div>
                                    </Show>
                                </div>
                                <label>{tr("kod_polecajacy_opcjonalnie")}<input prop:value=move || referral.get() on:input=move |e| referral.set(event_target_value(&e))/></label>
                                <label>{tr("lokalny_pin")}<input type="password" autocomplete="new-password" prop:value=move || pin.get() on:input=move |e| pin.set(event_target_value(&e))/></label>
                                <label class="check-label"><input type="checkbox" prop:checked=move || consent.get() on:change=move |e| consent.set(event_target_checked(&e))/><span>{tr("chce_otrzymywac_informacje_o_koncertach_premierach_i_nagrodach")}</span></label>
                                <button class="primary" disabled=move || busy.get() on:click=signup>{tr("doacz_do_sygnau")}</button>
                            </>
                        </Show>
                    </div>
                </div>
            }>
                <div class="access-card fan-card">
                    <Show when=move || recovery_open.get() fallback=move || view! {
                        <>
                            <p class="lock-copy">{tr("twoj_profil_i_bilety_sa_zaszyfrowane_na_urzadzeniu")}</p>
                            <div class="form-grid">
                                <label>"PIN"<input type="password" autocomplete="current-password" prop:value=move || pin.get() on:input=move |e| pin.set(event_target_value(&e))/></label>
                                <button class="primary" disabled=move || busy.get() on:click=unlock>{tr("otworz_moj_sygna")}</button>
                                <button type="button" class="text-button recovery-link" on:click=move |_| recovery_open.set(true)>{tr("nie_pamietam_pin_u_zaloguj_ponownie")}</button>
                            </div>
                        </>
                    }>
                        <div class="form-grid recovery-panel">
                            <div class="recovery-heading"><p class="eyebrow">{tr("odzyskiwanie_dostepu")}</p><h3>{tr("ustaw_nowy_pin")}</h3><p>{tr("podaj_e_mail_wyslij_swiezy_link_a_potem")}</p></div>
                            <label>{tr("e_mail")}<input type="email" autocomplete="email" prop:value=move || email.get() on:input=move |e| email.set(event_target_value(&e))/></label>
                            <button type="button" class="ghost" disabled=move || busy.get() || email.get().trim().is_empty() on:click=request_access>{tr("wyslij_link_logowania")}</button>
                            <label>{tr("link_lub_kod_z_e_maila")}<textarea rows="3" autocomplete="one-time-code" spellcheck="false" autocapitalize="none" placeholder=tr("wklej_link_lub_kod") prop:value=move || token.get() on:input=move |e| token.set(event_target_value(&e))></textarea></label>
                            <div class="confirmation-actions single">
                                <button type="button" class="confirmation-action primary-scan" disabled=move || busy.get() on:click=scan_confirmation><span aria-hidden="true">"▦"</span><strong>{tr("skanuj_qr")}</strong><small>{tr("albo_przytrzymaj_pole_i_wybierz_wklej")}</small></button>
                            </div>
                            <label>{tr("nowy_lokalny_pin")}<input type="password" autocomplete="new-password" inputmode="numeric" prop:value=move || pin.get() on:input=move |e| pin.set(event_target_value(&e))/></label>
                            <button class="primary" disabled=move || busy.get() || email.get().trim().is_empty() || token.get().trim().is_empty() || pin.get().chars().count() < 4 on:click=confirm>{tr("potwierdz_i_ustaw_nowy_pin")}</button>
                            <button type="button" class="text-button" on:click=move |_| recovery_open.set(false)>{tr("wroc_do_logowania_pin_em")}</button>
                        </div>
                    </Show>
                </div>
            </Show>
        </section>
    }
}

#[component]
fn FanApp(
    mode: RwSignal<RootMode>,
    status: RwSignal<FanSessionStatus>,
    public: RwSignal<Option<PublicHomeData>>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let tab = RwSignal::new(FanTab::Signal);
    let dashboard = RwSignal::new(None::<FanDashboardData>);
    let merch = RwSignal::new(None::<MerchCatalog>);
    let merch_bundles = RwSignal::new(None::<FanMerchBundleCatalog>);
    let wallets = RwSignal::new(Vec::<TicketWallet>::new());
    let checkout_event = RwSignal::new(None::<PublicEvent>);
    let admission_qr = RwSignal::new(None::<AdmissionQr>);
    let area = RwSignal::new(None::<AreaWallet>);
    let loading = RwSignal::new(FanLoadingState::all());

    let loaded = RwSignal::new(FanLoadedState::default());
    let menu_open = RwSignal::new(false);

    Effect::new(move |_| {
        if !status.get().unlocked {
            return;
        }
        if dashboard.get_untracked().is_none() {
            dashboard.set(Some(FanDashboardData::default()));
        }

        match tab.get() {
            FanTab::Signal => {
                if !loaded.get_untracked().referral {
                    loaded.update(|state| state.referral = true);
                    refresh_fan_referral(dashboard, loading, error);
                }
            }
            FanTab::Events => {
                let state = loaded.get_untracked();
                if !state.events {
                    loaded.update(|value| value.events = true);
                    refresh_fan_events(dashboard, loading, error);
                }
                if !state.interests {
                    loaded.update(|value| value.interests = true);
                    refresh_fan_interests(dashboard, loading, error);
                }
            }
            FanTab::Merch => {
                if !loaded.get_untracked().merch {
                    loaded.update(|state| state.merch = true);
                    refresh_fan_merch(merch, loading, error);
                    refresh_fan_merch_bundles(merch_bundles);
                }
            }
            FanTab::Game => {
                if !loaded.get_untracked().area {
                    loaded.update(|state| state.area = true);
                    refresh_fan_area(area, loading, error);
                }
            }
            FanTab::Wallet => {
                let state = loaded.get_untracked();
                if !state.admission_pass {
                    loaded.update(|value| value.admission_pass = true);
                    refresh_fan_admission_pass(dashboard, loading, error);
                }
                if !state.wallets {
                    loaded.update(|value| value.wallets = true);
                    refresh_wallets(wallets, Some(loading), error);
                }
            }
            FanTab::Profile => {
                let state = loaded.get_untracked();
                if !state.referral {
                    loaded.update(|value| value.referral = true);
                    refresh_fan_referral(dashboard, loading, error);
                }
                if !state.events {
                    loaded.update(|value| value.events = true);
                    refresh_fan_events(dashboard, loading, error);
                }
                if !state.interests {
                    loaded.update(|value| value.interests = true);
                    refresh_fan_interests(dashboard, loading, error);
                }
                if !state.admission_pass {
                    loaded.update(|value| value.admission_pass = true);
                    refresh_fan_admission_pass(dashboard, loading, error);
                }
                if !state.wallets {
                    loaded.update(|value| value.wallets = true);
                    refresh_wallets(wallets, Some(loading), error);
                }
                if !state.area {
                    loaded.update(|value| value.area = true);
                    refresh_fan_area(area, loading, error);
                }
            }
        }
    });
    Effect::new(move |_| {
        if tab.get() != FanTab::Events && checkout_event.get_untracked().is_some() {
            checkout_event.set(None);
        }
    });
    on_cleanup(move || bridge::invalidate_latest("fan:"));

    let close = move |_| {
        bridge::invalidate_latest("fan:");
        spawn_local(async move {
            match bridge::invoke::<FanSessionStatus, _>("fan_lock", &EmptyArgs {}).await {
                Ok(value) => {
                    dashboard.set(None);
                    merch.set(None);
                    merch_bundles.set(None);
                    wallets.set(Vec::new());
                    checkout_event.set(None);
                    admission_qr.set(None);
                    area.set(None);
                    loading.set(FanLoadingState::all());
                    status.set(value);
                    mode.set(RootMode::Fan);
                }
                Err(message) => error.set(Some(message)),
            }
        });
    };

    view! {
        <section class="authenticated fan-authenticated">
            <header class="topbar fan-topbar">
                <div on:dblclick=move |_| { loaded.set(FanLoadedState::default()); refresh_fan_parts(dashboard, loading, error); refresh_fan_merch(merch, loading, error); refresh_fan_merch_bundles(merch_bundles); refresh_wallets(wallets, Some(loading), error); refresh_fan_area(area, loading, error); } style="cursor:pointer"><p class="eyebrow">{tr("virya_signal")}</p><strong>{move || status.get().session.and_then(|s| s.display_name).value_or_else(|| tr("moj_sygna").to_owned())}</strong></div>
                <div class="topbar-actions"><span class="live-dot"></span><button class="menu-trigger" aria-label=tr("otworz_menu") aria-expanded=move || menu_open.get() on:click=move |_| menu_open.update(|value| *value = !*value)><i></i><i></i><i></i></button><button aria-label=tr("zamknij_i_zablokuj_sygna") on:click=close>"×"</button></div>
            </header>
            <Show when=move || menu_open.get()>
                <div class="overflow-backdrop" on:click=move |_| menu_open.set(false)></div>
                <nav class="overflow-menu">
                    <button class:active=move || tab.get() == FanTab::Game on:click=move |_| { tab.set(FanTab::Game); menu_open.set(false); }><span>"◇"</span>{tr("gra_area")}</button>
                    <button class:active=move || tab.get() == FanTab::Profile on:click=move |_| { tab.set(FanTab::Profile); menu_open.set(false); }><span>"◎"</span>{tr("profil")}</button>
                    <button on:click=move |_| { menu_open.set(false); mode.set(RootMode::StaffGate); }><span>"⌁"</span>{tr("strefa_staff")}</button>
                </nav>
            </Show>
            <div class="content">{move || match tab.get() {
                FanTab::Signal => view! { <FanSignal dashboard=dashboard loading=loading error=error /> }.into_any(),
                FanTab::Events => checkout_event.get().map(|event| view! {
                    <FanTicketCheckout
                        event=event
                        status=status
                        tab=tab
                        checkout_event=checkout_event
                        wallets=wallets
                        loading=loading
                        error=error
                    />
                }.into_any()).value_or_else(|| view! {
                    <FanEvents dashboard=dashboard public=public checkout_event=checkout_event loading=loading error=error />
                }.into_any()),
                FanTab::Merch => view! { <FanMerch merch=merch bundles=merch_bundles loading=loading error=error /> }.into_any(),
                FanTab::Game => view! { <FanGame area=area loading=loading error=error /> }.into_any(),
                FanTab::Wallet => view! { <FanWallet dashboard=dashboard wallets=wallets admission_qr=admission_qr loading=loading error=error /> }.into_any(),
                FanTab::Profile => view! { <FanProfileScreen status=status dashboard=dashboard wallets=wallets area=area loading=loading error=error /> }.into_any(),
            }}</div>
            <nav class="bottom-nav four primary-four"><FanNavButton tab=tab own=FanTab::Signal icon="signal" label=tr("sygna_2")/><FanNavButton tab=tab own=FanTab::Events icon="events" label=tr("koncerty")/><FanNavButton tab=tab own=FanTab::Merch icon="shop" label=tr("sklep")/><FanNavButton tab=tab own=FanTab::Wallet icon="ticket" label=tr("bilety")/></nav>
        </section>
    }
}

#[component]
fn FanNavButton(
    tab: RwSignal<FanTab>,
    own: FanTab,
    icon: &'static str,
    label: &'static str,
) -> impl IntoView {
    view! { <button class:active=move || tab.get() == own on:click=move |_| tab.set(own)><NavGlyph icon=icon/><small>{label}</small></button> }
}

#[component]
fn FanSignal(
    dashboard: RwSignal<Option<FanDashboardData>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <section class="screen fan-screen">
            <header class="signal-dashboard-hero">
                <p class="eyebrow">{tr("twoj_wpyw")}</p>
                <h2>{move || dashboard.with(|state| state.as_ref().map(|d| d.referral.qualified_referrals.to_string())).value_or_else(|| "—".to_owned())}</h2>
                <strong>{tr("potwierdzonych_polecen")}</strong>
                <p>{move || dashboard.with(|state| state.as_ref().map(|d| i18n::format("kod_value", std::slice::from_ref(&d.referral.referral_code)))).value_or_else(|| tr("adowanie_sygnau").to_owned())}</p>
            </header>
            <Show when=move || !loading.get().referral fallback=move || view! { <Skeleton /> }>
            {move || dashboard.with(|state| state.as_ref().map(|data| data.referral.clone())).map(|referral| {
                let entries_total = referral.draw_entries.iter().map(|draw| draw.total_entries).sum::<u32>();
                let draw_count = referral.draw_entries.len();
                let coupon_count = referral.coupons.len();
                let draws = referral.draw_entries;
                let coupons = referral.coupons;
                let rewards = referral.physical_rewards;
                let coupons_view = (!coupons.is_empty()).then(|| view! {
                    <div class="section-head"><h3>{tr("twoje_kupony")}</h3></div>
                    <div class="card-list">{coupons.into_iter().map(|coupon| view! {
                        <article class="fan-coupon"><div><span>{format!("-{}%", coupon.discount_percent)}</span><strong>{coupon.code}</strong></div><small>{coupon.status}</small></article>
                    }).collect_view()}</div>
                });
                let rewards_view = (!rewards.is_empty()).then(|| view! {
                    <div class="section-head"><h3>{tr("nagrody")}</h3></div>
                    <div class="card-list">{rewards.into_iter().map(|reward| view! {
                        <article class="reward-card"><div><strong>{reward.item_name}</strong><p>{reward.sku}</p></div><span>{reward.status}</span></article>
                    }).collect_view()}</div>
                });
                view! {
                    <div class="stats-grid"><Metric value=referral.pending_referrals.to_string() label=tr("oczekujace")/><Metric value=entries_total.to_string() label=tr("losy")/><Metric value=coupon_count.to_string() label=tr("kupony")/></div>
                    <div class="section-head"><h3>{tr("aktywne_losowania")}</h3><span>{draw_count}</span></div>
                    <div class="card-list">{draws.into_iter().map(|draw| {
                        let proof_url = (!draw.slug.is_empty()).then(|| format!(
                            "https://virya.music/pl/dowody/losowania/{}/?source=signal-app",
                            draw.slug,
                        ));
                        view! {
                            <article class="draw-card">
                                <div><p class="eyebrow">{draw.prize_kind}</p><strong>{draw.name}</strong><span>{i18n::format("losowanie_value", &[human_time(&draw.draw_at).to_string()])}</span></div>
                                <div class="draw-actions">
                                    <div class="entry-count"><b>{draw.total_entries}</b><small>{tr("losow")}</small></div>
                                    {proof_url.map(|url| view! { <ExternalLink url=url label=tr("dowod") error=error /> })}
                                </div>
                            </article>
                        }
                    }).collect_view()}</div>
                    {coupons_view}
                    {rewards_view}
                }.into_any()
            }).value_or_else(|| view! { <Skeleton /> }.into_any())}
            </Show>
        </section>
    }
}

#[component]
fn FanMerch(
    merch: RwSignal<Option<MerchCatalog>>,
    bundles: RwSignal<Option<FanMerchBundleCatalog>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <section class="screen fan-screen merch-screen">
            <header class="screen-title">
                <p class="eyebrow">{tr("virya_store")}</p>
                <h2>{tr("merch")}</h2>
                <p>{tr("produkty_i_zestawy_korzystaja_z_tego_samego_stanu")}</p>
            </header>
            <Show when=move || !loading.get().merch fallback=move || view! { <Skeleton rows=4 /> }>
                {move || merch.get().map(|catalog| {
                    let products = catalog.products.into_iter()
                        .filter(|product| product.active && product.public)
                        .collect::<Vec<_>>();
                    if products.is_empty() {
                        view! {
                            <div class="empty-state">
                                <strong>{tr("sklep_jest_chwilowo_niedostepny")}</strong>
                                <p>{tr("pozostae_czesci_sygnau_dziaaja_normalnie_sprobuj_ponownie_za")}</p>
                                <button class="ghost" on:click=move |_| {
                                    refresh_fan_merch(merch, loading, error);
                                    refresh_fan_merch_bundles(bundles);
                                }>{tr("odswiez_merch")}</button>
                            </div>
                        }.into_any()
                    } else {
                        let bundle_catalog = bundles.get();
                        view! {
                            <div class="fan-merch-list">
                                <div class="merch-grid-action">
                                    <ExternalLink url="https://virya.music/pl/merch/?source=signal-app".to_owned() label=tr("otworz_peny_sklep") error=error />
                                </div>
                                <div class="merch-grid-heading">
                                    <div><p class="eyebrow">{tr("zestawy")}</p><h3>{tr("bundle_ze_sklepu_online")}</h3></div>
                                    <span>{tr("do_30")}</span>
                                </div>
                                {bundle_catalog.map(|catalog| {
                                    if catalog.bundles.is_empty() {
                                        view! {
                                            <div class="merch-grid-message">
                                                <p>{tr("zestawy_sa_teraz_niedostepne_w_live_inventory")}</p>
                                                <ExternalLink url="https://virya.music/pl/merch/?source=signal-app&product=bundle-stage-pack".to_owned() label=tr("zobacz_zestawy") error=error />
                                            </div>
                                        }.into_any()
                                    } else {
                                        catalog.bundles.into_iter().map(|bundle| {
                                            let availability_label = match bundle.availability.as_str() {
                                                "low_stock" => {tr("ostatnie_sztuki")},
                                                "available" => {tr("dostepny")},
                                                _ => {tr("brak_na_stanie")},
                                            };
                                            let available = bundle.available;
                                            let product_url = bundle.product_url.clone();
                                            let bundle_name = bundle.name;
                                            let image_alt = i18n::format(
                                                "value_zestaw_merchu_virya",
                                                std::slice::from_ref(&bundle_name),
                                            );
                                            let original_price = (bundle.original_price_gross_minor > bundle.price_gross_minor)
                                                .then(|| money(bundle.original_price_gross_minor, &bundle.currency));
                                            let includes = bundle.includes;
                                            let includes_view = (!includes.is_empty()).then(|| view! {
                                                <ul class="fan-merch-includes">
                                                    {includes.into_iter().map(|item| view! { <li>{item}</li> }).collect_view()}
                                                </ul>
                                            });
                                            let variants = bundle.variants;
                                            let variants_view = (!variants.is_empty()).then(|| view! {
                                                <div class="fan-merch-variants">
                                                    {variants.into_iter().map(|variant| view! {
                                                        <span class:available=variant.available>{variant.label}</span>
                                                    }).collect_view()}
                                                </div>
                                            });
                                            view! {
                                                <article class="fan-merch-card fan-merch-bundle">
                                                    <div class="bundle-badge">"BUNDLE"</div>
                                                    {bundle.image_url.map(|url| view! {
                                                        <img src=url alt=image_alt width="720" height="720" loading="lazy" decoding="async" referrerpolicy="no-referrer" />
                                                    })}
                                                    <div class="fan-merch-body">
                                                        <div class="fan-merch-heading">
                                                            <div>
                                                                <h3>{bundle_name}</h3>
                                                                <div class="fan-merch-price">
                                                                    <strong>{money(bundle.price_gross_minor, &bundle.currency)}</strong>
                                                                    {original_price.map(|price| view! { <del>{price}</del> })}
                                                                </div>
                                                            </div>
                                                            <span class:available=available>{availability_label}</span>
                                                        </div>
                                                        {bundle.description.map(|description| view! { <p>{description}</p> })}
                                                        {includes_view}
                                                        {variants_view}
                                                        <Show when=move || available fallback=move || view! {
                                                            <button class="ghost" on:click=move |_| refresh_fan_merch_bundles(bundles)>{tr("sprawdz_ponownie")}</button>
                                                        }>
                                                            <ExternalLink url=product_url.clone() label=tr("kup_w_sklepie") error=error />
                                                        </Show>
                                                    </div>
                                                </article>
                                            }
                                        }).collect_view().into_any()
                                    }
                                }).value_or_else(|| view! {
                                    <div class="merch-grid-message">
                                        <p>{tr("zestawy_doczytuja_sie_niezaleznie_od_produktow")}</p>
                                        <ExternalLink url="https://virya.music/pl/merch/?source=signal-app&product=bundle-stage-pack".to_owned() label=tr("zobacz_zestawy") error=error />
                                    </div>
                                }.into_any())}
                                <div class="merch-grid-heading merch-products-heading">
                                    <div><p class="eyebrow">{tr("pojedyncze_produkty")}</p><h3>{tr("wybierz_swoj_merch")}</h3></div>
                                </div>
                                {products.into_iter().map(|product| {
                                    let available_variants = product.variants.iter()
                                        .filter(|variant| variant.active && variant.available)
                                        .cloned()
                                        .collect::<Vec<_>>();
                                    let has_stock = !available_variants.is_empty();
                                    let preorder = available_variants.iter()
                                        .any(|variant| variant.availability == "preorder");
                                    let low_stock = available_variants.iter()
                                        .any(|variant| variant.availability == "low_stock");
                                    let availability_label = if preorder {
                                        tr("przedsprzedaz")
                                    } else if low_stock {
                                        tr("ostatnie_sztuki")
                                    } else if has_stock {
                                        tr("dostepny")
                                    } else {
                                        tr("brak_na_stanie")
                                    };
                                    let shop_url = format!(
                                        "https://virya.music/pl/merch/?source=signal-app&product={}",
                                        product.slug,
                                    );
                                    let product_name = product.name;
                                    let image_alt = i18n::format(
                                        "value_merch_virya",
                                        std::slice::from_ref(&product_name),
                                    );
                                    let variants = product.variants.into_iter()
                                        .filter(|variant| variant.active)
                                        .collect::<Vec<_>>();
                                    let variants_view = (!variants.is_empty()).then(|| view! {
                                        <div class="fan-merch-variants">
                                            {variants.into_iter().map(|variant| view! {
                                                <span class:available=variant.available>{variant.label}</span>
                                            }).collect_view()}
                                        </div>
                                    });
                                    view! {
                                        <article class="fan-merch-card">
                                            {product.image_url.map(|url| view! {
                                                <img src=url alt=image_alt width="720" height="720" loading="lazy" decoding="async" referrerpolicy="no-referrer" />
                                            })}
                                            <div class="fan-merch-body">
                                                <div class="fan-merch-heading">
                                                    <div><h3>{product_name}</h3><strong>{money(product.price_gross_minor, &product.currency)}</strong></div>
                                                    <span class:available=has_stock>{availability_label}</span>
                                                </div>
                                                {product.description.map(|description| view! { <p>{description}</p> })}
                                                {variants_view}
                                                <Show when=move || has_stock fallback=move || view! {
                                                    <button class="ghost" on:click=move |_| refresh_fan_merch(merch, loading, error)>{tr("sprawdz_ponownie")}</button>
                                                }>
                                                    <ExternalLink url=shop_url.clone() label=tr("kup_w_sklepie") error=error />
                                                </Show>
                                            </div>
                                        </article>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }
                }).value_or_else(|| view! {
                    <div class="empty-state">
                        <strong>{tr("nie_udao_sie_pobrac_stanu_sklepu")}</strong>
                        <p>{tr("koncerty_bilety_i_profil_pozostaja_dostepne")}</p>
                        <button class="ghost" on:click=move |_| {
                            refresh_fan_merch(merch, loading, error);
                            refresh_fan_merch_bundles(bundles);
                        }>{tr("sprobuj_ponownie")}</button>
                    </div>
                }.into_any())}
            </Show>
        </section>
    }
}

#[component]
fn FanEvents(
    dashboard: RwSignal<Option<FanDashboardData>>,
    public: RwSignal<Option<PublicHomeData>>,
    checkout_event: RwSignal<Option<PublicEvent>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <section class="screen"><header class="screen-title"><p class="eyebrow">{tr("gdzie_gramy")}</p><h2>{tr("koncerty")}</h2></header><Show when=move || !loading.get().events fallback=move || view! { <Skeleton /> }>{move || { let events = fan_events(dashboard, public); if events.is_empty() { view! { <div class="empty-state"><strong>{tr("brak_koncertow_w_kalendarzu")}</strong><p>{tr("kiedy_pojawi_sie_nowy_event_bedzie_tutaj")}</p></div> }.into_any() } else { view! { <div class="card-list fan-event-list">{events.into_iter().map(|event| view! { <FanEventCard event=event checkout_event=checkout_event dashboard=dashboard loading=loading error=error /> }).collect_view()}</div> }.into_any() }}}</Show></section>
    }
}

#[component]
fn FanEventCard(
    event: PublicEvent,
    checkout_event: RwSignal<Option<PublicEvent>>,
    dashboard: RwSignal<Option<FanDashboardData>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let checkout = event.clone();
    let buy = move |_| checkout_event.set(Some(checkout.clone()));
    let event_slug = event.slug.clone();
    let interested = Signal::derive(move || {
        dashboard.with(|state| {
            state.as_ref().is_some_and(|data| {
                data.interests
                    .iter()
                    .any(|item| item.event.slug == event_slug)
            })
        })
    });
    let interest_slug = event.slug.clone();
    let busy = RwSignal::new(false);
    let interest = move |_| {
        if interested.get_untracked() || busy.get_untracked() {
            return;
        }
        let event_slug = interest_slug.clone();
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke_unit(
                "fan_register_interest",
                &EventArgs {
                    event_slug: &event_slug,
                },
            )
            .await
            {
                Ok(_) => {
                    error.set(Some(tr("koncert_zapisany_w_twoim_sygnale").to_owned()));
                    refresh_fan_interests(dashboard, loading, error);
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };
    let event_day = day(&event.starts_at);
    let event_month = month(&event.starts_at);
    let event_time = human_time(&event.starts_at);
    let location = event_location(&event);
    let image = event.image_thumbnail_url.or(event.image_url);
    let description = event.description;
    let title = event.title;
    let image_alt = i18n::format("value_koncert_virya", std::slice::from_ref(&title));
    view! {
        <article class="fan-event-card">
            {image.map(|url| view! {
                <img
                    src=url
                    alt=image_alt
                    width="720"
                    height="405"
                    loading="lazy"
                    decoding="async"
                    fetchpriority="low"
                    referrerpolicy="no-referrer"
                />
            })}
            <div class="fan-event-body">
                <div class="date-line"><span>{format!("{event_day} {event_month}")}</span><small>{event_time}</small></div>
                <h3>{title}</h3><p>{location}</p>
                {description.map(|text| view! { <p class="event-description">{text}</p> })}
                <div class="event-actions">
                    <button class:active=move || interested.get() on:click=interest disabled=move || busy.get() || interested.get()>{move || if busy.get() { tr("zapisuje") } else if interested.get() { tr("mam_to") } else { tr("interesuje_mnie") }}</button>
                    <button class="ticket-buy-button" on:click=buy>{tr("kup_bilet")}</button>
                </div>
            </div>
        </article>
    }
}

#[component]
fn FanTicketCheckout(
    event: PublicEvent,
    status: RwSignal<FanSessionStatus>,
    tab: RwSignal<FanTab>,
    checkout_event: RwSignal<Option<PublicEvent>>,
    wallets: RwSignal<Vec<TicketWallet>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let sale = RwSignal::new(None::<TicketSaleOffer>);
    let sale_loading = RwSignal::new(true);
    let sale_failed = RwSignal::new(false);
    let sale_refresh = RwSignal::new(0_u32);
    let load_slug = event.slug.clone();

    Effect::new(move |_| {
        sale_refresh.get();
        sale_loading.set(true);
        let event_slug = load_slug.clone();
        spawn_local(async move {
            match bridge::invoke_latest::<Option<TicketSaleOffer>, _>(
                "fan_ticket_sale",
                &EventArgs {
                    event_slug: &event_slug,
                },
                15_000,
                "fan:ticket-sale",
            )
            .await
            {
                Ok(Some(Some(value))) => {
                    sale.set(Some(value));
                    sale_failed.set(false);
                }
                Ok(Some(None)) => {
                    sale.set(None);
                    sale_failed.set(false);
                }
                Ok(None) => return,
                Err(message) => {
                    sale.set(None);
                    sale_failed.set(true);
                    error.set(Some(message));
                }
            }
            sale_loading.set(false);
        });
    });
    on_cleanup(move || bridge::invalidate_latest("fan:ticket-sale"));

    let back = move |_| checkout_event.set(None);
    let event_title = event.title.clone();
    let event_meta = event_time_location(&event.starts_at, event.venue.as_deref());
    let event_slug = event.slug.clone();
    let fallback_url = event
        .ticket_url
        .clone()
        .value_or_else(|| format!("https://virya.music/pl/live/{}/#tickets", event.slug));
    let full_form_url = format!("https://virya.music/pl/live/{}/#tickets", event.slug);

    view! {
        <section class="screen fan-ticket-checkout-screen">
            <button class="checkout-back" on:click=back>{tr("wroc_do_koncertow")}</button>
            <header class="ticket-checkout-hero">
                <p class="eyebrow">{tr("virya_bilety")}</p>
                <h2>{event_title}</h2>
                <p>{event_meta}</p>
            </header>
            {
                let render_sale = move || {
                    let event_slug = event_slug.clone();
                    let fallback_url = fallback_url.clone();
                    let full_form_url = full_form_url.clone();
                    match sale.get() {
                        Some(offer) => view! {
                            <FanTicketSale
                                offer=offer
                                event_slug=event_slug
                                fallback_url=fallback_url
                                full_form_url=full_form_url
                                status=status
                                tab=tab
                                checkout_event=checkout_event
                                wallets=wallets
                                loading=loading
                                sale_refresh=sale_refresh
                                error=error
                            />
                        }
                        .into_any(),
                        None => view! {
                            <div class="empty-state">
                                <strong>{if sale_failed.get() { tr("nie_udao_sie_sprawdzic_sprzedazy") } else { tr("brak_wasnej_puli_virya") }}</strong>
                                <p>{tr("mozesz_przejsc_do_strony_koncertu_lub_sprzedazy_prowadzonej")}</p>
                                <ExternalLink url=fallback_url label=tr("sprawdz_bilety") error=error />
                            </div>
                        }
                        .into_any(),
                    }
                };
                view! {
                    <Show when=move || !sale_loading.get() fallback=move || view! { <Skeleton rows=4 /> }>
                        {render_sale.clone()}
                    </Show>
                }
            }
        </section>
    }
}

#[component]
fn FanTicketSale(
    offer: TicketSaleOffer,
    event_slug: String,
    fallback_url: String,
    full_form_url: String,
    status: RwSignal<FanSessionStatus>,
    tab: RwSignal<FanTab>,
    checkout_event: RwSignal<Option<PublicEvent>>,
    wallets: RwSignal<Vec<TicketWallet>>,
    loading: RwSignal<FanLoadingState>,
    sale_refresh: RwSignal<u32>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let ticket_types = offer
        .ticket_types
        .iter()
        .filter(|ticket_type| ticket_type.active)
        .cloned()
        .collect::<Vec<_>>();
    let max_per_order = offer.max_per_order.max(0) as u32;
    let has_available_type = ticket_types
        .iter()
        .any(|ticket_type| ticket_type.available > 0);
    let is_open = offer.active
        && offer.sales_state == "open"
        && offer.available > 0
        && max_per_order > 0
        && has_available_type;
    let state_copy = match offer.sales_state.as_str() {
        "upcoming" => tr("sprzedaz_rozpocznie_sie_wkrotce"),
        "closed" => tr("sprzedaz_online_zostaa_zakonczona"),
        "sold_out" => tr("ta_pula_biletow_jest_wyprzedana"),
        "inactive" => tr("sprzedaz_jest_chwilowo_wyaczona"),
        "event_unavailable" => tr("ten_koncert_nie_jest_obecnie_dostepny_w_sprzedazy"),
        _ if !is_open => tr("bilety_nie_sa_teraz_dostepne"),
        _ => tr("wybierz_bilety_miejsca_zostana_zarezerwowane_na_czas_patnosci"),
    };
    let sale_available = offer.available.max(0);
    let sale_reserved = offer.reserved.max(0);
    let sale_sold = offer.sold.max(0);

    if !is_open {
        return view! {
            <div class="ticket-sale-summary">
                <div><strong>{sale_available}</strong><span>{tr("dostepne")}</span></div>
                <div><strong>{sale_reserved}</strong><span>{tr("w_patnosci")}</span></div>
                <div><strong>{sale_sold}</strong><span>{tr("sprzedane")}</span></div>
            </div>
            <p class="checkout-state-copy">{state_copy}</p>
            <div class="empty-state compact">
                <strong>{tr("sprawdz_strone_koncertu")}</strong>
                <p>{tr("jezeli_organizator_prowadzi_osobna_sprzedaz_znajdziesz_ja_pod")}</p>
                <ExternalLink url=fallback_url label=tr("sprawdz_bilety") error=error />
            </div>
        }
        .into_any();
    }

    let quantities = RwSignal::new(
        ticket_types
            .iter()
            .map(|ticket_type| TicketCheckoutItemInput {
                ticket_type_slug: ticket_type.slug.clone(),
                quantity: 0,
            })
            .collect::<Vec<_>>(),
    );
    let buyer_name = RwSignal::new(
        status
            .get_untracked()
            .session
            .and_then(|profile| profile.display_name)
            .unwrap_or_default(),
    );
    let busy = RwSignal::new(false);
    let pending_checkout = RwSignal::new(None::<TicketCheckoutStart>);
    let selected_count = Signal::derive(move || checkout_count(quantities));
    let gross_offer = offer.clone();
    let selected_gross = Signal::derive(move || checkout_gross(&gross_offer, quantities));
    let purchase_slug = event_slug.clone();

    let purchase = move |_| {
        if busy.get_untracked() || pending_checkout.get_untracked().is_some() {
            return;
        }
        let items = quantities
            .get_untracked()
            .into_iter()
            .filter(|item| item.quantity > 0)
            .collect::<Vec<_>>();
        if items.is_empty() {
            error.set(Some(tr("wybierz_co_najmniej_jeden_bilet").to_owned()));
            return;
        }
        let name = buyer_name.get_untracked().trim().to_owned();
        let input = TicketCheckoutInput {
            event_slug: purchase_slug.clone(),
            buyer_name: (!name.is_empty()).then_some(name),
            items,
        };
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke_timeout::<TicketCheckoutStart, _>(
                "fan_start_ticket_checkout",
                &TicketCheckoutArgs { input: &input },
                35_000,
            )
            .await
            {
                Ok(checkout) => {
                    pending_checkout.set(Some(checkout.clone()));
                    refresh_wallets(wallets, Some(loading), error);
                    let checkout_url = checkout.url.clone();
                    match bridge::invoke_unit("open_external_url", &UrlArgs { url: &checkout_url })
                        .await
                    {
                        Ok(_) => {
                            checkout_event.set(None);
                            tab.set(FanTab::Wallet);
                            error.set(Some(i18n::format(
                                "zamowienie_value_zapisane_dokoncz_bezpieczna_patnosc_stripe",
                                std::slice::from_ref(&checkout.order_reference),
                            )));
                        }
                        Err(message) => {
                            error.set(Some(i18n::format(
                                "message_zamowienie_value_jest_zapisane_uzyj_przycisku_ponownego",
                                &[message.to_string(), checkout.order_reference.to_string()],
                            )));
                        }
                    }
                }
                Err(message) => {
                    error.set(Some(message));
                    sale_refresh.update(|value| *value = value.wrapping_add(1));
                }
            }
            busy.set(false);
        });
    };

    let retry_payment = move |_| {
        let Some(checkout) = pending_checkout.get_untracked() else {
            return;
        };
        let checkout_url = checkout.url.clone();
        spawn_local(async move {
            match bridge::invoke_unit("open_external_url", &UrlArgs { url: &checkout_url }).await {
                Ok(_) => {
                    checkout_event.set(None);
                    tab.set(FanTab::Wallet);
                    error.set(Some(i18n::format(
                        "otworzono_patnosc_dla_zamowienia_value",
                        std::slice::from_ref(&checkout.order_reference),
                    )));
                }
                Err(message) => error.set(Some(message)),
            }
        });
    };

    let currency_for_total = offer.currency.clone();
    let purchase_disabled = Signal::derive(move || {
        busy.get() || selected_count.get() == 0 || pending_checkout.get().is_some()
    });
    view! {
        <div class="ticket-sale-summary">
            <div><strong>{sale_available}</strong><span>{tr("dostepne")}</span></div>
            <div><strong>{sale_reserved}</strong><span>{tr("w_patnosci")}</span></div>
            <div><strong>{sale_sold}</strong><span>{tr("sprzedane")}</span></div>
        </div>
        <p class="checkout-state-copy">{state_copy}</p>
        <div class="ticket-type-list">
            {ticket_types.into_iter().map(|ticket_type| {
                let quantity_slug = ticket_type.slug.clone();
                let decrement_slug = ticket_type.slug.clone();
                let increment_slug = ticket_type.slug.clone();
                let available = ticket_type.available.max(0) as u32;
                let currency = offer.currency.clone();
                let quantity = Signal::derive(move || checkout_quantity(quantities, &quantity_slug));
                let decrement_disabled = Signal::derive(move || quantity.get() == 0);
                let increment_disabled = Signal::derive(move || {
                    quantity.get() >= available || selected_count.get() >= max_per_order
                });
                let decrement = move |_| {
                    let current = checkout_quantity(quantities, &decrement_slug);
                    set_checkout_quantity(
                        quantities,
                        &decrement_slug,
                        current.saturating_sub(1),
                        available,
                        max_per_order,
                    );
                };
                let increment = move |_| {
                    let current = checkout_quantity(quantities, &increment_slug);
                    set_checkout_quantity(
                        quantities,
                        &increment_slug,
                        current.saturating_add(1),
                        available,
                        max_per_order,
                    );
                };
                view! {
                    <article class="ticket-type-card">
                        <div>
                            <h3>{ticket_type.name}</h3>
                            {ticket_type.description.map(|description| view! { <p>{description}</p> })}
                            <strong>{money(ticket_type.price_gross_minor, &currency)}</strong>
                            <small>{i18n::format("dostepne_value", &[ticket_type.available.max(0).to_string()])}</small>
                        </div>
                        <div class="ticket-stepper" aria-label=tr("liczba_biletow")>
                            <button type="button" aria-label=tr("zmniejsz_liczbe_biletow") on:click=decrement disabled=move || decrement_disabled.get()>"−"</button>
                            <output aria-live="polite">{move || quantity.get()}</output>
                            <button type="button" aria-label=tr("zwieksz_liczbe_biletow") on:click=increment disabled=move || increment_disabled.get()>"+"</button>
                        </div>
                    </article>
                }
            }).collect_view()}
        </div>
        <div class="ticket-buyer-panel">
            <label>{tr("imie_i_nazwisko_na_zamowieniu_opcjonalnie")}<input autocomplete="name" maxlength="160" prop:value=move || buyer_name.get() on:input=move |event| buyer_name.set(event_target_value(&event))/></label>
            <p>{move || status.get().session.map(|profile| i18n::format("bilety_i_potwierdzenie_trafia_na_value", std::slice::from_ref(&profile.email))).value_or_else(|| tr("bilety_trafia_na_e_mail_konta_fana").to_owned())}</p>
            <ExternalLink url=full_form_url label=tr("faktura_peny_formularz") error=error />
        </div>
        <footer class="ticket-checkout-total">
            <div><span>{tr("wybrane_bilety")}</span><strong>{move || selected_count.get()}</strong></div>
            <div><span>{tr("razem_brutto")}</span><strong>{move || money(selected_gross.get(), &currency_for_total)}</strong></div>
            <button type="button" class="primary" on:click=purchase disabled=move || purchase_disabled.get()>{move || if busy.get() { tr("rezerwuje") } else if pending_checkout.get().is_some() { tr("zamowienie_zapisane") } else { tr("przejdz_do_patnosci_stripe") }}</button>
            <Show when=move || pending_checkout.get().is_some()>
                <button type="button" class="ghost checkout-retry" on:click=retry_payment>{tr("otworz_patnosc_ponownie")}</button>
            </Show>
            <small>{tr("dane_karty_nie_trafiaja_do_virya_signal_patnosc")}</small>
        </footer>
    }
    .into_any()
}

fn checkout_quantity(
    quantities: RwSignal<Vec<TicketCheckoutItemInput>>,
    ticket_type_slug: &str,
) -> u32 {
    quantities.with(|items| {
        items
            .iter()
            .find(|item| item.ticket_type_slug == ticket_type_slug)
            .map(|item| item.quantity)
            .unwrap_or_default()
    })
}

fn checkout_count(quantities: RwSignal<Vec<TicketCheckoutItemInput>>) -> u32 {
    quantities.with(|items| items.iter().map(|item| item.quantity).sum())
}

fn checkout_gross(
    sale: &TicketSaleOffer,
    quantities: RwSignal<Vec<TicketCheckoutItemInput>>,
) -> i64 {
    quantities.with(|items| {
        items
            .iter()
            .filter_map(|item| {
                sale.ticket_types
                    .iter()
                    .find(|ticket_type| ticket_type.slug == item.ticket_type_slug)
                    .map(|ticket_type| {
                        ticket_type
                            .price_gross_minor
                            .saturating_mul(i64::from(item.quantity))
                    })
            })
            .fold(0_i64, i64::saturating_add)
    })
}

fn set_checkout_quantity(
    quantities: RwSignal<Vec<TicketCheckoutItemInput>>,
    ticket_type_slug: &str,
    requested: u32,
    available: u32,
    max_per_order: u32,
) {
    quantities.update(|items| {
        let other = items
            .iter()
            .filter(|item| item.ticket_type_slug != ticket_type_slug)
            .map(|item| item.quantity)
            .sum::<u32>();
        let allowed = available.min(max_per_order.saturating_sub(other));
        if let Some(item) = items
            .iter_mut()
            .find(|item| item.ticket_type_slug == ticket_type_slug)
        {
            item.quantity = requested.min(allowed);
        }
    });
}

#[component]
fn ExternalLink(
    url: String,
    label: &'static str,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let open_url = url.clone();
    let open = move |_| {
        let current = open_url.clone();
        spawn_local(async move {
            if let Err(message) =
                bridge::invoke_unit("open_external_url", &UrlArgs { url: &current }).await
            {
                error.set(Some(message));
            }
        });
    };
    view! { <button type="button" class="ticket-buy-button" on:click=open>{label}</button> }
}

fn open_area_game(error: RwSignal<Option<String>>) {
    spawn_local(async move {
        let url = "https://virya.music/pl/area/#area-map";
        if let Err(message) = bridge::invoke_unit("open_external_url", &UrlArgs { url }).await {
            error.set(Some(message));
        }
    });
}

#[component]
fn FanGame(
    area: RwSignal<Option<AreaWallet>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let refresh = move |_| refresh_fan_area(area, loading, error);
    view! {
        <section class="screen fan-screen area-screen">
            <header class="screen-title"><p class="eyebrow">{tr("virya_area")}</p><h2>{tr("znajdz_punkt_w_swoim_miescie")}</h2><p>{tr("otworz_mape_wybierz_aktywny_punkt_i_przejdz_do")}</p></header>
            <Show when=move || !loading.get().area fallback=move || view! { <Skeleton rows=3 /> }>
                {move || area.get().map(|wallet| {
                    let claimed = wallet.claims.len() as u32;
                    let total = wallet.collection_size.max(1);
                    let percent = (claimed.saturating_mul(100) / total).min(100);
                    let claims = wallet.claims;
                    let live_count = wallet.live_drops.len();
                    let voucher_count = wallet.vouchers.len();
                    let collection_size = wallet.collection_size;
                    let reward_credits = wallet.reward_credits;
                    let community_percent = wallet.community.percent.round();
                    let migration_notice = wallet.migration_required.then(|| view! {
                        <p class="security-note warning">{tr("poacz_portfel_przegladarkowy_z_kontem_na_stronie_area")}</p>
                    });
                    view! {
                        <div class="area-hero-card">
                            <div><p class="eyebrow">{tr("postep_kolekcji")}</p><h3>{format!("{claimed} / {collection_size}")}</h3></div>
                            <strong>{i18n::format("reward_credits_kredytow", &[reward_credits.to_string()])}</strong>
                            <div class="area-progress" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow=percent><span style=format!("width:{percent}%")></span></div>
                            <div class="area-stats"><span>{i18n::format("live_count_aktywne_punkty", &[live_count.to_string()])}</span><span>{i18n::format("voucher_count_nagrody", &[voucher_count.to_string()])}</span><span>{i18n::format("community_percent_spoecznosci", &[community_percent.to_string()])}</span></div>
                        </div>
                        {migration_notice}
                        <div class="section-head"><h3>{tr("odkryte_artefakty")}</h3><span>{claimed}</span></div>
                        <div class="artifact-list">
                            <For each=move || claims.clone() key=|claim| claim.drop_id.clone() children=move |claim| view! {
                                <article class="artifact-card"><span>{claim.number}</span><div><strong>{claim.track}</strong><p>{format!("{} · {}", claim.city, claim.edition)}</p><small>{claim.line}</small></div><em>{claim.edition_number.map(|number| format!("#{number}")).value_or_else(|| "✓".to_owned())}</em></article>
                            } />
                        </div>
                        <div class="area-actions"><button class="primary" on:click=move |_| open_area_game(error)>{tr("otworz_mape_i_zacznij")}</button><button class="ghost" on:click=refresh disabled=move || loading.get().area>{tr("odswiez_progres")}</button></div>
                        <p class="security-note">{tr("mapa_pokazuje_aktywne_punkty_i_prowadzi_do_startu")}</p>
                    }.into_any()
                }).value_or_else(|| view! { <div class="empty-state"><strong>{tr("area_chwilowo_niedostepna")}</strong><p>{tr("odswiez_dane_albo_otworz_pena_gre")}</p><button class="primary" on:click=move |_| open_area_game(error)>{tr("otworz_area")}</button></div> }.into_any())}
            </Show>
        </section>
    }
}

#[component]
fn FanWallet(
    dashboard: RwSignal<Option<FanDashboardData>>,
    wallets: RwSignal<Vec<TicketWallet>>,
    admission_qr: RwSignal<Option<AdmissionQr>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let order_id = RwSignal::new(String::new());
    let checkout_token = RwSignal::new(String::new());
    let claim_token = RwSignal::new(String::new());
    let busy = RwSignal::new(false);

    let import = move |_| {
        let order = order_id.get().trim().to_owned();
        let token = checkout_token.get().trim().to_owned();
        if order.is_empty() || token.is_empty() {
            error.set(Some(
                tr("podaj_identyfikator_zamowienia_i_prywatny_token").to_owned(),
            ));
            return;
        }
        // The recovery token must not remain rendered in the WebView while IPC runs.
        checkout_token.set(String::new());
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<TicketWallet, _>(
                "fan_import_wallet",
                &ImportWalletArgs {
                    order_id: &order,
                    checkout_token: &token,
                },
            )
            .await
            {
                Ok(_) => {
                    error.set(Some(tr("bilety_zapisane_w_portfelu").to_owned()));
                    refresh_wallets(wallets, Some(loading), error);
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let claim = move |_| {
        let token = claim_token.get().trim().to_owned();
        if token.is_empty() {
            error.set(Some(tr("wklej_token_wejsciowki").to_owned()));
            return;
        }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<AdmissionPass, _>(
                "fan_claim_pass",
                &ClaimArgs {
                    claim_token: &token,
                },
            )
            .await
            {
                Ok(_) => {
                    error.set(Some(tr("wejsciowka_przypisana_do_urzadzenia").to_owned()));
                    refresh_fan_admission_pass(dashboard, loading, error);
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let qr = move |_| {
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<AdmissionQr, _>("fan_admission_qr", &EmptyArgs {}).await {
                Ok(value) => admission_qr.set(Some(value)),
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    view! {
        <section class="screen"><header class="screen-title"><p class="eyebrow">{tr("mobile_wallet")}</p><h2>{tr("bilety_i_wejscie")}</h2></header><Show when=move || !loading.get().admission_pass fallback=move || view! { <Skeleton rows=1 /> }>{move || dashboard.with(|state| state.as_ref().and_then(|d| d.admission_pass.clone())).map(|pass| view! { <article class="admission-card"><p class="eyebrow">{tr("wejsciowka_virya")}</p><h3>{pass.event_title}</h3><p>{event_time_location(&pass.starts_at, pass.venue.as_deref())}</p><strong>{pass.public_reference}</strong><span>{pass.status}</span><button class="primary" on:click=qr disabled=move || busy.get()>{tr("pokaz_qr_na_wejscie")}</button>{move || admission_qr.get().map(|value| view! { <QrPanel svg=value.qr_svg token=value.token expires=value.expires_at /> })}</article> })}
        <Show when=move || dashboard.with(|state| state.as_ref().is_none_or(|d| d.admission_pass.is_none()))><div class="claim-box"><p class="eyebrow">{tr("wygraes_wejsciowke")}</p><h3>{tr("przypisz_ja_do_telefonu")}</h3><textarea rows="3" placeholder=tr("token_z_wiadomosci") prop:value=move || claim_token.get() on:input=move |e| claim_token.set(event_target_value(&e))></textarea><button class="primary" on:click=claim disabled=move || busy.get()>{tr("odbierz_wejsciowke")}</button></div></Show></Show>
        <div class="section-head"><h3>{tr("portfel_biletow")}</h3><span>{move || wallets.get().len()}</span></div><Show when=move || !loading.get().wallets fallback=move || view! { <Skeleton rows=2 /> }><div class="wallet-stack">{move || wallets.get().into_iter().map(|wallet| view! {
            <WalletCard wallet=wallet error=error />
        }).collect_view()}</div></Show><details class="import-box"><summary>{tr("dodaj_istniejace_zamowienie")}</summary><div class="form-grid"><label>"Order ID"<input placeholder=tr("uuid_zamowienia") prop:value=move || order_id.get() on:input=move |e| order_id.set(event_target_value(&e))/></label><label>{tr("prywatny_checkout_token")}<textarea rows="3" autocomplete="off" autocapitalize="none" spellcheck="false" prop:value=move || checkout_token.get() on:input=move |e| checkout_token.set(event_target_value(&e))></textarea></label><button class="primary" on:click=import disabled=move || busy.get()>{tr("dodaj_do_portfela")}</button></div></details></section>
    }
}

#[component]
fn WalletCard(wallet: TicketWallet, error: RwSignal<Option<String>>) -> impl IntoView {
    let order_id = wallet.order.order_id.clone();
    let delivery_order_id = order_id.clone();
    let busy = RwSignal::new(false);
    let resend = move |_| {
        if busy.get_untracked() {
            return;
        }
        let order = delivery_order_id.clone();
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke_unit("fan_request_delivery", &OrderArgs { order_id: &order }).await
            {
                Ok(_) => error.set(Some(tr("wysalismy_ponownie_portfel_na_e_mail").to_owned())),
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };
    view! {
        <article class="wallet-card"><header><div><p class="eyebrow">{wallet.order.status}</p><h3>{wallet.order.event_title}</h3><p>{event_time_location(&wallet.order.starts_at, wallet.order.venue.as_deref())}</p></div><strong>{wallet.order.public_reference}</strong></header><div class="ticket-stack">{wallet.tickets.into_iter().map(|ticket| view! { <WalletTicketCard order_id=order_id.clone() ticket=ticket error=error /> }).collect_view()}</div><button class="text-button" on:click=resend disabled=move || busy.get()>{move || if busy.get() { tr("wysyam") } else { tr("wyslij_bilety_ponownie_na_e_mail") }}</button></article>
    }
}

#[component]
fn WalletTicketCard(
    order_id: String,
    ticket: WalletTicket,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let public_reference = ticket.public_reference.clone();
    let qr_available = ticket.qr_available;
    let qr_svg = RwSignal::new(None::<String>);
    let qr_visible = RwSignal::new(false);
    let busy = RwSignal::new(false);
    let toggle_qr = move |_| {
        if busy.get_untracked() {
            return;
        }
        if qr_svg.get_untracked().is_some() {
            qr_visible.update(|visible| *visible = !*visible);
            return;
        }
        let order_id = order_id.clone();
        let public_reference = public_reference.clone();
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<String, _>(
                "render_wallet_qr",
                &WalletQrArgs {
                    order_id: &order_id,
                    public_reference: &public_reference,
                },
            )
            .await
            {
                Ok(svg) => {
                    qr_svg.set(Some(svg));
                    qr_visible.set(true);
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };
    view! {
        <article class="ticket-card"><div><p class="eyebrow">{ticket.ticket_type_name}</p><strong>{ticket.public_reference}</strong><span>{ticket.holder_name.value_or(ticket.holder_email_masked)}</span></div><button class="ticket-qr-button" on:click=toggle_qr disabled=move || busy.get() || !qr_available>{move || if busy.get() { tr("generuje") } else if qr_visible.get() { tr("ukryj_qr") } else if qr_available { tr("pokaz_qr") } else { tr("qr_niedostepny") }}</button><Show when=move || qr_visible.get()>{move || qr_svg.get().map(|svg| view! { <div class="mini-qr" inner_html=svg></div> })}</Show><small>{i18n::format("qr_wazny_do_value", &[human_time(&ticket.qr_expires_at).to_string()])}</small></article>
    }
}

#[component]
fn QrPanel(svg: Option<String>, token: String, expires: String) -> impl IntoView {
    view! { <div class="qr-panel">{svg.map(|markup| view! { <div class="qr-svg" inner_html=markup></div> })}<code>{token}</code><small>{i18n::format("wazny_do_value", &[human_time(&expires).to_string()])}</small></div> }
}

#[component]
fn FanProfileScreen(
    status: RwSignal<FanSessionStatus>,
    dashboard: RwSignal<Option<FanDashboardData>>,
    wallets: RwSignal<Vec<TicketWallet>>,
    area: RwSignal<Option<AreaWallet>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let refresh = move |_| {
        refresh_fan_parts(dashboard, loading, error);
        refresh_wallets(wallets, Some(loading), error);
        refresh_fan_area(area, loading, error);
    };
    let lock = move |_| {
        spawn_local(async move {
            match bridge::invoke::<FanSessionStatus, _>("fan_lock", &EmptyArgs {}).await {
                Ok(value) => {
                    dashboard.set(None);
                    wallets.set(Vec::new());
                    area.set(None);
                    loading.set(FanLoadingState::all());
                    status.set(value);
                }
                Err(message) => error.set(Some(message)),
            }
        })
    };
    let forget = move |_| {
        spawn_local(async move {
            match bridge::invoke::<FanSessionStatus, _>("fan_forget", &EmptyArgs {}).await {
                Ok(value) => {
                    dashboard.set(None);
                    wallets.set(Vec::new());
                    area.set(None);
                    loading.set(FanLoadingState::all());
                    status.set(value);
                }
                Err(message) => error.set(Some(message)),
            }
        })
    };
    view! {
        <section class="screen">
            <header class="screen-title"><p class="eyebrow">{tr("moj_profil")}</p><h2>{tr("ustawienia_sygnau")}</h2></header>
            {move || status.get().session.map(|profile| view! {
                <div class="profile-card"><div class="avatar">"V"</div><div><strong>{profile.display_name.value_or_else(|| tr("fan_viryi").to_owned())}</strong><p>{profile.email}</p></div></div>
                <div class="stats-grid"><Metric value=profile.wallet_count.to_string() label=tr("zamowienia")/><Metric value=if profile.has_admission_pass { "1".to_owned() } else { "0".to_owned() } label=tr("wejsciowki")/><Metric value=dashboard.with(|state| state.as_ref().map(|d| d.referral.qualified_referrals.to_string())).value_or_else(|| "—".to_owned()) label=tr("polecenia")/></div>
            })}
            <div class="settings-list">
                <LanguageSwitch />
                <button on:click=refresh disabled=move || { let state = loading.get(); state.events || state.referral || state.interests || state.admission_pass || state.wallets }>{move || { let state = loading.get(); if state.events || state.referral || state.interests || state.admission_pass || state.wallets { tr("odswiezam_2") } else { tr("odswiez_dane") } }}</button>
                <button on:click=lock>{tr("zablokuj_aplikacje")}</button>
                <button class="danger ghost" on:click=forget>{tr("usun_profil_i_bilety_z_urzadzenia")}</button>
            </div>
            <AnonymousFeedback error=error />
            <p class="security-note">{tr("sesja_fana_wejsciowka_oraz_prywatne_tokeny_portfela_sa")}</p>
        </section>
    }
}

#[component]
fn AnonymousFeedback(error: RwSignal<Option<String>>) -> impl IntoView {
    let category = RwSignal::new("idea".to_owned());
    let message = RwSignal::new(String::new());
    let busy = RwSignal::new(false);

    let submit = move |_| {
        if busy.get_untracked() {
            return;
        }
        let current_category = category.get_untracked();
        let current_message = message.get_untracked().trim().to_owned();
        let length = current_message.chars().count();
        if !(8..=2_000).contains(&length) {
            error.set(Some(
                tr("feedback_powinien_miec_od_8_do_2000_znakow").to_owned(),
            ));
            return;
        }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke_unit(
                "submit_anonymous_feedback",
                &AnonymousFeedbackArgs {
                    category: &current_category,
                    message: &current_message,
                },
            )
            .await
            {
                Ok(()) => {
                    message.set(String::new());
                    error.set(Some(
                        tr("feedback_zosta_wysany_anonimowo_dzieki").to_owned(),
                    ));
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    view! {
        <section class="feedback-card">
            <div class="feedback-heading">
                <div><p class="eyebrow">{tr("anonimowy_feedback")}</p><h3>{tr("powiedz_nam_co_poprawic")}</h3></div>
                <span aria-hidden="true">"◌"</span>
            </div>
            <p>{tr("aplikacja_wysya_tylko_kategorie_i_tresc_bez_e")}</p>
            <label class="select-label">
                {tr("kategoria")}
                <select prop:value=move || category.get() on:change=move |event| category.set(event_target_value(&event))>
                    <option value="idea">{tr("pomys")}</option>
                    <option value="bug">{tr("bad")}</option>
                    <option value="concert">{tr("koncerty_i_bilety")}</option>
                    <option value="merch">{tr("merch")}</option>
                    <option value="other">{tr("inne")}</option>
                </select>
            </label>
            <label>
                {tr("tresc")}
                <textarea
                    rows="6"
                    maxlength="2000"
                    placeholder=tr("napisz_wprost_co_dziaa_zle_albo_czego_brakuje")
                    prop:value=move || message.get()
                    on:input=move |event| message.set(event_target_value(&event))
                ></textarea>
            </label>
            <div class="feedback-submit-row">
                <small>{move || format!("{} / 2000", message.get().chars().count())}</small>
                <button type="button" class="primary" disabled=move || busy.get() || message.get().trim().chars().count() < 8 on:click=submit>
                    {move || if busy.get() { tr("wysyam_2") } else { tr("wyslij_anonimowo") }}
                </button>
            </div>
        </section>
    }
}

#[component]
fn Skeleton(#[prop(default = 3)] rows: usize) -> impl IntoView {
    view! { <div class="skeleton-stack" aria-label=tr("adowanie")>{(0..rows).map(|_| view! { <i></i> }).collect_view()}</div> }
}

#[component]
fn Toast(error: RwSignal<Option<String>>) -> impl IntoView {
    Effect::new(move |_| {
        if error.get().is_some() {
            set_timeout(move || error.set(None), std::time::Duration::from_secs(5));
        }
    });
    let is_success = move || {
        error.with(|msg| {
            msg.as_ref().is_some_and(|m| {
                let lower = m.to_lowercase();
                lower.contains("utworzon")
                    || lower.contains("zapisan")
                    || lower.contains(tr("wysalismy"))
                    || lower.contains("sent")
                    || lower.contains(tr("uniewaznion"))
                    || lower.contains("revoked")
                    || lower.contains("zrealizowan")
                    || lower.contains("gotowy")
                    || lower.contains("zeskanowany")
                    || lower.contains("ponownie")
                    || lower.contains(tr("feedback_zosta_wysany"))
            })
        })
    };
    view! { <Show when=move || error.get().is_some()><button class="toast" class:toast-success=is_success on:click=move |_| error.set(None)>{move || error.get().value_or_else(Default::default)}</button></Show> }
}

fn refresh_operator_parts(
    dashboard: RwSignal<Option<DashboardData>>,
    loading: RwSignal<OperatorLoadingState>,
    error: RwSignal<Option<String>>,
) {
    refresh_operator_events(dashboard, loading, error);
    refresh_operator_qr(dashboard, loading, error);
}

fn refresh_operator_events(
    dashboard: RwSignal<Option<DashboardData>>,
    loading: RwSignal<OperatorLoadingState>,
    error: RwSignal<Option<String>>,
) {
    loading.update(|state| state.events = true);
    spawn_local(async move {
        match bridge::invoke_latest::<Vec<PublicEvent>, _>(
            "operator_events",
            &EmptyArgs {},
            15_000,
            "operator:events",
        )
        .await
        {
            Ok(Some(value)) => dashboard.update(|state| {
                state.get_or_insert_with(DashboardData::default).events = value;
            }),
            Ok(None) => return,
            Err(message) => error.set(Some(message)),
        }
        loading.update(|state| state.events = false);
    });
}

fn refresh_operator_qr(
    dashboard: RwSignal<Option<DashboardData>>,
    loading: RwSignal<OperatorLoadingState>,
    error: RwSignal<Option<String>>,
) {
    loading.update(|state| state.qr = true);
    spawn_local(async move {
        match bridge::invoke_latest::<crate::models::ConcertQrOverview, _>(
            "operator_qr",
            &EmptyArgs {},
            15_000,
            "operator:qr",
        )
        .await
        {
            Ok(Some(value)) => dashboard.update(|state| {
                state.get_or_insert_with(DashboardData::default).qr = Some(value);
            }),
            Ok(None) => return,
            Err(message) => error.set(Some(message)),
        }
        loading.update(|state| state.qr = false);
    });
}

fn refresh_operator_signal(
    overview: RwSignal<Option<OperatorSignalOverview>>,
    loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) {
    if loading.get_untracked() {
        return;
    }
    loading.set(true);
    spawn_local(async move {
        let result = bridge::invoke_latest::<OperatorSignalOverview, _>(
            "operator_signal_overview",
            &EmptyArgs {},
            20_000,
            "operator:signal",
        )
        .await;
        match result {
            Ok(Some(value)) => overview.set(Some(value)),
            Ok(None) => {}
            Err(message) => error.set(Some(message)),
        }
        loading.set(false);
    });
}

fn refresh_operator_ops(
    overview: RwSignal<Option<OperatorOpsOverview>>,
    loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) {
    loading.set(true);
    spawn_local(async move {
        match bridge::invoke_latest::<OperatorOpsOverview, _>(
            "operator_ops_overview",
            &EmptyArgs {},
            20_000,
            "operator:ops",
        )
        .await
        {
            Ok(Some(value)) => overview.set(Some(value)),
            Ok(None) => return,
            Err(message) => error.set(Some(message)),
        }
        loading.set(false);
    });
}

fn refresh_fan_status(status: RwSignal<FanSessionStatus>, error: RwSignal<Option<String>>) {
    spawn_local(async move {
        match bridge::invoke::<FanSessionStatus, _>("fan_status", &EmptyArgs {}).await {
            Ok(value) => status.set(value),
            Err(message) => error.set(Some(message)),
        }
    });
}

fn refresh_fan_parts(
    dashboard: RwSignal<Option<FanDashboardData>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) {
    loading.set(FanLoadingState::all());
    refresh_fan_events(dashboard, loading, error);
    refresh_fan_referral(dashboard, loading, error);
    refresh_fan_interests(dashboard, loading, error);
    refresh_fan_admission_pass(dashboard, loading, error);
}

fn refresh_fan_events(
    dashboard: RwSignal<Option<FanDashboardData>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) {
    loading.update(|state| state.events = true);
    spawn_local(async move {
        match bridge::invoke_latest::<Vec<PublicEvent>, _>(
            "fan_events",
            &EmptyArgs {},
            15_000,
            "fan:events",
        )
        .await
        {
            Ok(Some(value)) => dashboard.update(|state| {
                state.get_or_insert_with(FanDashboardData::default).events =
                    stable_fan_events(value);
            }),
            Ok(None) => return,
            Err(message) => error.set(Some(message)),
        }
        loading.update(|state| state.events = false);
    });
}

fn refresh_fan_merch(
    merch: RwSignal<Option<MerchCatalog>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) {
    loading.update(|state| state.merch = true);
    spawn_local(async move {
        match bridge::invoke_latest::<MerchCatalog, _>(
            "fan_merch_catalog",
            &EmptyArgs {},
            15_000,
            "fan:merch",
        )
        .await
        {
            Ok(Some(value)) => merch.set(Some(value)),
            Ok(None) => return,
            Err(message) => {
                merch.set(None);
                error.set(Some(message));
            }
        }
        loading.update(|state| state.merch = false);
    });
}

fn refresh_fan_merch_bundles(bundles: RwSignal<Option<FanMerchBundleCatalog>>) {
    spawn_local(async move {
        match bridge::invoke_latest::<FanMerchBundleCatalog, _>(
            "fan_merch_bundles",
            &EmptyArgs {},
            12_000,
            "fan:merch-bundles",
        )
        .await
        {
            Ok(Some(value)) => bundles.set(Some(value)),
            Ok(None) => {}
            Err(_) => bundles.set(None),
        }
    });
}

fn refresh_fan_referral(
    dashboard: RwSignal<Option<FanDashboardData>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) {
    loading.update(|state| state.referral = true);
    spawn_local(async move {
        match bridge::invoke_latest::<ReferralProgress, _>(
            "fan_referral",
            &EmptyArgs {},
            15_000,
            "fan:referral",
        )
        .await
        {
            Ok(Some(value)) => dashboard.update(|state| {
                state.get_or_insert_with(FanDashboardData::default).referral = value;
            }),
            Ok(None) => return,
            Err(message) => error.set(Some(message)),
        }
        loading.update(|state| state.referral = false);
    });
}

fn refresh_fan_interests(
    dashboard: RwSignal<Option<FanDashboardData>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) {
    loading.update(|state| state.interests = true);
    spawn_local(async move {
        match bridge::invoke_latest::<Vec<FanEventInterest>, _>(
            "fan_interests",
            &EmptyArgs {},
            15_000,
            "fan:interests",
        )
        .await
        {
            Ok(Some(value)) => dashboard.update(|state| {
                state
                    .get_or_insert_with(FanDashboardData::default)
                    .interests = stable_fan_interests(value);
            }),
            Ok(None) => return,
            Err(message) => error.set(Some(message)),
        }
        loading.update(|state| state.interests = false);
    });
}

fn refresh_fan_admission_pass(
    dashboard: RwSignal<Option<FanDashboardData>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) {
    loading.update(|state| state.admission_pass = true);
    spawn_local(async move {
        match bridge::invoke_latest::<Option<AdmissionPass>, _>(
            "fan_admission_pass",
            &EmptyArgs {},
            15_000,
            "fan:admission",
        )
        .await
        {
            Ok(Some(value)) => dashboard.update(|state| {
                state
                    .get_or_insert_with(FanDashboardData::default)
                    .admission_pass = value;
            }),
            Ok(None) => return,
            Err(message) => error.set(Some(message)),
        }
        loading.update(|state| state.admission_pass = false);
    });
}

fn refresh_fan_area(
    area: RwSignal<Option<AreaWallet>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) {
    loading.update(|state| state.area = true);
    spawn_local(async move {
        match bridge::invoke_latest::<AreaWallet, _>(
            "fan_area_wallet",
            &EmptyArgs {},
            15_000,
            "fan:area",
        )
        .await
        {
            Ok(Some(value)) => area.set(Some(value)),
            Ok(None) => return,
            Err(message) => error.set(Some(message)),
        }
        loading.update(|state| state.area = false);
    });
}

fn refresh_wallets(
    wallets: RwSignal<Vec<TicketWallet>>,
    loading: Option<RwSignal<FanLoadingState>>,
    error: RwSignal<Option<String>>,
) {
    if let Some(loading) = loading {
        loading.update(|state| state.wallets = true);
    }
    spawn_local(async move {
        match bridge::invoke_latest::<WalletBatch, _>(
            "fan_wallets",
            &EmptyArgs {},
            35_000,
            "fan:wallets",
        )
        .await
        {
            Ok(Some(value)) => {
                wallets.set(stable_wallets(value.wallets));
                if value.failed_count > 0 {
                    error.set(Some(i18n::format(
                        "nie_udao_sie_odswiezyc_value_zamowien_pozostae_bilety",
                        &[value.failed_count.to_string()],
                    )));
                }
            }
            Ok(None) => return,
            Err(message) => error.set(Some(message)),
        }
        if let Some(loading) = loading {
            loading.update(|state| state.wallets = false);
        }
    });
}

fn operator_events(dashboard: RwSignal<Option<DashboardData>>) -> Vec<PublicEvent> {
    dashboard.with(|state| {
        state
            .as_ref()
            .map(|data| data.events.clone())
            .value_or_else(Default::default)
    })
}

fn operator_qr_events(
    dashboard: RwSignal<Option<DashboardData>>,
) -> Vec<crate::models::StaffEvent> {
    dashboard.with(|state| {
        state
            .as_ref()
            .and_then(|data| data.qr.as_ref())
            .map(|qr| qr.events.clone())
            .value_or_else(Default::default)
    })
}

fn operator_campaigns(dashboard: RwSignal<Option<DashboardData>>) -> Vec<QrCampaign> {
    dashboard.with(|state| {
        state
            .as_ref()
            .and_then(|data| data.qr.as_ref())
            .map(|qr| qr.campaigns.clone())
            .value_or_else(Default::default)
    })
}

fn fan_events(
    dashboard: RwSignal<Option<FanDashboardData>>,
    public: RwSignal<Option<PublicHomeData>>,
) -> Vec<PublicEvent> {
    stable_fan_events(
        dashboard
            .with(|state| state.as_ref().map(|data| data.events.clone()))
            .or_else(|| public.with(|state| state.as_ref().map(|data| data.events.clone())))
            .value_or_else(Default::default),
    )
}

fn stable_fan_events(mut events: Vec<PublicEvent>) -> Vec<PublicEvent> {
    events.retain(|event| {
        !event.slug.trim().is_empty()
            && !event.title.trim().is_empty()
            && event.slug.len() <= 128
            && event.title.chars().count() <= 240
    });
    events.sort_unstable_by(|left, right| {
        left.starts_at
            .cmp(&right.starts_at)
            .then_with(|| left.slug.cmp(&right.slug))
    });
    events.dedup_by(|left, right| left.slug == right.slug);
    events.truncate(100);
    events
}

fn stable_fan_interests(mut interests: Vec<FanEventInterest>) -> Vec<FanEventInterest> {
    interests.retain(|interest| {
        !interest.event.slug.trim().is_empty() && !interest.event.title.trim().is_empty()
    });
    interests.sort_unstable_by(|left, right| left.event.slug.cmp(&right.event.slug));
    interests.dedup_by(|left, right| left.event.slug == right.event.slug);
    interests.truncate(100);
    interests
}

fn stable_wallets(mut wallets: Vec<TicketWallet>) -> Vec<TicketWallet> {
    wallets.retain(|wallet| !wallet.order.order_id.trim().is_empty());
    wallets.sort_unstable_by(|left, right| left.order.order_id.cmp(&right.order.order_id));
    wallets.dedup_by(|left, right| left.order.order_id == right.order.order_id);
    wallets.truncate(100);
    wallets
}
