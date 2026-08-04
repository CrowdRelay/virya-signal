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
        FanDashboardData, FanEventInterest, FanSessionStatus, FanSignupInput, IssuePassInput,
        IssuedPass, OperatorOpsOverview, OperatorProfileInput, OperatorRole,
        OperatorSignalOverview, OpsDeliveryItem, OpsOutboxItem, OpsRetryResult, PublicEvent,
        PublicHomeData, QrCampaign, ReferralProgress, RequestedCityInput, RequestedCityResult,
        SessionStatus, ShowModeScanResult, ShowModeStatus, ShowModeSyncResult, TicketWallet,
        TicketingOverview, WalletBatch, WalletTicket,
    },
};

#[component]
pub fn App() -> impl IntoView {
    let mode = RwSignal::new(RootMode::Launcher);
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
            match bridge::invoke_timeout::<SessionStatus, _>(
                "session_status",
                &EmptyArgs {},
                10_000,
            )
            .await
            {
                Ok(value) => {
                    operator_status.set(value);
                    operator_status_failed.set(false);
                }
                Err(message) => {
                    operator_status_failed.set(true);
                    error.set(Some(message));
                }
            }
            operator_status_loading.set(false);
        });
        spawn_local(async move {
            match bridge::invoke_timeout::<FanSessionStatus, _>("fan_status", &EmptyArgs {}, 10_000)
                .await
            {
                Ok(value) => {
                    fan_status.set(value);
                    fan_status_failed.set(false);
                }
                Err(message) => {
                    fan_status_failed.set(true);
                    error.set(Some(message));
                }
            }
            fan_status_loading.set(false);
        });
    });

    view! {
        <main class="app-shell">
            {move || match mode.get() {
                RootMode::Launcher => view! {
                    <Launcher
                        mode=mode
                        fan_status=fan_status
                        operator_status=operator_status
                        fan_status_loading=fan_status_loading
                        operator_status_loading=operator_status_loading
                        fan_status_failed=fan_status_failed
                        operator_status_failed=operator_status_failed
                        status_refresh=status_refresh
                    />
                }.into_any(),
                RootMode::Fan => view! {
                    <FanPortal mode=mode status=fan_status status_loading=fan_status_loading error=error />
                }.into_any(),
                RootMode::Team => view! {
                    <OperatorPortal mode=mode status=operator_status status_loading=operator_status_loading error=error />
                }.into_any(),
            }}
            <Toast error=error />
        </main>
    }
}

#[component]
fn Launcher(
    mode: RwSignal<RootMode>,
    fan_status: RwSignal<FanSessionStatus>,
    operator_status: RwSignal<SessionStatus>,
    fan_status_loading: RwSignal<bool>,
    operator_status_loading: RwSignal<bool>,
    fan_status_failed: RwSignal<bool>,
    operator_status_failed: RwSignal<bool>,
    status_refresh: RwSignal<u32>,
) -> impl IntoView {
    let open_fan = move |_| {
        if fan_status_failed.get_untracked() {
            status_refresh.update(|value| *value = value.wrapping_add(1));
        } else {
            mode.set(RootMode::Fan);
        }
    };
    let open_team = move |_| {
        if operator_status_failed.get_untracked() {
            status_refresh.update(|value| *value = value.wrapping_add(1));
        } else {
            mode.set(RootMode::Team);
        }
    };

    view! {
        <section class="launcher">
            <header class="launcher-brand">
                <div class="signal-mark signal-logo" role="img" aria-label="Logo Virya Signal"><span></span><span></span><span></span></div>
                <p class="eyebrow">VIRYA SIGNAL</p>
                <h1>Sygnał w kieszeni.<br/><em>Kontrola na scenie.</em></h1>
                <p>Koncerty, bilety, nagrody i wejście dla fanów. Sprzedaż, skanowanie i obsługa wydarzeń dla zespołu.</p>
            </header>
            <div class="mode-grid">
                <button class="mode-card fan-mode" on:click=open_fan>
                    <span class="mode-index">01</span>
                    <div><p class="eyebrow">DLA FANÓW</p><h2>Virya Signal</h2><p>Koncerty, polecenia, nagrody, bilety i QR na wejście.</p></div>
                    <strong>{move || if fan_status_loading.get() { "SPRAWDZAM PROFIL…" } else if fan_status_failed.get() { "PONÓW ODCZYT PROFILU" } else if fan_status.get().configured { "OTWÓRZ MÓJ SYGNAŁ" } else { "DOŁĄCZ DO SYGNAŁU" }}</strong>
                </button>
                <button class="mode-card team-mode" on:click=open_team>
                    <span class="mode-index">02</span>
                    <div><p class="eyebrow">DLA ZESPOŁU</p><h2>Virya Control</h2><p>Bramka, bilety, kupony, wejściówki i kampanie koncertowe.</p></div>
                    <strong>{move || if operator_status_loading.get() { "SPRAWDZAM SEJF…" } else if operator_status_failed.get() { "PONÓW ODCZYT SEJFU" } else if operator_status.get().configured { "OTWÓRZ PANEL" } else { "SPARUJ URZĄDZENIE" }}</strong>
                </button>
            </div>
            <p class="launcher-foot">CROWDRELAY POWERED / RUST NATIVE CORE</p>
        </section>
    }
}

#[component]
fn BackButton(mode: RwSignal<RootMode>) -> impl IntoView {
    view! { <button class="back-button" on:click=move |_| mode.set(RootMode::Launcher)>"← VIRYA"</button> }
}

