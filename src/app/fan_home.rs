fn event_phase_label(phase: &str) -> &'static str {
    match phase {
        "live" => tr("signal_live_now"),
        "afterglow" => tr("signal_afterglow"),
        _ => tr("next_signal"),
    }
}

fn recommended_tab(action: &str) -> FanTab {
    match action {
        "open_wallet" => FanTab::Wallet,
        "share_post_show_feedback" => FanTab::Profile,
        "continue_synesthesia" => FanTab::Signal,
        _ => FanTab::Events,
    }
}

fn recommended_label(action: &str) -> &'static str {
    match action {
        "open_wallet" => tr("open_wallet_now"),
        "open_live_event" => tr("open_live_signal"),
        "share_post_show_feedback" => tr("share_post_show_feedback"),
        "get_ticket" => tr("get_ticket_now"),
        "follow_next_event" => tr("follow_this_signal"),
        _ => tr("show_details"),
    }
}

#[component]
fn FanHomeOverview(
    home: RwSignal<Option<FanHomeData>>,
    loading: RwSignal<FanLoadingState>,
    tab: RwSignal<FanTab>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <section class="fan-home-overview">
            <Show when=move || !loading.get().home fallback=move || view! { <Skeleton rows=3 /> }>
                {move || home.get().map(|snapshot| {
                    let stale = snapshot.stale;
                    let synesthesia = snapshot.synesthesia.clone();
                    let synesthesia_summary = synesthesia_best_summary(&synesthesia);
                    let counts = snapshot.counts.clone();
                    let referral = snapshot.referral.clone();
                    let next_event = snapshot.next_event.clone();
                    let city = snapshot.profile.primary_city.clone();
                    let generated_at = human_time(&snapshot.generated_at);
                    view! {
                        <header class="fan-home-header">
                            <div>
                                <p class="eyebrow">{tr("your_signal_now")}</p>
                                <h2>{snapshot.profile.display_name.value_or_else(|| tr("my_signal").to_owned())}</h2>
                                <p>{city.map(|value| i18n::format("signal_city_context", &[value])).value_or_else(|| tr("signal_home_context").to_owned())}</p>
                            </div>
                            <Show when=move || stale>
                                <span class="cache-badge">{tr("cached_data")}</span>
                            </Show>
                            <small>{i18n::format("signal_snapshot_updated", &[generated_at])}</small>
                        </header>
                        <div class="fan-home-grid">
                            <article class="home-action-card synesthesia-home-card">
                                <p class="eyebrow">"SYNESTHESIA"</p>
                                <strong>{if synesthesia.completed { tr("journey_completed") } else if synesthesia.started { tr("continue_the_journey") } else { tr("start_the_journey") }}</strong>
                                <p>{if synesthesia.completed {
                                    if synesthesia.linked_at.is_some() { tr("completion_linked_to_signal") } else { tr("completion_saved_link_it_to_signal") }
                                } else {
                                    tr("rooms_completed_count")
                                }}</p>
                                {synesthesia.client_total_elapsed_ms.map(|elapsed| view! {
                                    <small>{i18n::format("synesthesia_completed_in_minutes", &[((elapsed / 60_000).max(1)).to_string()])}</small>
                                })}
                                {synesthesia_summary.map(|summary| view! {
                                    <small class="synesthesia-best-summary">{summary}</small>
                                })}
                                <Show when=move || synesthesia.reward_entered>
                                    <span class="cache-badge">{tr("reward_entry_confirmed")}</span>
                                </Show>
                                <Show when=move || !synesthesia.completed>
                                    <div class="progress-track"><span style=format!("width:{}%", (i32::from(synesthesia.rooms_completed).clamp(0, 11) * 100) / 11)></span></div>
                                    <small>{format!("{}/11", synesthesia.rooms_completed.clamp(0, 11))}</small>
                                </Show>
                                <ExternalLink url="https://synesthesia.virya.music/?source=signal-app&resume=1".to_owned() label=if synesthesia.started { tr("open_synesthesia") } else { tr("enter_synesthesia") } error=error />
                            </article>
                            {next_event.map(|event| {
                                let ticket_url = event.ticket_url.clone();
                                let title = event.title.clone();
                                let is_live = event.phase == "live";
                                let is_afterglow = event.phase == "afterglow";
                                let is_upcoming = event.phase == "upcoming";
                                let location = event.venue.clone().or(event.city.clone());
                                let event_meta = event_time_location(&event.starts_at, location.as_deref());
                                let doors = event.doors_at.as_deref().map(human_time);
                                let ends = event.ends_at.as_deref().map(human_time);
                                let admission_ready = event.has_pass || event.has_paid_ticket;
                                let ticket_sale_active = event.ticket_sale_active;
                                let interested = event.interested;
                                view! {
                                    <article class="home-action-card next-show-card" class:live=is_live class:afterglow=is_afterglow>
                                        <p class="eyebrow">{event_phase_label(&event.phase)}</p>
                                        <strong>{title}</strong>
                                        <p>{event_meta}</p>
                                        {doors.map(|value| view! { <small>{i18n::format("doors_open_at", &[value])}</small> })}
                                        {ends.map(|value| view! { <small>{i18n::format("event_ends_at", &[value])}</small> })}
                                        <div class="home-card-statuses">
                                            <Show when=move || admission_ready><span class="cache-badge">{tr("entry_ready")}</span></Show>
                                            <Show when=move || interested><span class="cache-badge">{tr("following_event")}</span></Show>
                                            <Show when=move || ticket_sale_active && !admission_ready><span class="cache-badge">{tr("tickets_on_sale")}</span></Show>
                                        </div>
                                        <Show when=move || is_live><p class="signal-live-note">{tr("signal_live_note")}</p></Show>
                                        <Show when=move || is_afterglow><p class="signal-afterglow-note">{tr("signal_afterglow_note")}</p></Show>
                                        <div class="home-card-actions">
                                            <button class="ghost" on:click={
                                                let action = snapshot.recommended_action.clone();
                                                move |_| tab.set(recommended_tab(&action))
                                            }>{recommended_label(&snapshot.recommended_action)}</button>
                                            {ticket_url.filter(|_| is_upcoming).map(|url| view! { <ExternalLink url=url label=tr("tickets_tab") error=error /> })}
                                        </div>
                                    </article>
                                }
                            })}
                        </div>
                        <div class="stats-grid fan-home-stats">
                            <Metric value=counts.event_interests.to_string() label=tr("show_interests")/>
                            <Metric value=counts.active_passes.to_string() label=tr("active_passes")/>
                            <Metric value=counts.paid_orders.to_string() label=tr("paid_orders")/>
                            <Metric value=counts.area_claims.to_string() label=tr("area_findings")/>
                            <Metric value=referral.qualified.to_string() label=tr("confirmed_referrals")/>
                            <Metric value=referral.pending.to_string() label=tr("pending_referrals")/>
                        </div>
                        <section class="participation-history" aria-label=tr("your_participation")>
                            <div class="participation-history-heading">
                                <div><p class="eyebrow">{tr("your_participation")}</p><strong>{tr("participation_history_title")}</strong></div>
                                <small>{tr("participation_history_hint")}</small>
                            </div>
                            <div class="participation-history-grid">
                                <article class:active=synesthesia.started><strong>{if synesthesia.completed { "✓".to_owned() } else { format!("{}/11", synesthesia.rooms_completed.clamp(0, 11)) }}</strong><span>{tr("synesthesia_journey")}</span></article>
                                <article class:active={counts.area_claims > 0}><strong>{counts.area_claims.max(0)}</strong><span>{tr("area_discoveries")}</span></article>
                                <article class:active={counts.paid_orders > 0}><strong>{counts.paid_orders.max(0)}</strong><span>{tr("concert_orders")}</span></article>
                                <article class:active={counts.active_passes > 0}><strong>{counts.active_passes.max(0)}</strong><span>{tr("concert_passes")}</span></article>
                            </div>
                        </section>
                    }.into_any()
                }).value_or_else(|| view! {
                    <div class="empty-state"><strong>{tr("signal_home_unavailable")}</strong><p>{tr("signal_home_fallback_hint")}</p></div>
                }.into_any())}
            </Show>
        </section>
    }
}

