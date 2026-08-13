#[component]
fn BackButton(mode: RwSignal<RootMode>) -> impl IntoView {
    view! { <button class="back-button" on:click=move |_| mode.set(RootMode::Fan)>{tr("back_signal")}</button> }
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
            {tr("are_you_on_the_staff")}
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
            error.set(Some(tr("enter_the_staff_password_used_in_the").to_owned()));
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
                {tr("back_signal")}
            </button>
            <header class="hero compact staff-gate-hero">
                <p class="eyebrow">"VIRYA / STAFF"</p>
                <h1>{tr("zone_prefix")}<em>{tr("team_zone_suffix")}</em></h1>
                <p>{tr("gate_sales_and_show_operations_access_is")}</p>
            </header>
            <div class="access-card staff-gate-card">
                <div class="staff-gate-lock" aria-hidden="true">"⌁"</div>
                <div>
                    <p class="eyebrow">{tr("staff_verification")}</p>
                    <h2>{tr("virya_panel_password")}</h2>
                    <p>{tr("use_the_same_password_as_in_qr")}</p>
                </div>
                <label>
                    {tr("staff_password")}
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
                    {move || if busy.get() { tr("checking") } else { tr("open_staff_zone") }}
                </button>
                <p class="staff-gate-note">{tr("password_is_verified_by_virya_music_and")}</p>
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
            view! { <StatusFailure mode=mode status_refresh=status_refresh label=tr("failed_to_read_the_staff_vault") show_back=true /> }.into_any()
        } else if status_loading.get() {
            view! { <AccessLoader mode=mode label=tr("checking_the_secure_vault") show_back=true /> }.into_any()
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
        if !new_operator_pin_is_valid(&current_pin) {
            error.set(Some(
                tr("pin_must_contain_at_least_4_characters").to_owned(),
            ));
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
            error.set(Some(tr("enter_a_4_6_digit_pin_and").to_owned()));
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
            error.set(Some(tr("enter_a_4_6_digit_pin_and_2").to_owned()));
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
                <h1>{tr("pair")}<em>{tr("device_label")}</em></h1>
                <p>{tr("no_retyping_the_api_role_or_long")}</p>
            </header>
            <div class="access-card">
                <Show when=move || status.get().configured fallback=move || view! {
                    <div class="pairing-flow">
                        <Show when=move || pairing.get().trim().is_empty() fallback=move || view! {
                            <div class="pairing-scanned">
                                <span class="pairing-ok">"✓"</span>
                                <strong>{tr("code_scanned")}</strong>
                                <small>{tr("enter_the_pin_below_and_tap_pair")}</small>
                            </div>
                        }>
                            <button class="pairing-scan primary" on:click=scan_pairing disabled=move || busy.get()>
                                <span class="pairing-qr">"▦"</span>
                                <strong>{move || if busy.get() { tr("connecting") } else { tr("scan_qr_code") }}</strong>
                                <small>{tr("code_shown_in_the_virya_panel")}</small>
                            </button>
                        </Show>
                        <div class="pairing-divider"><span></span><small>{tr("or_label")}</small><span></span></div>
                        <label>{tr("pairing_code")}<textarea rows="2" placeholder="virya-signal://pair?…" prop:value=move || pairing.get() on:input=move |e| pairing.set(event_target_value(&e))></textarea></label>
                        <label class="pin-field">
                            <span class="pin-field-label">{tr("create_an_unlock_pin")}</span>
                            <small id="operator-new-pin-help">{tr("enter_4_6_digits_for_example_2580")}</small>
                            <input type="password" autocomplete="new-password" inputmode="numeric" pattern="[0-9]*" maxlength="6" placeholder=tr("pin_example") aria-describedby="operator-new-pin-help" prop:value=move || pin.get() on:input=move |e| pin.set(normalize_new_operator_pin(event_target_value(&e))) />
                        </label>
                        <button class="primary" on:click=submit_pairing disabled=move || busy.get() || pairing.get().trim().is_empty() || !new_operator_pin_is_valid(&pin.get())>{tr("pair_2")}</button>
                        <button class="text-button" type="button" on:click=move |_| advanced.update(|v| *v = !*v)>
                            {move || if advanced.get() { tr("hide_manual_settings") } else { tr("advanced_settings") }}
                        </button>
                        <Show when=move || advanced.get()>
                            <div class="advanced-config">
                                <label>{tr("device_person_name")}<input prop:value=move || name.get() on:input=move |e| name.set(event_target_value(&e)) /></label>
                                <label>"API CrowdRelay"<input prop:value=move || api.get() on:input=move |e| api.set(event_target_value(&e)) /></label>
                                <div class="segmented">
                                    <button class:active=move || role.get() == OperatorRole::Owner on:click=move |_| role.set(OperatorRole::Owner)>{tr("owner")}</button>
                                    <button class:active=move || role.get() == OperatorRole::Staff on:click=move |_| role.set(OperatorRole::Staff)>{tr("staff")}</button>
                                </div>
                                <label>{tr("device_token")}<textarea rows="3" prop:value=move || token.get() on:input=move |e| token.set(event_target_value(&e))></textarea></label>
                                <button class="ghost" on:click=configure_manual disabled=move || busy.get() || !new_operator_pin_is_valid(&pin.get())>{tr("save_manually")}</button>
                            </div>
                        </Show>
                    </div>
                }>
                    <div class="form-grid">
                        <p class="lock-copy">{tr("operator_profile_is_encrypted_locally")}</p>
                        <label class="pin-field">
                            <span class="pin-field-label">{tr("app_unlock_pin")}</span>
                            <small id="operator-unlock-pin-help">{tr("enter_the_pin_created_when_this_device")}</small>
                            <input type="password" autocomplete="current-password" inputmode="numeric" placeholder=tr("your_pin") aria-describedby="operator-unlock-pin-help" prop:value=move || pin.get() on:input=move |e| pin.set(event_target_value(&e)) />
                        </label>
                        <button class="primary" disabled=move || busy.get() || pin.get().chars().count() < 4 on:click=unlock>{tr("unlock")}</button>
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
                    <button class="menu-trigger" aria-label=tr("open_menu") aria-expanded=move || menu_open.get() on:click=move |_| menu_open.update(|v| *v = !*v)><i></i><i></i><i></i></button>
                    <button aria-label=tr("close_and_lock_panel") on:click=close>"×"</button>
                </div>
            </header>
            <Show when=move || menu_open.get()>
                <div class="overflow-backdrop" on:click=move |_| menu_open.set(false)></div>
                <nav class="overflow-menu">
                    <button class:active=move || tab.get() == OperatorTab::Discounts on:click=move |_| { tab.set(OperatorTab::Discounts); menu_open.set(false); }><span>"%"</span>{tr("discounts")}</button>
                    <button class:active=move || tab.get() == OperatorTab::Campaigns on:click=move |_| { tab.set(OperatorTab::Campaigns); menu_open.set(false); }><span>"◫"</span>{tr("qr_codes")}</button>
                    <button class:active=move || tab.get() == OperatorTab::Settings on:click=move |_| { tab.set(OperatorTab::Settings); menu_open.set(false); }><span>"⚙"</span>{tr("settings")}</button>
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
                <NavButton tab=tab own=OperatorTab::Home icon="home" label=tr("home_tab") />
                <NavButton tab=tab own=OperatorTab::Signal icon="signal" label=tr("signal_tab") />
                <NavButton tab=tab own=OperatorTab::Scan icon="scan" label=tr("scan_tab") />
                <NavButton tab=tab own=OperatorTab::Tickets icon="ticket" label=tr("tickets_tab") />
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
            <header class="screen-title"><p class="eyebrow">LIVE OPERATIONS</p><h2>{tr("today_under_control")}</h2></header>
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
                            <article class="hero-card"><p class="eyebrow">{tr("next_show")}</p><h3>{title}</h3><p>{location}</p><time>{time}</time></article>
                        }
                    })}
                    <div class="stats-grid"><Metric value=event_count.to_string() label=tr("shows_count_label")/><Metric value=if qr_loading { "…".to_owned() } else { active.to_string() } label=tr("active_qr")/><Metric value=if qr_loading { "…".to_owned() } else { checkins.to_string() } label=tr("check_ins")/></div>
                    <div class="section-head"><h3>{tr("upcoming")}</h3><span>{event_count}</span></div>
                    {if events.is_empty() {
                        view! { <div class="empty-state"><strong>{tr("no_upcoming_shows")}</strong><p>{tr("new_events_will_appear_here")}</p></div> }.into_any()
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
                <h2>{tr("community_and_growth")}</h2>
                <p class="screen-copy">{tr("combined_signal_overview_without_fans_personal_data")}</p>
            </header>
            <Show
                when=move || owner.get()
                fallback=move || view! {
                    <div class="empty-state">
                        <strong>{tr("owner_only_view")}</strong>
                        <p>{tr("consent_growth_and_city_statistics_are_available")}</p>
                    </div>
                }
            >
                <div class="signal-admin-toolbar">
                    <p>{tr("data_is_aggregated_in_crowdrelay_and_contains")}</p>
                    <button class="text-button" on:click=refresh disabled=move || loading.get()>
                        {move || if loading.get() { tr("refreshing") } else { tr("refresh") }}
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
                                        <strong>{tr("no_signal_snapshot")}</strong>
                                        <p>{tr("refresh_the_data_if_the_backend_is")}</p>
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
    let audience = data.audience;
    let ticket_revenue = data.ticket_revenue;
    let has_ticket_revenue = !ticket_revenue.is_empty();
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
                {i18n::format("partial_snapshot_unavailable_sources", &[unavailable.join(", ").to_string()])}
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
                    <span>{i18n::format("active_2", &[city.active_fans.max(0).to_string()])}</span>
                </article>
            }
        })
        .collect_view();
    let cities_view = if city_count == 0 {
        view! {
            <div class="empty-state compact">
                <strong>{tr("no_city_aggregate")}</strong>
                <p>{tr("signal_has_no_confirmed_city_data_yet")}</p>
            </div>
        }
        .into_any()
    } else {
        view! { <div class="signal-city-list">{city_cards}</div> }.into_any()
    };

    view! {
        <div class="signal-admin-content">
            <div class="stats-grid">
                <Metric value=summary.active_fans.max(0).to_string() label=tr("active_status")/>
                <Metric value=summary.marketing_opted_in.max(0).to_string() label=tr("marketing_consents")/>
                <Metric value=activity.new_fans_30d.max(0).to_string() label=tr("new_30_days")/>
            </div>
            <article class="signal-health-card">
                <div>
                    <p class="eyebrow">{tr("database_health")}</p>
                    <strong>{confirmation_rate}</strong>
                    <span>{tr("confirmed_among_active_and_pending")}</span>
                </div>
                <dl>
                    <div><dt>{tr("all")}</dt><dd>{summary.total_fans.max(0)}</dd></div>
                    <div><dt>{tr("pending")}</dt><dd>{summary.pending_fans.max(0)}</dd></div>
                    <div><dt>{tr("unsubscribed")}</dt><dd>{summary.unsubscribed_fans.max(0)}</dd></div>
                    <div><dt>{tr("muted")}</dt><dd>{summary.suppressed_fans.max(0)}</dd></div>
                    <div><dt>{tr("nearby_notifications")}</dt><dd>{summary.nearby_enabled.max(0)}</dd></div>
                </dl>
            </article>
            <div class="section-head"><h3>{tr("activity")}</h3><span>{tr("text_30_days_total")}</span></div>
            <div class="signal-activity-grid">
                <article><strong>{activity.new_fans_7d.max(0)}</strong><span>{tr("new_7_days")}</span></article>
                <article><strong>{format!("{} / {}", activity.referral_attributions_30d.max(0), activity.referral_attributions_total.max(0))}</strong><span>{tr("referrals")}</span></article>
                <article><strong>{format!("{} / {}", activity.event_interests_30d.max(0), activity.event_interests_total.max(0))}</strong><span>{tr("show_interests")}</span></article>
                <article><strong>{activity.nearby_notifications_30d.max(0)}</strong><span>{tr("nearby_notifications_2")}</span></article>
                <article><strong>{activity.pending_city_requests.max(0)}</strong><span>{tr("cities_awaiting_moderation")}</span></article>
            </div>
            <div class="section-head"><h3>{tr("audience_intelligence")}</h3><span>{tr("fan_360_summary")}</span></div>
            <div class="signal-activity-grid">
                <article><strong>{audience.ticket_buyers.max(0)}</strong><span>{tr("ticket_buyers")}</span></article>
                <article><strong>{audience.attendees.max(0)}</strong><span>{tr("concert_attendees")}</span></article>
                <article><strong>{audience.synesthesia_participants.max(0)}</strong><span>{tr("synesthesia_participants")}</span></article>
                <article><strong>{audience.qualified_referrals.max(0)}</strong><span>{tr("qualified_referrals")}</span></article>
            </div>
            <Show when=move || has_ticket_revenue>
                <div class="section-head"><h3>{tr("ticket_revenue")}</h3><span>{tr("after_refunds")}</span></div>
                <div class="signal-city-list">
                    {ticket_revenue.clone().into_iter().map(|row| view! {
                        <article class="signal-city-card">
                            <div><strong>{money(row.after_refunds_minor, &row.currency)}</strong><small>{i18n::format("paid_orders_count", &[row.paid_orders.max(0).to_string()])}</small></div>
                            <span>{row.currency}</span>
                        </article>
                    }).collect_view()}
                </div>
            </Show>
            {degraded_view}
            <div class="section-head"><h3>{tr("strongest_cities")}</h3><span>{city_count}</span></div>
            {cities_view}
            <p class="security-note">{i18n::format("snapshot_generated_at_aggregated_data_only", std::slice::from_ref(&generated_at))}</p>
        </div>
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
    let event_snapshot = RwSignal::new(None::<StaffEventDashboard>);
    let busy = RwSignal::new(false);
    let fan_email = RwSignal::new(String::new());
    let pool_slug = RwSignal::new("tickets".to_owned());
    let revoke_ref = RwSignal::new(String::new());
    let issued = RwSignal::new(None::<IssuedPass>);

    let load = move |_| {
        let slug = event_slug.get();
        if slug.is_empty() {
            error.set(Some(tr("select_a_show").to_owned()));
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
            match bridge::invoke::<StaffEventDashboard, _>(
                "staff_event_dashboard",
                &EventArgs { event_slug: &slug },
            )
            .await
            {
                Ok(value) if value.has_supported_schema() => event_snapshot.set(Some(value)),
                Ok(value) => error.set(Some(i18n::format(
                    "unsupported_staff_snapshot_version",
                    &[value.schema_version.to_string()],
                ))),
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
            error.set(Some(tr("select_a_show_and_enter_the_fan").to_owned()));
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
            error.set(Some(
                tr("enter_the_admission_pass_public_reference").to_owned(),
            ));
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
                Ok(_) => error.set(Some(tr("admission_pass_has_been_revoked").to_owned())),
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    view! {
        <section class="screen">
            <header class="screen-title"><p class="eyebrow">TICKETING</p><h2>{tr("tickets_and_admission_passes")}</h2></header>
            <div class="toolbar"><select disabled=move || loading.get().events prop:value=move || event_slug.get() on:change=move |e| event_slug.set(event_target_value(&e))><option value="">{move || if loading.get().events { tr("loading_shows") } else { tr("select_a_show_2") }}</option>{move || operator_events(dashboard).into_iter().map(|event| view! { <option value=event.slug.clone()>{event.title}</option> }).collect_view()}</select><button on:click=load disabled=move || busy.get() || loading.get().events>{tr("refresh")}</button></div>
            {move || event_snapshot.get().map(|snapshot| view! {
                <article class="event-snapshot-heading">
                    <p class="eyebrow">{snapshot.slug}</p>
                    <h3>{snapshot.title}</h3>
                    <p>{event_time_location(&snapshot.starts_at, snapshot.venue.as_deref())}</p>
                </article>
                <div class="stats-grid wide event-context-stats">
                    <Metric value=snapshot.interested_fans.to_string() label=tr("interested")/>
                    <Metric value=snapshot.paid_orders.to_string() label=tr("paid_orders")/>
                    <Metric value=snapshot.paid_tickets.to_string() label=tr("sold")/>
                    <Metric value=snapshot.passes_issued.to_string() label=tr("passes_issued")/>
                    <Metric value=snapshot.passes_claimed.to_string() label=tr("claimed")/>
                    <Metric value=snapshot.passes_redeemed.to_string() label=tr("redeemed")/>
                </div>
            })}
            {move || overview.get().map(|data| view! {
                <div class="stats-grid wide"><Metric value=data.paid_tickets.to_string() label=tr("sold")/><Metric value=data.sale.reserved.to_string() label=tr("in_checkout")/><Metric value=data.sale.available.to_string() label=tr("available_label")/></div>
                <div class="revenue-card"><p>{tr("gross_revenue")}</p><strong>{money(data.gross_sales_minor, &data.sale.currency)}</strong><span>{format!("zwroty: {}", money(data.refunded_minor, &data.sale.currency))}</span></div>
                <div class="section-head"><h3>{tr("recent_orders")}</h3><span>{data.recent_orders.len()}</span></div>
                <div class="card-list">{data.recent_orders.into_iter().map(|order| view! { <article class="order-card"><div><strong>{order.public_reference}</strong><p>{order.buyer_name.value_or(order.buyer_email_masked)}</p></div><span>{money(order.amount_gross_minor, &order.currency)}</span></article> }).collect_view()}</div>
            })}
            <Show when=move || owner.get()><div class="admin-box"><p class="eyebrow">OWNER ONLY</p><h3>{tr("manual_admission_pass")}</h3><p class="inline-note">{tr("admission_pass_number_is_a_safe_public")}</p><input placeholder="fan@email.com" prop:value=move || fan_email.get() on:input=move |e| fan_email.set(event_target_value(&e))/><input placeholder="pool slug" prop:value=move || pool_slug.get() on:input=move |e| pool_slug.set(event_target_value(&e))/><button class="primary" on:click=issue disabled=move || busy.get()>{tr("issue_pass")}</button>{move || issued.get().map(|pass| view! { <div class="token-box"><strong>{pass.public_reference}</strong><p>Token roszczenia: {pass.claim_token}</p></div> })}<hr/><input placeholder=tr("admission_pass_number_e_g_vry") prop:value=move || revoke_ref.get() on:input=move |e| revoke_ref.set(event_target_value(&e))/><button class="danger" on:click=revoke disabled=move || busy.get()>{tr("revoke")}</button></div></Show>
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
            error.set(Some(tr("enter_the_code_and_sale_number").to_owned()));
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
        <section class="screen"><header class="screen-title"><p class="eyebrow">MERCH DESK</p><h2>{tr("redeem_a_discount")}</h2></header><div class="coupon-visual"><span>%</span><div><strong>{tr("virya_signal")}</strong><p>{tr("fan_coupon_controlled_use")}</p></div></div><div class="form-grid panel"><label>{tr("discount_code")}<input autocapitalize="characters" placeholder="VIRYA-…" prop:value=move || code.get() on:input=move |e| code.set(event_target_value(&e))/></label><label>{tr("sale_number")}<input placeholder="MERCH-WRO-001" prop:value=move || order.get() on:input=move |e| order.set(event_target_value(&e))/></label><button class="primary" on:click=redeem disabled=move || busy.get()>{tr("redeem_coupon")}</button></div>{move || result.get().map(|envelope| view! { <article class="scan-result scan-success"><strong>{tr("coupon_redeemed")}</strong><span>{envelope.result.status}</span><p>{i18n::format("usage", &[envelope.result.used_count.to_string(), envelope.result.max_uses.to_string()])}</p></article> })}</section>
    }
}

#[component]
fn Campaigns(
    dashboard: RwSignal<Option<DashboardData>>,
    loading: RwSignal<OperatorLoadingState>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let event_slug = RwSignal::new(String::new());
    let label = RwSignal::new(tr("main_entrance").to_owned());
    let valid_from = RwSignal::new(String::new());
    let valid_until = RwSignal::new(String::new());
    let max_checkins = RwSignal::new(String::new());
    let busy = RwSignal::new(false);

    let create = move |_| {
        let Some(from) = local_to_rfc3339(&valid_from.get()) else {
            error.set(Some(tr("enter_a_valid_start_date").to_owned()));
            return;
        };
        let Some(until) = local_to_rfc3339(&valid_until.get()) else {
            error.set(Some(tr("enter_a_valid_end_date").to_owned()));
            return;
        };
        let max_value = max_checkins.get();
        let max = if max_value.trim().is_empty() {
            None
        } else {
            match max_value.trim().parse::<u32>() {
                Ok(value) if value > 0 => Some(value),
                _ => {
                    error.set(Some(tr("limit_must_be_a_positive_number").to_owned()));
                    return;
                }
            }
        };
        if until <= from {
            error.set(Some(tr("campaign_end_must_be_after_its_start").to_owned()));
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
            error.set(Some(tr("select_a_show_and_name_the_campaign").to_owned()));
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
                    error.set(Some(tr("qr_campaign_created").to_owned()));
                    refresh_operator_qr(dashboard, loading, error);
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    view! {
        <section class="screen"><header class="screen-title"><p class="eyebrow">CONCERT SIGNAL</p><h2>{tr("qr_campaigns")}</h2></header><div class="form-grid panel"><label>{tr("show")}<select disabled=move || loading.get().qr prop:value=move || event_slug.get() on:change=move |e| event_slug.set(event_target_value(&e))><option value="">{move || if loading.get().qr { tr("loading_campaigns") } else { tr("select_a_show_2") }}</option><For each=move || operator_qr_events(dashboard) key=|event| event.slug.clone() children=move |event| view! { <option value=event.slug.clone()>{event.title}</option> } /></select></label><label>{tr("point_campaign_name")}<input prop:value=move || label.get() on:input=move |e| label.set(event_target_value(&e))/></label><div class="two-cols"><label>{tr("valid_from")}<input type="datetime-local" prop:value=move || valid_from.get() on:input=move |e| valid_from.set(event_target_value(&e))/></label><label>{tr("valid_until")}<input type="datetime-local" prop:value=move || valid_until.get() on:input=move |e| valid_until.set(event_target_value(&e))/></label></div><label>{tr("check_in_limit_optional")}<input inputmode="numeric" prop:value=move || max_checkins.get() on:input=move |e| max_checkins.set(event_target_value(&e))/></label><button class="primary" on:click=create disabled=move || busy.get() || loading.get().qr>{tr("create_campaign")}</button></div><div class="section-head"><h3>{tr("active_and_historical")}</h3></div><Show when=move || !loading.get().qr fallback=move || view! { <Skeleton /> }><div class="card-list"><For each=move || operator_campaigns(dashboard) key=|campaign| campaign.id.clone() children=move |campaign| view! { <CampaignCard campaign=campaign dashboard=dashboard loading=loading error=error /> } /></div></Show></section>
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
                    error.set(Some(tr("campaign_has_been_disabled").to_owned()));
                    refresh_operator_qr(dashboard, loading, error);
                }
                Err(message) => error.set(Some(message)),
            }
        });
    };
    let revoke_button = if active {
        Some(view! {
            <button class="danger ghost" on:click=revoke>
                {tr("disable_campaign")}
            </button>
        })
    } else {
        None
    };
    view! {
        <article class="campaign-card"><div class="campaign-head"><div><strong>{campaign.label}</strong><p>{campaign.event_title}</p></div><span class:online=active class:offline=!active>{if active { tr("active_status") } else { tr("closed") }}</span></div><div class="campaign-stats"><span>{i18n::format("check_ins_2", &[campaign.checkin_count.to_string()])}</span><span>{campaign.max_checkins.map(|v| i18n::format("limit_v", &[v.to_string()])).value_or_else(|| tr("no_limit").to_owned())}</span></div>{campaign.token.map(|token| view! { <code>{token}</code> })}{revoke_button}</article>
    }
}

#[component]
fn LanguageSwitch() -> impl IntoView {
    let selected = i18n::current();
    view! {
        <article class="language-setting">
            <div>
                <strong>{tr("app_language")}</strong>
                <p>{tr("changing_the_language_reloads_the_interface_your")}</p>
            </div>
            <div class="language-switch" role="group" aria-label=tr("language")>
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
    let autopilot = RwSignal::new(None::<OperatorAutopilotOverview>);
    let autopilot_loading = RwSignal::new(false);
    let autopilot_chief = RwSignal::new(None::<AutopilotChiefOfStaff>);
    let autopilot_chief_loading = RwSignal::new(false);
    let ops = RwSignal::new(None::<OperatorOpsOverview>);
    let ops_loading = RwSignal::new(false);
    let owner = Signal::derive(move || {
        status
            .get()
            .session
            .is_some_and(|session| session.role == OperatorRole::Owner)
    });
    Effect::new(move |_| {
        if owner.get() {
            if autopilot.get().is_none() && !autopilot_loading.get() {
                refresh_operator_autopilot(autopilot, autopilot_loading, error);
            }
            if autopilot_chief.get().is_none() && !autopilot_chief_loading.get() {
                refresh_operator_chief(autopilot_chief, autopilot_chief_loading, error);
            }
            if ops.get().is_none() && !ops_loading.get() {
                refresh_operator_ops(ops, ops_loading, error);
            }
        }
    });
    let refresh = move |_| {
        refresh_operator_parts(dashboard, loading, error);
        if owner.get_untracked() {
            refresh_operator_autopilot(autopilot, autopilot_loading, error);
            refresh_operator_chief(autopilot_chief, autopilot_chief_loading, error);
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
            <header class="screen-title"><p class="eyebrow">{tr("device_label")}</p><h2>{tr("settings")}</h2></header>
            <div class="settings-list">
                <LanguageSwitch />
                <article><div><strong>{tr("connection")}</strong><p>{move || status.get().session.map(|s| s.api_base_url).value_or_else(Default::default)}</p></div><span class:online=move || !loading.get().events && !loading.get().qr>{move || if loading.get().events || loading.get().qr { tr("connecting_2") } else { tr("online") }}</span></article>
                <article><div><strong>{tr("permissions")}</strong><p>{move || status.get().session.map(|s| s.role.label().to_owned()).value_or_else(Default::default)}</p></div></article>
                <button on:click=refresh disabled=move || loading.get().events || loading.get().qr>{tr("refresh_all_data")}</button>
                <button on:click=lock>{tr("lock_panel")}</button>
                <button class="danger ghost" on:click=forget>{tr("remove_operator_profile")}</button>
            </div>
            <Show when=move || owner.get()>
                <AutopilotPanel overview=autopilot loading=autopilot_loading chief=autopilot_chief chief_loading=autopilot_chief_loading error=error />
                <OpsPanel overview=ops loading=ops_loading error=error />
            </Show>
            <AnonymousFeedback error=error />
            <p class="security-note">{tr("operator_token_is_stored_in_an_encrypted")}</p>
        </section>
    }
}


#[component]
fn AutopilotPanel(
    overview: RwSignal<Option<OperatorAutopilotOverview>>,
    loading: RwSignal<bool>,
    chief: RwSignal<Option<AutopilotChiefOfStaff>>,
    chief_loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let refresh = move |_| {
        refresh_operator_autopilot(overview, loading, error);
        refresh_operator_chief(chief, chief_loading, error);
    };
    view! {
        <section class="ops-panel autopilot-panel">
            <div class="section-head">
                <div><p class="eyebrow">VIRYAOS AUTOPILOT</p><h3>{tr("autopilot_control")}</h3></div>
                <button class="text-button" on:click=refresh disabled=move || loading.get()>{tr("refresh_2")}</button>
            </div>
            <Show when=move || !chief_loading.get() fallback=move || view! { <Skeleton rows=2 /> }>
                {move || chief.get().map(|brief| {
                    let attention = brief.attention_items.into_iter().take(6).collect::<Vec<_>>();
                    let opportunities = brief.top_opportunities.into_iter().take(5).collect::<Vec<_>>();
                    let show_tasks = brief.show_tasks.into_iter().take(5).collect::<Vec<_>>();
                    let attention_view = (!attention.is_empty()).then(|| view! {
                        <div class="section-head"><h3>{tr("autopilot_deadline_radar")}</h3></div>
                        <div class="ops-list"><For each=move || attention.clone() key=|item| format!("{}:{}:{}", item.kind, item.subject_kind, item.subject_id) children=move |item| {
                            let title = if item.kind == "approval" { autopilot_action_kind_label(&item.title).to_string() } else { item.title.clone() };
                            let detail = if item.kind == "approval" { autopilot_context_label(&item.detail).to_string() } else { item.detail.clone() };
                            let due_at = human_time(&item.due_at);
                            view! { <article class=format!("ops-item autopilot-attention attention-{}", item.urgency)><div><strong>{title}</strong><p>{detail}</p><small>{format!("{} · {} · {}", autopilot_attention_label(&item.kind), autopilot_urgency_label(&item.urgency), due_at)}</small></div></article> }
                        } /></div>
                    });
                    let opportunities_view = (!opportunities.is_empty()).then(|| view! {
                        <div class="section-head"><h3>{tr("autopilot_opportunities")}</h3></div>
                        <div class="ops-list"><For each=move || opportunities.clone() key=|item| format!("{}:{}:{}", item.context, item.subject_kind, item.subject_id) children=move |item| view! {
                            <article class="ops-item"><div><strong>{autopilot_context_label(&item.context)}</strong><p>{item.reason}</p><small>{format!("{}% · {}", item.confidence / 100, if item.needs_approval { tr("autopilot_approval") } else { item.decision_kind.as_str() })}</small></div></article>
                        } /></div>
                    });
                    let show_tasks_view = (!show_tasks.is_empty()).then(|| view! {
                        <div class="section-head"><h3>{tr("autopilot_show_tasks")}</h3></div>
                        <div class="ops-list"><For each=move || show_tasks.clone() key=|item| format!("{}:{}", item.event_id, item.task_key) children=move |item| view! {
                            <article class="ops-item"><div><strong>{item.event_title}</strong><p>{autopilot_show_task_label(&item.task_key)}</p><small>{item.status}</small></div></article>
                        } /></div>
                    });
                    view! {
                        <div class="autopilot-chief">
                            <div class="section-head"><h3>{tr("autopilot_chief")}</h3><span>{format!("{}", brief.needs_you)}</span></div>
                            <div class="ops-metrics">
                                <Metric value=brief.executed_24h.to_string() label=tr("autopilot_actions_24h") />
                                <Metric value=format!("~{}m", brief.estimated_minutes_saved_24h) label=tr("autopilot_time_saved") />
                                <Metric value=brief.measured_improved_7d.to_string() label=tr("autopilot_improved_7d") />
                                <Metric value=brief.needs_you.to_string() label=tr("autopilot_needs_you") />
                            </div>
                            {attention_view}
                            {opportunities_view}
                            {show_tasks_view}
                        </div>
                    }
                })}
            </Show>
            <Show when=move || !loading.get() fallback=move || view! { <Skeleton rows=3 /> }>
                {move || overview.get().map(|data| {
                    let runtime_enabled = data.runtime_enabled;
                    let policies = data.policies;
                    let promotion_budget_guardrails = data.promotion_budget_guardrails;
                    let needs_you = data.needs_you;
                    let available_assignees = data.available_assignees;
                    let recent_actions = data.recent_actions.into_iter().take(10).collect::<Vec<_>>();
                    let recent_effects = data.recent_effects.into_iter().take(6).collect::<Vec<_>>();
                    let queue = data.queued_actions.saturating_add(data.processing_actions);
                    let release_ledger = data.release_ledger;
                    let rum_metrics = data.rum_metrics_24h;
                    let release_drift = release_ledger.backend_sha_drift
                        || !release_ledger.missing_components.is_empty()
                        || release_ledger.active_executor_count == 0
                        || release_ledger.guarded_executor_count > 0
                        || release_ledger.active_executor_manifest_shas.len() > 1
                        || release_ledger.components.iter().any(|component| component.stale);
                    let release_components = release_ledger.components.clone();
                    let release_missing = release_ledger.missing_components.join(", ");
                    let release_view = view! {
                        <div class="section-head"><h3>{tr("autopilot_release_ledger")}</h3><span>{if release_drift { tr("autopilot_release_drift") } else { tr("autopilot_release_sync") }}</span></div>
                        <div class="ops-metrics">
                            <Metric value=release_ledger.active_executor_count.to_string() label=tr("autopilot_n8n_executors") />
                            <Metric value=release_ledger.guarded_executor_count.to_string() label=tr("autopilot_executor_guards") />
                            <Metric value=release_missing.clone() label=tr("autopilot_release_missing") />
                        </div>
                        <div class="ops-list"><For each=move || release_components.clone() key=|component| component.component_key.clone() children=move |component| view! {
                            <article class="ops-item"><div>
                                <strong>{component.component_key}</strong>
                                <p>{component.version.unwrap_or_else(|| component.source_sha.chars().take(12).collect())}</p>
                                <small>{if component.stale { tr("autopilot_release_stale") } else { tr("autopilot_release_production") }}</small>
                            </div></article>
                        } /></div>
                    };
                    let rum_view = (!rum_metrics.is_empty()).then(|| view! {
                        <div class="section-head"><h3>{tr("autopilot_rum_24h")}</h3><span>{rum_metrics.len()}</span></div>
                        <div class="ops-list"><For each=move || rum_metrics.clone() key=|metric| format!("{}:{}", metric.surface, metric.metric_key) children=move |metric| view! {
                            <article class="ops-item"><div>
                                <strong>{autopilot_rum_metric_label(&metric.surface, &metric.metric_key)}</strong>
                                <p>{autopilot_rum_value(&metric.metric_key, metric.p75, metric.p95)}</p>
                                <small>{format!("{} {}", metric.samples_24h, tr("autopilot_samples"))}</small>
                            </div></article>
                        } /></div>
                    });
                    let runtime_view = (!runtime_enabled).then(|| view! {
                        <p class="security-note warning">{tr("autopilot_runtime_disabled")}</p>
                    });
                    let needs_view = if needs_you.is_empty() {
                        view! { <p class="ops-healthy">{tr("autopilot_nothing_needs_you")}</p> }.into_any()
                    } else {
                        view! {
                            <div class="section-head"><h3>{tr("autopilot_needs_you")}</h3><span>{needs_you.len()}</span></div>
                            <div class="ops-list">
                                <For each=move || needs_you.clone() key=|action| action.id.clone() children={
                                    let available_assignees = available_assignees.clone();
                                    move |action| view! {
                                        <AutopilotPendingCard action=action available_assignees=available_assignees.clone() overview=overview loading=loading error=error />
                                    }
                                } />
                            </div>
                        }.into_any()
                    };
                    let effects_view = (!recent_effects.is_empty()).then(|| view! {
                        <div class="section-head"><h3>{tr("autopilot_measured_effects")}</h3></div>
                        <div class="ops-list"><For each=move || recent_effects.clone() key=|effect| effect.measurement_id.clone() children=move |effect| {
                            let delta = effect.delta_basis_points as f64 / 100.0;
                            view! {
                                <article class="ops-item"><div>
                                    <strong>{autopilot_context_label(&effect.context)}</strong>
                                    <p>{autopilot_measurement_kind_label(&effect.measurement_kind)}</p>
                                    <small>{format!("{} · {delta:+.1}%", autopilot_effect_label(&effect.assessment))}</small>
                                </div></article>
                            }
                        } /></div>
                    });
                    let recent_view = (!recent_actions.is_empty()).then(|| view! {
                        <div class="section-head"><h3>{tr("autopilot_recent_actions")}</h3></div>
                        <div class="ops-list"><For each=move || recent_actions.clone() key=|action| action.id.clone() children=move |action| {
                            let manual = action.manual_steps.iter().take(3)
                                .map(|step| format!("{}: {}", step.destination, step.what_to_do))
                                .collect::<Vec<_>>()
                                .join(" · ");
                            view! {
                                <article class="ops-item"><div>
                                    <strong>{autopilot_context_label(&action.context)}</strong>
                                    <p>{autopilot_action_kind_label(&action.action_kind)}</p>
                                    <small>{format!("{} · #{} · executor:{}", action.status, action.attempt_count, action.executor_status.as_deref().unwrap_or("pending"))}</small>
                                    {(!manual.is_empty()).then(|| view! { <small class="ops-note">{i18n::format("autopilot_manual_steps", std::slice::from_ref(&manual))}</small> })}
                                </div></article>
                            }
                        } /></div>
                    });
                    let guardrails_view = (!promotion_budget_guardrails.is_empty()).then(|| view! {
                        <div class="section-head"><h3>{tr("autopilot_financial_guardrails")}</h3></div>
                        <div class="ops-list"><For each=move || promotion_budget_guardrails.clone() key=|guardrail| guardrail.currency.clone() children=move |guardrail| view! {
                            <article class="ops-item"><div><strong>{guardrail.currency}</strong><p>{format!("≤{:.2}/day · ≤{:.2}/month", guardrail.maximum_total_daily_budget_minor as f64 / 100.0, guardrail.maximum_monthly_spend_minor as f64 / 100.0)}</p><small>{format!("v{}", guardrail.version)}</small></div></article>
                        } /></div>
                    });
                    view! {
                        <div class="ops-metrics">
                            <Metric value=data.succeeded_24h.to_string() label=tr("autopilot_actions_24h") />
                            <Metric value=queue.to_string() label=tr("autopilot_queue") />
                            <Metric value=data.executor_confirmed_24h.to_string() label=tr("autopilot_executor_confirmed") />
                            <Metric value=data.executor_failed_24h.to_string() label=tr("autopilot_executor_failed") />
                            <Metric value=data.failed_24h.to_string() label=tr("autopilot_failed_24h") />
                            <Metric value=if runtime_enabled { "ON".to_owned() } else { "OFF".to_owned() } label="runtime" />
                        </div>
                        {runtime_view}
                        <div class="section-head"><h3>{tr("autopilot_authority")}</h3></div>
                        <div class="ops-list autopilot-policies"><For each=move || policies.clone() key=|policy| policy.context.clone() children=move |policy| view! {
                            <AutopilotPolicyCard policy=policy overview=overview loading=loading error=error />
                        } /></div>
                        {guardrails_view}
                        {release_view}
                        {rum_view}
                        {needs_view}
                        {effects_view}
                        {recent_view}
                    }
                })}
            </Show>
        </section>
    }
}

#[component]
fn AutopilotPolicyCard(
    policy: AutopilotPolicySummary,
    overview: RwSignal<Option<OperatorAutopilotOverview>>,
    loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let current = if policy.enabled { policy.autonomy_level.as_str() } else { "off" };
    let observe_policy = policy.clone();
    let recommend_policy = policy.clone();
    let approval_policy = policy.clone();
    let auto_policy = policy.clone();
    let off_policy = policy.clone();
    view! {
        <article class="ops-item autopilot-policy-card">
            <div>
                <strong>{autopilot_context_label(&policy.context)}</strong>
                <p>{format!("{} · {}% · ≤{}/24h · v{}", current, policy.minimum_confidence / 100, policy.max_actions_24h, policy.version)}</p>
                {policy.guarded_until.as_ref().map(|until| view! {
                    <small class="warning">{format!("{} · {}", tr("autopilot_guarded"), until)}</small>
                })}
            </div>
            <div class="autopilot-policy-actions" role="group" aria-label=tr("autopilot_authority")>
                <button class="text-button" class:active=current == "off" on:click=move |_| set_autopilot_policy(off_policy.clone(), false, "observe", overview, loading, error)>{tr("autopilot_off")}</button>
                <button class="text-button" class:active=current == "observe" on:click=move |_| set_autopilot_policy(observe_policy.clone(), true, "observe", overview, loading, error)>{tr("autopilot_observe")}</button>
                <button class="text-button" class:active=current == "recommend" on:click=move |_| set_autopilot_policy(recommend_policy.clone(), true, "recommend", overview, loading, error)>{tr("autopilot_recommend")}</button>
                <button class="text-button" class:active=current == "require_approval" on:click=move |_| set_autopilot_policy(approval_policy.clone(), true, "require_approval", overview, loading, error)>{tr("autopilot_approval")}</button>
                <button class="text-button" class:active=current == "bounded_auto" on:click=move |_| set_autopilot_policy(auto_policy.clone(), true, "bounded_auto", overview, loading, error)>{tr("autopilot_auto")}</button>
            </div>
        </article>
    }
}

include!("operator/autopilot_cards.rs");

fn set_autopilot_policy(
    policy: AutopilotPolicySummary,
    enabled: bool,
    autonomy_level: &'static str,
    overview: RwSignal<Option<OperatorAutopilotOverview>>,
    loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) {
    if loading.get_untracked() { return; }
    loading.set(true);
    spawn_local(async move {
        let result = bridge::invoke::<AutopilotMutation, _>(
            "operator_autopilot_set_authority",
            &AutopilotAuthorityArgs {
                context: &policy.context,
                enabled,
                autonomy_level,
                minimum_confidence_basis_points: policy.minimum_confidence,
                max_actions_24h: policy.max_actions_24h,
                expected_version: policy.version,
            },
        ).await;
        match result {
            Ok(_) => {
                loading.set(false);
                refresh_operator_autopilot(overview, loading, error);
            }
            Err(message) => { loading.set(false); error.set(Some(message)); }
        }
    });
}

fn assign_autopilot_action(
    action_id: String,
    member_key: String,
    overview: RwSignal<Option<OperatorAutopilotOverview>>,
    loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) {
    if loading.get_untracked() || member_key.is_empty() { return; }
    loading.set(true);
    spawn_local(async move {
        let result = bridge::invoke::<AutopilotMutation, _>(
            "operator_autopilot_assign",
            &AutopilotAssignArgs { action_id: &action_id, member_key: &member_key },
        ).await;
        match result {
            Ok(_) => { loading.set(false); refresh_operator_autopilot(overview, loading, error); }
            Err(message) => { loading.set(false); error.set(Some(message)); }
        }
    });
}

fn mutate_autopilot_action(
    command: &'static str,
    action_id: String,
    overview: RwSignal<Option<OperatorAutopilotOverview>>,
    loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) {
    if loading.get_untracked() { return; }
    loading.set(true);
    spawn_local(async move {
        let result = match command {
            "operator_autopilot_approve" => bridge::invoke::<AutopilotMutation, _>(
                "operator_autopilot_approve", &AutopilotActionArgs { action_id: &action_id }
            ).await,
            "operator_autopilot_cancel" => bridge::invoke::<AutopilotMutation, _>(
                "operator_autopilot_cancel", &AutopilotActionArgs { action_id: &action_id }
            ).await,
            _ => Err(tr("autopilot_invalid_action").to_owned()),
        };
        match result {
            Ok(_) => { loading.set(false); refresh_operator_autopilot(overview, loading, error); }
            Err(message) => { loading.set(false); error.set(Some(message)); }
        }
    });
}

include!("operator/autopilot_labels.rs");

#[component]
fn OpsPanel(
    overview: RwSignal<Option<OperatorOpsOverview>>,
    loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let refresh = move |_| refresh_operator_ops(overview, loading, error);
    view! {
        <section class="ops-panel"><div class="section-head"><div><p class="eyebrow">CONTROL PLANE</p><h3>{tr("queues_and_deliveries")}</h3></div><button class="text-button" on:click=refresh disabled=move || loading.get()>{tr("refresh_2")}</button></div>
            <Show when=move || !loading.get() fallback=move || view! { <Skeleton rows=2 /> }>
                {move || overview.get().map(|data| {
                    let summary = data.summary;
                    let deliveries = data.dead_deliveries;
                    let outbox = data.dead_outbox;
                    let unavailable_sources = data.unavailable_sources;
                    let healthy = deliveries.is_empty() && outbox.is_empty() && unavailable_sources.is_empty();
                    let degraded_view = (!unavailable_sources.is_empty()).then(|| view! {
                        <p class="security-note warning">{i18n::format("cockpit_is_partially_available_unavailable", &[unavailable_sources.join(", ").to_string()])}</p>
                    });
                    let deliveries_view = (!deliveries.is_empty()).then(|| view! {
                        <div class="section-head"><h3>{tr("dead_deliveries")}</h3></div>
                        <div class="ops-list"><For each=move || deliveries.clone() key=|item| item.id.clone() children=move |item| view! { <OpsDeliveryCard item=item overview=overview loading=loading error=error /> } /></div>
                    });
                    let outbox_view = (!outbox.is_empty()).then(|| view! {
                        <div class="section-head"><h3>{tr("dead_outbox")}</h3></div>
                        <div class="ops-list"><For each=move || outbox.clone() key=|item| item.id.clone() children=move |item| view! { <OpsOutboxCard item=item overview=overview loading=loading error=error /> } /></div>
                    });
                    let healthy_view = healthy.then(|| view! { <p class="ops-healthy">{tr("no_dead_entries_the_delivery_pipeline_is")}</p> });
                    view! {
                        <div class="ops-metrics"><Metric value=summary.outbox.pending.to_string() label="outbox pending"/><Metric value=summary.outbox.dead.to_string() label="outbox dead"/><Metric value=summary.deliveries.pending.to_string() label="delivery pending"/><Metric value=summary.deliveries.dead.to_string() label="delivery dead"/></div>
                        <div class="ops-metrics http-ops-metrics"><Metric value=summary.http.requests.to_string() label="http requests"/><Metric value=format!("{} ms", summary.http.average_ms) label="avg"/><Metric value=format!("≤{} ms", summary.http.p50_ms) label="p50"/><Metric value=format!("≤{} ms", summary.http.p95_ms) label="p95"/><Metric value=summary.http.errors_4xx.to_string() label="4xx"/><Metric value=summary.http.errors_5xx.to_string() label="5xx"/><Metric value=if summary.release.is_empty() { "—".to_owned() } else { summary.release.clone() } label="release"/></div>
                        <div class="ops-metrics database-ops-metrics"><Metric value={if summary.database.server_version_num > 0 { format!("{}.{}", summary.database.server_version_num / 10_000, (summary.database.server_version_num / 100) % 100) } else { "—".to_owned() }} label=tr("database_runtime")/><Metric value=summary.database.io_method.clone().unwrap_or_else(|| "—".to_owned()) label="io_method"/><Metric value=summary.database.io_workers.map_or_else(|| "—".to_owned(), |value| value.to_string()) label="io_workers"/><Metric value=summary.database.effective_io_concurrency.map_or_else(|| "—".to_owned(), |value| value.to_string()) label="effective_io"/><Metric value=summary.database.maintenance_io_concurrency.map_or_else(|| "—".to_owned(), |value| value.to_string()) label="maintenance_io"/><Metric value=summary.database.io_max_concurrency.map_or_else(|| "—".to_owned(), |value| if value < 0 { "auto".to_owned() } else { value.to_string() }) label="io_max"/><Metric value=summary.database.io_combine_limit_bytes.map_or_else(|| "—".to_owned(), |value| format!("{} KiB", value / 1024)) label="io_combine"/><Metric value=summary.database.io_max_combine_limit_bytes.map_or_else(|| "—".to_owned(), |value| format!("{} KiB", value / 1024)) label="io_max_combine"/><Metric value={if summary.database.async_io_active { "ACTIVE".to_owned() } else { "OFF".to_owned() }} label=tr("async_io")/></div>
                        <div class="section-head"><h3>{tr("area_runtime")}</h3><span>{format!("schema v{}", summary.schema_version)}</span></div>
                        <div class="ops-metrics area-ops-metrics"><Metric value=summary.area.credits_total.to_string() label=tr("area_credits")/><Metric value=summary.area.vouchers_issued.to_string() label=tr("area_vouchers")/><Metric value=summary.area.ticket_rewards_issued.to_string() label=tr("area_ticket_rewards")/><Metric value=summary.area.legacy_imported_players.to_string() label=tr("area_legacy_imported")/><Metric value=summary.area.stale_voucher_reservations.to_string() label=tr("stale_voucher_leases")/><Metric value=summary.area.stale_ticket_reward_reservations.to_string() label=tr("stale_ticket_leases")/></div>
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
    view! { <article class="ops-item"><div><strong>{item.event_type}</strong><p>{i18n::format("attempt_2", &[item.endpoint_name.to_string(), item.attempt_count.to_string(), item.max_attempts.to_string()])}</p><small>{item.last_error_kind.value_or_else(|| tr("no_error_code").to_owned())}</small></div><button class="danger ghost" on:click=retry disabled=move || loading.get() || !endpoint_active>{tr("retry")}</button></article> }
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
    view! { <article class="ops-item"><div><strong>{item.event_type}</strong><p>{i18n::format("attempt", &[item.attempts.to_string(), item.max_attempts.to_string()])}</p><small>{item.last_error_kind.value_or_else(|| tr("no_error_code").to_owned())}</small></div><button class="danger ghost" on:click=retry disabled=move || loading.get()>{tr("retry")}</button></article> }
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
                    tr("retry_had_already_been_accepted").to_owned()
                } else {
                    tr("item_returned_to_the_queue").to_owned()
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
