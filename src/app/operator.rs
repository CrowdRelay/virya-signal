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
            {degraded_view}
            <div class="section-head"><h3>{tr("strongest_cities")}</h3><span>{city_count}</span></div>
            {cities_view}
            <p class="security-note">{i18n::format("snapshot_generated_at_aggregated_data_only", std::slice::from_ref(&generated_at))}</p>
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
            error.set(Some(tr("select_a_show_first").to_owned()));
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
            error.set(Some(tr("select_a_show").to_owned()));
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
                    show_message.set(tr("show_data_removed_from_the_device").to_owned());
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    view! {
        <section class="screen">
            <header class="screen-title"><p class="eyebrow">GATE MODE</p><h2>{tr("scan_entry")}</h2></header>
            <label class="select-label">{tr("show")}<select disabled=move || loading.get().events prop:value=move || event_slug.get() on:change=select_event><option value="">{move || if loading.get().events { tr("loading_shows") } else { tr("select_an_event") }}</option><For each=move || operator_events(dashboard) key=|event| event.slug.clone() children=move |event| view! { <option value=event.slug.clone()>{event.title}</option> } /></select></label>
            <article class:show-mode-active=move || offline.get() class="show-mode-card">
                <div class="section-head"><div><p class="eyebrow">OFFLINE SHOW MODE</p><h3>{move || if offline.get() { tr("gate_works_locally") } else { tr("works_without_lte") }}</h3></div><button type="button" class:active=move || offline.get() disabled=move || !show_mode.get().prepared || busy.get() on:click=move |_| offline.update(|value| *value = !*value)>{move || if offline.get() { tr("offline_on") } else { tr("offline_off") }}</button></div>
                <p>{move || if show_mode.get().prepared { i18n::format("tickets_pending_conflicts", &[show_mode.get().eligible_passes.to_string(), show_mode.get().pending.to_string(), show_mode.get().conflicts.to_string()]) } else { tr("download_a_secure_snapshot_before_opening_the").to_owned() }}</p>
                <div class="show-mode-actions"><button type="button" on:click=prepare disabled=move || busy.get() || event_slug.get().is_empty()>{tr("prepare_offline")}</button><button type="button" on:click=sync disabled=move || busy.get() || !show_mode.get().prepared>{tr("sync")}</button><button type="button" class="danger ghost" on:click=clear disabled=move || busy.get() || !show_mode.get().prepared>{tr("clear")}</button></div>
                <Show when=move || !show_message.get().is_empty()><small>{move || show_message.get()}</small></Show>
            </article>
            <button class="scanner-button" on:click=scan disabled=move || busy.get()><span class="scanner-frame"></span><strong>{move || if busy.get() { tr("verifying") } else if offline.get() { tr("scan_locally") } else { tr("open_camera") }}</strong><small>{move || if offline.get() { tr("durable_t1_ticket_qr_only") } else { tr("ticket_or_admission_pass_qr") }}</small></button>
            <Show when=move || !offline.get()><div class="manual-row"><input placeholder=tr("qr_code_or_admission_pass_number") prop:value=move || manual.get() on:input=move |e| manual.set(event_target_value(&e))/><button on:click=manual_submit disabled=move || busy.get()>{tr("check")}</button></div></Show>
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
            <header class="screen-title"><p class="eyebrow">{tr("device_label")}</p><h2>{tr("settings")}</h2></header>
            <div class="settings-list">
                <LanguageSwitch />
                <article><div><strong>{tr("connection")}</strong><p>{move || status.get().session.map(|s| s.api_base_url).value_or_else(Default::default)}</p></div><span class:online=move || !loading.get().events && !loading.get().qr>{move || if loading.get().events || loading.get().qr { tr("connecting_2") } else { tr("online") }}</span></article>
                <article><div><strong>{tr("permissions")}</strong><p>{move || status.get().session.map(|s| s.role.label().to_owned()).value_or_else(Default::default)}</p></div></article>
                <button on:click=refresh disabled=move || loading.get().events || loading.get().qr>{tr("refresh_all_data")}</button>
                <button on:click=lock>{tr("lock_panel")}</button>
                <button class="danger ghost" on:click=forget>{tr("remove_operator_profile")}</button>
            </div>
            <Show when=move || owner.get()><OpsPanel overview=ops loading=ops_loading error=error /></Show>
            <AnonymousFeedback error=error />
            <p class="security-note">{tr("operator_token_is_stored_in_an_encrypted")}</p>
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