#[component]
fn OperatorPortal(
    mode: RwSignal<RootMode>,
    status: RwSignal<SessionStatus>,
    status_loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let dashboard = RwSignal::new(None::<DashboardData>);
    let tab = RwSignal::new(OperatorTab::Home);

    view! {
        {move || if status_loading.get() {
            view! { <AccessLoader mode=mode label="SPRAWDZAM BEZPIECZNY SEJF" /> }.into_any()
        } else if status.get().unlocked {
            view! { <OperatorApp mode=mode status=status dashboard=dashboard tab=tab error=error /> }.into_any()
        } else {
            view! { <OperatorAccess mode=mode status=status error=error /> }.into_any()
        }}
    }
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
    let name = RwSignal::new("Virya staff".to_owned());
    let token = RwSignal::new(String::new());
    let api = RwSignal::new(API_BASE.to_owned());
    let role = RwSignal::new(OperatorRole::Staff);
    let busy = RwSignal::new(false);

    let unlock = move |_| {
        let current_pin = pin.get();
        if current_pin.chars().count() < 6 {
            error.set(Some("PIN musi mieć co najmniej 6 znaków.".to_owned()));
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
        if current_pin.chars().count() < 6 || payload.trim().is_empty() {
            error.set(Some(
                "Podaj PIN i zeskanuj albo wklej kod parowania.".to_owned(),
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
                    if pin.get().chars().count() >= 6 {
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
        if current_pin.chars().count() < 6 || profile.bearer_token.trim().len() < 24 {
            error.set(Some("Podaj PIN i poprawny token urządzenia.".to_owned()));
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
                <p class="eyebrow">VIRYA CONTROL</p>
                <h1>Sparuj <em>urządzenie.</em></h1>
                <p>Bez przepisywania API, roli i długiego sekretu.</p>
            </header>
            <div class="access-card">
                <Show when=move || status.get().configured fallback=move || view! {
                    <div class="pairing-flow">
                        <Show when=move || pairing.get().trim().is_empty() fallback=move || view! {
                            <div class="pairing-scanned">
                                <span class="pairing-ok">"✓"</span>
                                <strong>"Kod zeskanowany"</strong>
                                <small>"Wpisz PIN poniżej i kliknij SPARUJ."</small>
                            </div>
                        }>
                            <button class="pairing-scan primary" on:click=scan_pairing disabled=move || busy.get()>
                                <span class="pairing-qr">"▦"</span>
                                <strong>{move || if busy.get() { "ŁĄCZĘ…" } else { "ZESKANUJ KOD QR" }}</strong>
                                <small>"Kod pokazany w panelu Virya"</small>
                            </button>
                        </Show>
                        <div class="pairing-divider"><span></span><small>"ALBO"</small><span></span></div>
                        <label>"Kod parowania"<textarea rows="2" placeholder="virya-signal://pair?…" prop:value=move || pairing.get() on:input=move |e| pairing.set(event_target_value(&e))></textarea></label>
                        <label>"Lokalny PIN"<input type="password" autocomplete="new-password" prop:value=move || pin.get() on:input=move |e| pin.set(event_target_value(&e)) /></label>
                        <button class="primary" on:click=submit_pairing disabled=move || busy.get() || pairing.get().trim().is_empty()>"SPARUJ"</button>
                        <button class="text-button" type="button" on:click=move |_| advanced.update(|v| *v = !*v)>
                            {move || if advanced.get() { "UKRYJ USTAWIENIA RĘCZNE" } else { "USTAWIENIA ZAAWANSOWANE" }}
                        </button>
                        <Show when=move || advanced.get()>
                            <div class="advanced-config">
                                <label>"Nazwa urządzenia / osoby"<input prop:value=move || name.get() on:input=move |e| name.set(event_target_value(&e)) /></label>
                                <label>"API CrowdRelay"<input prop:value=move || api.get() on:input=move |e| api.set(event_target_value(&e)) /></label>
                                <div class="segmented">
                                    <button class:active=move || role.get() == OperatorRole::Owner on:click=move |_| role.set(OperatorRole::Owner)>"OWNER"</button>
                                    <button class:active=move || role.get() == OperatorRole::Staff on:click=move |_| role.set(OperatorRole::Staff)>"STAFF"</button>
                                </div>
                                <label>"Token urządzenia"<textarea rows="3" prop:value=move || token.get() on:input=move |e| token.set(event_target_value(&e))></textarea></label>
                                <button class="ghost" on:click=configure_manual disabled=move || busy.get()>"ZAPISZ RĘCZNIE"</button>
                            </div>
                        </Show>
                    </div>
                }>
                    <div class="form-grid">
                        <p class="lock-copy">Profil operatora jest zaszyfrowany lokalnie.</p>
                        <label>"PIN"<input type="password" autocomplete="current-password" prop:value=move || pin.get() on:input=move |e| pin.set(event_target_value(&e)) /></label>
                        <button class="primary" disabled=move || busy.get() on:click=unlock>"ODBLOKUJ"</button>
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
                    mode.set(RootMode::Launcher);
                }
                Err(message) => error.set(Some(message)),
            }
        });
    };

    view! {
        <section class="authenticated">
            <header class="topbar">
                <div on:dblclick=move |_| refresh_operator_parts(dashboard, loading, error) style="cursor:pointer"><p class="eyebrow">"VIRYA CONTROL"</p><strong>{move || status.get().session.map(|s| s.display_name).value_or_else(Default::default)}</strong></div>
                <div class="topbar-actions">
                    <span class="role-pill">{move || role().label()}</span>
                    <button class="menu-trigger" aria-label="Więcej opcji" on:click=move |_| menu_open.update(|v| *v = !*v)>"⋯"</button>
                    <button aria-label="Zamknij i zablokuj panel" on:click=close>"×"</button>
                </div>
            </header>
            <Show when=move || menu_open.get()>
                <div class="overflow-backdrop" on:click=move |_| menu_open.set(false)></div>
                <nav class="overflow-menu">
                    <button class:active=move || tab.get() == OperatorTab::Discounts on:click=move |_| { tab.set(OperatorTab::Discounts); menu_open.set(false); }><span>"%"</span>"Zniżki"</button>
                    <button class:active=move || tab.get() == OperatorTab::Campaigns on:click=move |_| { tab.set(OperatorTab::Campaigns); menu_open.set(false); }><span>"◫"</span>"Kody QR"</button>
                    <button class:active=move || tab.get() == OperatorTab::Settings on:click=move |_| { tab.set(OperatorTab::Settings); menu_open.set(false); }><span>"⚙"</span>"Ustawienia"</button>
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
            <nav class="bottom-nav" class:four=move || owner.get() class:three=move || !owner.get()>
                <NavButton tab=tab own=OperatorTab::Home icon="⌁" label="Start" />
                <Show when=move || owner.get()>
                    <NavButton tab=tab own=OperatorTab::Signal icon="◉" label="Sygnał" />
                </Show>
                <NavButton tab=tab own=OperatorTab::Scan icon="▣" label="Skan" />
                <NavButton tab=tab own=OperatorTab::Tickets icon="▤" label="Bilety" />
            </nav>
        </section>
    }
}

#[component]
fn NavButton<T>(tab: RwSignal<T>, own: T, icon: &'static str, label: &'static str) -> impl IntoView
where
    T: Copy + PartialEq + Send + Sync + 'static,
{
    view! { <button class:active=move || tab.get() == own on:click=move |_| tab.set(own)><span>{icon}</span><small>{label}</small></button> }
}

#[component]
fn OperatorHome(
    dashboard: RwSignal<Option<DashboardData>>,
    loading: RwSignal<OperatorLoadingState>,
) -> impl IntoView {
    view! {
        <section class="screen">
            <header class="screen-title"><p class="eyebrow">LIVE OPERATIONS</p><h2>Dzisiaj pod kontrolą</h2></header>
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
                            <article class="hero-card"><p class="eyebrow">NASTĘPNY KONCERT</p><h3>{title}</h3><p>{location}</p><time>{time}</time></article>
                        }
                    })}
                    <div class="stats-grid"><Metric value=event_count.to_string() label="koncerty"/><Metric value=if qr_loading { "…".to_owned() } else { active.to_string() } label="aktywne QR"/><Metric value=if qr_loading { "…".to_owned() } else { checkins.to_string() } label="check-iny"/></div>
                    <div class="section-head"><h3>Nadchodzące</h3><span>{event_count}</span></div>
                    {if events.is_empty() {
                        view! { <div class="empty-state"><strong>"Brak nadchodzących koncertów"</strong><p>"Kiedy pojawi się nowe wydarzenie, zobaczysz je tutaj."</p></div> }.into_any()
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
                <p class="eyebrow">VIRYA SIGNAL</p>
                <h2>Społeczność i wzrost</h2>
                <p class="screen-copy">Zbiorczy obraz Sygnału bez danych osobowych fanów.</p>
            </header>
            <Show
                when=move || owner.get()
                fallback=move || view! {
                    <div class="empty-state">
                        <strong>"Widok tylko dla ownera"</strong>
                        <p>"Statystyki zgód, wzrostu i miast są dostępne wyłącznie dla właściciela."</p>
                    </div>
                }
            >
                <div class="signal-admin-toolbar">
                    <p>"Dane są agregowane w CrowdRelay i nie zawierają adresów e-mail ani identyfikatorów fanów."</p>
                    <button class="text-button" on:click=refresh disabled=move || loading.get()>
                        {move || if loading.get() { "ODŚWIEŻAM…" } else { "ODŚWIEŻ" }}
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
                                        <strong>"Brak snapshotu Sygnału"</strong>
                                        <p>"Odśwież dane. Jeżeli backend jest jeszcze w trakcie wdrożenia, panel pokaże bezpieczny błąd zamiast pustego ekranu."</p>
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
                {format!(
                    "Snapshot częściowy. Niedostępne źródła: {}.",
                    unavailable.join(", ")
                )}
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
                    <span>{format!("{} aktywnych", city.active_fans.max(0))}</span>
                </article>
            }
        })
        .collect_view();
    let cities_view = if city_count == 0 {
        view! {
            <div class="empty-state compact">
                <strong>"Brak agregatu miast"</strong>
                <p>"Sygnał nie ma jeszcze potwierdzonych danych miejskich albo źródło jest chwilowo niedostępne."</p>
            </div>
        }
        .into_any()
    } else {
        view! { <div class="signal-city-list">{city_cards}</div> }.into_any()
    };

    view! {
        <div class="signal-admin-content">
            <div class="stats-grid">
                <Metric value=summary.active_fans.max(0).to_string() label="aktywni"/>
                <Metric value=summary.marketing_opted_in.max(0).to_string() label="zgody marketingowe"/>
                <Metric value=activity.new_fans_30d.max(0).to_string() label="nowi / 30 dni"/>
            </div>
            <article class="signal-health-card">
                <div>
                    <p class="eyebrow">ZDROWIE BAZY</p>
                    <strong>{confirmation_rate}</strong>
                    <span>"potwierdzonych spośród aktywnych i oczekujących"</span>
                </div>
                <dl>
                    <div><dt>"wszyscy"</dt><dd>{summary.total_fans.max(0)}</dd></div>
                    <div><dt>"oczekujący"</dt><dd>{summary.pending_fans.max(0)}</dd></div>
                    <div><dt>"wypisani"</dt><dd>{summary.unsubscribed_fans.max(0)}</dd></div>
                    <div><dt>"wyciszeni"</dt><dd>{summary.suppressed_fans.max(0)}</dd></div>
                    <div><dt>"powiadomienia w pobliżu"</dt><dd>{summary.nearby_enabled.max(0)}</dd></div>
                </dl>
            </article>
            <div class="section-head"><h3>"Aktywność"</h3><span>"30 dni / całość"</span></div>
            <div class="signal-activity-grid">
                <article><strong>{activity.new_fans_7d.max(0)}</strong><span>"nowi / 7 dni"</span></article>
                <article><strong>{format!("{} / {}", activity.referral_attributions_30d.max(0), activity.referral_attributions_total.max(0))}</strong><span>"polecenia"</span></article>
                <article><strong>{format!("{} / {}", activity.event_interests_30d.max(0), activity.event_interests_total.max(0))}</strong><span>"zainteresowania koncertami"</span></article>
                <article><strong>{activity.nearby_notifications_30d.max(0)}</strong><span>"powiadomienia nearby"</span></article>
                <article><strong>{activity.pending_city_requests.max(0)}</strong><span>"miasta do moderacji"</span></article>
            </div>
            {degraded_view}
            <div class="section-head"><h3>"Najsilniejsze miasta"</h3><span>{city_count}</span></div>
            {cities_view}
            <p class="security-note">{format!("Snapshot: {generated_at}. Dane wyłącznie zagregowane.")}</p>
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
            error.set(Some("Najpierw wybierz koncert.".to_owned()));
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
    let manual_submit = move |_| redeem_code(manual.get());

    let prepare = move |_| {
        let slug = event_slug.get();
        if slug.is_empty() {
            error.set(Some("Wybierz koncert.".to_owned()));
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
                    show_message.set(format!(
                        "Snapshot gotowy: {} trwałych biletów.",
                        value.eligible_passes
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
                    show_message.set(format!(
                        "Sync: {} zapisane, {} konfliktów, {} nadal czeka.",
                        value.synced, value.conflicts, value.pending
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
                    show_message.set("Dane koncertu usunięte z urządzenia.".to_owned());
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    view! {
        <section class="screen">
            <header class="screen-title"><p class="eyebrow">GATE MODE</p><h2>Skanuj wejście</h2></header>
            <label class="select-label">"Koncert"<select disabled=move || loading.get().events prop:value=move || event_slug.get() on:change=select_event><option value="">{move || if loading.get().events { "Ładuję koncerty…" } else { "Wybierz wydarzenie" }}</option><For each=move || operator_events(dashboard) key=|event| event.slug.clone() children=move |event| view! { <option value=event.slug.clone()>{event.title}</option> } /></select></label>
            <article class:show-mode-active=move || offline.get() class="show-mode-card">
                <div class="section-head"><div><p class="eyebrow">OFFLINE SHOW MODE</p><h3>{move || if offline.get() { "Bramka pracuje lokalnie" } else { "Odporność na brak LTE" }}</h3></div><button type="button" class:active=move || offline.get() disabled=move || !show_mode.get().prepared || busy.get() on:click=move |_| offline.update(|value| *value = !*value)>{move || if offline.get() { "OFFLINE ON" } else { "OFFLINE OFF" }}</button></div>
                <p>{move || if show_mode.get().prepared { format!("{} biletów · {} oczekuje · {} konfliktów", show_mode.get().eligible_passes, show_mode.get().pending, show_mode.get().conflicts) } else { "Pobierz bezpieczny snapshot przed otwarciem bram.".to_owned() }}</p>
                <div class="show-mode-actions"><button type="button" on:click=prepare disabled=move || busy.get() || event_slug.get().is_empty()>"PRZYGOTUJ OFFLINE"</button><button type="button" on:click=sync disabled=move || busy.get() || !show_mode.get().prepared>"SYNCHRONIZUJ"</button><button type="button" class="danger ghost" on:click=clear disabled=move || busy.get() || !show_mode.get().prepared>"WYCZYŚĆ"</button></div>
                <Show when=move || !show_message.get().is_empty()><small>{move || show_message.get()}</small></Show>
            </article>
            <button class="scanner-button" on:click=scan disabled=move || busy.get()><span class="scanner-frame"></span><strong>{move || if busy.get() { "WERYFIKUJĘ…" } else if offline.get() { "SKANUJ LOKALNIE" } else { "URUCHOM APARAT" }}</strong><small>{move || if offline.get() { "Wyłącznie trwały QR biletu t1" } else { "QR biletu lub wejściówki" }}</small></button>
            <Show when=move || !offline.get()><div class="manual-row"><input placeholder="Kod QR lub numer wejściówki" prop:value=move || manual.get() on:input=move |e| manual.set(event_target_value(&e))/><button on:click=manual_submit disabled=move || busy.get()>"SPRAWDŹ"</button></div></Show>
            {move || result.get().map(|entry| {
                let success = matches!(entry.status.as_str(), "redeemed" | "already_redeemed" | "offline_queued" | "offline_duplicate");
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
            error.set(Some("Wybierz koncert.".to_owned()));
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
            pool_slug: pool_slug.get(),
            fan_email: fan_email.get(),
            claim_expires_hours: 72,
        };
        if input.event_slug.is_empty() || input.fan_email.trim().is_empty() {
            error.set(Some("Wybierz koncert i podaj e-mail fana.".to_owned()));
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
        let reference = revoke_ref.get();
        if reference.trim().is_empty() {
            error.set(Some("Podaj public reference wejściówki.".to_owned()));
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
                Ok(_) => error.set(Some("Wejściówka została unieważniona.".to_owned())),
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    view! {
        <section class="screen">
            <header class="screen-title"><p class="eyebrow">TICKETING</p><h2>Bilety i wejściówki</h2></header>
            <div class="toolbar"><select disabled=move || loading.get().events prop:value=move || event_slug.get() on:change=move |e| event_slug.set(event_target_value(&e))><option value="">{move || if loading.get().events { "Ładuję koncerty…" } else { "Wybierz koncert" }}</option>{move || operator_events(dashboard).into_iter().map(|event| view! { <option value=event.slug.clone()>{event.title}</option> }).collect_view()}</select><button on:click=load disabled=move || busy.get() || loading.get().events>"ODŚWIEŻ"</button></div>
            {move || overview.get().map(|data| view! {
                <div class="stats-grid wide"><Metric value=data.paid_tickets.to_string() label="sprzedane"/><Metric value=data.sale.reserved.to_string() label="w trakcie"/><Metric value=data.sale.available.to_string() label="dostępne"/></div>
                <div class="revenue-card"><p>OBRÓT BRUTTO</p><strong>{money(data.gross_sales_minor, &data.sale.currency)}</strong><span>{format!("zwroty: {}", money(data.refunded_minor, &data.sale.currency))}</span></div>
                <div class="section-head"><h3>Ostatnie zamówienia</h3><span>{data.recent_orders.len()}</span></div>
                <div class="card-list">{data.recent_orders.into_iter().map(|order| view! { <article class="order-card"><div><strong>{order.public_reference}</strong><p>{order.buyer_name.value_or(order.buyer_email_masked)}</p></div><span>{money(order.amount_gross_minor, &order.currency)}</span></article> }).collect_view()}</div>
            })}
            <Show when=move || owner.get()><div class="admin-box"><p class="eyebrow">OWNER ONLY</p><h3>Ręczna wejściówka</h3><p class="inline-note">"Numer wejściówki to bezpieczny publiczny identyfikator, np. VRY-... Nie jest tokenem QR ani prywatnym tokenem zamówienia."</p><input placeholder="fan@email.com" prop:value=move || fan_email.get() on:input=move |e| fan_email.set(event_target_value(&e))/><input placeholder="pool slug" prop:value=move || pool_slug.get() on:input=move |e| pool_slug.set(event_target_value(&e))/><button class="primary" on:click=issue disabled=move || busy.get()>"WYDAJ WEJŚCIÓWKĘ"</button>{move || issued.get().map(|pass| view! { <div class="token-box"><strong>{pass.public_reference}</strong><p>Token roszczenia: {pass.claim_token}</p></div> })}<hr/><input placeholder="Numer wejściówki, np. VRY-…" prop:value=move || revoke_ref.get() on:input=move |e| revoke_ref.set(event_target_value(&e))/><button class="danger" on:click=revoke disabled=move || busy.get()>"UNIEWAŻNIJ"</button></div></Show>
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
        let c = code.get();
        let o = order.get();
        if c.trim().is_empty() || o.trim().is_empty() {
            error.set(Some("Podaj kod i numer sprzedaży.".to_owned()));
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
        <section class="screen"><header class="screen-title"><p class="eyebrow">MERCH DESK</p><h2>Realizuj zniżkę</h2></header><div class="coupon-visual"><span>%</span><div><strong>VIRYA SIGNAL</strong><p>kupon fanowski / kontrolowane użycie</p></div></div><div class="form-grid panel"><label>"Kod zniżkowy"<input autocapitalize="characters" placeholder="VIRYA-…" prop:value=move || code.get() on:input=move |e| code.set(event_target_value(&e))/></label><label>"Numer sprzedaży"<input placeholder="MERCH-WRO-001" prop:value=move || order.get() on:input=move |e| order.set(event_target_value(&e))/></label><button class="primary" on:click=redeem disabled=move || busy.get()>"ZREALIZUJ KUPON"</button></div>{move || result.get().map(|envelope| view! { <article class="scan-result scan-success"><strong>KUPON ZREALIZOWANY</strong><span>{envelope.result.status}</span><p>{format!("Użycie {}/{}", envelope.result.used_count, envelope.result.max_uses)}</p></article> })}</section>
    }
}

#[component]
fn Campaigns(
    dashboard: RwSignal<Option<DashboardData>>,
    loading: RwSignal<OperatorLoadingState>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let event_slug = RwSignal::new(String::new());
    let label = RwSignal::new("Wejście główne".to_owned());
    let valid_from = RwSignal::new(String::new());
    let valid_until = RwSignal::new(String::new());
    let max_checkins = RwSignal::new(String::new());
    let busy = RwSignal::new(false);

    let create = move |_| {
        let Some(from) = local_to_rfc3339(&valid_from.get()) else {
            error.set(Some("Podaj poprawny początek ważności.".to_owned()));
            return;
        };
        let Some(until) = local_to_rfc3339(&valid_until.get()) else {
            error.set(Some("Podaj poprawny koniec ważności.".to_owned()));
            return;
        };
        let max_value = max_checkins.get();
        let max = if max_value.trim().is_empty() {
            None
        } else {
            match max_value.trim().parse::<u32>() {
                Ok(value) if value > 0 => Some(value),
                _ => {
                    error.set(Some("Limit musi być dodatnią liczbą.".to_owned()));
                    return;
                }
            }
        };
        if until <= from {
            error.set(Some(
                "Koniec kampanii musi być później niż początek.".to_owned(),
            ));
            return;
        }
        let input = CreateQrCampaignInput {
            event_slug: event_slug.get(),
            label: label.get(),
            valid_from: from,
            valid_until: until,
            max_checkins: max,
        };
        if input.event_slug.is_empty() || input.label.trim().is_empty() {
            error.set(Some("Wybierz koncert i nazwij kampanię.".to_owned()));
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
                    error.set(Some("Kampania QR utworzona.".to_owned()));
                    refresh_operator_qr(dashboard, loading, error);
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    view! {
        <section class="screen"><header class="screen-title"><p class="eyebrow">CONCERT SIGNAL</p><h2>Kampanie QR</h2></header><div class="form-grid panel"><label>"Koncert"<select disabled=move || loading.get().qr prop:value=move || event_slug.get() on:change=move |e| event_slug.set(event_target_value(&e))><option value="">{move || if loading.get().qr { "Ładuję kampanie…" } else { "Wybierz koncert" }}</option><For each=move || operator_qr_events(dashboard) key=|event| event.slug.clone() children=move |event| view! { <option value=event.slug.clone()>{event.title}</option> } /></select></label><label>"Nazwa punktu / kampanii"<input prop:value=move || label.get() on:input=move |e| label.set(event_target_value(&e))/></label><div class="two-cols"><label>"Ważna od"<input type="datetime-local" prop:value=move || valid_from.get() on:input=move |e| valid_from.set(event_target_value(&e))/></label><label>"Ważna do"<input type="datetime-local" prop:value=move || valid_until.get() on:input=move |e| valid_until.set(event_target_value(&e))/></label></div><label>"Limit check-inów (opcjonalnie)"<input inputmode="numeric" prop:value=move || max_checkins.get() on:input=move |e| max_checkins.set(event_target_value(&e))/></label><button class="primary" on:click=create disabled=move || busy.get() || loading.get().qr>"UTWÓRZ KAMPANIĘ"</button></div><div class="section-head"><h3>Aktywne i historyczne</h3></div><Show when=move || !loading.get().qr fallback=move || view! { <Skeleton /> }><div class="card-list"><For each=move || operator_campaigns(dashboard) key=|campaign| campaign.id.clone() children=move |campaign| view! { <CampaignCard campaign=campaign dashboard=dashboard loading=loading error=error /> } /></div></Show></section>
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
                    error.set(Some("Kampania została wyłączona.".to_owned()));
                    refresh_operator_qr(dashboard, loading, error);
                }
                Err(message) => error.set(Some(message)),
            }
        });
    };
    let revoke_button = if active {
        Some(view! {
            <button class="danger ghost" on:click=revoke>
                "WYŁĄCZ KAMPANIĘ"
            </button>
        })
    } else {
        None
    };
    view! {
        <article class="campaign-card"><div class="campaign-head"><div><strong>{campaign.label}</strong><p>{campaign.event_title}</p></div><span class:online=active class:offline=!active>{if active { "ACTIVE" } else { "CLOSED" }}</span></div><div class="campaign-stats"><span>{format!("{} check-inów", campaign.checkin_count)}</span><span>{campaign.max_checkins.map(|v| format!("limit {v}")).value_or_else(|| "bez limitu".to_owned())}</span></div>{campaign.token.map(|token| view! { <code>{token}</code> })}{revoke_button}</article>
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
        <section class="screen"><header class="screen-title"><p class="eyebrow">DEVICE</p><h2>Ustawienia</h2></header><div class="settings-list"><article><div><strong>Połączenie</strong><p>{move || status.get().session.map(|s| s.api_base_url).value_or_else(Default::default)}</p></div><span class:online=move || !loading.get().events && !loading.get().qr>{move || if loading.get().events || loading.get().qr { "ŁĄCZĘ" } else { "ONLINE" }}</span></article><article><div><strong>Uprawnienia</strong><p>{move || status.get().session.map(|s| s.role.label().to_owned()).value_or_else(Default::default)}</p></div></article><button on:click=refresh disabled=move || loading.get().events || loading.get().qr>"Odśwież wszystkie dane"</button><button on:click=lock>"Zablokuj panel"</button><button class="danger ghost" on:click=forget>"Usuń profil operatora"</button></div><Show when=move || owner.get()><OpsPanel overview=ops loading=ops_loading error=error /></Show><p class="security-note">Token operatora przechowuje zaszyfrowany sejf Stronghold. Warstwa WebView nigdy go nie odczytuje.</p></section>
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
        <section class="ops-panel"><div class="section-head"><div><p class="eyebrow">CONTROL PLANE</p><h3>Kolejki i dostawy</h3></div><button class="text-button" on:click=refresh disabled=move || loading.get()>"Odśwież"</button></div>
            <Show when=move || !loading.get() fallback=move || view! { <Skeleton rows=2 /> }>
                {move || overview.get().map(|data| {
                    let summary = data.summary;
                    let deliveries = data.dead_deliveries;
                    let outbox = data.dead_outbox;
                    let unavailable_sources = data.unavailable_sources;
                    let healthy = deliveries.is_empty() && outbox.is_empty() && unavailable_sources.is_empty();
                    let degraded_view = (!unavailable_sources.is_empty()).then(|| view! {
                        <p class="security-note warning">{format!("Cockpit działa częściowo. Niedostępne: {}.", unavailable_sources.join(", "))}</p>
                    });
                    let deliveries_view = (!deliveries.is_empty()).then(|| view! {
                        <div class="section-head"><h3>Martwe dostawy</h3></div>
                        <div class="ops-list"><For each=move || deliveries.clone() key=|item| item.id.clone() children=move |item| view! { <OpsDeliveryCard item=item overview=overview loading=loading error=error /> } /></div>
                    });
                    let outbox_view = (!outbox.is_empty()).then(|| view! {
                        <div class="section-head"><h3>Martwy outbox</h3></div>
                        <div class="ops-list"><For each=move || outbox.clone() key=|item| item.id.clone() children=move |item| view! { <OpsOutboxCard item=item overview=overview loading=loading error=error /> } /></div>
                    });
                    let healthy_view = healthy.then(|| view! { <p class="ops-healthy">Brak martwych wpisów. Tor dostaw jest czysty.</p> });
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
    view! { <article class="ops-item"><div><strong>{item.event_type}</strong><p>{format!("{} · próba {}/{}", item.endpoint_name, item.attempt_count, item.max_attempts)}</p><small>{item.last_error_kind.value_or_else(|| "brak kodu błędu".to_owned())}</small></div><button class="danger ghost" on:click=retry disabled=move || loading.get() || !endpoint_active>"RETRY"</button></article> }
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
    view! { <article class="ops-item"><div><strong>{item.event_type}</strong><p>{format!("próba {}/{}", item.attempts, item.max_attempts)}</p><small>{item.last_error_kind.value_or_else(|| "brak kodu błędu".to_owned())}</small></div><button class="danger ghost" on:click=retry disabled=move || loading.get()>"RETRY"</button></article> }
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
                    "Retry był już wcześniej przyjęty.".to_owned()
                } else {
                    "Wpis wrócił do kolejki.".to_owned()
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
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    // Entering the fan zone must be a local-only transition. City data is
    // fetched only when the user explicitly asks for the canonical list.
    let public = RwSignal::new(Some(PublicHomeData::default()));

    view! {
        {move || if status_loading.get() {
            view! { <AccessLoader mode=mode label="SPRAWDZAM TWÓJ SYGNAŁ" /> }.into_any()
        } else if status.get().unlocked {
            view! { <FanApp mode=mode status=status public=public error=error /> }.into_any()
        } else {
            view! { <FanAccess mode=mode status=status error=error /> }.into_any()
        }}
    }
}

#[component]
fn AccessLoader(mode: RwSignal<RootMode>, label: &'static str) -> impl IntoView {
    view! {
        <section class="access-screen status-loader">
            <BackButton mode=mode />
            <div class="access-card">
                <p class="eyebrow">{label}</p>
                <Skeleton rows=2 />
            </div>
        </section>
    }
}

#[component]
fn FanAccess(
    mode: RwSignal<RootMode>,
    status: RwSignal<FanSessionStatus>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
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
                "Zgoda marketingowa jest wymagana do dołączenia do Sygnału.".to_owned(),
            ));
            return;
        }
        let current_pin = pin.get();
        let requested = RequestedCityInput {
            name: custom_city_name.get(),
            region: optional(custom_region.get()),
            country_code: "PL".to_owned(),
        };
        let input_email = email.get();
        let input_name = optional(name.get());
        let input_referral = optional(referral.get());
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
                    error.set(Some(format!("Nie udało się zapisać miasta: {message}")));
                    busy.set(false);
                    return;
                }
            };
            if city_slug.trim().is_empty() {
                error.set(Some("Wybierz miasto albo wpisz własne.".to_owned()));
                busy.set(false);
                return;
            }
            let input = FanSignupInput {
                api_base_url: API_BASE.to_owned(),
                email: input_email,
                display_name: input_name,
                city_slug,
                locale: "pl".to_owned(),
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
                    }
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let confirm = move |_| {
        let current_pin = pin.get();
        let input = FanConfirmationInput {
            api_base_url: API_BASE.to_owned(),
            email: email.get(),
            display_name: optional(name.get()),
            token: token.get(),
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
    };

    view! {
        <section class="fan-access">
            <BackButton mode=mode />
            <header class="fan-access-hero">
                <p class="eyebrow">"VIRYA SIGNAL"</p>
                <h1>"Koncerty, bilety"<br/><em>"i nagrody."</em></h1>
                <p class="hero-subtitle">"Dołącz w 3 krokach:"</p>
                <ol class="signal-steps" aria-label="Jak dołączyć">
                    <li><span class="step-num">"1"</span>" Podaj e-mail i miasto"</li>
                    <li><span class="step-num">"2"</span>" Potwierdź kod z wiadomości"</li>
                    <li><span class="step-num">"3"</span>" Odkrywaj koncerty blisko Ciebie"</li>
                </ol>
                <div class="signal-purpose-grid" aria-label="Co daje Virya Signal">
                    <span><b aria-hidden="true">"⌁"</b>" koncerty blisko Ciebie"</span>
                    <span><b aria-hidden="true">"▣"</b>" bilety i QR w telefonie"</span>
                    <span><b aria-hidden="true">"✦"</b>" nagrody za proste akcje"</span>
                </div>
            </header>
            <Show when=move || status.get().configured fallback=move || view! {
                <div class="access-card fan-card">
                    <div class="segmented">
                        <button class:active=move || access_mode.get() == FanAccessMode::Signup on:click=move |_| access_mode.set(FanAccessMode::Signup)>"ZACZYNAM"</button>
                        <button class:active=move || access_mode.get() == FanAccessMode::Confirm on:click=move |_| access_mode.set(FanAccessMode::Confirm)>"MAM KOD"</button>
                    </div>
                    <div class="form-grid fan-form">
                        <label>"E-mail"<input type="email" autocomplete="email" prop:value=move || email.get() on:input=move |e| email.set(event_target_value(&e))/></label>
                        <label>"Imię / nazwa (opcjonalnie)"<input prop:value=move || name.get() on:input=move |e| name.set(event_target_value(&e))/></label>
                        <Show when=move || access_mode.get() == FanAccessMode::Signup fallback=move || view! {
                            <>
                                <p class="confirm-hint">"Sprawdź skrzynkę e-mail. Wklej poniżej kod, który wysłaliśmy."</p>
                                <label>"Kod z e-maila"<textarea rows="3" placeholder="np. eyJhbGci..." prop:value=move || token.get() on:input=move |e| token.set(event_target_value(&e))></textarea></label>
                                <label>"Lokalny PIN"<input type="password" autocomplete="new-password" prop:value=move || pin.get() on:input=move |e| pin.set(event_target_value(&e))/></label>
                                <button class="primary" disabled=move || busy.get() on:click=confirm>"POTWIERDŹ I WEJDŹ"</button>
                            </>
                        }>
                            <>
                                <div class="custom-city-fields city-stable-entry">
                                    <label>"Miejscowość"<input placeholder="np. Bielawa" prop:value=move || custom_city_name.get() on:input=move |e| custom_city_name.set(event_target_value(&e))/></label>
                                    <label>"Województwo / region (opcjonalnie)"<input placeholder="dolnośląskie" prop:value=move || custom_region.get() on:input=move |e| custom_region.set(event_target_value(&e))/></label>
                                    <p class="inline-note">"Wpisz miejscowość ręcznie — dopasujemy ją do mapy Sygnału."</p>
                                </div>
                                <div class="nearby-pref">
                                    <label class="check-label"><input type="checkbox" prop:checked=move || nearby_enabled.get() on:change=move |e| nearby_enabled.set(event_target_checked(&e))/><span>"Powiadamiaj mnie o koncertach w pobliżu"</span></label>
                                    <Show when=move || nearby_enabled.get()>
                                        <div class="radius-picker">
                                            <button type="button" class:active=move || radius_km.get()==50 on:click=move |_| radius_km.set(50)>"50 km"</button>
                                            <button type="button" class:active=move || radius_km.get()==100 on:click=move |_| radius_km.set(100)>"100 km"</button>
                                            <button type="button" class:active=move || radius_km.get()==150 on:click=move |_| radius_km.set(150)>"150 km"</button>
                                            <button type="button" class:active=move || radius_km.get()==250 on:click=move |_| radius_km.set(250)>"250 km"</button>
                                        </div>
                                    </Show>
                                </div>
                                <label>"Kod polecający (opcjonalnie)"<input prop:value=move || referral.get() on:input=move |e| referral.set(event_target_value(&e))/></label>
                                <label>"Lokalny PIN"<input type="password" autocomplete="new-password" prop:value=move || pin.get() on:input=move |e| pin.set(event_target_value(&e))/></label>
                                <label class="check-label"><input type="checkbox" prop:checked=move || consent.get() on:change=move |e| consent.set(event_target_checked(&e))/><span>"Chcę otrzymywać informacje o koncertach, premierach i nagrodach Viryi."</span></label>
                                <button class="primary" disabled=move || busy.get() on:click=signup>"DOŁĄCZ DO SYGNAŁU"</button>
                            </>
                        </Show>
                    </div>
                </div>
            }>
                <div class="access-card fan-card">
                    <p class="lock-copy">Twój profil i bilety są zaszyfrowane na urządzeniu.</p>
                    <div class="form-grid">
                        <label>"PIN"<input type="password" autocomplete="current-password" prop:value=move || pin.get() on:input=move |e| pin.set(event_target_value(&e))/></label>
                        <button class="primary" disabled=move || busy.get() on:click=unlock>"OTWÓRZ MÓJ SYGNAŁ"</button>
                    </div>
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
    let wallets = RwSignal::new(Vec::<TicketWallet>::new());
    let admission_qr = RwSignal::new(None::<AdmissionQr>);
    let area = RwSignal::new(None::<AreaWallet>);
    let loading = RwSignal::new(FanLoadingState::all());

    let loaded = RwSignal::new(FanLoadedState::default());

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
    on_cleanup(move || bridge::invalidate_latest("fan:"));

    let close = move |_| {
        bridge::invalidate_latest("fan:");
        spawn_local(async move {
            match bridge::invoke::<FanSessionStatus, _>("fan_lock", &EmptyArgs {}).await {
                Ok(value) => {
                    dashboard.set(None);
                    wallets.set(Vec::new());
                    admission_qr.set(None);
                    area.set(None);
                    loading.set(FanLoadingState::all());
                    status.set(value);
                    mode.set(RootMode::Launcher);
                }
                Err(message) => error.set(Some(message)),
            }
        });
    };

    view! {
        <section class="authenticated fan-authenticated"><header class="topbar fan-topbar"><div on:dblclick=move |_| { loaded.set(FanLoadedState::default()); refresh_fan_parts(dashboard, loading, error); refresh_wallets(wallets, Some(loading), error); refresh_fan_area(area, loading, error); } style="cursor:pointer"><p class="eyebrow">VIRYA SIGNAL</p><strong>{move || status.get().session.and_then(|s| s.display_name).value_or_else(|| "Mój Sygnał".to_owned())}</strong></div><div class="topbar-actions"><span class="live-dot"></span><button aria-label="Zamknij i zablokuj Sygnał" on:click=close>"×"</button></div></header><div class="content">{move || match tab.get() {
            FanTab::Signal => view! { <FanSignal dashboard=dashboard loading=loading /> }.into_any(),
            FanTab::Events => view! { <FanEvents dashboard=dashboard public=public loading=loading error=error /> }.into_any(),
            FanTab::Game => view! { <FanGame area=area loading=loading error=error /> }.into_any(),
            FanTab::Wallet => view! { <FanWallet dashboard=dashboard wallets=wallets admission_qr=admission_qr loading=loading error=error /> }.into_any(),
            FanTab::Profile => view! { <FanProfileScreen status=status dashboard=dashboard wallets=wallets area=area loading=loading error=error /> }.into_any(),
        }}</div><nav class="bottom-nav five"><FanNavButton tab=tab own=FanTab::Signal icon="◉" label="Sygnał"/><FanNavButton tab=tab own=FanTab::Events icon="⌁" label="Koncerty"/><FanNavButton tab=tab own=FanTab::Game icon="◇" label="Gra"/><FanNavButton tab=tab own=FanTab::Wallet icon="▣" label="Bilety"/><FanNavButton tab=tab own=FanTab::Profile icon="◎" label="Profil"/></nav></section>
    }
}

#[component]
fn FanNavButton(
    tab: RwSignal<FanTab>,
    own: FanTab,
    icon: &'static str,
    label: &'static str,
) -> impl IntoView {
    view! { <button class:active=move || tab.get() == own on:click=move |_| tab.set(own)><span>{icon}</span><small>{label}</small></button> }
}

#[component]
fn FanSignal(
    dashboard: RwSignal<Option<FanDashboardData>>,
    loading: RwSignal<FanLoadingState>,
) -> impl IntoView {
    view! {
        <section class="screen fan-screen">
            <header class="signal-dashboard-hero">
                <p class="eyebrow">TWÓJ WPŁYW</p>
                <h2>{move || dashboard.with(|state| state.as_ref().map(|d| d.referral.qualified_referrals.to_string())).value_or_else(|| "—".to_owned())}</h2>
                <strong>potwierdzonych poleceń</strong>
                <p>{move || dashboard.with(|state| state.as_ref().map(|d| format!("Kod: {}", d.referral.referral_code))).value_or_else(|| "Ładowanie Sygnału…".to_owned())}</p>
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
                    <div class="section-head"><h3>Twoje kupony</h3></div>
                    <div class="card-list">{coupons.into_iter().map(|coupon| view! {
                        <article class="fan-coupon"><div><span>{format!("-{}%", coupon.discount_percent)}</span><strong>{coupon.code}</strong></div><small>{coupon.status}</small></article>
                    }).collect_view()}</div>
                });
                let rewards_view = (!rewards.is_empty()).then(|| view! {
                    <div class="section-head"><h3>Nagrody</h3></div>
                    <div class="card-list">{rewards.into_iter().map(|reward| view! {
                        <article class="reward-card"><div><strong>{reward.item_name}</strong><p>{reward.sku}</p></div><span>{reward.status}</span></article>
                    }).collect_view()}</div>
                });
                view! {
                    <div class="stats-grid"><Metric value=referral.pending_referrals.to_string() label="oczekujące"/><Metric value=entries_total.to_string() label="losy"/><Metric value=coupon_count.to_string() label="kupony"/></div>
                    <div class="section-head"><h3>Aktywne losowania</h3><span>{draw_count}</span></div>
                    <div class="card-list">{draws.into_iter().map(|draw| view! {
                        <article class="draw-card"><div><p class="eyebrow">{draw.prize_kind}</p><strong>{draw.name}</strong><span>{format!("Losowanie {}", human_time(&draw.draw_at))}</span></div><div class="entry-count"><b>{draw.total_entries}</b><small>LOSÓW</small></div></article>
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
fn FanEvents(
    dashboard: RwSignal<Option<FanDashboardData>>,
    public: RwSignal<Option<PublicHomeData>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <section class="screen"><header class="screen-title"><p class="eyebrow">GDZIE GRAMY</p><h2>Koncerty</h2></header><Show when=move || !loading.get().events fallback=move || view! { <Skeleton /> }>{move || { let events = fan_events(dashboard, public); if events.is_empty() { view! { <div class="empty-state"><strong>"Brak koncertów w kalendarzu"</strong><p>"Kiedy pojawi się nowy event, będzie tutaj."</p></div> }.into_any() } else { view! { <div class="card-list fan-event-list">{events.into_iter().map(|event| view! { <FanEventCard event=event dashboard=dashboard loading=loading error=error /> }).collect_view()}</div> }.into_any() }}}</Show></section>
    }
}

#[component]
fn FanEventCard(
    event: PublicEvent,
    dashboard: RwSignal<Option<FanDashboardData>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
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
                    error.set(Some("Koncert zapisany w Twoim Sygnale.".to_owned()));
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
    let ticket = event.ticket_url;
    let description = event.description;
    let title = event.title;
    view! {
        <article class="fan-event-card">
            {image.map(|url| view! {
                <img
                    src=url
                    alt=""
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
                    <button class:active=move || interested.get() on:click=interest disabled=move || busy.get() || interested.get()>{move || if busy.get() { "ZAPISUJĘ…" } else if interested.get() { "✓ MAM TO" } else { "+ INTERESUJE MNIE" }}</button>
                    {ticket.map(|url| view! { <ExternalLink url=url label="BILETY ↗" error=error /> })}
                </div>
            </div>
        </article>
    }
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
    view! { <button on:click=open>{label}</button> }
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
            <header class="screen-title"><p class="eyebrow">VIRYA AREA</p><h2>Znajdź punkt w swoim mieście</h2><p>Otwórz mapę, wybierz aktywny punkt i przejdź do niego. Nie musisz zbierać wszystkiego ani jeździć po kraju.</p></header>
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
                        <p class="security-note warning">Połącz portfel przeglądarkowy z kontem na stronie AREA, aby zachować cały postęp.</p>
                    });
                    view! {
                        <div class="area-hero-card">
                            <div><p class="eyebrow">POSTĘP KOLEKCJI</p><h3>{format!("{claimed} / {collection_size}")}</h3></div>
                            <strong>{format!("{reward_credits} kredytów")}</strong>
                            <div class="area-progress" role="progressbar" aria-valuemin="0" aria-valuemax="100" aria-valuenow=percent><span style=format!("width:{percent}%")></span></div>
                            <div class="area-stats"><span>{format!("{live_count} aktywne punkty")}</span><span>{format!("{voucher_count} nagrody")}</span><span>{format!("{community_percent}% społeczności")}</span></div>
                        </div>
                        {migration_notice}
                        <div class="section-head"><h3>Odkryte artefakty</h3><span>{claimed}</span></div>
                        <div class="artifact-list">
                            <For each=move || claims.clone() key=|claim| claim.drop_id.clone() children=move |claim| view! {
                                <article class="artifact-card"><span>{claim.number}</span><div><strong>{claim.track}</strong><p>{format!("{} · {}", claim.city, claim.edition)}</p><small>{claim.line}</small></div><em>{claim.edition_number.map(|number| format!("#{number}")).value_or_else(|| "✓".to_owned())}</em></article>
                            } />
                        </div>
                        <div class="area-actions"><button class="primary" on:click=move |_| open_area_game(error)>"OTWÓRZ MAPĘ I ZACZNIJ"</button><button class="ghost" on:click=refresh disabled=move || loading.get().area>"Odśwież progres"</button></div>
                        <p class="security-note">Mapa pokazuje aktywne punkty i prowadzi do startu. Dokładna lokalizacja odsłania się w grze dopiero wtedy, gdy jest potrzebna.</p>
                    }.into_any()
                }).value_or_else(|| view! { <div class="empty-state"><strong>AREA chwilowo niedostępna</strong><p>Odśwież dane albo otwórz pełną grę.</p><button class="primary" on:click=move |_| open_area_game(error)>"OTWÓRZ AREA"</button></div> }.into_any())}
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
        let order = order_id.get();
        let token = checkout_token.get();
        if order.trim().is_empty() || token.trim().is_empty() {
            error.set(Some(
                "Podaj identyfikator zamówienia i prywatny token.".to_owned(),
            ));
            return;
        }
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
                    error.set(Some("Bilety zapisane w portfelu.".to_owned()));
                    refresh_wallets(wallets, Some(loading), error);
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let claim = move |_| {
        let token = claim_token.get();
        if token.trim().is_empty() {
            error.set(Some("Wklej token wejściówki.".to_owned()));
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
                    error.set(Some("Wejściówka przypisana do urządzenia.".to_owned()));
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
        <section class="screen"><header class="screen-title"><p class="eyebrow">MOBILE WALLET</p><h2>Bilety i wejście</h2></header><Show when=move || !loading.get().admission_pass fallback=move || view! { <Skeleton rows=1 /> }>{move || dashboard.with(|state| state.as_ref().and_then(|d| d.admission_pass.clone())).map(|pass| view! { <article class="admission-card"><p class="eyebrow">WEJŚCIÓWKA VIRYA</p><h3>{pass.event_title}</h3><p>{event_time_location(&pass.starts_at, pass.venue.as_deref())}</p><strong>{pass.public_reference}</strong><span>{pass.status}</span><button class="primary" on:click=qr disabled=move || busy.get()>"POKAŻ QR NA WEJŚCIE"</button>{move || admission_qr.get().map(|value| view! { <QrPanel svg=value.qr_svg token=value.token expires=value.expires_at /> })}</article> })}
        <Show when=move || dashboard.with(|state| state.as_ref().is_none_or(|d| d.admission_pass.is_none()))><div class="claim-box"><p class="eyebrow">WYGRAŁEŚ WEJŚCIÓWKĘ?</p><h3>Przypisz ją do telefonu</h3><textarea rows="3" placeholder="Token z wiadomości" prop:value=move || claim_token.get() on:input=move |e| claim_token.set(event_target_value(&e))></textarea><button class="primary" on:click=claim disabled=move || busy.get()>"ODBIERZ WEJŚCIÓWKĘ"</button></div></Show></Show>
        <div class="section-head"><h3>Portfel biletów</h3><span>{move || wallets.get().len()}</span></div><Show when=move || !loading.get().wallets fallback=move || view! { <Skeleton rows=2 /> }><div class="wallet-stack">{move || wallets.get().into_iter().map(|wallet| view! {
            <WalletCard wallet=wallet error=error />
        }).collect_view()}</div></Show><details class="import-box"><summary>"Dodaj istniejące zamówienie"</summary><div class="form-grid"><label>"Order ID"<input placeholder="UUID zamówienia" prop:value=move || order_id.get() on:input=move |e| order_id.set(event_target_value(&e))/></label><label>"Prywatny checkout token"<textarea rows="3" prop:value=move || checkout_token.get() on:input=move |e| checkout_token.set(event_target_value(&e))></textarea></label><button class="primary" on:click=import disabled=move || busy.get()>"DODAJ DO PORTFELA"</button></div></details></section>
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
                Ok(_) => error.set(Some("Wysłaliśmy ponownie portfel na e-mail.".to_owned())),
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };
    view! {
        <article class="wallet-card"><header><div><p class="eyebrow">{wallet.order.status}</p><h3>{wallet.order.event_title}</h3><p>{event_time_location(&wallet.order.starts_at, wallet.order.venue.as_deref())}</p></div><strong>{wallet.order.public_reference}</strong></header><div class="ticket-stack">{wallet.tickets.into_iter().map(|ticket| view! { <WalletTicketCard order_id=order_id.clone() ticket=ticket error=error /> }).collect_view()}</div><button class="text-button" on:click=resend disabled=move || busy.get()>{move || if busy.get() { "Wysyłam…" } else { "Wyślij bilety ponownie na e-mail" }}</button></article>
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
        <article class="ticket-card"><div><p class="eyebrow">{ticket.ticket_type_name}</p><strong>{ticket.public_reference}</strong><span>{ticket.holder_name.value_or(ticket.holder_email_masked)}</span></div><button class="ticket-qr-button" on:click=toggle_qr disabled=move || busy.get() || !qr_available>{move || if busy.get() { "GENERUJĘ…" } else if qr_visible.get() { "UKRYJ QR" } else if qr_available { "POKAŻ QR" } else { "QR NIEDOSTĘPNY" }}</button><Show when=move || qr_visible.get()>{move || qr_svg.get().map(|svg| view! { <div class="mini-qr" inner_html=svg></div> })}</Show><small>{format!("QR ważny do {}", human_time(&ticket.qr_expires_at))}</small></article>
    }
}

#[component]
fn QrPanel(svg: Option<String>, token: String, expires: String) -> impl IntoView {
    view! { <div class="qr-panel">{svg.map(|markup| view! { <div class="qr-svg" inner_html=markup></div> })}<code>{token}</code><small>{format!("ważny do {}", human_time(&expires))}</small></div> }
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
        <section class="screen"><header class="screen-title"><p class="eyebrow">MÓJ PROFIL</p><h2>Ustawienia Sygnału</h2></header>{move || status.get().session.map(|profile| view! { <div class="profile-card"><div class="avatar">V</div><div><strong>{profile.display_name.value_or_else(|| "Fan Viryi".to_owned())}</strong><p>{profile.email}</p></div></div><div class="stats-grid"><Metric value=profile.wallet_count.to_string() label="zamówienia"/><Metric value=if profile.has_admission_pass { "1".to_owned() } else { "0".to_owned() } label="wejściówki"/><Metric value=dashboard.with(|state| state.as_ref().map(|d| d.referral.qualified_referrals.to_string())).value_or_else(|| "—".to_owned()) label="polecenia"/></div> })}<div class="settings-list"><button on:click=refresh disabled=move || { let state = loading.get(); state.events || state.referral || state.interests || state.admission_pass || state.wallets }>{move || { let state = loading.get(); if state.events || state.referral || state.interests || state.admission_pass || state.wallets { "Odświeżam…" } else { "Odśwież dane" } }}</button><button on:click=lock>"Zablokuj aplikację"</button><button class="danger ghost" on:click=forget>"Usuń profil i bilety z urządzenia"</button></div><p class="security-note">Sesja fana, wejściówka oraz prywatne tokeny portfela są przechowywane w osobnym, zaszyfrowanym sejfie Stronghold.</p></section>
    }
}

#[component]
fn Skeleton(#[prop(default = 3)] rows: usize) -> impl IntoView {
    view! { <div class="skeleton-stack" aria-label="Ładowanie">{(0..rows).map(|_| view! { <i></i> }).collect_view()}</div> }
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
                    || lower.contains("wysłaliśmy")
                    || lower.contains("unieważnion")
                    || lower.contains("zrealizowan")
                    || lower.contains("gotowy")
                    || lower.contains("zeskanowany")
                    || lower.contains("ponownie")
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
                    error.set(Some(format!(
                        "Nie udało się odświeżyć {} zamówień. Pozostałe bilety są dostępne.",
                        value.failed_count
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
