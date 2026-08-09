#[component]
pub fn Skeleton(#[prop(default = 3)] rows: usize) -> impl IntoView {
    view! { <div class="skeleton-stack" aria-label=tr("loading")>{(0..rows).map(|_| view! { <i></i> }).collect_view()}</div> }
}

#[component]
fn Toast(error: RwSignal<Option<String>>) -> impl IntoView {
    Effect::new(move |_| {
        if error.get().is_some() {
            set_timeout(move || error.set(None), std::time::Duration::from_secs(5));
        }
    });
    let is_success = move || {
        error.with(|msg| {
            msg.as_ref().is_some_and(|m| {
                let lower = m.to_lowercase();
                lower.contains("utworzon")
                    || lower.contains("zapisan")
                    || lower.contains(tr("sent"))
                    || lower.contains("sent")
                    || lower.contains(tr("revoked"))
                    || lower.contains("revoked")
                    || lower.contains("zrealizowan")
                    || lower.contains("gotowy")
                    || lower.contains("zeskanowany")
                    || lower.contains("ponownie")
                    || lower.contains(tr("feedback_was_sent"))
            })
        })
    };
    view! { <Show when=move || error.get().is_some()><button class="toast" class:toast-success=is_success on:click=move |_| error.set(None)>{move || error.get().value_or_else(Default::default)}</button></Show> }
}

fn refresh_operator_parts(
    dashboard: RwSignal<Option<DashboardData>>,
    loading: RwSignal<OperatorLoadingState>,
    error: RwSignal<Option<String>>,
) {
    refresh_operator_events(dashboard, loading, error);
    refresh_operator_qr(dashboard, loading, error);
}

fn refresh_operator_events(
    dashboard: RwSignal<Option<DashboardData>>,
    loading: RwSignal<OperatorLoadingState>,
    error: RwSignal<Option<String>>,
) {
    loading.update(|state| state.events = true);
    spawn_local(async move {
        match bridge::invoke_latest::<Vec<PublicEvent>, _>(
            "operator_events",
            &EmptyArgs {},
            15_000,
            "operator:events",
        )
        .await
        {
            Ok(Some(value)) => dashboard.update(|state| {
                state.get_or_insert_with(DashboardData::default).events = value;
            }),
            Ok(None) => return,
            Err(message) => error.set(Some(message)),
        }
        loading.update(|state| state.events = false);
    });
}

fn refresh_operator_qr(
    dashboard: RwSignal<Option<DashboardData>>,
    loading: RwSignal<OperatorLoadingState>,
    error: RwSignal<Option<String>>,
) {
    loading.update(|state| state.qr = true);
    spawn_local(async move {
        match bridge::invoke_latest::<crate::models::ConcertQrOverview, _>(
            "operator_qr",
            &EmptyArgs {},
            15_000,
            "operator:qr",
        )
        .await
        {
            Ok(Some(value)) => dashboard.update(|state| {
                state.get_or_insert_with(DashboardData::default).qr = Some(value);
            }),
            Ok(None) => return,
            Err(message) => error.set(Some(message)),
        }
        loading.update(|state| state.qr = false);
    });
}

fn refresh_operator_signal(
    overview: RwSignal<Option<OperatorSignalOverview>>,
    loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) {
    if loading.get_untracked() {
        return;
    }
    loading.set(true);
    spawn_local(async move {
        let result = bridge::invoke_latest::<OperatorSignalOverview, _>(
            "operator_signal_overview",
            &EmptyArgs {},
            20_000,
            "operator:signal",
        )
        .await;
        match result {
            Ok(Some(value)) => overview.set(Some(value)),
            Ok(None) => {}
            Err(message) => error.set(Some(message)),
        }
        loading.set(false);
    });
}

fn refresh_operator_ops(
    overview: RwSignal<Option<OperatorOpsOverview>>,
    loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) {
    loading.set(true);
    spawn_local(async move {
        match bridge::invoke_latest::<OperatorOpsOverview, _>(
            "operator_ops_overview",
            &EmptyArgs {},
            20_000,
            "operator:ops",
        )
        .await
        {
            Ok(Some(value)) => overview.set(Some(value)),
            Ok(None) => return,
            Err(message) => error.set(Some(message)),
        }
        loading.set(false);
    });
}

fn refresh_fan_status(status: RwSignal<FanSessionStatus>, error: RwSignal<Option<String>>) {
    spawn_local(async move {
        match bridge::invoke::<FanSessionStatus, _>("fan_status", &EmptyArgs {}).await {
            Ok(value) => status.set(value),
            Err(message) => error.set(Some(message)),
        }
    });
}

