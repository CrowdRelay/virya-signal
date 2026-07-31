use leptos::prelude::*;
use serde::Serialize;
use wasm_bindgen_futures::spawn_local;

use crate::{
    bridge,
    models::{
        AdmissionRedemption, CouponEnvelope, DashboardData, IssuePassInput, IssuedPass,
        OperatorProfileInput, OperatorRole, PublicEvent, SessionStatus, TicketingOverview,
    },
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Tab {
    Home,
    Scan,
    Tickets,
    Discounts,
    Fan,
    Settings,
}

#[derive(Serialize)]
struct EmptyArgs {}

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

#[component]
pub fn App() -> impl IntoView {
    let status = RwSignal::new(SessionStatus::default());
    let loading = RwSignal::new(true);
    let error = RwSignal::new(None::<String>);
    let dashboard = RwSignal::new(None::<DashboardData>);
    let tab = RwSignal::new(Tab::Home);

    Effect::new(move |_| {
        spawn_local(async move {
            match bridge::invoke::<SessionStatus, _>("session_status", &EmptyArgs {}).await {
                Ok(value) => status.set(value),
                Err(message) => error.set(Some(message)),
            }
            loading.set(false);
        });
    });

    view! {
        <main class="app-shell">
            <Show
                when=move || !loading.get()
                fallback=move || view! { <Splash /> }
            >
                {move || {
                    if status.get().unlocked {
                        view! {
                            <AuthenticatedApp
                                status=status
                                dashboard=dashboard
                                tab=tab
                                error=error
                            />
                        }.into_any()
                    } else {
                        view! {
                            <AccessScreen status=status error=error />
                        }.into_any()
                    }
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
            <p>CONTROL / SIGNAL</p>
        </section>
    }
}

#[component]
fn AccessScreen(status: RwSignal<SessionStatus>, error: RwSignal<Option<String>>) -> impl IntoView {
    let pin = RwSignal::new(String::new());
    let name = RwSignal::new("Wojciech".to_owned());
    let token = RwSignal::new(String::new());
    let api = RwSignal::new("https://signal-api.virya.music/v1/".to_owned());
    let role = RwSignal::new(OperatorRole::Owner);
    let busy = RwSignal::new(false);

    let unlock = move |_| {
        if pin.get().len() < 6 {
            error.set(Some("PIN musi mieć co najmniej 6 znaków.".to_owned()));
            return;
        }
        busy.set(true);
        spawn_local(async move {
            let current_pin = pin.get_untracked();
            match bridge::invoke::<SessionStatus, _>("unlock", &PinArgs { pin: &current_pin }).await
            {
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
        if current_pin.len() < 6 || profile.bearer_token.trim().len() < 24 {
            error.set(Some(
                "Podaj PIN (min. 6 znaków) i poprawny token urządzenia.".to_owned(),
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
                Ok(value) => status.set(value),
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    view! {
        <section class="access-screen">
            <header class="hero compact">
                <p class="eyebrow">PRIVATE BAND OPERATIONS</p>
                <h1>Virya <em>Control</em></h1>
                <p>Wejście, bilety, zniżki i koncertowy chaos - w jednym miejscu.</p>
            </header>

            <div class="access-card">
                <Show
                    when=move || status.get().configured
                    fallback=move || view! {
                        <div class="form-grid">
                            <label>"Nazwa urządzenia / osoby"<input prop:value=move || name.get() on:input=move |e| name.set(event_target_value(&e)) /></label>
                            <label>"API"<input prop:value=move || api.get() on:input=move |e| api.set(event_target_value(&e)) /></label>
                            <div class="segmented">
                                <button class:active=move || role.get() == OperatorRole::Owner on:click=move |_| role.set(OperatorRole::Owner)>"OWNER"</button>
                                <button class:active=move || role.get() == OperatorRole::Staff on:click=move |_| role.set(OperatorRole::Staff)>"STAFF"</button>
                            </div>
                            <label>"Token CrowdRelay"<textarea rows="3" prop:value=move || token.get() on:input=move |e| token.set(event_target_value(&e))></textarea></label>
                            <label>"Lokalny PIN"<input type="password" inputmode="numeric" autocomplete="new-password" prop:value=move || pin.get() on:input=move |e| pin.set(event_target_value(&e)) /></label>
                            <button class="primary" disabled=move || busy.get() on:click=configure>{move || if busy.get() { "ŁĄCZĘ…" } else { "SPARUJ URZĄDZENIE" }}</button>
                        </div>
                    }
                >
                    <div class="form-grid">
                        <p class="lock-copy">"Profil urządzenia jest zaszyfrowany lokalnie. Podaj PIN, żeby odblokować sesję."</p>
                        <label>"PIN"<input type="password" inputmode="numeric" autocomplete="current-password" prop:value=move || pin.get() on:input=move |e| pin.set(event_target_value(&e)) /></label>
                        <button class="primary" disabled=move || busy.get() on:click=unlock>{move || if busy.get() { "ODBLOKOWUJĘ…" } else { "ODBLOKUJ" }}</button>
                    </div>
                </Show>
            </div>
        </section>
    }
}

#[component]
fn AuthenticatedApp(
    status: RwSignal<SessionStatus>,
    dashboard: RwSignal<Option<DashboardData>>,
    tab: RwSignal<Tab>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    Effect::new(move |_| {
        if status.get().unlocked && dashboard.get().is_none() {
            spawn_local(async move {
                match bridge::invoke::<DashboardData, _>("dashboard", &EmptyArgs {}).await {
                    Ok(value) => dashboard.set(Some(value)),
                    Err(message) => error.set(Some(message)),
                }
            });
        }
    });

    let role = move || {
        status
            .get()
            .session
            .map(|s| s.role)
            .unwrap_or(OperatorRole::Staff)
    };

    view! {
        <section class="authenticated">
            <header class="topbar">
                <div>
                    <p class="eyebrow">VIRYA CONTROL</p>
                    <strong>{move || status.get().session.map(|s| s.display_name).unwrap_or_default()}</strong>
                </div>
                <span class="role-pill">{move || role().label()}</span>
            </header>

            <div class="content">
                {move || match tab.get() {
                    Tab::Home => view! { <Home dashboard=dashboard /> }.into_any(),
                    Tab::Scan => view! { <Scanner dashboard=dashboard error=error /> }.into_any(),
                    Tab::Tickets => view! { <Tickets dashboard=dashboard error=error owner=Signal::derive(move || role() == OperatorRole::Owner) /> }.into_any(),
                    Tab::Discounts => view! { <Discounts error=error /> }.into_any(),
                    Tab::Fan => view! { <FanSection dashboard=dashboard /> }.into_any(),
                    Tab::Settings => view! { <Settings status=status dashboard=dashboard error=error /> }.into_any(),
                }}
            </div>

            <nav class="bottom-nav">
                <NavButton tab=tab own=Tab::Home icon="⌁" label="Start" />
                <NavButton tab=tab own=Tab::Scan icon="▣" label="Skan" />
                <NavButton tab=tab own=Tab::Tickets icon="▤" label="Bilety" />
                <NavButton tab=tab own=Tab::Discounts icon="%" label="Zniżki" />
                <NavButton tab=tab own=Tab::Fan icon="◉" label="Fan" />
                <NavButton tab=tab own=Tab::Settings icon="⚙" label="Opcje" />
            </nav>
        </section>
    }
}

#[component]
fn NavButton(
    tab: RwSignal<Tab>,
    own: Tab,
    icon: &'static str,
    label: &'static str,
) -> impl IntoView {
    view! {
        <button class:active=move || tab.get() == own on:click=move |_| tab.set(own)>
            <span>{icon}</span><small>{label}</small>
        </button>
    }
}

#[component]
fn Home(dashboard: RwSignal<Option<DashboardData>>) -> impl IntoView {
    view! {
        <section class="screen">
            <header class="screen-title"><p class="eyebrow">LIVE OPERATIONS</p><h2>Co gramy dalej?</h2></header>
            {move || match dashboard.get() {
                None => view! { <Skeleton /> }.into_any(),
                Some(data) => {
                    let next = data.events.first().cloned();
                    let active_campaigns = data.qr.as_ref().map(|q| q.campaigns.iter().filter(|c| c.active).count()).unwrap_or(0);
                    view! {
                        <div class="hero-card">
                            <p class="eyebrow">NAJBLIŻSZY KONCERT</p>
                            <h3>{next.as_ref().map(|e| e.title.clone()).unwrap_or_else(|| "Brak opublikowanego koncertu".to_owned())}</h3>
                            <p>{next.as_ref().and_then(|e| e.venue.clone()).unwrap_or_else(|| "Dodaj wydarzenie w CrowdRelay".to_owned())}</p>
                            <time>{next.as_ref().map(|e| human_time(&e.starts_at)).unwrap_or_default()}</time>
                        </div>
                        <div class="stats-grid">
                            <Metric value=data.events.len().to_string() label="koncerty" />
                            <Metric value=active_campaigns.to_string() label="aktywne QR" />
                            <Metric value=data.qr.as_ref().map(|q| q.campaigns.iter().map(|c| c.checkin_count).sum::<u64>()).unwrap_or(0).to_string() label="check-in" />
                        </div>
                        <div class="section-head"><h3>Koncerty</h3><span>{data.events.len()}</span></div>
                        <div class="card-list">
                            {data.events.into_iter().map(|event| view! { <EventCard event=event /> }).collect_view()}
                        </div>
                    }.into_any()
                }
            }}
        </section>
    }
}

#[component]
fn Metric(value: String, label: &'static str) -> impl IntoView {
    view! { <article class="metric"><strong>{value}</strong><span>{label}</span></article> }
}

#[component]
fn EventCard(event: PublicEvent) -> impl IntoView {
    let city = event
        .city
        .as_ref()
        .map(|c| c.name.clone())
        .unwrap_or_default();
    view! {
        <article class="event-card">
            <div class="date-block"><strong>{day(&event.starts_at)}</strong><span>{month(&event.starts_at)}</span></div>
            <div><h4>{event.title}</h4><p>{event.venue.unwrap_or(city)}</p></div>
            <span class="chevron">></span>
        </article>
    }
}

#[component]
fn Scanner(
    dashboard: RwSignal<Option<DashboardData>>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let event_slug = RwSignal::new(String::new());
    let manual = RwSignal::new(String::new());
    let result = RwSignal::new(None::<AdmissionRedemption>);
    let busy = RwSignal::new(false);

    Effect::new(move |_| {
        if event_slug.get().is_empty() {
            if let Some(first) = dashboard.get().and_then(|d| d.events.first().cloned()) {
                event_slug.set(first.slug);
            }
        }
    });

    let redeem_code = move |code: String| {
        let event = event_slug.get();
        if event.is_empty() || code.trim().is_empty() {
            error.set(Some("Wybierz koncert i podaj kod.".to_owned()));
            return;
        }
        busy.set(true);
        result.set(None);
        spawn_local(async move {
            match bridge::invoke::<AdmissionRedemption, _>(
                "redeem_admission",
                &RedeemArgs {
                    event_slug: &event,
                    code: &code,
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

    let scan = move |_| {
        busy.set(true);
        spawn_local(async move {
            match bridge::scan_qr().await {
                Ok(code) => redeem_code(code),
                Err(message) => {
                    error.set(Some(message));
                    busy.set(false);
                }
            }
        });
    };
    let manual_redeem = move |_| redeem_code(manual.get());

    view! {
        <section class="screen scanner-screen">
            <header class="screen-title"><p class="eyebrow">GATE MODE</p><h2>Skanuj wejście</h2></header>
            <label class="select-label">"Koncert"
                <select prop:value=move || event_slug.get() on:change=move |e| event_slug.set(event_target_value(&e))>
                    {move || dashboard.get().map(|d| d.events.into_iter().map(|e| view! { <option value=e.slug.clone()>{e.title}</option> }).collect_view())}
                </select>
            </label>
            <button class="scanner-button" disabled=move || busy.get() on:click=scan>
                <span class="scanner-frame"></span>
                <strong>{move || if busy.get() { "WERYFIKUJĘ…" } else { "OTWÓRZ SKANER QR" }}</strong>
                <small>kamera + natychmiastowy wynik</small>
            </button>
            <div class="manual-row">
                <input placeholder="VIRYA-… / kod ręczny" prop:value=move || manual.get() on:input=move |e| manual.set(event_target_value(&e)) />
                <button on:click=manual_redeem>"SPRAWDŹ"</button>
            </div>
            {move || result.get().map(|value| {
                let success = value.status == "redeemed";
                view! {
                    <article class:scan-success=success class:scan-warning=!success class="scan-result">
                        <strong>{if success { "WEJŚCIE OK" } else { "SPRAWDŹ STATUS" }}</strong>
                        <span>{value.public_reference}</span>
                        <p>{value.holder_name.unwrap_or(value.holder_email_masked)}</p>
                        <small>{value.status}</small>
                    </article>
                }
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
    let fan_email = RwSignal::new(String::new());
    let pool_slug = RwSignal::new("paid-tickets".to_owned());
    let issued = RwSignal::new(None::<IssuedPass>);
    let revoke_ref = RwSignal::new(String::new());
    let busy = RwSignal::new(false);

    let load = move |_| {
        let event = event_slug.get();
        if event.is_empty() {
            return;
        }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<TicketingOverview, _>(
                "ticketing_overview",
                &EventArgs { event_slug: &event },
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
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<serde_json::Value, _>(
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
            <div class="toolbar">
                <select prop:value=move || event_slug.get() on:change=move |e| event_slug.set(event_target_value(&e))>
                    <option value="">"Wybierz koncert"</option>
                    {move || dashboard.get().map(|d| d.events.into_iter().map(|e| view! { <option value=e.slug.clone()>{e.title}</option> }).collect_view())}
                </select>
                <button on:click=load disabled=move || busy.get()>"ODŚWIEŻ"</button>
            </div>
            {move || overview.get().map(|data| view! {
                <div class="stats-grid wide">
                    <Metric value=data.paid_tickets.to_string() label="sprzedane" />
                    <Metric value=data.sale.available.to_string() label="dostępne" />
                    <Metric value=money(data.gross_sales_minor, &data.sale.currency) label="obrót" />
                </div>
                <div class="section-head"><h3>Ostatnie zamówienia</h3><span>{data.recent_orders.len()}</span></div>
                <div class="card-list">
                    {data.recent_orders.into_iter().map(|order| view! {
                        <article class="order-card"><div><strong>{order.public_reference}</strong><p>{order.buyer_name.unwrap_or(order.buyer_email_masked)}</p></div><span>{money(order.amount_gross_minor, &order.currency)}</span></article>
                    }).collect_view()}
                </div>
            })}
            <Show when=move || owner.get()>
                <div class="admin-box">
                    <p class="eyebrow">OWNER ONLY</p><h3>Ręczna wejściówka</h3>
                    <input placeholder="fan@email.com" prop:value=move || fan_email.get() on:input=move |e| fan_email.set(event_target_value(&e)) />
                    <input placeholder="pool slug" prop:value=move || pool_slug.get() on:input=move |e| pool_slug.set(event_target_value(&e)) />
                    <button class="primary" on:click=issue disabled=move || busy.get()>"WYDAJ WEJŚCIÓWKĘ"</button>
                    {move || issued.get().map(|pass| view! { <p class="success-copy">{format!("{} — token wygenerowany", pass.public_reference)}</p> })}
                    <hr />
                    <input placeholder="public reference do unieważnienia" prop:value=move || revoke_ref.get() on:input=move |e| revoke_ref.set(event_target_value(&e)) />
                    <button class="danger" on:click=revoke disabled=move || busy.get()>"UNIEWAŻNIJ"</button>
                </div>
            </Show>
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
        <section class="screen">
            <header class="screen-title"><p class="eyebrow">MERCH DESK</p><h2>Realizuj zniżkę</h2></header>
            <div class="coupon-visual"><span>%</span><div><strong>VIRYA SIGNAL</strong><p>kupon fanowski / jednorazowe użycie</p></div></div>
            <div class="form-grid panel">
                <label>"Kod zniżkowy"<input autocapitalize="characters" placeholder="VIRYA-…" prop:value=move || code.get() on:input=move |e| code.set(event_target_value(&e)) /></label>
                <label>"Numer zamówienia / sprzedaży"<input placeholder="MERCH-WRO-001" prop:value=move || order.get() on:input=move |e| order.set(event_target_value(&e)) /></label>
                <button class="primary" on:click=redeem disabled=move || busy.get()>"ZREALIZUJ KUPON"</button>
            </div>
            {move || result.get().map(|envelope| view! {
                <article class="scan-result scan-success"><strong>"KUPON ZREALIZOWANY"</strong><span>{envelope.result.status}</span><p>{format!("Użycie {}/{}", envelope.result.used_count, envelope.result.max_uses)}</p></article>
            })}
        </section>
    }
}

#[component]
fn FanSection(dashboard: RwSignal<Option<DashboardData>>) -> impl IntoView {
    view! {
        <section class="screen fan-screen">
            <header class="screen-title"><p class="eyebrow">PUBLIC LAYER</p><h2>Virya Signal</h2></header>
            <div class="fan-hero">
                <p class="eyebrow">MÓJ SYGNAŁ</p><h3>Koncerty. Nagrody. Dostęp.</h3>
                <p>Ten ekran jest już oddzielony od panelu zespołu. Kolejny etap podłączy konto fana, jego QR-y, polecenia i wallet biletów.</p>
                <button disabled>"DOŁĄCZ DO SIGNAL — ETAP 2"</button>
            </div>
            <div class="section-head"><h3>Nadchodzące</h3></div>
            <div class="card-list">
                {move || dashboard.get().map(|d| d.events.into_iter().map(|event| view! { <EventCard event=event /> }).collect_view())}
            </div>
            <div class="roadmap-mini"><span>01</span><p><strong>Konto fana</strong><br/>magic link + zgody + profil miasta</p></div>
            <div class="roadmap-mini"><span>02</span><p><strong>Wallet</strong><br/>bilety, wejściówki i obracane QR-y</p></div>
            <div class="roadmap-mini"><span>03</span><p><strong>Signal</strong><br/>polecenia, zniżki, AREA i nagrody</p></div>
        </section>
    }
}

#[component]
fn Settings(
    status: RwSignal<SessionStatus>,
    dashboard: RwSignal<Option<DashboardData>>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let busy = RwSignal::new(false);
    let refresh = move |_| {
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<DashboardData, _>("dashboard", &EmptyArgs {}).await {
                Ok(value) => dashboard.set(Some(value)),
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };
    let lock = move |_| {
        spawn_local(async move {
            let _ = bridge::invoke::<SessionStatus, _>("lock", &EmptyArgs {})
                .await
                .map(|s| status.set(s));
        })
    };
    let forget = move |_| {
        spawn_local(async move {
            let _ = bridge::invoke::<SessionStatus, _>("forget_device", &EmptyArgs {})
                .await
                .map(|s| status.set(s));
        })
    };
    view! {
        <section class="screen">
            <header class="screen-title"><p class="eyebrow">DEVICE</p><h2>Ustawienia</h2></header>
            <div class="settings-list">
                <article><div><strong>Połączenie</strong><p>{move || status.get().session.map(|s| s.api_base_url).unwrap_or_default()}</p></div><span class="online">ONLINE</span></article>
                <article><div><strong>Uprawnienia</strong><p>{move || status.get().session.map(|s| s.role.label().to_owned()).unwrap_or_default()}</p></div></article>
                <button on:click=refresh disabled=move || busy.get()>"Odśwież wszystkie dane"</button>
                <button on:click=lock>"Zablokuj aplikację"</button>
                <button class="danger ghost" on:click=forget>"Usuń profil z urządzenia"</button>
            </div>
            <p class="security-note">"Token operatora nie wraca do warstwy UI po odblokowaniu. Żądania do CrowdRelay wykonuje natywna warstwa Rust."</p>
        </section>
    }
}

#[component]
fn Skeleton() -> impl IntoView {
    view! { <div class="skeleton-stack"><i></i><i></i><i></i></div> }
}

#[component]
fn Toast(error: RwSignal<Option<String>>) -> impl IntoView {
    view! {
        <Show when=move || error.get().is_some()>
            <button class="toast" on:click=move |_| error.set(None)>{move || error.get().unwrap_or_default()}</button>
        </Show>
    }
}

fn money(minor: i64, currency: &str) -> String {
    format!("{:.2} {}", minor as f64 / 100.0, currency.to_uppercase())
}

fn human_time(value: &str) -> String {
    value.replace('T', " • ").chars().take(16).collect()
}

fn day(value: &str) -> String {
    value.get(8..10).unwrap_or("--").to_owned()
}
fn month(value: &str) -> String {
    match value.get(5..7).unwrap_or("") {
        "01" => "STY",
        "02" => "LUT",
        "03" => "MAR",
        "04" => "KWI",
        "05" => "MAJ",
        "06" => "CZE",
        "07" => "LIP",
        "08" => "SIE",
        "09" => "WRZ",
        "10" => "PAŹ",
        "11" => "LIS",
        "12" => "GRU",
        _ => "---",
    }
    .to_owned()
}