#[component]
fn NativePushControl(error: RwSignal<Option<String>>) -> impl IntoView {
    let status = RwSignal::new(None::<FanPushStatus>);
    let busy = RwSignal::new(false);
    let resume_refresh = RwSignal::new(0_u32);
    let enable_after_settings = RwSignal::new(false);
    install_resume_refresh(resume_refresh);

    Effect::new(move |_| {
        resume_refresh.get();
        if !bridge::native_available() || busy.get() {
            return;
        }

        // A permanently denied Android permission must be changed in system
        // Settings. Treat returning from that screen as continuation of the
        // original enable intent: if permission is now granted, finish FCM +
        // CrowdRelay registration automatically instead of requiring a second
        // tap. Clearing the flag before spawning also makes duplicate resume
        // events harmless.
        if enable_after_settings.get_untracked() {
            enable_after_settings.set(false);
            busy.set(true);
            spawn_local(async move {
                match bridge::invoke::<FanPushStatus, _>("fan_push_enable", &EmptyArgs {}).await {
                    Ok(value) => status.set(Some(value)),
                    Err(message) => error.set(Some(message)),
                }
                busy.set(false);
            });
            return;
        }

        spawn_local(async move {
            match bridge::invoke_latest::<FanPushStatus, _>(
                "fan_push_status",
                &EmptyArgs {},
                10_000,
                "fan:push-status",
            )
            .await
            {
                Ok(Some(value)) => status.set(Some(value)),
                Ok(None) => {}
                Err(message) => error.set(Some(message)),
            }
        });
    });

    let toggle = move |_| {
        if busy.get_untracked() {
            return;
        }
        let current = status.get_untracked();
        let opens_settings = current
            .as_ref()
            .is_some_and(|value| !value.enabled && value.permission == "denied");
        let command = if current.as_ref().is_some_and(|value| value.enabled) {
            "fan_push_disable"
        } else if opens_settings {
            "fan_push_open_settings"
        } else {
            "fan_push_enable"
        };
        if opens_settings {
            enable_after_settings.set(true);
        }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<FanPushStatus, _>(command, &EmptyArgs {}).await {
                Ok(value) => status.set(Some(value)),
                Err(message) => {
                    if opens_settings {
                        enable_after_settings.set(false);
                    }
                    error.set(Some(message));
                }
            }
            busy.set(false);
        });
    };

    view! {
        <Show when=move || bridge::native_available()>
            <article class="push-setting-card">
                <div class="push-setting-copy">
                    <div class="push-setting-heading">
                        <span class="push-setting-icon" aria-hidden="true">"◉"</span>
                        <div><strong>{tr("push_notifications")}</strong><p>{tr("push_notifications_hint")}</p></div>
                    </div>
                    {move || status.get().map(|current| {
                        let message = if !current.supported {
                            tr("push_notifications_degraded")
                        } else if !current.backend_enabled {
                            tr("push_notifications_waiting_backend")
                        } else if current.permission == "denied" {
                            tr("push_notifications_blocked")
                        } else if current.enabled {
                            tr("push_notifications_on")
                        } else if current.detail.is_some() {
                            tr("push_notifications_degraded")
                        } else {
                            tr("push_notifications_off")
                        };
                        view! { <small class:success=current.enabled class:warning=!current.enabled>{message}</small> }
                    })}
                </div>
                <Show when=move || status.get().is_some_and(|value| value.supported)>
                    <button
                        type="button"
                        class="ghost push-setting-action"
                        disabled=move || busy.get()
                        on:click=toggle
                    >
                        {move || if busy.get() {
                            tr("syncing_push_notifications")
                        } else if status.get().is_some_and(|value| value.enabled) {
                            tr("disable_push_notifications")
                        } else if status.get().as_ref().is_some_and(|value| value.permission == "denied") {
                            tr("open_notification_settings")
                        } else {
                            tr("enable_push_notifications")
                        }}
                    </button>
                </Show>
            </article>
        </Show>
    }
}
