#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct BeaconExchangeArgs<'a> {
    api_base_url: &'a str,
    invite: &'a str,
    pin: &'a str,
    radius_km: i32,
    locale: &'a str,
    topics: &'a [String],
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct BeaconPrepareArgs<'a> {
    api_base_url: &'a str,
    pin: &'a str,
    radius_km: i32,
    locale: &'a str,
    topics: &'a [String],
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct BeaconPreferencesArgs<'a> {
    radius_km: i32,
    locale: &'a str,
    topics: &'a [String],
    nearby_gigs_enabled: bool,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct BeaconEventArgs<'a> {
    event_id: Option<&'a str>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct BeaconPressRequestArgs<'a> {
    event_id: Option<&'a str>,
    request_kind: &'a str,
    details: Option<&'a str>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct BeaconEngagementArgs<'a> {
    event_id: &'a str,
    action: &'a str,
    help_kind: Option<&'a str>,
    help_details: Option<&'a str>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct BeaconCoverageArgs<'a> {
    event_id: &'a str,
    coverage_kind: &'a str,
    url: &'a str,
    title: Option<&'a str>,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct BeaconReleaseConfirmArgs<'a> {
    campaign_id: &'a str,
    recipient_name: &'a str,
    recipient_phone: &'a str,
    parcel_locker_code: &'a str,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct BeaconCampaignArgs<'a> {
    campaign_id: &'a str,
}

#[derive(serde::Serialize)]
#[serde(rename_all = "camelCase")]
struct BeaconLeaveArgs {
    do_not_contact: bool,
}

fn default_beacon_topics() -> Vec<String> {
    ["shows", "press_materials", "releases", "interviews", "accreditation"]
        .into_iter()
        .map(str::to_owned)
        .collect()
}

fn beacon_news_text(value: &crate::models::SignalLocalizedText) -> String {
    if i18n::current() == Language::En { value.en.clone() } else { value.pl.clone() }
}

fn beacon_open_url(url: String, error: RwSignal<Option<String>>) {
    spawn_local(async move {
        if let Err(message) = bridge::invoke_unit("open_external_url", &UrlArgs { url: &url }).await {
            error.set(Some(message));
        }
    });
}

fn refresh_beacon_home(
    home: RwSignal<Option<BeaconHomeData>>,
    loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) {
    loading.set(true);
    spawn_local(async move {
        match bridge::invoke_latest::<BeaconHomeData, _>("beacon_home", &EmptyArgs {}, 15_000, "beacon:home").await {
            Ok(Some(value)) => home.set(Some(value)),
            Ok(None) => {}
            Err(message) => error.set(Some(message)),
        }
        loading.set(false);
    });
}

fn refresh_beacon_requests(
    requests: RwSignal<Option<BeaconPressRequestsData>>,
    error: RwSignal<Option<String>>,
) {
    spawn_local(async move {
        match bridge::invoke_latest::<BeaconPressRequestsData, _>("beacon_press_requests", &EmptyArgs {}, 15_000, "beacon:requests").await {
            Ok(Some(value)) => requests.set(Some(value)),
            Ok(None) => {}
            Err(message) => error.set(Some(message)),
        }
    });
}

fn refresh_beacon_releases(
    releases: RwSignal<Option<BeaconReleasesData>>,
    error: RwSignal<Option<String>>,
) {
    spawn_local(async move {
        match bridge::invoke_latest::<BeaconReleasesData, _>("beacon_releases", &EmptyArgs {}, 15_000, "beacon:releases").await {
            Ok(Some(value)) => releases.set(Some(value)),
            Ok(None) => {}
            Err(message) => error.set(Some(message)),
        }
    });
}

fn refresh_beacon_press_room(
    press_room: RwSignal<Option<BeaconPressRoomData>>,
    event_id: Option<String>,
    error: RwSignal<Option<String>>,
) {
    spawn_local(async move {
        match bridge::invoke_latest::<BeaconPressRoomData, _>(
            "beacon_press_room",
            &BeaconEventArgs { event_id: event_id.as_deref() },
            15_000,
            "beacon:press-room",
        ).await {
            Ok(Some(value)) => press_room.set(Some(value)),
            Ok(None) => {}
            Err(message) => error.set(Some(message)),
        }
    });
}

#[component]
fn BeaconPortal(
    mode: RwSignal<RootMode>,
    status: RwSignal<BeaconSessionStatus>,
    status_loading: RwSignal<bool>,
    status_failed: RwSignal<bool>,
    status_refresh: RwSignal<u32>,
    pending_link: RwSignal<bool>,
    push_target: RwSignal<Option<String>>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    view! {
        {move || if status_failed.get() {
            view! { <StatusFailure mode=mode status_refresh=status_refresh label=tr("latarnik_vault_failed") show_back=true /> }.into_any()
        } else if status_loading.get() {
            view! { <AccessLoader mode=mode label=tr("latarnik_vault_checking") show_back=true /> }.into_any()
        } else if status.get().unlocked && !pending_link.get() {
            // A pending one-time invite is a fresh trust ceremony. It must win
            // over a stale launcher snapshot that raced the native beacon_lock.
            view! { <BeaconApp mode=mode status=status push_target=push_target error=error /> }.into_any()
        } else {
            view! { <BeaconAccess mode=mode status=status status_refresh=status_refresh pending_link=pending_link error=error /> }.into_any()
        }}
    }
}

#[component]
fn BeaconAccess(
    mode: RwSignal<RootMode>,
    status: RwSignal<BeaconSessionStatus>,
    status_refresh: RwSignal<u32>,
    pending_link: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let pin = RwSignal::new(String::new());
    let invite = RwSignal::new(String::new());
    let radius = RwSignal::new(100_i32);
    let topics = RwSignal::new(default_beacon_topics());
    let busy = RwSignal::new(false);
    let reactivation = RwSignal::new(false);

    let adopt = move |value: BeaconSessionStatus| {
        pin.set(String::new());
        invite.set(String::new());
        pending_link.set(false);
        reactivation.set(false);
        status.set(value);
        status_refresh.update(|value| *value = value.wrapping_add(1));
    };

    let unlock = move |_| {
        let current = pin.get_untracked();
        if current.chars().count() < 4 { return; }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<BeaconSessionStatus, _>("beacon_unlock", &PinArgs { pin: &current }).await {
                Ok(value) => adopt(value),
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let activate = move |_| {
        let current_pin = pin.get_untracked();
        let current_invite = invite.get_untracked();
        if !new_operator_pin_is_valid(&current_pin) || current_invite.trim().is_empty() { return; }
        let current_topics = topics.get_untracked();
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<BeaconSessionStatus, _>(
                "beacon_exchange_invite",
                &BeaconExchangeArgs {
                    api_base_url: API_BASE,
                    invite: &current_invite,
                    pin: &current_pin,
                    radius_km: radius.get_untracked(),
                    locale: i18n::current().code(),
                    topics: &current_topics,
                },
            ).await {
                Ok(value) => adopt(value),
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let activate_pending = move |_| {
        let current_pin = pin.get_untracked();
        if !new_operator_pin_is_valid(&current_pin) { return; }
        let current_topics = topics.get_untracked();
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<BeaconSessionStatus, _>(
                "beacon_exchange_pending",
                &BeaconPrepareArgs {
                    api_base_url: API_BASE,
                    pin: &current_pin,
                    radius_km: radius.get_untracked(),
                    locale: i18n::current().code(),
                    topics: &current_topics,
                },
            ).await {
                Ok(value) => adopt(value),
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let scan = move |_| {
        let current_pin = pin.get_untracked();
        if !new_operator_pin_is_valid(&current_pin) { return; }
        let current_topics = topics.get_untracked();
        busy.set(true);
        spawn_lifecycle_task(async move {
            let prepared = bridge::invoke_unit(
                "beacon_prepare_invite",
                &BeaconPrepareArgs {
                    api_base_url: API_BASE,
                    pin: &current_pin,
                    radius_km: radius.get_untracked(),
                    locale: i18n::current().code(),
                    topics: &current_topics,
                },
            ).await;
            if let Err(message) = prepared {
                let _ = error.try_set(Some(message));
                let _ = busy.try_set(false);
                return;
            }
            match bridge::scan_and_confirm_beacon().await {
                Ok(Some(value)) => {
                    let _ = pin.try_set(String::new());
                    let _ = invite.try_set(String::new());
                    let _ = pending_link.try_set(false);
                    let _ = status.try_set(value);
                    let _ = status_refresh.try_update(|value| *value = value.wrapping_add(1));
                }
                Ok(None) => {}
                Err(message) => {
                    if let Ok(value) = bridge::invoke_timeout::<BeaconSessionStatus, _>("beacon_status", &EmptyArgs {}, 5_000).await
                        && value.unlocked
                    {
                        let _ = status.try_set(value);
                    } else {
                        let _ = error.try_set(Some(message));
                    }
                }
            }
            let _ = busy.try_set(false);
        });
    };

    let cancel_pending = move |_| {
        if busy.get_untracked() { return; }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke_unit("beacon_clear_pending_invite", &EmptyArgs {}).await {
                Ok(()) => match bridge::invoke::<BeaconSessionStatus, _>("beacon_status", &EmptyArgs {}).await {
                    Ok(value) => {
                        pending_link.set(false);
                        status.set(value);
                    }
                    Err(message) => error.set(Some(message)),
                },
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let toggle_topic = move |topic: &'static str| {
        topics.update(|values| {
            if let Some(index) = values.iter().position(|value| value == topic) {
                if values.len() > 1 { values.remove(index); }
            } else {
                values.push(topic.to_owned());
            }
        });
    };

    let browser = move |_| beacon_open_url(
        if i18n::current() == Language::En { "https://virya.music/latarnik/".to_owned() } else { "https://virya.music/pl/latarnik/".to_owned() },
        error,
    );

    view! {
        <section class="access-screen beacon-access">
            <BackButton mode=mode />
            <div class="access-card beacon-access-card">
                <p class="eyebrow">{tr("latarnik_native_label")}</p>
                <h1>{tr("latarnik_private_network")}</h1>
                <p>{tr("latarnik_access_pitch")}</p>
                <Show when=move || status.get().configured && !pending_link.get() && !reactivation.get() fallback=move || view! {
                    <div class="form-grid beacon-invite-form">
                        <Show when=move || pending_link.get()>
                            <div class="beacon-pending-invite"><strong>{tr("latarnik_invite_received")}</strong><p>{tr("latarnik_invite_received_hint")}</p></div>
                        </Show>
                        <label class="pin-field"><span>{tr("latarnik_pin_create")}</span><small>{tr("latarnik_pin_hint")}</small><input type="password" inputmode="numeric" maxlength="6" autocomplete="new-password" placeholder=tr("pin_example") prop:value=move || pin.get() on:input=move |e| pin.set(normalize_new_operator_pin(event_target_value(&e)))/></label>
                        <div class="radius-picker beacon-radius"><button class:active=move || radius.get()==25 on:click=move |_| radius.set(25)>"25 km"</button><button class:active=move || radius.get()==50 on:click=move |_| radius.set(50)>"50 km"</button><button class:active=move || radius.get()==100 on:click=move |_| radius.set(100)>"100 km"</button><button class:active=move || radius.get()==200 on:click=move |_| radius.set(200)>"200 km"</button></div>
                        <div class="beacon-topic-grid">
                            <button type="button" class:active=move || topics.get().iter().any(|v| v=="shows") on:click=move |_| toggle_topic("shows")>{tr("latarnik_topic_shows")}</button>
                            <button type="button" class:active=move || topics.get().iter().any(|v| v=="press_materials") on:click=move |_| toggle_topic("press_materials")>{tr("latarnik_topic_press")}</button>
                            <button type="button" class:active=move || topics.get().iter().any(|v| v=="releases") on:click=move |_| toggle_topic("releases")>{tr("latarnik_topic_releases")}</button>
                            <button type="button" class:active=move || topics.get().iter().any(|v| v=="interviews") on:click=move |_| toggle_topic("interviews")>{tr("latarnik_topic_interviews")}</button>
                            <button type="button" class:active=move || topics.get().iter().any(|v| v=="accreditation") on:click=move |_| toggle_topic("accreditation")>{tr("latarnik_topic_accreditation")}</button>
                        </div>
                        <Show when=move || pending_link.get()>
                            <button class="primary" disabled=move || busy.get() || !new_operator_pin_is_valid(&pin.get()) on:click=activate_pending>{tr("latarnik_activate_invite")}</button>
                            <button type="button" class="text-button" disabled=move || busy.get() on:click=cancel_pending>{tr("cancel")}</button>
                        </Show>
                        <button type="button" class="confirmation-action primary-scan" disabled=move || busy.get() || !new_operator_pin_is_valid(&pin.get()) on:click=scan><span aria-hidden="true">"▦"</span><strong>{tr("latarnik_scan_invite")}</strong><small>{tr("latarnik_scan_hint")}</small></button>
                        <label>{tr("latarnik_invite_link")}<textarea rows="3" autocomplete="one-time-code" spellcheck="false" placeholder=tr("latarnik_paste_invite") prop:value=move || invite.get() on:input=move |e| invite.set(event_target_value(&e))></textarea></label>
                        <button class="ghost" disabled=move || busy.get() || invite.get().trim().is_empty() || !new_operator_pin_is_valid(&pin.get()) on:click=activate>{tr("latarnik_activate_pasted")}</button>
                        <Show when=move || status.get().configured && !pending_link.get()>
                            <button type="button" class="text-button" disabled=move || busy.get() on:click=move |_| reactivation.set(false)>{tr("cancel")}</button>
                        </Show>
                    </div>
                }>
                    <div class="form-grid">
                        <p>{tr("latarnik_vault_locked")}</p>
                        <label class="pin-field"><span>{tr("latarnik_pin")}</span><input type="password" autocomplete="current-password" placeholder=tr("your_pin") prop:value=move || pin.get() on:input=move |e| pin.set(event_target_value(&e))/></label>
                        <button class="primary" disabled=move || busy.get() || pin.get().chars().count()<4 on:click=unlock>{tr("latarnik_unlock")}</button>
                        <button type="button" class="text-button" disabled=move || busy.get() on:click=move |_| { pin.set(String::new()); reactivation.set(true); }>{tr("latarnik_use_new_invite")}</button>
                    </div>
                </Show>
                <button type="button" class="text-button" on:click=browser>{tr("latarnik_open_web")}</button>
                <p class="inline-note">{tr("latarnik_not_street_team")}</p>
            </div>
        </section>
    }
}

#[component]
fn BeaconApp(
    mode: RwSignal<RootMode>,
    status: RwSignal<BeaconSessionStatus>,
    push_target: RwSignal<Option<String>>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let tab = RwSignal::new(BeaconTab::Briefing);
    let home = RwSignal::new(None::<BeaconHomeData>);
    let news = RwSignal::new(None::<SignalNewsFeed>);
    let press_room = RwSignal::new(None::<BeaconPressRoomData>);
    let requests = RwSignal::new(None::<BeaconPressRequestsData>);
    let releases = RwSignal::new(None::<BeaconReleasesData>);
    let selected_event = RwSignal::new(None::<String>);
    let loading_home = RwSignal::new(true);
    let refresh = RwSignal::new(1_u32);
    let menu_open = RwSignal::new(false);

    Effect::new(move |_| {
        refresh.get();
        match tab.get() {
            BeaconTab::Briefing => {
                refresh_beacon_home(home, loading_home, error);
                spawn_local(async move {
                    match bridge::invoke_latest::<SignalNewsFeed, _>("beacon_news", &EmptyArgs {}, 15_000, "beacon:news").await {
                        Ok(Some(value)) => news.set(Some(value)), Ok(None) => {}, Err(message) => error.set(Some(message)),
                    }
                });
                refresh_beacon_requests(requests, error);
                refresh_beacon_releases(releases, error);
            }
            BeaconTab::Radar => refresh_beacon_home(home, loading_home, error),
            // BeaconPressRoom owns this fetch: it already reacts to both the
            // refresh counter and the selected event, so firing here too would
            // just duplicate the request on every entry into the tab.
            BeaconTab::Press => {}
            BeaconTab::Access => {
                // Access contains Beacon preferences/settings as well as requests/releases.
                // Load the authoritative profile even when a push deep-links straight here.
                refresh_beacon_home(home, loading_home, error);
                refresh_beacon_requests(requests, error);
                refresh_beacon_releases(releases, error);
            }
        }
    });

    Effect::new(move |_| {
        let Some(target) = push_target.get() else { return; };
        push_target.set(None);
        if target.contains("/press") { tab.set(BeaconTab::Press); }
        else if target.contains("/access") || target.contains("/requests") { tab.set(BeaconTab::Access); }
        else if target.contains("/radar") { tab.set(BeaconTab::Radar); }
        else { tab.set(BeaconTab::Briefing); }
        if let Some((_, id)) = target.split_once("event=") {
            let id = id.split('&').next().unwrap_or(id).trim();
            if !id.is_empty() { selected_event.set(Some(id.to_owned())); }
        }
    });

    on_cleanup(move || bridge::invalidate_latest("beacon:"));

    let lock = move |_| {
        bridge::invalidate_latest("beacon:");
        spawn_local(async move {
            match bridge::invoke::<BeaconSessionStatus, _>("beacon_lock", &EmptyArgs {}).await {
                Ok(value) => status.set(value), Err(message) => error.set(Some(message)),
            }
        });
    };
    let switch_fan = move |_| { menu_open.set(false); mode.set(RootMode::Fan); };
    let refresh_all = move |_| { menu_open.set(false); refresh.update(|value| *value = value.wrapping_add(1)); };

    view! {
        <section class="authenticated beacon-authenticated">
            <header class="topbar beacon-topbar">
                <div><p class="eyebrow">{tr("latarnik_native_label")}</p><strong>{move || status.get().session.map(|s| s.display_name).unwrap_or_else(|| tr("latarnik_name").to_owned())}</strong></div>
                <div class="topbar-actions"><span class="live-dot"></span><button class="menu-trigger" aria-label=tr("open_menu") on:click=move |_| menu_open.update(|v| *v = !*v)><i></i><i></i><i></i></button><button aria-label=tr("latarnik_lock") on:click=lock>"×"</button></div>
            </header>
            <Show when=move || menu_open.get()>
                <div class="overflow-backdrop" on:click=move |_| menu_open.set(false)></div>
                <nav class="overflow-menu beacon-menu">
                    <button on:click=switch_fan><span>"⌁"</span>{tr("latarnik_my_signal")}</button>
                    <button on:click=refresh_all><span>"↻"</span>{tr("refresh_all_data")}</button>
                    <button on:click=move |_| { menu_open.set(false); mode.set(RootMode::StaffGate); }><span>"◎"</span>{tr("staff_zone")}</button>
                </nav>
            </Show>
            <div class="content beacon-content">
                {move || match tab.get() {
                    BeaconTab::Briefing => view! { <BeaconBriefing home=home news=news requests=requests releases=releases loading=loading_home tab=tab selected_event=selected_event error=error /> }.into_any(),
                    BeaconTab::Radar => view! { <BeaconRadar home=home loading=loading_home tab=tab selected_event=selected_event refresh=refresh error=error /> }.into_any(),
                    BeaconTab::Press => view! { <BeaconPressRoom press_room=press_room selected_event=selected_event requests=requests refresh=refresh error=error /> }.into_any(),
                    BeaconTab::Access => view! { <BeaconAccessHub home=home requests=requests releases=releases refresh=refresh error=error /> }.into_any(),
                }}
            </div>
            <nav class="bottom-nav four primary-four beacon-bottom-nav">
                <BeaconNav tab=tab own=BeaconTab::Briefing icon="◉" label=tr("latarnik_tab_briefing")/>
                <BeaconNav tab=tab own=BeaconTab::Radar icon="⌁" label=tr("latarnik_tab_radar")/>
                <BeaconNav tab=tab own=BeaconTab::Press icon="▤" label=tr("latarnik_tab_press")/>
                <BeaconNav tab=tab own=BeaconTab::Access icon="◇" label=tr("latarnik_tab_access")/>
            </nav>
        </section>
    }
}

#[component]
fn BeaconNav(tab: RwSignal<BeaconTab>, own: BeaconTab, icon: &'static str, label: &'static str) -> impl IntoView {
    view! { <button class:active=move || tab.get()==own on:click=move |_| tab.set(own)><span class="beacon-nav-glyph" aria-hidden="true">{icon}</span><small>{label}</small></button> }
}

#[component]
fn BeaconBriefing(
    home: RwSignal<Option<BeaconHomeData>>,
    news: RwSignal<Option<SignalNewsFeed>>,
    requests: RwSignal<Option<BeaconPressRequestsData>>,
    releases: RwSignal<Option<BeaconReleasesData>>,
    loading: RwSignal<bool>,
    tab: RwSignal<BeaconTab>,
    selected_event: RwSignal<Option<String>>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <section class="screen beacon-screen">
            <header class="screen-title beacon-hero"><p class="eyebrow">{tr("latarnik_briefing_label")}</p><h2>{tr("latarnik_briefing_title")}</h2><p>{tr("latarnik_briefing_subtitle")}</p></header>
            <Show when=move || !loading.get() || home.with(|value| value.is_some()) fallback=move || view! { <Skeleton rows=3/> }>
                {move || home.get().and_then(|data| data.nearby_events.first().cloned()).map(|event| {
                    let id=event.id.clone(); let title=event.title.clone();
                    view! { <article class="beacon-feature-card"><p class="eyebrow">{tr("latarnik_near_you")}</p><h3>{title}</h3><p>{format!("{} · {} km", event.city, event.distance_km)}</p><button class="primary" on:click=move |_| { selected_event.set(Some(id.clone())); tab.set(BeaconTab::Radar); }>{tr("latarnik_open_radar")}</button></article> }
                })}
            </Show>
            <div class="beacon-status-strip">
                <article><strong>{move || home.get().map(|h| h.nearby_events.len()).unwrap_or(0)}</strong><span>{tr("latarnik_local_signals")}</span></article>
                <article><strong>{move || requests.get().map(|r| r.requests.iter().filter(|v| v.status=="open").count()).unwrap_or(0)}</strong><span>{tr("latarnik_open_requests")}</span></article>
                <article><strong>{move || releases.get().map(|r| r.campaigns.iter().filter(|v| matches!(v.recipient_status.as_str(),"eligible"|"notified")).count()).unwrap_or(0)}</strong><span>{tr("latarnik_allocations")}</span></article>
            </div>
            <div class="section-head"><div><p class="eyebrow">{tr("latarnik_news_label")}</p><h3>{tr("latarnik_news_title")}</h3></div></div>
            <div class="card-list beacon-news-list">
                {move || news.get().map(|feed| feed.items.into_iter().take(6).map(|item| {
                    let title=beacon_news_text(&item.title); let summary=beacon_news_text(&item.summary); let url=beacon_news_text(&item.url); let tag=beacon_news_text(&item.tag);
                    view! { <article class="beacon-news-card"><p class="eyebrow">{tag}</p><h3>{title}</h3><p>{summary}</p><button class="ghost" on:click=move |_| beacon_open_url(url.clone(), error)>{tr("latarnik_read")}</button></article> }
                }).collect_view())}
            </div>
        </section>
    }
}

#[component]
fn BeaconRadar(
    home: RwSignal<Option<BeaconHomeData>>,
    loading: RwSignal<bool>,
    tab: RwSignal<BeaconTab>,
    selected_event: RwSignal<Option<String>>,
    refresh: RwSignal<u32>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let helping = RwSignal::new(None::<String>);
    let help_kind = RwSignal::new("photos".to_owned());
    let help_details = RwSignal::new(String::new());
    let busy = RwSignal::new(false);

    let action = move |event_id: String, action: &'static str, kind: Option<String>| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        let details = help_details.get_untracked();
        spawn_local(async move {
            match bridge::invoke::<BeaconEngagementResult, _>(
                "beacon_engagement",
                &BeaconEngagementArgs {
                    event_id: &event_id,
                    action,
                    help_kind: kind.as_deref(),
                    help_details: (!details.trim().is_empty()).then_some(details.trim()),
                },
            )
            .await
            {
                Ok(_) => {
                    helping.set(None);
                    help_details.set(String::new());
                    refresh.update(|value| *value = value.wrapping_add(1));
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    view! {
        <section class="screen beacon-screen">
            <header class="screen-title">
                <p class="eyebrow">{tr("latarnik_radar_label")}</p>
                <h2>{tr("latarnik_radar_title")}</h2>
                <p>{tr("latarnik_radar_subtitle")}</p>
            </header>
            <Show when=move || !loading.get() || home.with(|value| value.is_some()) fallback=move || view! { <Skeleton rows=4/> }>
                <div class="card-list beacon-event-list">
                    {move || home.get().map(|data| data.nearby_events.into_iter().map(|event| {
                        let event_id = event.id.clone();
                        let selected_id = event_id.clone();
                        let interest_id = event_id.clone();
                        let decline_id = event_id.clone();
                        let help_open_id = event_id.clone();
                        let press_id = event_id.clone();
                        let event_city = event.city.clone();
                        let event_location = event.venue.clone().unwrap_or_else(|| event_city.clone());
                        let engagement = event.engagement_status.clone();
                        view! {
                            <article class="beacon-event-card" class:selected=move || selected_event.get().as_deref() == Some(selected_id.as_str())>
                                <p class="eyebrow">{format!("{} KM · {}", event.distance_km, event_city.to_uppercase())}</p>
                                <h3>{event.title}</h3>
                                <p>{format!("{} · {}", human_time(&event.starts_at), event_location)}</p>
                                {engagement.map(|status| view! { <span class="cache-badge">{status.to_uppercase()}</span> })}
                                <div class="beacon-action-row">
                                    <button class="ghost" disabled=move || busy.get() on:click=move |_| action(interest_id.clone(), "interested", None)>{tr("latarnik_interested")}</button>
                                    <button class="ghost" on:click=move |_| helping.set(Some(help_open_id.clone()))>{tr("latarnik_can_help")}</button>
                                    <button class="text-button" disabled=move || busy.get() on:click=move |_| action(decline_id.clone(), "declined", None)>{tr("latarnik_not_this_time")}</button>
                                </div>
                                <Show when=move || helping.get().as_deref() == Some(event_id.as_str())>
                                    <div class="beacon-help-box">
                                        <label>{tr("latarnik_help_kind")}<select prop:value=move || help_kind.get() on:change=move |e| help_kind.set(event_target_value(&e))><option value="article">{tr("latarnik_help_article")}</option><option value="radio">{tr("latarnik_help_radio")}</option><option value="podcast">{tr("latarnik_help_podcast")}</option><option value="photos">{tr("latarnik_help_photos")}</option><option value="share">{tr("latarnik_help_share")}</option><option value="contact">{tr("latarnik_help_contact")}</option><option value="other">{tr("latarnik_help_other")}</option></select></label>
                                        <label>{tr("latarnik_details_optional")}<textarea rows="2" prop:value=move || help_details.get() on:input=move |e| help_details.set(event_target_value(&e))></textarea></label>
                                        <button class="primary" disabled=move || busy.get() on:click=move |_| if let Some(id) = helping.get_untracked() { action(id, "helping", Some(help_kind.get_untracked())); }>{tr("latarnik_confirm_help")}</button>
                                    </div>
                                </Show>
                                <button class="primary beacon-press-cta" on:click=move |_| { selected_event.set(Some(press_id.clone())); tab.set(BeaconTab::Press); }>{tr("latarnik_open_press_room")}</button>
                            </article>
                        }
                    }).collect_view())}
                </div>
            </Show>
        </section>
    }
}

#[component]
fn BeaconPressRoom(
    press_room: RwSignal<Option<BeaconPressRoomData>>,
    selected_event: RwSignal<Option<String>>,
    requests: RwSignal<Option<BeaconPressRequestsData>>,
    refresh: RwSignal<u32>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let request_kind = RwSignal::new("press_photo".to_owned());
    let details = RwSignal::new(String::new());
    let coverage_kind = RwSignal::new("article".to_owned());
    let coverage_url = RwSignal::new(String::new());
    let coverage_title = RwSignal::new(String::new());
    let coverage_status = RwSignal::new(None::<String>);
    let busy = RwSignal::new(false);

    Effect::new(move |_| {
        refresh.get();
        let event_selected = selected_event.get().is_some();
        let current_kind = request_kind.get_untracked();
        if event_selected && current_kind == "press_photo" {
            request_kind.set("accreditation".to_owned());
        } else if !event_selected && current_kind == "accreditation" {
            request_kind.set("press_photo".to_owned());
        }
        refresh_beacon_press_room(press_room, selected_event.get_untracked(), error);
    });

    let request = move |_| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        let kind = request_kind.get_untracked();
        let text = details.get_untracked();
        let event = selected_event.get_untracked();
        spawn_local(async move {
            match bridge::invoke::<BeaconMutationResult, _>(
                "beacon_press_request_create",
                &BeaconPressRequestArgs {
                    event_id: event.as_deref(),
                    request_kind: &kind,
                    details: (!text.trim().is_empty()).then_some(text.trim()),
                },
            )
            .await
            {
                Ok(_) => {
                    details.set(String::new());
                    refresh_beacon_requests(requests, error);
                    refresh.update(|value| *value = value.wrapping_add(1));
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let submit_coverage = move |_| {
        if busy.get_untracked() {
            return;
        }
        let Some(event_id) = selected_event.get_untracked() else {
            return;
        };
        let url = coverage_url.get_untracked();
        if url.trim().is_empty() {
            return;
        }
        let kind = coverage_kind.get_untracked();
        let title = coverage_title.get_untracked();
        busy.set(true);
        coverage_status.set(None);
        spawn_local(async move {
            match bridge::invoke::<BeaconMutationResult, _>(
                "beacon_coverage",
                &BeaconCoverageArgs {
                    event_id: &event_id,
                    coverage_kind: &kind,
                    url: url.trim(),
                    title: (!title.trim().is_empty()).then_some(title.trim()),
                },
            )
            .await
            {
                Ok(_) => {
                    coverage_url.set(String::new());
                    coverage_title.set(String::new());
                    coverage_status.set(Some(tr("latarnik_coverage_saved").to_owned()));
                    refresh.update(|value| *value = value.wrapping_add(1));
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    view! {
        <section class="screen beacon-screen">
            <header class="screen-title">
                <p class="eyebrow">{tr("latarnik_press_label")}</p>
                <h2>{tr("latarnik_press_title")}</h2>
                {move || press_room.get().and_then(|room| room.event).map(|event| {
                    view! { <p>{format!("{} · {}", event.title, event.city.unwrap_or_default())}</p> }
                })}
            </header>
            <div class="beacon-assets">
                {move || press_room.get().map(|room| room.assets.into_iter().map(|asset| {
                    let label = if i18n::current() == Language::En { asset.label_en } else { asset.label_pl };
                    let url = asset.url;
                    view! {
                        <article class="beacon-asset-card">
                            <span class="cache-badge">{asset.asset_kind.to_uppercase()}</span>
                            <strong>{label}</strong>
                            <button class="ghost" on:click=move |_| beacon_open_url(url.clone(), error)>{tr("latarnik_open_asset")}</button>
                        </article>
                    }
                }).collect_view())}
            </div>
            <article class="beacon-request-card">
                <p class="eyebrow">{tr("latarnik_need_something")}</p>
                <h3>{tr("latarnik_request_material")}</h3>
                <label>{tr("type")}<select prop:value=move || request_kind.get() on:change=move |e| request_kind.set(event_target_value(&e))><Show when=move || selected_event.get().is_some()><option value="accreditation">{tr("latarnik_request_accreditation")}</option></Show><option value="press_photo">{tr("latarnik_request_photos")}</option><option value="wav">WAV</option><option value="clean_version">{tr("latarnik_request_clean")}</option><option value="interview">{tr("latarnik_request_interview")}</option><option value="custom">{tr("latarnik_request_other")}</option></select></label>
                <label>{tr("latarnik_details_optional")}<textarea rows="3" prop:value=move || details.get() on:input=move |e| details.set(event_target_value(&e))></textarea></label>
                <button class="primary" disabled=move || busy.get() || (request_kind.get()=="custom" && details.get().trim().is_empty()) || (request_kind.get()=="accreditation" && selected_event.get().is_none()) on:click=request>{tr("latarnik_send_request")}</button>
                <p class="inline-note">{tr("latarnik_accreditation_note")}</p>
            </article>
            <Show when=move || selected_event.get().is_some()>
                <article class="beacon-request-card beacon-coverage-card">
                    <p class="eyebrow">{tr("latarnik_coverage_label")}</p>
                    <h3>{tr("latarnik_coverage_title")}</h3>
                    <p>{tr("latarnik_coverage_hint")}</p>
                    <label>{tr("latarnik_coverage_kind")}<select prop:value=move || coverage_kind.get() on:change=move |e| coverage_kind.set(event_target_value(&e))><option value="article">{tr("latarnik_help_article")}</option><option value="radio">{tr("latarnik_help_radio")}</option><option value="video">{tr("latarnik_coverage_video")}</option><option value="photo">{tr("latarnik_help_photos")}</option><option value="social">{tr("latarnik_coverage_social")}</option><option value="podcast">{tr("latarnik_help_podcast")}</option><option value="other">{tr("latarnik_help_other")}</option></select></label>
                    <label>{tr("latarnik_coverage_url")}<input type="url" inputmode="url" placeholder="https://" prop:value=move || coverage_url.get() on:input=move |e| coverage_url.set(event_target_value(&e))/></label>
                    <label>{tr("latarnik_coverage_title_optional")}<input prop:value=move || coverage_title.get() on:input=move |e| coverage_title.set(event_target_value(&e))/></label>
                    <button class="primary" disabled=move || busy.get() || coverage_url.get().trim().is_empty() on:click=submit_coverage>{tr("latarnik_coverage_submit")}</button>
                    {move || coverage_status.get().map(|message| view! { <small class="success">{message}</small> })}
                </article>
            </Show>
        </section>
    }
}

#[component]
fn BeaconAccessHub(
    home: RwSignal<Option<BeaconHomeData>>,
    requests: RwSignal<Option<BeaconPressRequestsData>>,
    releases: RwSignal<Option<BeaconReleasesData>>,
    refresh: RwSignal<u32>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let campaign = RwSignal::new(None::<String>);
    let name = RwSignal::new(String::new());
    let phone = RwSignal::new(String::new());
    let locker = RwSignal::new(String::new());
    let decline_candidate = RwSignal::new(None::<(String, String)>);
    let busy = RwSignal::new(false);

    let confirm_release = move |_| {
        let Some(id) = campaign.get_untracked() else {
            return;
        };
        let recipient_name = name.get_untracked();
        let recipient_phone = phone.get_untracked();
        let parcel_locker = locker.get_untracked();
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<BeaconMutationResult, _>(
                "beacon_release_confirm",
                &BeaconReleaseConfirmArgs {
                    campaign_id: &id,
                    recipient_name: &recipient_name,
                    recipient_phone: &recipient_phone,
                    parcel_locker_code: &parcel_locker,
                },
            )
            .await
            {
                Ok(_) => {
                    campaign.set(None);
                    name.set(String::new());
                    phone.set(String::new());
                    locker.set(String::new());
                    refresh_beacon_releases(releases, error);
                    refresh.update(|value| *value = value.wrapping_add(1));
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let decline_release = move |id: String| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<BeaconMutationResult, _>(
                "beacon_release_decline",
                &BeaconCampaignArgs { campaign_id: &id },
            )
            .await
            {
                Ok(_) => {
                    refresh_beacon_releases(releases, error);
                    refresh.update(|value| *value = value.wrapping_add(1));
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let confirm_decline_release = move |_| {
        let Some((id, _)) = decline_candidate.get_untracked() else { return; };
        decline_candidate.set(None);
        decline_release(id);
    };

    view! {
        <section class="screen beacon-screen">
            <header class="screen-title">
                <p class="eyebrow">{tr("latarnik_access_label")}</p>
                <h2>{tr("latarnik_access_title")}</h2>
                <p>{tr("latarnik_access_subtitle")}</p>
            </header>
            <div class="section-head"><h3>{tr("latarnik_requests")}</h3></div>
            <div class="card-list">
                {move || requests.get().map(|data| data.requests.into_iter().map(|request| {
                    let title = request.event_title.unwrap_or_else(|| request.request_kind.clone());
                    let note = request.resolution_note.unwrap_or_else(|| request.details.unwrap_or_default());
                    view! {
                        <article class="beacon-request-status">
                            <span class="cache-badge">{request.status.to_uppercase()}</span>
                            <strong>{title}</strong>
                            <p>{note}</p>
                        </article>
                    }
                }).collect_view())}
            </div>
            <div class="section-head"><h3>{tr("latarnik_release_allocations")}</h3></div>
            <div class="card-list">
                {move || releases.get().map(|data| data.campaigns.into_iter().map(|release| {
                    let campaign_id = release.campaign_id.clone();
                    let decline_id = campaign_id.clone();
                    let status = release.recipient_status.clone();
                    let can_respond = matches!(status.as_str(), "eligible" | "notified");
                    let status_label = status.to_uppercase();
                    let title = release.title;
                    let decline_title = title.clone();
                    let product = format!("{} · {}", release.product_name, release.variant_label);
                    let deadline = format!("{} {}", tr("latarnik_claim_until"), human_time(&release.claim_deadline));
                    let actions = can_respond.then(|| {
                        let confirm_id = campaign_id.clone();
                        view! {
                            <div class="beacon-action-row">
                                <button class="primary" on:click=move |_| campaign.set(Some(confirm_id.clone()))>{tr("latarnik_confirm_delivery")}</button>
                                <button class="text-button" disabled=move || busy.get() on:click=move |_| decline_candidate.set(Some((decline_id.clone(), decline_title.clone())) )>{tr("latarnik_decline")}</button>
                            </div>
                        }
                    });
                    view! {
                        <article class="beacon-release-card">
                            <p class="eyebrow">{status_label}</p>
                            <h3>{title}</h3>
                            <p>{product}</p>
                            <small>{deadline}</small>
                            {actions}
                        </article>
                    }
                }).collect_view())}
            </div>
            <Show when=move || decline_candidate.get().is_some()>
                {move || decline_candidate.get().map(|(_, title)| view! {
                    <article class="beacon-delivery-form beacon-decline-confirm">
                        <h3>{tr("latarnik_decline_release_confirm_title")}</h3>
                        <p>{i18n::format("latarnik_decline_release_confirm_hint", &[title])}</p>
                        <button class="danger" disabled=move || busy.get() on:click=confirm_decline_release>{tr("latarnik_decline")}</button>
                        <button class="text-button" disabled=move || busy.get() on:click=move |_| decline_candidate.set(None)>{tr("cancel")}</button>
                    </article>
                })}
            </Show>
            <Show when=move || campaign.get().is_some()>
                <article class="beacon-delivery-form">
                    <h3>{tr("latarnik_delivery_details")}</h3>
                    <label>{tr("latarnik_recipient_name")}<input autocomplete="name" prop:value=move || name.get() on:input=move |e| name.set(event_target_value(&e))/></label>
                    <label>{tr("latarnik_phone")}<input type="tel" prop:value=move || phone.get() on:input=move |e| phone.set(event_target_value(&e))/></label>
                    <label>{tr("latarnik_parcel_locker")}<input prop:value=move || locker.get() on:input=move |e| locker.set(event_target_value(&e))/></label>
                    <button class="primary" disabled=move || busy.get() || name.get().trim().is_empty() || phone.get().trim().is_empty() || locker.get().trim().is_empty() on:click=confirm_release>{tr("latarnik_save_delivery")}</button>
                    <button class="text-button" on:click=move |_| campaign.set(None)>{tr("cancel")}</button>
                </article>
            </Show>
            <BeaconSettings home=home refresh=refresh error=error />
        </section>
    }
}

include!("beacon/settings.rs");
