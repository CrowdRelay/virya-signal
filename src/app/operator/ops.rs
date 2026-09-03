#[component]
fn OpsPanel(
    overview: RwSignal<Option<OperatorOpsOverview>>,
    loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let refresh = move |_| refresh_operator_ops(overview, loading, error);
    view! {
        <section class="ops-panel"><div class="section-head"><div><p class="eyebrow">CONTROL PLANE</p><h3>{tr("queues_and_deliveries")}</h3></div><button class="text-button" on:click=refresh disabled=move || loading.get()>{tr("refresh_2")}</button></div>
            <p class="panel-hint">{tr("ops_panel_hint")}</p>
            <Show when=move || !loading.get() || overview.with(|value| value.is_some()) fallback=move || view! { <Skeleton rows=2 height=120 /> }>
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
                        <div class="ops-metrics"><Metric value=summary.outbox.pending.to_string() label="outbox pending"/><Metric value=summary.outbox.dead.to_string() label="outbox dead"/><Metric value=summary.deliveries.pending.to_string() label="webhook pending"/><Metric value=summary.deliveries.dead.to_string() label="webhook dead"/><Metric value=summary.push.pending.to_string() label="push pending"/><Metric value=summary.push.processing.to_string() label="push in-flight"/><Metric value=summary.push.dead.to_string() label="push failed"/><Metric value=summary.push.delivered_24h.to_string() label="push 24h"/></div>
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
