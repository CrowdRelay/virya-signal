use leptos::prelude::*;
use serde::Serialize;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::spawn_local;

use crate::{
    bridge,
    models::{
        AdmissionPass, AdmissionQr, AdmissionRedemption,
        CouponEnvelope, CreateQrCampaignInput, DashboardData, FanAuthResult,
        FanConfirmationInput, FanDashboardData, FanSessionStatus, FanSignupInput, IssuePassInput,
        IssuedPass, OperatorProfileInput, OperatorRole, PublicEvent, PublicHomeData, QrCampaign,
        SessionStatus, TicketWallet, TicketingOverview,
    },
};

const API_BASE: &str = "https://signal-api.virya.music/v1/";
const POLICY_VERSION: &str = "2026-07";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum RootMode {
    Launcher,
    Fan,
    Team,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OperatorTab {
    Home,
    Scan,
    Tickets,
    Discounts,
    Campaigns,
    Settings,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FanTab {
    Signal,
    Events,
    Wallet,
    Profile,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum FanAccessMode {
    Signup,
    Confirm,
}

#[derive(Serialize)]
struct EmptyArgs {}

#[derive(Serialize)]
struct ApiArgs<'a> {
    api_base_url: &'a str,
}

#[derive(Serialize)]
struct PinArgs<'a> {
    pin: &'a str,
}

#[derive(Serialize)]
struct ConfigureArgs<'a> {
    pin: &'a str,
    profile: &'a OperatorProfileInput,
}

#[derive(Serialize)]
struct EventArgs<'a> {
    event_slug: &'a str,
}

#[derive(Serialize)]
struct RedeemArgs<'a> {
    event_slug: &'a str,
    code: &'a str,
}

#[derive(Serialize)]
struct CouponArgs<'a> {
    code: &'a str,
    order_reference: &'a str,
}

#[derive(Serialize)]
struct IssueArgs<'a> {
    input: &'a IssuePassInput,
}

#[derive(Serialize)]
struct ReferenceArgs<'a> {
    public_reference: &'a str,
}

#[derive(Serialize)]
struct CampaignArgs<'a> {
    input: &'a CreateQrCampaignInput,
}

#[derive(Serialize)]
struct CampaignIdArgs<'a> {
    campaign_id: &'a str,
}

#[derive(Serialize)]
struct FanSignupArgs<'a> {
    input: &'a FanSignupInput,
    pin: &'a str,
}

#[derive(Serialize)]
struct FanConfirmArgs<'a> {
    input: &'a FanConfirmationInput,
    pin: &'a str,
}

#[derive(Serialize)]
struct ClaimArgs<'a> {
    claim_token: &'a str,
}

#[derive(Serialize)]
struct ImportWalletArgs<'a> {
    order_id: &'a str,
    checkout_token: &'a str,
}

#[derive(Serialize)]
struct OrderArgs<'a> {
    order_id: &'a str,
}

#[derive(Serialize)]
struct UrlArgs<'a> {
    url: &'a str,
}

#[component]
pub fn App() -> impl IntoView {
    let mode = RwSignal::new(RootMode::Launcher);
    let operator_status = RwSignal::new(SessionStatus::default());
    let fan_status = RwSignal::new(FanSessionStatus::default());
    let loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);

    Effect::new(move |_| {
        spawn_local(async move {
            match bridge::invoke::<SessionStatus, _>("session_status", &EmptyArgs {}).await {
                Ok(value) => operator_status.set(value),
                Err(message) => error.set(Some(message)),
            }
            match bridge::invoke::<FanSessionStatus, _>("fan_status", &EmptyArgs {}).await {
                Ok(value) => fan_status.set(value),
                Err(message) => error.set(Some(message)),
            }
            loading.set(false);
        });
    });

    view! {
        <main class="app-shell">
            <Show when=move || !loading.get() fallback=move || view! { <Splash /> }>
                {move || match mode.get() {
                    RootMode::Launcher => view! {
                        <Launcher mode=mode fan_status=fan_status operator_status=operator_status />
                    }.into_any(),
                    RootMode::Fan => view! {
                        <FanPortal mode=mode status=fan_status error=error />
                    }.into_any(),
                    RootMode::Team => view! {
                        <OperatorPortal mode=mode status=operator_status error=error />
                    }.into_any(),
                }}
            </Show>
            <Toast error=error />
        </main>
    }
}

#[component]
fn Splash() -> impl IntoView {
    view! {
        <section class="splash">
            <div class="signal-mark"><span></span><span></span><span></span></div>
            <h1>VIRYA</h1>
            <p>SIGNAL / CONTROL</p>
        </section>
    }
}

