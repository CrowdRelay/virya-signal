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
                <Show when=move || !loading.get() || overview.with(|value| value.is_some()) fallback=move || view! { <Skeleton rows=4 height=140 /> }>
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