fn refresh_fan_home(
    home: RwSignal<Option<FanHomeData>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) {
    loading.update(|state| state.home = true);
    spawn_local(async move {
        match bridge::invoke_latest::<FanHomeData, _>(
            "fan_home",
            &EmptyArgs {},
            12_000,
            "fan:home",
        )
        .await
        {
            Ok(Some(value)) if value.has_supported_schema() => home.set(Some(value)),
            Ok(Some(value)) => error.set(Some(i18n::format(
                "unsupported_signal_snapshot_version",
                &[value.schema_version.to_string()],
            ))),
            Ok(None) => return,
            Err(message) => error.set(Some(message)),
        }
        loading.update(|state| state.home = false);
    });
}

fn refresh_fan_parts(
    dashboard: RwSignal<Option<FanDashboardData>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) {
    loading.set(FanLoadingState::all());
    refresh_fan_events(dashboard, loading, error);
    refresh_fan_referral(dashboard, loading, error);
    refresh_fan_interests(dashboard, loading, error);
    refresh_fan_admission_pass(dashboard, loading, error);
}

fn refresh_fan_events(
    dashboard: RwSignal<Option<FanDashboardData>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) {
    loading.update(|state| state.events = true);
    spawn_local(async move {
        match bridge::invoke_latest::<Vec<PublicEvent>, _>(
            "fan_events",
            &EmptyArgs {},
            15_000,
            "fan:events",
        )
        .await
        {
            Ok(Some(value)) => dashboard.update(|state| {
                state.get_or_insert_with(FanDashboardData::default).events =
                    stable_fan_events(value);
            }),
            Ok(None) => return,
            Err(message) => error.set(Some(message)),
        }
        loading.update(|state| state.events = false);
    });
}

fn refresh_fan_merch(
    merch: RwSignal<Option<MerchCatalog>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) {
    loading.update(|state| state.merch = true);
    spawn_local(async move {
        match bridge::invoke_latest::<MerchCatalog, _>(
            "fan_merch_catalog",
            &EmptyArgs {},
            15_000,
            "fan:merch",
        )
        .await
        {
            Ok(Some(value)) => merch.set(Some(value)),
            Ok(None) => return,
            Err(message) => {
                merch.set(None);
                error.set(Some(message));
            }
        }
        loading.update(|state| state.merch = false);
    });
}

fn refresh_fan_merch_bundles(bundles: RwSignal<Option<FanMerchBundleCatalog>>) {
    spawn_local(async move {
        match bridge::invoke_latest::<FanMerchBundleCatalog, _>(
            "fan_merch_bundles",
            &EmptyArgs {},
            12_000,
            "fan:merch-bundles",
        )
        .await
        {
            Ok(Some(value)) => bundles.set(Some(value)),
            Ok(None) => {}
            Err(_) => bundles.set(None),
        }
    });
}

fn refresh_fan_referral(
    dashboard: RwSignal<Option<FanDashboardData>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) {
    loading.update(|state| state.referral = true);
    spawn_local(async move {
        match bridge::invoke_latest::<ReferralProgress, _>(
            "fan_referral",
            &EmptyArgs {},
            15_000,
            "fan:referral",
        )
        .await
        {
            Ok(Some(value)) => dashboard.update(|state| {
                state.get_or_insert_with(FanDashboardData::default).referral = value;
            }),
            Ok(None) => return,
            Err(message) => error.set(Some(message)),
        }
        loading.update(|state| state.referral = false);
    });
}

fn refresh_fan_interests(
    dashboard: RwSignal<Option<FanDashboardData>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) {
    loading.update(|state| state.interests = true);
    spawn_local(async move {
        match bridge::invoke_latest::<Vec<FanEventInterest>, _>(
            "fan_interests",
            &EmptyArgs {},
            15_000,
            "fan:interests",
        )
        .await
        {
            Ok(Some(value)) => dashboard.update(|state| {
                state
                    .get_or_insert_with(FanDashboardData::default)
                    .interests = stable_fan_interests(value);
            }),
            Ok(None) => return,
            Err(message) => error.set(Some(message)),
        }
        loading.update(|state| state.interests = false);
    });
}

fn refresh_fan_admission_pass(
    dashboard: RwSignal<Option<FanDashboardData>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) {
    loading.update(|state| state.admission_pass = true);
    spawn_local(async move {
        match bridge::invoke_latest::<Option<AdmissionPass>, _>(
            "fan_admission_pass",
            &EmptyArgs {},
            15_000,
            "fan:admission",
        )
        .await
        {
            Ok(Some(value)) => dashboard.update(|state| {
                state
                    .get_or_insert_with(FanDashboardData::default)
                    .admission_pass = value;
            }),
            Ok(None) => return,
            Err(message) => error.set(Some(message)),
        }
        loading.update(|state| state.admission_pass = false);
    });
}

