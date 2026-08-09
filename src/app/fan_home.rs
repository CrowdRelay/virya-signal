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
                    let counts = snapshot.counts.clone();
                    let referral = snapshot.referral.clone();
                    let next_event = snapshot.next_event.clone();
                    let city = snapshot.profile.primary_city.clone();
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
                                <Show when=move || !synesthesia.completed>
                                    <div class="progress-track"><span style=format!("width:{}%", (i32::from(synesthesia.rooms_completed).clamp(0, 11) * 100) / 11)></span></div>
                                    <small>{format!("{}/11", synesthesia.rooms_completed.clamp(0, 11))}</small>
                                </Show>
                                <ExternalLink url="https://synesthesia.virya.music/?source=signal-app&resume=1".to_owned() label=if synesthesia.started { tr("open_synesthesia") } else { tr("enter_synesthesia") } error=error />
                            </article>
                            {next_event.map(|event| {
                                let ticket_url = event.ticket_url.clone();
                                let title = event.title.clone();
                                view! {
                                    <article class="home-action-card next-show-card" class:live=event.phase == "live" class:afterglow=event.phase == "afterglow">
                                        <p class="eyebrow">{event_phase_label(&event.phase)}</p>
                                        <strong>{title}</strong>
                                        <p>{event_time_location(&event.starts_at, event.venue.as_deref())}</p>
                                        <Show when=move || event.phase == "live"><p class="signal-live-note">{tr("signal_live_note")}</p></Show>
                                        <Show when=move || event.phase == "afterglow"><p class="signal-afterglow-note">{tr("signal_afterglow_note")}</p></Show>
                                        <div class="home-card-actions">
                                            <button class="ghost" on:click={
                                                let action = snapshot.recommended_action.clone();
                                                move |_| tab.set(recommended_tab(&action))
                                            }>{recommended_label(&snapshot.recommended_action)}</button>
                                            {ticket_url.filter(|_| event.phase == "upcoming").map(|url| view! { <ExternalLink url=url label=tr("tickets_tab") error=error /> })}
                                        </div>
                                    </article>
                                }
                            })}
                        </div>
                        <div class="stats-grid fan-home-stats">
                            <Metric value=counts.active_passes.to_string() label=tr("active_passes")/>
                            <Metric value=counts.area_claims.to_string() label=tr("area_findings")/>
                            <Metric value=referral.qualified.to_string() label=tr("confirmed_referrals")/>
                        </div>
                    }.into_any()
                }).value_or_else(|| view! {
                    <div class="empty-state"><strong>{tr("signal_home_unavailable")}</strong><p>{tr("signal_home_fallback_hint")}</p></div>
                }.into_any())}
            </Show>
        </section>
    }
}