#[component]
fn Launcher(
    mode: RwSignal<RootMode>,
    fan_status: RwSignal<FanSessionStatus>,
    operator_status: RwSignal<SessionStatus>,
) -> impl IntoView {
    view! {
        <section class="launcher">
            <header class="launcher-brand">
                <div class="signal-mark small"><span></span><span></span><span></span></div>
                <p class="eyebrow">VIRYA MOBILE</p>
                <h1>Sygnał w kieszeni.<br/><em>Kontrola na scenie.</em></h1>
                <p>Koncerty, bilety, nagrody i wejście dla fanów. Sprzedaż, skanowanie i obsługa wydarzeń dla zespołu.</p>
            </header>
            <div class="mode-grid">
                <button class="mode-card fan-mode" on:click=move |_| mode.set(RootMode::Fan)>
                    <span class="mode-index">01</span>
                    <div><p class="eyebrow">DLA FANÓW</p><h2>Virya Signal</h2><p>Koncerty, polecenia, nagrody, bilety i QR na wejście.</p></div>
                    <strong>{move || if fan_status.get().configured { "OTWÓRZ MÓJ SYGNAŁ" } else { "DOŁĄCZ DO SYGNAŁU" }}</strong>
                </button>
                <button class="mode-card team-mode" on:click=move |_| mode.set(RootMode::Team)>
                    <span class="mode-index">02</span>
                    <div><p class="eyebrow">DLA ZESPOŁU</p><h2>Virya Control</h2><p>Bramka, bilety, kupony, wejściówki i kampanie koncertowe.</p></div>
                    <strong>{move || if operator_status.get().configured { "OTWÓRZ PANEL" } else { "SPARUJ URZĄDZENIE" }}</strong>
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
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let dashboard = RwSignal::new(None::<DashboardData>);
    let tab = RwSignal::new(OperatorTab::Home);

    view! {
        {move || if status.get().unlocked {
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
    let name = RwSignal::new("Virya".to_owned());
    let token = RwSignal::new(String::new());
    let api = RwSignal::new(API_BASE.to_owned());
    let role = RwSignal::new(OperatorRole::Owner);
    let busy = RwSignal::new(false);

    let unlock = move |_| {
        let current_pin = pin.get();
        if current_pin.chars().count() < 6 {
            error.set(Some("PIN musi mieć co najmniej 6 znaków.".to_owned()));
            return;
        }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<SessionStatus, _>("unlock", &PinArgs { pin: &current_pin }).await {
                Ok(value) => status.set(value),
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let configure = move |_| {
        let current_pin = pin.get();
        let profile = OperatorProfileInput {
            display_name: name.get(),
            api_base_url: api.get(),
            role: role.get(),
            bearer_token: token.get(),
        };
        if current_pin.chars().count() < 6 || profile.bearer_token.trim().len() < 24 {
            error.set(Some("Podaj PIN i poprawny token CrowdRelay.".to_owned()));
            return;
        }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<SessionStatus, _>(
                "configure",
                &ConfigureArgs { pin: &current_pin, profile: &profile },
            ).await {
                Ok(value) => status.set(value),
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    view! {
        <section class="access-screen">
            <BackButton mode=mode />
            <header class="hero compact">
                <p class="eyebrow">PRIVATE BAND OPERATIONS</p>
                <h1>Virya <em>Control</em></h1>
                <p>Wejście, bilety, zniżki i koncertowy chaos — w jednym miejscu.</p>
            </header>
            <div class="access-card">
                <Show when=move || status.get().configured fallback=move || view! {
                    <div class="form-grid">
                        <label>"Nazwa urządzenia / osoby"<input prop:value=move || name.get() on:input=move |e| name.set(event_target_value(&e)) /></label>
                        <label>"API CrowdRelay"<input prop:value=move || api.get() on:input=move |e| api.set(event_target_value(&e)) /></label>
                        <div class="segmented">
                            <button class:active=move || role.get() == OperatorRole::Owner on:click=move |_| role.set(OperatorRole::Owner)>"OWNER"</button>
                            <button class:active=move || role.get() == OperatorRole::Staff on:click=move |_| role.set(OperatorRole::Staff)>"STAFF"</button>
                        </div>
                        <label>"Token urządzenia"<textarea rows="3" prop:value=move || token.get() on:input=move |e| token.set(event_target_value(&e))></textarea></label>
                        <label>"Lokalny PIN"<input type="password" autocomplete="new-password" prop:value=move || pin.get() on:input=move |e| pin.set(event_target_value(&e)) /></label>
                        <button class="primary" disabled=move || busy.get() on:click=configure>{move || if busy.get() { "ŁĄCZĘ…" } else { "SPARUJ URZĄDZENIE" }}</button>
                    </div>
                }>
                    <div class="form-grid">
                        <p class="lock-copy">Profil operatora jest zaszyfrowany lokalnie. Token nigdy nie trafia do interfejsu po odblokowaniu.</p>
                        <label>"PIN"<input type="password" autocomplete="current-password" prop:value=move || pin.get() on:input=move |e| pin.set(event_target_value(&e)) /></label>
                        <button class="primary" disabled=move || busy.get() on:click=unlock>{move || if busy.get() { "ODBLOKOWUJĘ…" } else { "ODBLOKUJ" }}</button>
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
    Effect::new(move |_| {
        if status.get().unlocked && dashboard.get().is_none() {
            refresh_operator_dashboard(dashboard, error);
        }
    });

    let role = move || status.get().session.map(|s| s.role).unwrap_or(OperatorRole::Staff);

    view! {
        <section class="authenticated">
            <header class="topbar">
                <div><p class="eyebrow">VIRYA CONTROL</p><strong>{move || status.get().session.map(|s| s.display_name).unwrap_or_default()}</strong></div>
                <div class="topbar-actions"><span class="role-pill">{move || role().label()}</span><button on:click=move |_| mode.set(RootMode::Launcher)>"×"</button></div>
            </header>
            <div class="content">
                {move || match tab.get() {
                    OperatorTab::Home => view! { <OperatorHome dashboard=dashboard /> }.into_any(),
                    OperatorTab::Scan => view! { <Scanner dashboard=dashboard error=error /> }.into_any(),
                    OperatorTab::Tickets => view! { <Tickets dashboard=dashboard error=error owner=Signal::derive(move || role() == OperatorRole::Owner) /> }.into_any(),
                    OperatorTab::Discounts => view! { <Discounts error=error /> }.into_any(),
                    OperatorTab::Campaigns => view! { <Campaigns dashboard=dashboard error=error /> }.into_any(),
                    OperatorTab::Settings => view! { <OperatorSettings status=status dashboard=dashboard error=error /> }.into_any(),
                }}
            </div>
            <nav class="bottom-nav six">
                <NavButton tab=tab own=OperatorTab::Home icon="⌁" label="Start" />
                <NavButton tab=tab own=OperatorTab::Scan icon="▣" label="Skan" />
                <NavButton tab=tab own=OperatorTab::Tickets icon="▤" label="Bilety" />
                <NavButton tab=tab own=OperatorTab::Discounts icon="%" label="Zniżki" />
                <NavButton tab=tab own=OperatorTab::Campaigns icon="◫" label="QR" />
                <NavButton tab=tab own=OperatorTab::Settings icon="⚙" label="Opcje" />
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
fn OperatorHome(dashboard: RwSignal<Option<DashboardData>>) -> impl IntoView {
    view! {
        <section class="screen">
            <header class="screen-title"><p class="eyebrow">LIVE OPERATIONS</p><h2>Dzisiaj pod kontrolą</h2></header>
            {move || dashboard.get().map(|data| {
                let next = data.events.first().cloned();
                let active = data.qr.as_ref().map(|q| q.campaigns.iter().filter(|c| c.active).count()).unwrap_or(0);
                let checkins = data.qr.as_ref().map(|q| q.campaigns.iter().map(|c| c.checkin_count).sum::<u64>()).unwrap_or(0);
                view! {
                    {next.map(|event| {
                        let location = event_location(&event);
                        let time = human_time(&event.starts_at);
                        let title = event.title;
                        view! {
                            <article class="hero-card"><p class="eyebrow">NASTĘPNY KONCERT</p><h3>{title}</h3><p>{location}</p><time>{time}</time></article>
                        }
                    })}
                    <div class="stats-grid"><Metric value=data.events.len().to_string() label="koncerty"/><Metric value=active.to_string() label="aktywne QR"/><Metric value=checkins.to_string() label="check-iny"/></div>
                    <div class="section-head"><h3>Nadchodzące</h3><span>{data.events.len()}</span></div>
                    <div class="card-list">{data.events.into_iter().take(8).map(|event| view! { <EventCard event=event /> }).collect_view()}</div>
                }
            }.into_any()).unwrap_or_else(|| view! { <Skeleton /> }.into_any())}
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
            <div><h4>{title}</h4><p>{location}</p></div><span class="chevron">›</span>
        </article>
    }
}

#[component]
fn Scanner(dashboard: RwSignal<Option<DashboardData>>, error: RwSignal<Option<String>>) -> impl IntoView {
    let event_slug = RwSignal::new(String::new());
    let manual = RwSignal::new(String::new());
    let result = RwSignal::new(None::<AdmissionRedemption>);
    let busy = RwSignal::new(false);

    let redeem_code = move |code: String| {
        let slug = event_slug.get_untracked();
        if slug.is_empty() {
            error.set(Some("Najpierw wybierz koncert.".to_owned()));
            return;
        }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<AdmissionRedemption, _>("redeem_admission", &RedeemArgs { event_slug: &slug, code: &code }).await {
                Ok(value) => result.set(Some(value)),
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let scan = move |_| {
        spawn_local(async move {
            match bridge::scan_qr().await {
                Ok(code) => redeem_code(code),
                Err(message) => error.set(Some(message)),
            }
        });
    };
    let manual_submit = move |_| redeem_code(manual.get());

    view! {
        <section class="screen">
            <header class="screen-title"><p class="eyebrow">GATE MODE</p><h2>Skanuj wejście</h2></header>
            <label class="select-label">"Koncert"<select prop:value=move || event_slug.get() on:change=move |e| event_slug.set(event_target_value(&e))><option value="">"Wybierz wydarzenie"</option>{move || operator_events(dashboard).into_iter().map(|event| view! { <option value=event.slug.clone()>{event.title}</option> }).collect_view()}</select></label>
            <button class="scanner-button" on:click=scan disabled=move || busy.get()><span class="scanner-frame"></span><strong>{move || if busy.get() { "WERYFIKUJĘ…" } else { "URUCHOM APARAT" }}</strong><small>QR biletu lub wejściówki</small></button>
            <div class="manual-row"><input placeholder="Kod / public reference" prop:value=move || manual.get() on:input=move |e| manual.set(event_target_value(&e))/><button on:click=manual_submit disabled=move || busy.get()>"SPRAWDŹ"</button></div>
            {move || result.get().map(|entry| {
                let success = entry.status == "redeemed" || entry.status == "already_redeemed";
                view! { <article class:scan-success=success class:scan-warning=!success class="scan-result"><strong>{entry.status.to_uppercase()}</strong><span>{entry.public_reference}</span><p>{entry.holder_name.unwrap_or(entry.holder_email_masked)}</p></article> }
            })}
        </section>
    }
}

#[component]
fn Tickets(
    dashboard: RwSignal<Option<DashboardData>>,
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
        if slug.is_empty() { error.set(Some("Wybierz koncert.".to_owned())); return; }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<TicketingOverview, _>("ticketing_overview", &EventArgs { event_slug: &slug }).await {
                Ok(value) => overview.set(Some(value)), Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let issue = move |_| {
        let input = IssuePassInput { event_slug: event_slug.get(), pool_slug: pool_slug.get(), fan_email: fan_email.get(), claim_expires_hours: 72 };
        if input.event_slug.is_empty() || input.fan_email.trim().is_empty() { error.set(Some("Wybierz koncert i podaj e-mail fana.".to_owned())); return; }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<IssuedPass, _>("issue_pass", &IssueArgs { input: &input }).await {
                Ok(value) => issued.set(Some(value)), Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let revoke = move |_| {
        let reference = revoke_ref.get();
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<serde_json::Value, _>("revoke_pass", &ReferenceArgs { public_reference: &reference }).await {
                Ok(_) => error.set(Some("Wejściówka została unieważniona.".to_owned())), Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    view! {
        <section class="screen">
            <header class="screen-title"><p class="eyebrow">TICKETING</p><h2>Bilety i wejściówki</h2></header>
            <div class="toolbar"><select prop:value=move || event_slug.get() on:change=move |e| event_slug.set(event_target_value(&e))><option value="">"Wybierz koncert"</option>{move || operator_events(dashboard).into_iter().map(|event| view! { <option value=event.slug.clone()>{event.title}</option> }).collect_view()}</select><button on:click=load disabled=move || busy.get()>"ODŚWIEŻ"</button></div>
            {move || overview.get().map(|data| view! {
                <div class="stats-grid wide"><Metric value=data.paid_tickets.to_string() label="sprzedane"/><Metric value=data.sale.reserved.to_string() label="w trakcie"/><Metric value=data.sale.available.to_string() label="dostępne"/></div>
                <div class="revenue-card"><p>OBRÓT BRUTTO</p><strong>{money(data.gross_sales_minor, &data.sale.currency)}</strong><span>{format!("zwroty: {}", money(data.refunded_minor, &data.sale.currency))}</span></div>
                <div class="section-head"><h3>Ostatnie zamówienia</h3><span>{data.recent_orders.len()}</span></div>
                <div class="card-list">{data.recent_orders.into_iter().map(|order| view! { <article class="order-card"><div><strong>{order.public_reference}</strong><p>{order.buyer_name.unwrap_or(order.buyer_email_masked)}</p></div><span>{money(order.amount_gross_minor, &order.currency)}</span></article> }).collect_view()}</div>
            })}
            <Show when=move || owner.get()><div class="admin-box"><p class="eyebrow">OWNER ONLY</p><h3>Ręczna wejściówka</h3><input placeholder="fan@email.com" prop:value=move || fan_email.get() on:input=move |e| fan_email.set(event_target_value(&e))/><input placeholder="pool slug" prop:value=move || pool_slug.get() on:input=move |e| pool_slug.set(event_target_value(&e))/><button class="primary" on:click=issue disabled=move || busy.get()>"WYDAJ WEJŚCIÓWKĘ"</button>{move || issued.get().map(|pass| view! { <div class="token-box"><strong>{pass.public_reference}</strong><p>Token roszczenia: {pass.claim_token}</p></div> })}<hr/><input placeholder="public reference do unieważnienia" prop:value=move || revoke_ref.get() on:input=move |e| revoke_ref.set(event_target_value(&e))/><button class="danger" on:click=revoke disabled=move || busy.get()>"UNIEWAŻNIJ"</button></div></Show>
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
        let c = code.get(); let o = order.get();
        if c.trim().is_empty() || o.trim().is_empty() { error.set(Some("Podaj kod i numer sprzedaży.".to_owned())); return; }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<CouponEnvelope, _>("redeem_coupon", &CouponArgs { code: &c, order_reference: &o }).await {
                Ok(value) => result.set(Some(value)), Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };
    view! {
        <section class="screen"><header class="screen-title"><p class="eyebrow">MERCH DESK</p><h2>Realizuj zniżkę</h2></header><div class="coupon-visual"><span>%</span><div><strong>VIRYA SIGNAL</strong><p>kupon fanowski / kontrolowane użycie</p></div></div><div class="form-grid panel"><label>"Kod zniżkowy"<input autocapitalize="characters" placeholder="VIRYA-…" prop:value=move || code.get() on:input=move |e| code.set(event_target_value(&e))/></label><label>"Numer sprzedaży"<input placeholder="MERCH-WRO-001" prop:value=move || order.get() on:input=move |e| order.set(event_target_value(&e))/></label><button class="primary" on:click=redeem disabled=move || busy.get()>"ZREALIZUJ KUPON"</button></div>{move || result.get().map(|envelope| view! { <article class="scan-result scan-success"><strong>KUPON ZREALIZOWANY</strong><span>{envelope.result.status}</span><p>{format!("Użycie {}/{}", envelope.result.used_count, envelope.result.max_uses)}</p></article> })}</section>
    }
}

#[component]
fn Campaigns(dashboard: RwSignal<Option<DashboardData>>, error: RwSignal<Option<String>>) -> impl IntoView {
    let event_slug = RwSignal::new(String::new());
    let label = RwSignal::new("Wejście główne".to_owned());
    let valid_from = RwSignal::new(String::new());
    let valid_until = RwSignal::new(String::new());
    let max_checkins = RwSignal::new(String::new());
    let busy = RwSignal::new(false);

    let create = move |_| {
        let Some(from) = local_to_rfc3339(&valid_from.get()) else { error.set(Some("Podaj poprawny początek ważności.".to_owned())); return; };
        let Some(until) = local_to_rfc3339(&valid_until.get()) else { error.set(Some("Podaj poprawny koniec ważności.".to_owned())); return; };
        let max = max_checkins.get().trim().parse::<u32>().ok();
        let input = CreateQrCampaignInput { event_slug: event_slug.get(), label: label.get(), valid_from: from, valid_until: until, max_checkins: max };
        if input.event_slug.is_empty() || input.label.trim().is_empty() { error.set(Some("Wybierz koncert i nazwij kampanię.".to_owned())); return; }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<QrCampaign, _>("create_qr_campaign", &CampaignArgs { input: &input }).await {
                Ok(_) => { error.set(Some("Kampania QR utworzona.".to_owned())); refresh_operator_dashboard(dashboard, error); },
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    view! {
        <section class="screen"><header class="screen-title"><p class="eyebrow">CONCERT SIGNAL</p><h2>Kampanie QR</h2></header><div class="form-grid panel"><label>"Koncert"<select prop:value=move || event_slug.get() on:change=move |e| event_slug.set(event_target_value(&e))><option value="">"Wybierz koncert"</option>{move || operator_qr_events(dashboard).into_iter().map(|event| view! { <option value=event.slug.clone()>{event.title}</option> }).collect_view()}</select></label><label>"Nazwa punktu / kampanii"<input prop:value=move || label.get() on:input=move |e| label.set(event_target_value(&e))/></label><div class="two-cols"><label>"Ważna od"<input type="datetime-local" prop:value=move || valid_from.get() on:input=move |e| valid_from.set(event_target_value(&e))/></label><label>"Ważna do"<input type="datetime-local" prop:value=move || valid_until.get() on:input=move |e| valid_until.set(event_target_value(&e))/></label></div><label>"Limit check-inów (opcjonalnie)"<input inputmode="numeric" prop:value=move || max_checkins.get() on:input=move |e| max_checkins.set(event_target_value(&e))/></label><button class="primary" on:click=create disabled=move || busy.get()>"UTWÓRZ KAMPANIĘ"</button></div><div class="section-head"><h3>Aktywne i historyczne</h3></div><div class="card-list">{move || operator_campaigns(dashboard).into_iter().map(|campaign| view! { <CampaignCard campaign=campaign dashboard=dashboard error=error /> }).collect_view()}</div></section>
    }
}

#[component]
fn CampaignCard(
    campaign: QrCampaign,
    dashboard: RwSignal<Option<DashboardData>>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let id = campaign.id.clone();
    let active = campaign.active;
    let revoke = move |_| {
        let campaign_id = id.clone();
        spawn_local(async move {
            match bridge::invoke::<serde_json::Value, _>("revoke_qr_campaign", &CampaignIdArgs { campaign_id: &campaign_id }).await {
                Ok(_) => { error.set(Some("Kampania została wyłączona.".to_owned())); refresh_operator_dashboard(dashboard, error); },
                Err(message) => error.set(Some(message)),
            }
        });
    };
    view! {
        <article class="campaign-card"><div class="campaign-head"><div><strong>{campaign.label}</strong><p>{campaign.event_title}</p></div><span class:online=active class:offline=!active>{if active { "ACTIVE" } else { "CLOSED" }}</span></div><div class="campaign-stats"><span>{format!("{} check-inów", campaign.checkin_count)}</span><span>{campaign.max_checkins.map(|v| format!("limit {v}")).unwrap_or_else(|| "bez limitu".to_owned())}</span></div>{campaign.token.map(|token| view! { <code>{token}</code> })}<Show when=move || active><button class="danger ghost" on:click=revoke>"WYŁĄCZ KAMPANIĘ"</button></Show></article>
    }
}

#[component]
fn OperatorSettings(
    status: RwSignal<SessionStatus>,
    dashboard: RwSignal<Option<DashboardData>>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let busy = RwSignal::new(false);
    let refresh = move |_| { busy.set(true); refresh_operator_dashboard(dashboard, error); busy.set(false); };
    let lock = move |_| spawn_local(async move { match bridge::invoke::<SessionStatus, _>("lock", &EmptyArgs {}).await { Ok(value) => status.set(value), Err(message) => error.set(Some(message)) } });
    let forget = move |_| spawn_local(async move { match bridge::invoke::<SessionStatus, _>("forget_device", &EmptyArgs {}).await { Ok(value) => status.set(value), Err(message) => error.set(Some(message)) } });
    view! {
        <section class="screen"><header class="screen-title"><p class="eyebrow">DEVICE</p><h2>Ustawienia</h2></header><div class="settings-list"><article><div><strong>Połączenie</strong><p>{move || status.get().session.map(|s| s.api_base_url).unwrap_or_default()}</p></div><span class="online">ONLINE</span></article><article><div><strong>Uprawnienia</strong><p>{move || status.get().session.map(|s| s.role.label().to_owned()).unwrap_or_default()}</p></div></article><button on:click=refresh disabled=move || busy.get()>"Odśwież wszystkie dane"</button><button on:click=lock>"Zablokuj panel"</button><button class="danger ghost" on:click=forget>"Usuń profil operatora"</button></div><p class="security-note">Token operatora przechowuje zaszyfrowany sejf Stronghold. Warstwa WebView nigdy go nie odczytuje.</p></section>
    }
}

#[component]
fn FanPortal(
    mode: RwSignal<RootMode>,
    status: RwSignal<FanSessionStatus>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let public = RwSignal::new(None::<PublicHomeData>);
    Effect::new(move |_| {
        if public.get().is_none() {
            spawn_local(async move {
                match bridge::invoke::<PublicHomeData, _>("public_home", &ApiArgs { api_base_url: API_BASE }).await {
                    Ok(value) => public.set(Some(value)), Err(message) => error.set(Some(message)),
                }
            });
        }
    });
    view! {
        {move || if status.get().unlocked {
            view! { <FanApp mode=mode status=status public=public error=error /> }.into_any()
        } else {
            view! { <FanAccess mode=mode status=status public=public error=error /> }.into_any()
        }}
    }
}

#[component]
fn FanAccess(
    mode: RwSignal<RootMode>,
    status: RwSignal<FanSessionStatus>,
    public: RwSignal<Option<PublicHomeData>>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let access_mode = RwSignal::new(FanAccessMode::Signup);
    let email = RwSignal::new(String::new());
    let name = RwSignal::new(String::new());
    let city = RwSignal::new(String::new());
    let referral = RwSignal::new(String::new());
    let token = RwSignal::new(String::new());
    let pin = RwSignal::new(String::new());
    let consent = RwSignal::new(false);
    let busy = RwSignal::new(false);

    let unlock = move |_| {
        let current_pin = pin.get();
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<FanSessionStatus, _>("fan_unlock", &PinArgs { pin: &current_pin }).await {
                Ok(value) => status.set(value), Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let signup = move |_| {
        if !consent.get() { error.set(Some("Zgoda marketingowa jest wymagana do dołączenia do Sygnału.".to_owned())); return; }
        let current_pin = pin.get();
        let input = FanSignupInput { api_base_url: API_BASE.to_owned(), email: email.get(), display_name: optional(name.get()), city_slug: city.get(), locale: "pl".to_owned(), referral_code: optional(referral.get()), policy_version: POLICY_VERSION.to_owned() };
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<FanAuthResult, _>("fan_signup", &FanSignupArgs { input: &input, pin: &current_pin }).await {
                Ok(result) => {
                    if result.session_created { refresh_fan_status(status, error); }
                    else { access_mode.set(FanAccessMode::Confirm); error.set(Some("Sprawdź e-mail i wklej kod potwierdzający.".to_owned())); }
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let confirm = move |_| {
        let current_pin = pin.get();
        let input = FanConfirmationInput { api_base_url: API_BASE.to_owned(), email: email.get(), display_name: optional(name.get()), token: token.get() };
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<FanAuthResult, _>("fan_confirm", &FanConfirmArgs { input: &input, pin: &current_pin }).await {
                Ok(_) => refresh_fan_status(status, error), Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    view! {
        <section class="fan-access">
            <BackButton mode=mode />
            <header class="fan-access-hero"><p class="eyebrow">VIRYA SIGNAL</p><h1>Nie obserwuj z boku.<br/><em>Wejdź do środka.</em></h1><p>Koncerty w Twoim mieście, własny link poleceń, losy, nagrody oraz wszystkie bilety w jednym miejscu.</p></header>
            <Show when=move || status.get().configured fallback=move || view! {
                <div class="access-card fan-card"><div class="segmented"><button class:active=move || access_mode.get() == FanAccessMode::Signup on:click=move |_| access_mode.set(FanAccessMode::Signup)>"DOŁĄCZAM"</button><button class:active=move || access_mode.get() == FanAccessMode::Confirm on:click=move |_| access_mode.set(FanAccessMode::Confirm)>"MAM KOD"</button></div>
                    <div class="form-grid fan-form"><label>"E-mail"<input type="email" autocomplete="email" prop:value=move || email.get() on:input=move |e| email.set(event_target_value(&e))/></label><label>"Imię / nazwa (opcjonalnie)"<input prop:value=move || name.get() on:input=move |e| name.set(event_target_value(&e))/></label>
                    <Show when=move || access_mode.get() == FanAccessMode::Signup fallback=move || view! { <><label>"Kod z e-maila"<textarea rows="3" prop:value=move || token.get() on:input=move |e| token.set(event_target_value(&e))></textarea></label><label>"Nowy lokalny PIN"<input type="password" autocomplete="new-password" prop:value=move || pin.get() on:input=move |e| pin.set(event_target_value(&e))/></label><button class="primary" disabled=move || busy.get() on:click=confirm>"POTWIERDŹ I WEJDŹ"</button></> }>
                        <><label>"Twoje miasto"<select prop:value=move || city.get() on:change=move |e| city.set(event_target_value(&e))><option value="">"Wybierz miasto"</option>{move || public.get().map(|data| data.cities.into_iter().map(|city_item| view! { <option value=city_item.slug.clone()>{format!("{} · {} fanów", city_item.name, city_item.fan_count)}</option> }).collect_view())}</select></label><label>"Kod polecający (opcjonalnie)"<input prop:value=move || referral.get() on:input=move |e| referral.set(event_target_value(&e))/></label><label>"Lokalny PIN"<input type="password" autocomplete="new-password" prop:value=move || pin.get() on:input=move |e| pin.set(event_target_value(&e))/></label><label class="check-label"><input type="checkbox" prop:checked=move || consent.get() on:change=move |e| consent.set(event_target_checked(&e))/><span>Chcę otrzymywać informacje o koncertach, premierach i nagrodach Viryi.</span></label><button class="primary" disabled=move || busy.get() on:click=signup>"DOŁĄCZ DO SYGNAŁU"</button></>
                    </Show></div>
                </div>
            }>
                <div class="access-card fan-card"><p class="lock-copy">Twój profil, sesja fana i tokeny biletów są zaszyfrowane na urządzeniu.</p><div class="form-grid"><label>"PIN"<input type="password" autocomplete="current-password" prop:value=move || pin.get() on:input=move |e| pin.set(event_target_value(&e))/></label><button class="primary" disabled=move || busy.get() on:click=unlock>"OTWÓRZ MÓJ SYGNAŁ"</button></div></div>
            </Show>
            <PublicEventStrip public=public />
        </section>
    }
}

#[component]
fn PublicEventStrip(public: RwSignal<Option<PublicHomeData>>) -> impl IntoView {
    view! { <div class="public-strip"><div class="section-head"><h3>Najbliższe koncerty</h3></div><div class="card-list">{move || public.get().map(|data| data.events.into_iter().take(4).map(|event| view! { <EventCard event=event /> }).collect_view())}</div></div> }
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

    Effect::new(move |_| {
        if status.get().unlocked && dashboard.get().is_none() {
            refresh_fan_dashboard(dashboard, error);
            refresh_wallets(wallets, error);
        }
    });

    view! {
        <section class="authenticated fan-authenticated"><header class="topbar fan-topbar"><div><p class="eyebrow">VIRYA SIGNAL</p><strong>{move || status.get().session.and_then(|s| s.display_name).unwrap_or_else(|| "Mój Sygnał".to_owned())}</strong></div><div class="topbar-actions"><span class="live-dot"></span><button on:click=move |_| mode.set(RootMode::Launcher)>"×"</button></div></header><div class="content">{move || match tab.get() {
            FanTab::Signal => view! { <FanSignal dashboard=dashboard /> }.into_any(),
            FanTab::Events => view! { <FanEvents dashboard=dashboard public=public error=error /> }.into_any(),
            FanTab::Wallet => view! { <FanWallet dashboard=dashboard wallets=wallets admission_qr=admission_qr error=error /> }.into_any(),
            FanTab::Profile => view! { <FanProfileScreen status=status dashboard=dashboard wallets=wallets error=error /> }.into_any(),
        }}</div><nav class="bottom-nav four"><FanNavButton tab=tab own=FanTab::Signal icon="◉" label="Sygnał"/><FanNavButton tab=tab own=FanTab::Events icon="⌁" label="Koncerty"/><FanNavButton tab=tab own=FanTab::Wallet icon="▣" label="Bilety"/><FanNavButton tab=tab own=FanTab::Profile icon="◎" label="Profil"/></nav></section>
    }
}

#[component]
fn FanNavButton(tab: RwSignal<FanTab>, own: FanTab, icon: &'static str, label: &'static str) -> impl IntoView {
    view! { <button class:active=move || tab.get() == own on:click=move |_| tab.set(own)><span>{icon}</span><small>{label}</small></button> }
}

#[component]
fn FanSignal(dashboard: RwSignal<Option<FanDashboardData>>) -> impl IntoView {
    view! {
        <section class="screen fan-screen">
            <header class="signal-dashboard-hero">
                <p class="eyebrow">TWÓJ WPŁYW</p>
                <h2>{move || dashboard.get().map(|d| d.referral.qualified_referrals.to_string()).unwrap_or_else(|| "—".to_owned())}</h2>
                <strong>potwierdzonych poleceń</strong>
                <p>{move || dashboard.get().map(|d| format!("Kod: {}", d.referral.referral_code)).unwrap_or_else(|| "Ładowanie Sygnału…".to_owned())}</p>
            </header>
            {move || dashboard.get().map(|data| {
                let referral = data.referral;
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
            }).unwrap_or_else(|| view! { <Skeleton /> }.into_any())}
        </section>
    }
}

#[component]
fn FanEvents(
    dashboard: RwSignal<Option<FanDashboardData>>,
    public: RwSignal<Option<PublicHomeData>>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <section class="screen"><header class="screen-title"><p class="eyebrow">GDZIE GRAMY</p><h2>Koncerty</h2></header><div class="card-list fan-event-list">{move || fan_events(dashboard, public).into_iter().map(|event| view! { <FanEventCard event=event dashboard=dashboard error=error /> }).collect_view()}</div></section>
    }
}

#[component]
fn FanEventCard(event: PublicEvent, dashboard: RwSignal<Option<FanDashboardData>>, error: RwSignal<Option<String>>) -> impl IntoView {
    let event_slug = event.slug.clone();
    let interested = Signal::derive(move || {
        dashboard
            .get()
            .is_some_and(|data| data.interests.iter().any(|item| item.event.slug == event_slug))
    });
    let interest_slug = event.slug.clone();
    let interest = move |_| {
        let event_slug = interest_slug.clone();
        spawn_local(async move {
            match bridge::invoke::<serde_json::Value, _>("fan_register_interest", &EventArgs { event_slug: &event_slug }).await {
                Ok(_) => {
                    error.set(Some("Koncert zapisany w Twoim Sygnale.".to_owned()));
                    refresh_fan_dashboard(dashboard, error);
                }
                Err(message) => error.set(Some(message)),
            }
        });
    };
    let event_day = day(&event.starts_at);
    let event_month = month(&event.starts_at);
    let event_time = human_time(&event.starts_at);
    let location = event_location(&event);
    let image = event.image_url;
    let ticket = event.ticket_url;
    let description = event.description;
    let title = event.title;
    view! {
        <article class="fan-event-card">
            {image.map(|url| view! { <img src=url alt="" /> })}
            <div class="fan-event-body">
                <div class="date-line"><span>{format!("{event_day} {event_month}")}</span><small>{event_time}</small></div>
                <h3>{title}</h3><p>{location}</p>
                {description.map(|text| view! { <p class="event-description">{text}</p> })}
                <div class="event-actions">
                    <button class:active=move || interested.get() on:click=interest>{move || if interested.get() { "✓ MAM TO" } else { "+ INTERESUJE MNIE" }}</button>
                    {ticket.map(|url| view! { <ExternalLink url=url label="BILETY ↗" error=error /> })}
                </div>
            </div>
        </article>
    }
}

#[component]
fn ExternalLink(url: String, label: &'static str, error: RwSignal<Option<String>>) -> impl IntoView {
    let open_url = url.clone();
    let open = move |_| {
        let current = open_url.clone();
        spawn_local(async move {
            if let Err(message) = bridge::invoke_unit("open_external_url", &UrlArgs { url: &current }).await {
                error.set(Some(message));
            }
        });
    };
    view! { <button on:click=open>{label}</button> }
}

#[component]
fn FanWallet(
    dashboard: RwSignal<Option<FanDashboardData>>,
    wallets: RwSignal<Vec<TicketWallet>>,
    admission_qr: RwSignal<Option<AdmissionQr>>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let order_id = RwSignal::new(String::new());
    let checkout_token = RwSignal::new(String::new());
    let claim_token = RwSignal::new(String::new());
    let busy = RwSignal::new(false);

    let import = move |_| {
        let order = order_id.get(); let token = checkout_token.get();
        if order.trim().is_empty() || token.trim().is_empty() { error.set(Some("Podaj identyfikator zamówienia i prywatny token.".to_owned())); return; }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<TicketWallet, _>("fan_import_wallet", &ImportWalletArgs { order_id: &order, checkout_token: &token }).await {
                Ok(_) => { error.set(Some("Bilety zapisane w portfelu.".to_owned())); refresh_wallets(wallets, error); },
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let claim = move |_| {
        let token = claim_token.get();
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<AdmissionPass, _>("fan_claim_pass", &ClaimArgs { claim_token: &token }).await {
                Ok(_) => { error.set(Some("Wejściówka przypisana do urządzenia.".to_owned())); refresh_fan_dashboard(dashboard, error); },
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let qr = move |_| {
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<AdmissionQr, _>("fan_admission_qr", &EmptyArgs {}).await {
                Ok(value) => admission_qr.set(Some(value)), Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    view! {
        <section class="screen"><header class="screen-title"><p class="eyebrow">MOBILE WALLET</p><h2>Bilety i wejście</h2></header>{move || dashboard.get().and_then(|d| d.admission_pass).map(|pass| view! { <article class="admission-card"><p class="eyebrow">WEJŚCIÓWKA VIRYA</p><h3>{pass.event_title}</h3><p>{event_time_location(&pass.starts_at, pass.venue.as_deref())}</p><strong>{pass.public_reference}</strong><span>{pass.status}</span><button class="primary" on:click=qr disabled=move || busy.get()>"POKAŻ QR NA WEJŚCIE"</button>{move || admission_qr.get().map(|value| view! { <QrPanel svg=value.qr_svg token=value.token expires=value.expires_at /> })}</article> })}
        <Show when=move || dashboard.get().and_then(|d| d.admission_pass).is_none()><div class="claim-box"><p class="eyebrow">WYGRAŁEŚ WEJŚCIÓWKĘ?</p><h3>Przypisz ją do telefonu</h3><textarea rows="3" placeholder="Token z wiadomości" prop:value=move || claim_token.get() on:input=move |e| claim_token.set(event_target_value(&e))></textarea><button class="primary" on:click=claim disabled=move || busy.get()>"ODBIERZ WEJŚCIÓWKĘ"</button></div></Show>
        <div class="section-head"><h3>Portfel biletów</h3><span>{move || wallets.get().len()}</span></div><div class="wallet-stack">{move || wallets.get().into_iter().map(|wallet| view! { <WalletCard wallet=wallet error=error /> }).collect_view()}</div><details class="import-box"><summary>"Dodaj istniejące zamówienie"</summary><div class="form-grid"><label>"Order ID"<input placeholder="UUID zamówienia" prop:value=move || order_id.get() on:input=move |e| order_id.set(event_target_value(&e))/></label><label>"Prywatny checkout token"<textarea rows="3" prop:value=move || checkout_token.get() on:input=move |e| checkout_token.set(event_target_value(&e))></textarea></label><button class="primary" on:click=import disabled=move || busy.get()>"DODAJ DO PORTFELA"</button></div></details></section>
    }
}

#[component]
fn WalletCard(wallet: TicketWallet, error: RwSignal<Option<String>>) -> impl IntoView {
    let order_id = wallet.order.order_id.clone();
    let resend = move |_| {
        let order = order_id.clone();
        spawn_local(async move {
            match bridge::invoke::<serde_json::Value, _>("fan_request_delivery", &OrderArgs { order_id: &order }).await {
                Ok(_) => error.set(Some("Wysłaliśmy ponownie portfel na e-mail.".to_owned())), Err(message) => error.set(Some(message)),
            }
        });
    };
    view! {
        <article class="wallet-card"><header><div><p class="eyebrow">{wallet.order.status}</p><h3>{wallet.order.event_title}</h3><p>{event_time_location(&wallet.order.starts_at, wallet.order.venue.as_deref())}</p></div><strong>{wallet.order.public_reference}</strong></header><div class="ticket-stack">{wallet.tickets.into_iter().map(|ticket| view! { <article class="ticket-card"><div><p class="eyebrow">{ticket.ticket_type_name}</p><strong>{ticket.public_reference}</strong><span>{ticket.holder_name.unwrap_or(ticket.holder_email_masked)}</span></div>{ticket.qr_svg.map(|svg| view! { <div class="mini-qr" inner_html=svg></div> })}<small>{format!("QR ważny do {}", human_time(&ticket.qr_expires_at))}</small></article> }).collect_view()}</div><button class="text-button" on:click=resend>"Wyślij bilety ponownie na e-mail"</button></article>
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
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let refresh = move |_| { refresh_fan_dashboard(dashboard, error); refresh_wallets(wallets, error); };
    let lock = move |_| spawn_local(async move { match bridge::invoke::<FanSessionStatus, _>("fan_lock", &EmptyArgs {}).await { Ok(value) => status.set(value), Err(message) => error.set(Some(message)) } });
    let forget = move |_| spawn_local(async move { match bridge::invoke::<FanSessionStatus, _>("fan_forget", &EmptyArgs {}).await { Ok(value) => status.set(value), Err(message) => error.set(Some(message)) } });
    view! {
        <section class="screen"><header class="screen-title"><p class="eyebrow">MÓJ PROFIL</p><h2>Ustawienia Sygnału</h2></header>{move || status.get().session.map(|profile| view! { <div class="profile-card"><div class="avatar">V</div><div><strong>{profile.display_name.unwrap_or_else(|| "Fan Viryi".to_owned())}</strong><p>{profile.email}</p></div></div><div class="stats-grid"><Metric value=profile.wallet_count.to_string() label="zamówienia"/><Metric value=if profile.has_admission_pass { "1".to_owned() } else { "0".to_owned() } label="wejściówki"/><Metric value=dashboard.get().map(|d| d.referral.qualified_referrals.to_string()).unwrap_or_else(|| "—".to_owned()) label="polecenia"/></div> })}<div class="settings-list"><button on:click=refresh>"Odśwież dane"</button><button on:click=lock>"Zablokuj aplikację"</button><button class="danger ghost" on:click=forget>"Usuń profil i bilety z urządzenia"</button></div><p class="security-note">Sesja fana, wejściówka oraz prywatne tokeny portfela są przechowywane w osobnym, zaszyfrowanym sejfie Stronghold.</p></section>
    }
}

#[component]
fn Skeleton() -> impl IntoView { view! { <div class="skeleton-stack"><i></i><i></i><i></i></div> } }

#[component]
fn Toast(error: RwSignal<Option<String>>) -> impl IntoView {
    view! { <Show when=move || error.get().is_some()><button class="toast" on:click=move |_| error.set(None)>{move || error.get().unwrap_or_default()}</button></Show> }
}

fn refresh_operator_dashboard(dashboard: RwSignal<Option<DashboardData>>, error: RwSignal<Option<String>>) {
    spawn_local(async move {
        match bridge::invoke::<DashboardData, _>("dashboard", &EmptyArgs {}).await {
            Ok(value) => dashboard.set(Some(value)), Err(message) => error.set(Some(message)),
        }
    });
}

fn refresh_fan_status(status: RwSignal<FanSessionStatus>, error: RwSignal<Option<String>>) {
    spawn_local(async move {
        match bridge::invoke::<FanSessionStatus, _>("fan_status", &EmptyArgs {}).await {
            Ok(value) => status.set(value), Err(message) => error.set(Some(message)),
        }
    });
}

fn refresh_fan_dashboard(dashboard: RwSignal<Option<FanDashboardData>>, error: RwSignal<Option<String>>) {
    spawn_local(async move {
        match bridge::invoke::<FanDashboardData, _>("fan_dashboard", &EmptyArgs {}).await {
            Ok(value) => dashboard.set(Some(value)), Err(message) => error.set(Some(message)),
        }
    });
}

fn refresh_wallets(wallets: RwSignal<Vec<TicketWallet>>, error: RwSignal<Option<String>>) {
    spawn_local(async move {
        match bridge::invoke::<Vec<TicketWallet>, _>("fan_wallets", &EmptyArgs {}).await {
            Ok(value) => wallets.set(value), Err(message) => error.set(Some(message)),
        }
    });
}

fn operator_events(dashboard: RwSignal<Option<DashboardData>>) -> Vec<PublicEvent> {
    dashboard.get().map(|data| data.events).unwrap_or_default()
}

fn operator_qr_events(dashboard: RwSignal<Option<DashboardData>>) -> Vec<crate::models::StaffEvent> {
    dashboard.get().and_then(|data| data.qr).map(|qr| qr.events).unwrap_or_default()
}

fn operator_campaigns(dashboard: RwSignal<Option<DashboardData>>) -> Vec<QrCampaign> {
    dashboard.get().and_then(|data| data.qr).map(|qr| qr.campaigns).unwrap_or_default()
}

fn fan_events(dashboard: RwSignal<Option<FanDashboardData>>, public: RwSignal<Option<PublicHomeData>>) -> Vec<PublicEvent> {
    dashboard.get().map(|data| data.events).or_else(|| public.get().map(|data| data.events)).unwrap_or_default()
}

fn optional(value: String) -> Option<String> {
    let value = value.trim().to_owned();
    (!value.is_empty()).then_some(value)
}

fn local_to_rfc3339(value: &str) -> Option<String> {
    if value.trim().is_empty() { return None; }
    let date = js_sys::Date::new(&JsValue::from_str(value));
    let time = date.get_time();
    if time.is_nan() { None } else { date.to_iso_string().as_string() }
}

fn money(minor: i64, currency: &str) -> String { format!("{:.2} {}", minor as f64 / 100.0, currency.to_uppercase()) }
fn human_time(value: &str) -> String { value.replace('T', " • ").replace('Z', "").chars().take(19).collect() }
fn event_location(event: &PublicEvent) -> String { event.venue.clone().or_else(|| event.city.as_ref().map(|city| city.name.clone())).unwrap_or_else(|| "Szczegóły wkrótce".to_owned()) }
fn event_time_location(starts_at: &str, venue: Option<&str>) -> String { format!("{} · {}", human_time(starts_at), venue.unwrap_or("miejsce wkrótce")) }
fn day(value: &str) -> String { value.get(8..10).unwrap_or("--").to_owned() }
fn month(value: &str) -> String {
    match value.get(5..7).unwrap_or("") { "01" => "STY", "02" => "LUT", "03" => "MAR", "04" => "KWI", "05" => "MAJ", "06" => "CZE", "07" => "LIP", "08" => "SIE", "09" => "WRZ", "10" => "PAŹ", "11" => "LIS", "12" => "GRU", _ => "---" }.to_owned()
}