fn refresh_fan_area(
    area: RwSignal<Option<AreaWallet>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) {
    loading.update(|state| state.area = true);
    spawn_local(async move {
        match bridge::invoke_latest::<AreaWallet, _>(
            "fan_area_wallet",
            &EmptyArgs {},
            15_000,
            "fan:area",
        )
        .await
        {
            Ok(Some(value)) => area.set(Some(value)),
            Ok(None) => return,
            Err(message) => error.set(Some(message)),
        }
        loading.update(|state| state.area = false);
    });
}

fn refresh_wallets(
    wallets: RwSignal<Vec<TicketWallet>>,
    loading: Option<RwSignal<FanLoadingState>>,
    error: RwSignal<Option<String>>,
) {
    if let Some(loading) = loading {
        loading.update(|state| state.wallets = true);
    }
    spawn_local(async move {
        match bridge::invoke_latest::<WalletBatch, _>(
            "fan_wallets",
            &EmptyArgs {},
            35_000,
            "fan:wallets",
        )
        .await
        {
            Ok(Some(value)) => {
                wallets.set(stable_wallets(value.wallets));
                if value.failed_count > 0 {
                    let key = if value.cached_count > 0 {
                        "could_not_refresh_orders_cached_orders_available"
                    } else {
                        "could_not_refresh_orders_count_other_tickets_remain_available"
                    };
                    error.set(Some(i18n::format(
                        key,
                        &[
                            value.failed_count.to_string(),
                            value.cached_count.to_string(),
                        ],
                    )));
                }
            }
            Ok(None) => return,
            Err(message) => error.set(Some(message)),
        }
        if let Some(loading) = loading {
            loading.update(|state| state.wallets = false);
        }
    });
}

fn operator_events(dashboard: RwSignal<Option<DashboardData>>) -> Vec<PublicEvent> {
    dashboard.with(|state| {
        state
            .as_ref()
            .map(|data| data.events.clone())
            .value_or_else(Default::default)
    })
}

fn operator_qr_events(
    dashboard: RwSignal<Option<DashboardData>>,
) -> Vec<crate::models::StaffEvent> {
    dashboard.with(|state| {
        state
            .as_ref()
            .and_then(|data| data.qr.as_ref())
            .map(|qr| qr.events.clone())
            .value_or_else(Default::default)
    })
}

fn operator_campaigns(dashboard: RwSignal<Option<DashboardData>>) -> Vec<QrCampaign> {
    dashboard.with(|state| {
        state
            .as_ref()
            .and_then(|data| data.qr.as_ref())
            .map(|qr| qr.campaigns.clone())
            .value_or_else(Default::default)
    })
}

fn fan_events(
    dashboard: RwSignal<Option<FanDashboardData>>,
    public: RwSignal<Option<PublicHomeData>>,
) -> Vec<PublicEvent> {
    stable_fan_events(
        dashboard
            .with(|state| state.as_ref().map(|data| data.events.clone()))
            .or_else(|| public.with(|state| state.as_ref().map(|data| data.events.clone())))
            .value_or_else(Default::default),
    )
}

fn stable_fan_events(mut events: Vec<PublicEvent>) -> Vec<PublicEvent> {
    events.retain(|event| {
        !event.slug.trim().is_empty()
            && !event.title.trim().is_empty()
            && event.slug.len() <= 128
            && event.title.chars().count() <= 240
    });
    events.sort_unstable_by(|left, right| {
        left.starts_at
            .cmp(&right.starts_at)
            .then_with(|| left.slug.cmp(&right.slug))
    });
    events.dedup_by(|left, right| left.slug == right.slug);
    events.truncate(100);
    events
}

fn stable_fan_interests(mut interests: Vec<FanEventInterest>) -> Vec<FanEventInterest> {
    interests.retain(|interest| {
        !interest.event.slug.trim().is_empty() && !interest.event.title.trim().is_empty()
    });
    interests.sort_unstable_by(|left, right| left.event.slug.cmp(&right.event.slug));
    interests.dedup_by(|left, right| left.event.slug == right.event.slug);
    interests.truncate(100);
    interests
}

fn stable_wallets(mut wallets: Vec<TicketWallet>) -> Vec<TicketWallet> {
    wallets.retain(|wallet| !wallet.order.order_id.trim().is_empty());
    wallets.sort_unstable_by(|left, right| left.order.order_id.cmp(&right.order.order_id));
    wallets.dedup_by(|left, right| left.order.order_id == right.order.order_id);
    wallets.truncate(100);
    wallets
}
