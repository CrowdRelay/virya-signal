use crate::app::formatters::synesthesia_best_summary;

fn event_phase_label(phase: &str) -> &'static str {
    match phase {
        "live" => tr("signal_live_now"),
        "afterglow" => tr("signal_afterglow"),
        _ => tr("next_signal"),
    }
}


#[component]
fn FanHomeOverview(
    home: RwSignal<Option<FanHomeData>>,
    loading: RwSignal<FanLoadingState>,
    tab: RwSignal<FanTab>,
    focused_event_slug: RwSignal<Option<String>>,
    focused_event_preview: RwSignal<Option<PublicEvent>>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let leaderboard_busy = RwSignal::new(false);
    view! {
        <section class="fan-home-overview">
            <Show when=move || !loading.get().home fallback=move || view! { <Skeleton rows=3 /> }>
                {move || home.get().map(|snapshot| {
                    let stale = snapshot.stale;
                    let synesthesia = snapshot.synesthesia.clone();
                    let show_synesthesia = synesthesia.started || synesthesia.completed;
                    let synesthesia_summary = synesthesia_best_summary(&synesthesia);
                    let next_event = snapshot.next_event.clone();
                    let city = snapshot.profile.primary_city.clone();
                    view! {
                        <header class="fan-home-header">
                            <div>
                                <p class="eyebrow">{tr("your_signal_now")}</p>
                                <h2>{snapshot.profile.display_name.clone().value_or_else(|| tr("my_signal").to_owned())}</h2>
                                <p>{city.map(|value| i18n::format("signal_city_context", &[value])).value_or_else(|| tr("signal_home_context").to_owned())}</p>
                            </div>
                            {stale.then(|| view! {
                                <span class="cache-badge">{tr("cached_data")}</span>
                            })}
                        </header>
                        <div class="fan-home-grid">
                            // Synesthesia is a side album experiment, not a primary Home CTA.
                            // Only people who already engaged with it see the progress card.
                            <Show when=move || show_synesthesia>
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
                                    {synesthesia_summary.clone().map(|summary| view! {
                                        <small class="synesthesia-best-summary">{summary}</small>
                                    })}
                                    {synesthesia.reward_entered.then(|| view! {
                                        <span class="cache-badge">{tr("reward_entry_confirmed")}</span>
                                    })}
                                    {(!synesthesia.completed).then(|| view! {
                                        <div class="progress-track"><span style=format!("width:{}%", (i32::from(synesthesia.rooms_completed).clamp(0, 11) * 100) / 11)></span></div>
                                        <small>{format!("{}/11", synesthesia.rooms_completed.clamp(0, 11))}</small>
                                    })}
                                    <ExternalLink url="https://synesthesia.virya.music/?source=signal-app&resume=1".to_owned() label=if synesthesia.started { tr("open_synesthesia") } else { tr("enter_synesthesia") } error=error />
                                    {(bridge::native_available() && synesthesia.leaderboard_published).then(|| view! {
                                        <button
                                            class="ghost"
                                            disabled=move || leaderboard_busy.get()
                                            on:click=move |_| {
                                                if leaderboard_busy.get_untracked() {
                                                    return;
                                                }
                                                leaderboard_busy.set(true);
                                                spawn_local(async move {
                                                    match bridge::invoke_timeout::<bool, _>(
                                                        "fan_unpublish_synesthesia_leaderboard",
                                                        &EmptyArgs {},
                                                        12_000,
                                                    )
                                                    .await
                                                    {
                                                        Ok(true) => home.update(|current| {
                                                            if let Some(snapshot) = current {
                                                                snapshot.synesthesia.leaderboard_published = false;
                                                                snapshot.synesthesia.leaderboard_rank = None;
                                                            }
                                                        }),
                                                        Ok(false) => error.set(Some(tr("leaderboard_unpublish_failed").to_owned())),
                                                        Err(message) => error.set(Some(message)),
                                                    }
                                                    leaderboard_busy.set(false);
                                                });
                                            }
                                        >{move || if leaderboard_busy.get() { tr("removing_from_leaderboard") } else { tr("remove_from_leaderboard") }}</button>
                                    })}
                                </article>
                            </Show>
                            {next_event.map(|event| {
                                let ticket_url = event.ticket_url.clone();
                                let title = event.title.clone();
                                let event_slug = event.slug.clone();
                                let event_preview = PublicEvent {
                                    slug: event.slug.clone(),
                                    title: event.title.clone(),
                                    description: None,
                                    city: event.city.clone().map(|name| EventCity { name }),
                                    venue: event.venue.clone(),
                                    starts_at: event.starts_at.clone(),
                                    ticket_url: event.ticket_url.clone(),
                                    image_url: None,
                                    image_thumbnail_url: None,
                                };
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
                                            {admission_ready.then(|| view! { <span class="cache-badge">{tr("entry_ready")}</span> })}
                                            {interested.then(|| view! { <span class="cache-badge">{tr("following_event")}</span> })}
                                            {(ticket_sale_active && !admission_ready).then(|| view! { <span class="cache-badge">{tr("tickets_on_sale")}</span> })}
                                        </div>
                                        {is_live.then(|| view! { <p class="signal-live-note">{tr("signal_live_note")}</p> })}
                                        {is_afterglow.then(|| view! { <p class="signal-afterglow-note">{tr("signal_afterglow_note")}</p> })}
                                        <div class="home-card-actions">
                                            <button class="ghost" on:click={
                                                let event_slug = event_slug.clone();
                                                let event_preview = event_preview.clone();
                                                move |_| {
                                                    focused_event_preview.set(Some(event_preview.clone()));
                                                    focused_event_slug.set(Some(event_slug.clone()));
                                                    tab.set(FanTab::Events);
                                                }
                                            }>{tr("show_details")}</button>
                                            // First-party ticketing wins whenever the show has an
                                            // active sale: send the fan into the Events tab focused on
                                            // this show, where the in-app checkout lives, exactly like
                                            // the details button beside it. The external link is only a
                                            // fallback for shows we do not sell ourselves, so a fan is
                                            // never pushed out to a resale page for a ticket Virya has.
                                            {(is_upcoming && ticket_sale_active).then(|| {
                                                let event_slug = event_slug.clone();
                                                let event_preview = event_preview.clone();
                                                view! {
                                                    <button class="ticket-buy-button" on:click=move |_| {
                                                        focused_event_preview.set(Some(event_preview.clone()));
                                                        focused_event_slug.set(Some(event_slug.clone()));
                                                        tab.set(FanTab::Events);
                                                    }>{tr("buy_ticket")}</button>
                                                }
                                            })}
                                            {ticket_url
                                                .filter(|_| is_upcoming && !ticket_sale_active)
                                                .map(|url| view! { <ExternalLink url=url label=tr("tickets_tab") error=error /> })}
                                        </div>
                                    </article>
                                }
                            })}
                        </div>
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
    let resume_pending = RwSignal::new(false);
    let enable_after_settings = RwSignal::new(false);
    install_resume_refresh(resume_refresh);

    Effect::new(move |_| {
        resume_refresh.get();
        if !bridge::native_available() {
            return;
        }
        if busy.get_untracked() {
            // Android can resume while the permission/settings command is still
            // completing. Preserve that edge and reconcile once it terminates.
            resume_pending.set(true);
            return;
        }
        resume_pending.set(false);
        busy.set(true);
        spawn_lifecycle_task(async move {
            let retry_after_settings =
                enable_after_settings.try_get_untracked().unwrap_or(false);
            let synced = bridge::invoke_latest::<FanPushStatus, _>(
                "fan_push_sync",
                &EmptyArgs {},
                15_000,
                "fan:push-sync",
            )
            .await;
            match synced {
                Ok(Some(value))
                    if retry_after_settings && value.permission != "denied" && !value.enabled =>
                {
                    let _ = status.try_set(Some(value));
                    match bridge::invoke_timeout::<FanPushStatus, _>(
                        "fan_push_enable",
                        &EmptyArgs {},
                        45_000,
                    )
                    .await
                    {
                        Ok(value) => {
                            let _ = enable_after_settings
                                .try_set(!value.enabled && value.permission != "denied");
                            let _ = status.try_set(Some(value));
                        }
                        Err(message) => {
                            let _ = error.try_set(Some(message));
                        }
                    }
                }
                Ok(Some(value)) => {
                    if value.enabled || value.permission == "denied" {
                        let _ = enable_after_settings.try_set(false);
                    }
                    let _ = status.try_set(Some(value));
                }
                Ok(None) => {}
                Err(message) => {
                    let _ = error.try_set(Some(message));
                }
            }
            finish_resumable_ui_task(busy, resume_pending, resume_refresh);
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
        busy.set(true);
        if opens_settings {
            enable_after_settings.set(true);
        }
        spawn_lifecycle_task(async move {
            match bridge::invoke_timeout::<FanPushStatus, _>(command, &EmptyArgs {}, 45_000).await {
                Ok(value) => {
                    let _ = status.try_set(Some(value));
                }
                Err(message) => {
                    let _ = error.try_set(Some(message));
                }
            }
            finish_resumable_ui_task(busy, resume_pending, resume_refresh);
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
                        } else if current.detail.as_deref() == Some("firebase_not_configured") {
                            tr("push_firebase_not_configured")
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
            </article>
            <PushPreferencesControl error=error />
        </Show>
    }
}


const PUSH_PREF_SHOWS: u8 = 0;
const PUSH_PREF_RELEASES: u8 = 1;
const PUSH_PREF_COMMUNITY: u8 = 2;
const PUSH_PREF_MERCH: u8 = 3;
const PUSH_PREF_QUIET: u8 = 4;

fn push_preference_enabled(value: &FanPushPreferences, key: u8) -> bool {
    match key {
        PUSH_PREF_SHOWS => value.shows,
        PUSH_PREF_RELEASES => value.releases,
        PUSH_PREF_COMMUNITY => value.community,
        PUSH_PREF_MERCH => value.merch,
        PUSH_PREF_QUIET => value.quiet_hours_enabled,
        _ => false,
    }
}

fn update_push_preference(
    preferences: RwSignal<Option<FanPushPreferences>>,
    busy: RwSignal<bool>,
    error: RwSignal<Option<String>>,
    key: u8,
    checked: bool,
) {
    let Some(current) = preferences.get_untracked() else {
        return;
    };
    if busy.get_untracked() {
        return;
    }

    let mut next = FanPushPreferencesUpdate {
        shows: current.shows,
        releases: current.releases,
        community: current.community,
        merch: current.merch,
        quiet_hours_enabled: current.quiet_hours_enabled,
        quiet_start: current.quiet_start.clone(),
        quiet_end: current.quiet_end.clone(),
    };
    match key {
        PUSH_PREF_SHOWS => next.shows = checked,
        PUSH_PREF_RELEASES => next.releases = checked,
        PUSH_PREF_COMMUNITY => next.community = checked,
        PUSH_PREF_MERCH => next.merch = checked,
        PUSH_PREF_QUIET => next.quiet_hours_enabled = checked,
        _ => return,
    }

    busy.set(true);
    spawn_lifecycle_task(async move {
        match bridge::invoke_timeout::<FanPushPreferences, _>(
            "fan_push_update_preferences",
            &FanPushPreferencesArgs { preferences: &next },
            15_000,
        )
        .await
        {
            Ok(value) => {
                let _ = preferences.try_set(Some(value));
            }
            Err(message) => {
                let _ = error.try_set(Some(message));
            }
        }
        let _ = busy.try_set(false);
    });
}

#[component]
fn PushPreferencesControl(error: RwSignal<Option<String>>) -> impl IntoView {
    let preferences = RwSignal::new(None::<FanPushPreferences>);
    let busy = RwSignal::new(false);

    Effect::new(move |_| {
        if !bridge::native_available()
            || preferences.get_untracked().is_some()
            || busy.get_untracked()
        {
            return;
        }
        busy.set(true);
        spawn_lifecycle_task(async move {
            match bridge::invoke_timeout::<FanPushPreferences, _>(
                "fan_push_preferences",
                &EmptyArgs {},
                15_000,
            )
            .await
            {
                Ok(value) => {
                    let _ = preferences.try_set(Some(value));
                }
                Err(message) => {
                    let _ = error.try_set(Some(message));
                }
            }
            let _ = busy.try_set(false);
        });
    });

    let items = [
        (PUSH_PREF_SHOWS, tr("push_category_shows")),
        (PUSH_PREF_RELEASES, tr("push_category_releases")),
        (PUSH_PREF_COMMUNITY, tr("push_category_community")),
        (PUSH_PREF_MERCH, tr("push_category_merch")),
        (PUSH_PREF_QUIET, tr("push_quiet_hours")),
    ];

    view! {
        <Show when=move || preferences.get().is_some()>
            <article class="push-setting-card push-preferences-card">
                <div class="push-setting-copy">
                    <div class="push-setting-heading">
                        <span class="push-setting-icon" aria-hidden="true">"◎"</span>
                        <div><strong>{tr("push_what_you_want")}</strong><p>{tr("push_what_you_want_hint")}</p></div>
                    </div>
                    <div class="pref-list">
                        {items.into_iter().map(|(key, label)| view! {
                            <label class="pref-row">
                                <span class="pref-row-label">{label}</span>
                                <input
                                    type="checkbox"
                                    class="pref-switch"
                                    disabled=move || busy.get()
                                    prop:checked=move || preferences
                                        .get()
                                        .as_ref()
                                        .is_some_and(|value| push_preference_enabled(value, key))
                                    on:change=move |event| {
                                        update_push_preference(
                                            preferences,
                                            busy,
                                            error,
                                            key,
                                            event_target_checked(&event),
                                        );
                                    }
                                />
                            </label>
                        }).collect_view()}
                    </div>
                    <small>{tr("push_essential_always")}</small>
                </div>
            </article>
        </Show>
    }
}
