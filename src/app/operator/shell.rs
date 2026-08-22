#[component]
fn OperatorApp(
    mode: RwSignal<RootMode>,
    status: RwSignal<SessionStatus>,
    dashboard: RwSignal<Option<DashboardData>>,
    tab: RwSignal<OperatorTab>,
    push_target: RwSignal<Option<String>>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let loading = RwSignal::new(OperatorLoadingState::all());
    let bootstrap_requested = RwSignal::new(false);
    let refresh_requested = RwSignal::new(0_u32);

    let signal_overview = RwSignal::new(None::<OperatorSignalOverview>);
    let signal_loading = RwSignal::new(false);
    let signal_requested = RwSignal::new(false);
    let staff_push_bootstrapped = RwSignal::new(false);
    let menu_open = RwSignal::new(false);
    Effect::new(move |_| {
        let unlocked = status.get().unlocked;
        if !unlocked {
            bootstrap_requested.set(false);
            return;
        }
        if bootstrap_requested.get_untracked() {
            return;
        }
        // `dashboard` lives in OperatorPortal and can survive an OperatorApp
        // remount. Never use its placeholder presence as proof that network
        // reads completed: a resume/status refresh can cancel the old owner
        // after the placeholder was created and otherwise leave the connection state stuck forever.
        bootstrap_requested.set(true);
        refresh_operator_parts(dashboard, loading, error);
        // Disk first, network behind it. Each panel is gated on its own loading
        // bit so a live answer that already landed is never overwritten by the
        // older snapshot.
        with_operator_cached_sections(move |snapshot| {
            let crate::models::OperatorSectionsSnapshot {
                events, qr, signal, ..
            } = snapshot;
            if !events.is_empty() && loading.get_untracked().events {
                dashboard.update(|state| {
                    state.get_or_insert_with(DashboardData::default).events = events;
                });
                loading.update(|state| state.events = false);
            }
            if qr.is_some() && loading.get_untracked().qr {
                dashboard.update(|state| {
                    state.get_or_insert_with(DashboardData::default).qr = qr;
                });
                loading.update(|state| state.qr = false);
            }
            if signal.is_some()
                && !signal_loading.get_untracked()
                && signal_overview.get_untracked().is_none()
            {
                signal_overview.set(signal);
            }
        });
    });

    Effect::new(move |_| {
        let generation = refresh_requested.get();
        if generation == 0 {
            return;
        }
        refresh_operator_parts(dashboard, loading, error);
        // Do not forcibly clear an in-flight request's loading bit. If one is
        // active, let it finish; the `signal_requested=false` edge will then
        // trigger exactly one fresh read from this stable OperatorApp owner.
        signal_requested.set(false);
        signal_overview.set(None);
    });

    Effect::new(move |_| {
        if status.get().unlocked && !staff_push_bootstrapped.get_untracked() {
            staff_push_bootstrapped.set(true);
            spawn_local(async move {
                // Silent bootstrap only: if Android permission is already granted,
                // bind this authenticated staff session to FCM. Permission prompting
                // stays explicit in the checklist screen.
                let _ = bridge::invoke_timeout::<FanPushStatus, _>("operator_push_sync", &EmptyArgs {}, 15_000).await;
            });
        }
    });

    let role = move || {
        status
            .get()
            .session
            .map(|s| s.role)
            .value_or(OperatorRole::Staff)
    };

    Effect::new(move |_| {
        if push_target.get().as_deref().is_some_and(|target| target.starts_with("/staff/checklist")) {
            tab.set(OperatorTab::Checklist);
        }
    });

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

    let refresh_all = move |_| {
        // The menu is a short-lived reactive owner. Trigger the actual reads
        // from the stable OperatorApp owner instead of spawning tasks here and
        // immediately cancelling them by closing the menu.
        menu_open.set(false);
        bridge::invalidate_latest("operator:");
        refresh_requested.update(|value| *value = value.wrapping_add(1).max(1));
    };

    on_cleanup(move || bridge::invalidate_latest("operator:"));

    let close = move |_| {
        bridge::invalidate_latest("operator:");
        // Locking is local and has no remote leg. Leave the staff surface now
        // and let the native lock land behind us instead of holding an
        // authenticated screen open until it replies.
        dashboard.set(None);
        loading.set(OperatorLoadingState::all());
        signal_overview.set(None);
        signal_loading.set(false);
        signal_requested.set(false);
        status.set(SessionStatus {
            configured: status.get_untracked().configured,
            unlocked: false,
            session: None,
        });
        mode.set(RootMode::Fan);
        spawn_local(async move {
            match bridge::invoke::<SessionStatus, _>("lock", &EmptyArgs {}).await {
                Ok(value) => {
                    let _ = status.try_set(value);
                }
                Err(message) => {
                    let _ = error.try_set(Some(message));
                }
            }
        });
    };

    view! {
        <section class="authenticated">
            <header class="topbar">
                <div><p class="eyebrow">{tr("virya_control")}</p><strong>{move || status.get().session.map(|s| s.display_name).value_or_else(Default::default)}</strong></div>
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
                    <button class:active=move || tab.get() == OperatorTab::Checklist on:click=move |_| { tab.set(OperatorTab::Checklist); menu_open.set(false); }><span>"✓"</span>{tr("gig_checklist")}</button>
                    <button class:active=move || tab.get() == OperatorTab::Settings on:click=move |_| { tab.set(OperatorTab::Settings); menu_open.set(false); }><span>"⚙"</span>{tr("settings")}</button>
                    <button on:click=refresh_all><span>"↻"</span>{tr("refresh_all_data")}</button>
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
                    OperatorTab::Checklist => view! { <OperatorChecklist dashboard=dashboard loading=loading push_target=push_target error=error /> }.into_any(),
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
