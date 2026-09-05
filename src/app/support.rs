/// Unit separator used to delimit the error kind from the message in the
/// error string. The bridge embeds Tauri errors as `kind\x1fmessage`; errors
/// without a structured kind (JS bridge errors, timeouts) have no prefix.
const ERROR_KIND_SEP: char = '\x1f';

/// Extracts the stable error kind from an error string. Returns `"unknown"`
/// for errors that don't carry a structured kind (e.g. JS bridge timeouts).
fn error_kind(msg: &str) -> &str {
    msg.split(ERROR_KIND_SEP).next().unwrap_or("unknown")
}

/// Extracts the human-readable message from an error string. For errors
/// without a structured kind prefix, returns the whole string.
pub fn error_message(msg: &str) -> &str {
    msg.split_once(ERROR_KIND_SEP)
        .map(|(_, message)| message)
        .unwrap_or(msg)
}

/// Classifies a toast message as success, error, or transient. Success
/// messages are identified by exact match against the translated values of
/// known success keys, plus prefix matching for format-string templates
/// (where `{}` is replaced with a dynamic value). This replaces the old
/// fragile substring matching that relied on hardcoded Polish fragments
/// and missed messages whose translation didn't contain those fragments.
fn is_success_message(msg: &str) -> bool {
    let display = error_message(msg);
    // Direct exact matches against translated success keys.
    let exact = [
        tr("admission_pass_has_been_revoked"),
        tr("qr_campaign_created"),
        tr("campaign_has_been_disabled"),
        tr("show_saved_to_your_signal"),
        tr("tickets_saved_to_the_wallet"),
        tr("we_resent_the_wallet_by_email"),
        tr("synesthesia_result_saved_in_signal"),
        tr("feedback_was_sent_anonymously_thank_you"),
        tr("location_saved"),
        tr("admission_pass_assigned_to_this_device"),
    ];
    if exact.contains(&display) {
        return true;
    }
    // Format-string templates: match on the prefix before the first `{}`.
    // The formatted message will start with the same prefix.
    let format_prefixes = [
        tr("order_saved_complete_the_secure_stripe_payment"),
        tr("payment_opened_for_order"),
    ];
    format_prefixes.iter().any(|template| {
        let prefix = template.split('{').next().unwrap_or("");
        !prefix.is_empty() && display.starts_with(prefix)
    })
}


///
/// `height` is the height of one row, and it matters: the point of a skeleton
/// is that the content replacing it lands in the same place. A fixed 82px row
/// standing in for a 184px card moves everything below it down by a hundred
/// pixels per row the moment data arrives, which is the shift the skeleton
/// exists to avoid. Pass the height of whatever this panel actually renders.
#[component]
pub fn Skeleton(
    #[prop(default = 3)] rows: usize,
    #[prop(default = 82)] height: u32,
) -> impl IntoView {
    let row_style = format!("height:{height}px");
    view! {
        <div class="skeleton-stack" role="status" aria-label=tr("loading")>
            {(0..rows).map(|_| view! { <i style=row_style.clone()></i> }).collect_view()}
        </div>
    }
}

/// `suppress_transient` keeps network blips, timeouts and cancellations off the
/// surface while still showing the two things a fan must see: the confirmation
/// for something they just did, and a real failure of it. Fan-facing writes
/// (saving a show, importing a wallet, opening Stripe, setting a city) all
/// reported through this signal and nothing rendered it in fan mode, so every
/// one of those confirmations was written and thrown away.
#[component]
fn Toast(
    error: RwSignal<Option<String>>,
    #[prop(default = false)] suppress_transient: bool,
) -> impl IntoView {
    // Each toast owns its dismissal generation. An older timeout must never
    // clear a newer message that replaced it before the five-second window.
    let dismiss_generation = RwSignal::new(0_u64);
    Effect::new(move |_| {
        if error.get().is_some() {
            let generation = dismiss_generation.get_untracked().wrapping_add(1);
            dismiss_generation.set(generation);
            // Transient/noisy errors get a shorter display window; real
            // errors and success messages get the full 5 seconds.
            let timeout = error.with(|msg| {
                msg.as_ref()
                    .map(|m| classify_error_timeout(m.as_str()))
                    .unwrap_or(std::time::Duration::from_secs(5))
            });
            set_timeout(
                move || {
                    if dismiss_generation.try_get_untracked() == Some(generation) {
                        let _ = error.try_set(None);
                    }
                },
                timeout,
            );
        }
    });
    let is_success = move || {
        error.with(|msg| {
            msg.as_ref().is_some_and(|m| is_success_message(m))
        })
    };
    let is_transient = move || {
        error.with(|msg| {
            msg.as_ref().is_some_and(|m| is_transient_kind(error_kind(m)))
        })
    };
    // A suppressed message still expires on its own timer above, so nothing
    // stays wedged in the signal waiting for a reader that never comes.
    let visible = move || error.get().is_some() && !(suppress_transient && is_transient());
    view! {
        <Show when=visible>
            <button
                class="toast"
                class:toast-success=is_success
                class:toast-transient=is_transient
                role="status"
                aria-live="polite"
                on:click=move |_| error.set(None)
            >
                {move || error.get().map(|m| error_message(&m).to_owned()).unwrap_or_default()}
            </button>
        </Show>
    }
}

/// Runs `action` when Enter is pressed in a field.
///
/// None of the access fields sit inside a `<form>`, so the browser's implicit
/// submit never existed: a fan who typed their PIN and pressed the keyboard's
/// Go key got nothing and had to reach for the button. On a screen that is
/// opened several times a day that is the difference between two gestures and
/// one.
fn on_enter(
    mut action: impl FnMut() + 'static,
) -> impl FnMut(leptos::ev::KeyboardEvent) + 'static {
    move |event: leptos::ev::KeyboardEvent| {
        if event.key() == "Enter" {
            event.prevent_default();
            action();
        }
    }
}

/// Classifies an error by kind and returns the appropriate display timeout.
/// Transient/network errors get a shorter window (2.5s) because they're
/// noisy and self-resolving. Real errors and success messages get 5s.
fn classify_error_timeout(message: &str) -> std::time::Duration {
    if is_transient_kind(error_kind(message)) {
        std::time::Duration::from_millis(2500)
    } else {
        std::time::Duration::from_secs(5)
    }
}

/// Returns true for transient error kinds that auto-resolve (network blips,
/// timeouts, cancellations). Classified by the structured `kind` field, not
/// by substring matching on the translated message — so a translation change
/// can never break classification.
fn is_transient_kind(kind: &str) -> bool {
    matches!(
        kind,
        "network" | "background_task" | "timeout" | "cancelled" | "offline"
    )
}

/// Sets an error on the shared RwSignal, but suppresses rapid-fire
/// duplicates. If the same message is already showing, it's silently
/// dropped instead of re-triggering the toast. This prevents error storms
/// when multiple parallel requests fail simultaneously (e.g. all API
/// calls timing out at once).
pub fn set_error_debounced(error: RwSignal<Option<String>>, message: String) {
    let is_duplicate = error.with(|current| {
        current.as_ref().is_some_and(|m| m == &message)
    });
    if is_duplicate {
        return;
    }
    error.set(Some(message));
}

/// Returns true for transient/background errors that should never surface
/// to the fan. The app recovers silently via cached data + the next refresh
/// cycle (tab switch, pull-to-refresh, or status_refresh). The fan never
/// sees these — the data signal stays at its last value and the skeleton
/// only shows when there is genuinely nothing to display.
#[allow(dead_code)]
pub fn is_transient_error(message: &str) -> bool {
    // Structured kind takes priority — it's stable across translations.
    let kind = error_kind(message);
    if kind != "unknown" {
        return is_transient_kind(kind);
    }
    // Fallback for errors without a structured kind (JS bridge errors that
    // don't come through Tauri's AppError serialization).
    let lower = error_message(message).to_lowercase();
    lower.contains("timeout")
        || lower.contains("timed out")
        || lower.contains("network")
        || lower.contains("connection")
        || lower.contains("cancelled")
        || lower.contains("offline")
        || lower.contains("unavailable")
        || lower.contains("temporarily")
        || lower.contains("reset by peer")
        || lower.contains("broken pipe")
}

fn latest_request_completed<T>(result: &Result<Option<T>, String>) -> bool {
    // `invoke_latest` maps an invalidated/stale invocation to `Ok(None)`. The
    // newer invocation owns the loading flag, so the stale completion must not
    // clear it while the replacement request is still in flight.
    !matches!(result, Ok(None))
}

/// Hands the operator's cached panel snapshot to a caller that owns some of
/// those panels. The panels are split across two components, and the native
/// side keeps the decrypted record in memory after the first read, so each
/// caller can ask for it without paying for a second vault decrypt.
fn with_operator_cached_sections(
    apply: impl FnOnce(crate::models::OperatorSectionsSnapshot) + 'static,
) {
    if !bridge::native_available() {
        return;
    }
    spawn_local(async move {
        if let Ok(Some(snapshot)) = bridge::invoke_timeout::<
            Option<crate::models::OperatorSectionsSnapshot>,
            _,
        >("operator_cached_sections", &EmptyArgs {}, 2_000)
        .await
        {
            apply(snapshot);
        }
    });
}

fn refresh_operator_parts(
    dashboard: RwSignal<Option<DashboardData>>,
    loading: RwSignal<OperatorLoadingState>,
    error: RwSignal<Option<String>>,
) {
    refresh_operator_events(dashboard, loading, error);
    refresh_operator_qr(dashboard, loading, error);
}

/// `blank_only_when_empty` for the operator dashboard's loading struct.
fn blank_operator_only_when_empty(
    loading: RwSignal<OperatorLoadingState>,
    has_data: bool,
    mark: impl Fn(&mut OperatorLoadingState, bool) + 'static,
) {
    if !has_data {
        loading.update(|state| mark(state, true));
    }
}

fn refresh_operator_events(
    dashboard: RwSignal<Option<DashboardData>>,
    loading: RwSignal<OperatorLoadingState>,
    error: RwSignal<Option<String>>,
) {
    // Same rule as the fan sections: a refresh over a populated list stays
    // silent instead of dropping the list back to a skeleton.
    blank_operator_only_when_empty(
        loading,
        dashboard.with_untracked(|value| value.as_ref().is_some_and(|data| !data.events.is_empty())),
        |state, value| state.events = value,
    );
    spawn_local(async move {
        let result = bridge::invoke_latest::<Vec<PublicEvent>, _>(
            "operator_events",
            &EmptyArgs {},
            15_000,
            "operator:events",
        )
        .await;
        let completed = latest_request_completed(&result);
        match result {
            Ok(Some(value)) => dashboard.update(|state| {
                state.get_or_insert_with(DashboardData::default).events = value;
            }),
            Ok(None) => {}
            Err(message) => set_error_debounced(error, message),
        }
        if completed {
            loading.update(|state| state.events = false);
        }
    });
}

fn refresh_operator_qr(
    dashboard: RwSignal<Option<DashboardData>>,
    loading: RwSignal<OperatorLoadingState>,
    error: RwSignal<Option<String>>,
) {
    loading.update(|state| state.qr = true);
    spawn_local(async move {
        let result = bridge::invoke_latest::<crate::models::ConcertQrOverview, _>(
            "operator_qr",
            &EmptyArgs {},
            15_000,
            "operator:qr",
        )
        .await;
        let completed = latest_request_completed(&result);
        match result {
            Ok(Some(value)) => dashboard.update(|state| {
                state.get_or_insert_with(DashboardData::default).qr = Some(value);
            }),
            Ok(None) => {}
            Err(message) => set_error_debounced(error, message),
        }
        if completed {
            loading.update(|state| state.qr = false);
        }
    });
}

/// The `RwSignal<bool>` counterpart of `blank_only_when_empty`, for the
/// operator screens whose loading state is a single flag rather than a struct.
///
/// Without it those panels set the flag on every refresh, so a periodic or
/// manual refresh replaced a populated panel with a skeleton and then put the
/// same content back — a blank flash on data the app was already holding. The
/// fan screens never did this; the operator screens did, on every cycle.
fn blank_flag_only_when_empty(loading: RwSignal<bool>, has_data: bool) {
    if !has_data {
        loading.set(true);
    }
}

fn refresh_operator_signal(
    overview: RwSignal<Option<OperatorSignalOverview>>,
    loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) {
    if loading.get_untracked() {
        return;
    }
    blank_flag_only_when_empty(loading, overview.with_untracked(|value| value.is_some()));
    spawn_lifecycle_task(async move {
        match bridge::invoke_timeout::<OperatorSignalOverview, _>(
            "operator_signal_overview",
            &EmptyArgs {},
            20_000,
        )
        .await
        {
            Ok(value) => {
                let _ = overview.try_set(Some(value));
            }
            Err(message) => {
                let _ = error.try_set(Some(message));
            }
        }
        let _ = loading.try_set(false);
    });
}

/// A section only drops to its skeleton when it has nothing to show. Refreshing
/// over data that is already on screen stays silent and swaps the new value in
/// when it lands, so switching tabs, reopening the app or resuming from the
/// background no longer flashes placeholders over content we already hold.
fn blank_only_when_empty(
    loading: RwSignal<FanLoadingState>,
    has_data: bool,
    mark: impl Fn(&mut FanLoadingState, bool) + 'static,
) {
    if !has_data {
        loading.update(|state| mark(state, true));
    }
}

fn refresh_fan_home(
    home: RwSignal<Option<FanHomeData>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) {
    blank_only_when_empty(loading, home.with_untracked(|value| value.is_some()), |state, value| state.home = value);
    // The cached snapshot used to be awaited before the live request even
    // started, so a slow vault read delayed the network call it was supposed to
    // cover for. Both run at once now and the live answer always wins.
    if bridge::native_available() && home.with_untracked(|value| value.is_none()) {
        spawn_local(async move {
            if let Ok(Some(value)) = bridge::invoke_timeout::<Option<FanHomeData>, _>(
                "fan_cached_home",
                &EmptyArgs {},
                2_000,
            )
            .await
                && value.has_supported_schema()
                && home.get_untracked().is_none()
            {
                home.set(Some(value));
                loading.update(|state| state.home = false);
            }
        });
    }
    spawn_local(async move {
        let result = bridge::invoke_latest::<FanHomeData, _>(
            "fan_home",
            &EmptyArgs {},
            12_000,
            "fan:home",
        )
        .await;
        let completed = latest_request_completed(&result);
        match result {
            Ok(Some(value)) if value.has_supported_schema() => home.set(Some(value)),
            Ok(Some(value)) => error.set(Some(i18n::format(
                "unsupported_signal_snapshot_version",
                &[value.schema_version.to_string()],
            ))),
            Ok(None) => {}
            // Fan background refresh errors are silent: the cache or empty
            // state handles the UI, and the next refresh cycle retries
            // naturally. The fan never sees a transient error toast.
            Err(_) => {}
        }
        if completed {
            loading.update(|state| state.home = false);
        }
    });
}

/// Paints referral, interests, the admission pass and the AREA wallet from the
/// native encrypted snapshot. One bridge call covers all four, and every slot is
/// gated on its own loading bit: a section whose live answer already landed has
/// its bit cleared, so a stale snapshot can never overwrite fresh data.
fn prime_fan_sections(
    dashboard: RwSignal<Option<FanDashboardData>>,
    area: RwSignal<Option<AreaWallet>>,
    loading: RwSignal<FanLoadingState>,
) {
    if !bridge::native_available() {
        return;
    }
    spawn_local(async move {
        let Ok(Some(snapshot)) =
            bridge::invoke_timeout::<Option<crate::models::FanSectionsSnapshot>, _>(
                "fan_cached_sections",
                &EmptyArgs {},
                2_000,
            )
            .await
        else {
            return;
        };
        if let Some(referral) = snapshot.referral
            && loading.get_untracked().referral
        {
            dashboard.update(|state| {
                state.get_or_insert_with(FanDashboardData::default).referral = referral;
            });
            loading.update(|state| state.referral = false);
        }
        if !snapshot.interests.is_empty() && loading.get_untracked().interests {
            dashboard.update(|state| {
                state
                    .get_or_insert_with(FanDashboardData::default)
                    .interests = stable_fan_interests(snapshot.interests);
            });
            loading.update(|state| state.interests = false);
        }
        if snapshot.admission_pass.is_some() && loading.get_untracked().admission_pass {
            dashboard.update(|state| {
                state
                    .get_or_insert_with(FanDashboardData::default)
                    .admission_pass = snapshot.admission_pass;
            });
            loading.update(|state| state.admission_pass = false);
        }
        if snapshot.area.is_some() && loading.get_untracked().area {
            area.set(snapshot.area);
            loading.update(|state| state.area = false);
        }
    });
}

/// Claim a fan section for exactly one loader. The tab effect and the
/// background warm-up both reach for the same sections, and neither should
/// issue a request the other already owns.
fn claim_fan_section(
    loaded: RwSignal<FanLoadedState>,
    pick: fn(&mut FanLoadedState) -> &mut bool,
) -> bool {
    let mut first = false;
    loaded.update(|state| {
        let slot = pick(state);
        first = !*slot;
        *slot = true;
    });
    first
}

fn refresh_fan_parts(
    dashboard: RwSignal<Option<FanDashboardData>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) {
    // Each child request owns exactly one loading bit. Do not mark unrelated
    // home/merch/wallet/AREA sections busy when refreshing dashboard fragments.
    refresh_fan_events(dashboard, loading, error);
    refresh_fan_referral(dashboard, loading, error);
    refresh_fan_interests(dashboard, loading, error);
    refresh_fan_admission_pass(dashboard, loading, error);
}

fn refresh_fan_events(
    dashboard: RwSignal<Option<FanDashboardData>>,
    loading: RwSignal<FanLoadingState>,
    _error: RwSignal<Option<String>>,
) {
    let has_events = dashboard.with_untracked(|value| value.as_ref().is_some_and(|data| !data.events.is_empty()));
    blank_only_when_empty(loading, has_events, |state, value| state.events = value);
    // The last successful list is already on disk. Paint it beside the live
    // request instead of showing a skeleton until the network answers.
    if !has_events && bridge::native_available() {
        spawn_local(async move {
            if let Ok(value) = bridge::invoke_timeout::<Vec<PublicEvent>, _>(
                "fan_cached_events",
                &EmptyArgs {},
                2_000,
            )
            .await
                && !value.is_empty()
                && dashboard.with_untracked(|state| state.as_ref().is_none_or(|data| data.events.is_empty()))
            {
                dashboard.update(|state| {
                    state.get_or_insert_with(FanDashboardData::default).events =
                        stable_fan_events(value);
                });
                loading.update(|state| state.events = false);
            }
        });
    }
    spawn_local(async move {
        let result = bridge::invoke_latest::<Vec<PublicEvent>, _>(
            "fan_events",
            &EmptyArgs {},
            15_000,
            "fan:events",
        )
        .await;
        let completed = latest_request_completed(&result);
        match result {
            Ok(Some(value)) => dashboard.update(|state| {
                state.get_or_insert_with(FanDashboardData::default).events =
                    stable_fan_events(value);
            }),
            Ok(None) => {}
            // Silent: cached events or empty state handles the UI.
            Err(_) => {}
        }
        if completed {
            loading.update(|state| state.events = false);
        }
    });
}

fn refresh_fan_merch(
    merch: RwSignal<Option<MerchCatalog>>,
    loading: RwSignal<FanLoadingState>,
    _error: RwSignal<Option<String>>,
) {
    let has_merch = merch.with_untracked(|value| value.is_some());
    blank_only_when_empty(loading, has_merch, |state, value| state.merch = value);
    // See refresh_fan_events: the storefront paints from the last snapshot and
    // is replaced silently when the live catalog lands.
    if !has_merch && bridge::native_available() {
        spawn_local(async move {
            if let Ok(Some(value)) = bridge::invoke_timeout::<Option<MerchCatalog>, _>(
                "fan_cached_merch_catalog",
                &EmptyArgs {},
                2_000,
            )
            .await
                && merch.get_untracked().is_none()
            {
                merch.set(Some(value));
                loading.update(|state| state.merch = false);
            }
        });
    }
    spawn_local(async move {
        let result = bridge::invoke_latest::<MerchCatalog, _>(
            "fan_merch_catalog",
            &EmptyArgs {},
            15_000,
            "fan:merch",
        )
        .await;
        let completed = latest_request_completed(&result);
        match result {
            // The cached snapshot painted first and the live catalog is usually
            // identical to it. Setting it anyway would tear down and rebuild
            // every product card and product image for no visible change.
            Ok(Some(value)) => {
                if merch.with_untracked(|current| current.as_ref() != Some(&value)) {
                    merch.set(Some(value));
                }
            }
            Ok(None) => {}
            // Silent: keep the last catalog (or cached snapshot) instead of
            // clearing it. The fan keeps browsing; the next refresh retries.
            Err(_) => {}
        }
        if completed {
            loading.update(|state| state.merch = false);
        }
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
            Ok(Some(value)) => {
                if bundles.with_untracked(|current| current.as_ref() != Some(&value)) {
                    bundles.set(Some(value));
                }
            }
            Ok(None) => {}
            // Silent: keep last bundles instead of clearing.
            Err(_) => {}
        }
    });
}

fn refresh_fan_referral(
    dashboard: RwSignal<Option<FanDashboardData>>,
    loading: RwSignal<FanLoadingState>,
    _error: RwSignal<Option<String>>,
) {
    blank_only_when_empty(loading, dashboard.with_untracked(|value| value.is_some()), |state, value| state.referral = value);
    spawn_local(async move {
        let result = bridge::invoke_latest::<ReferralProgress, _>(
            "fan_referral",
            &EmptyArgs {},
            15_000,
            "fan:referral",
        )
        .await;
        let completed = latest_request_completed(&result);
        match result {
            Ok(Some(value)) => dashboard.update(|state| {
                state.get_or_insert_with(FanDashboardData::default).referral = value;
            }),
            Ok(None) => {}
            // Silent: cached referral data or empty state handles the UI.
            Err(_) => {}
        }
        if completed {
            loading.update(|state| state.referral = false);
        }
    });
}

fn refresh_fan_interests(
    dashboard: RwSignal<Option<FanDashboardData>>,
    loading: RwSignal<FanLoadingState>,
    _error: RwSignal<Option<String>>,
) {
    blank_only_when_empty(loading, dashboard.with_untracked(|value| value.as_ref().is_some_and(|data| !data.interests.is_empty())), |state, value| state.interests = value);
    spawn_local(async move {
        let result = bridge::invoke_latest::<Vec<FanEventInterest>, _>(
            "fan_interests",
            &EmptyArgs {},
            15_000,
            "fan:interests",
        )
        .await;
        let completed = latest_request_completed(&result);
        match result {
            Ok(Some(value)) => dashboard.update(|state| {
                state
                    .get_or_insert_with(FanDashboardData::default)
                    .interests = stable_fan_interests(value);
            }),
            Ok(None) => {}
            // Silent: cached interests or empty state handles the UI.
            Err(_) => {}
        }
        if completed {
            loading.update(|state| state.interests = false);
        }
    });
}

fn refresh_fan_admission_pass(
    dashboard: RwSignal<Option<FanDashboardData>>,
    loading: RwSignal<FanLoadingState>,
    _error: RwSignal<Option<String>>,
) {
    blank_only_when_empty(loading, dashboard.with_untracked(|value| value.is_some()), |state, value| state.admission_pass = value);
    spawn_local(async move {
        let result = bridge::invoke_latest::<Option<AdmissionPass>, _>(
            "fan_admission_pass",
            &EmptyArgs {},
            15_000,
            "fan:admission",
        )
        .await;
        let completed = latest_request_completed(&result);
        match result {
            Ok(Some(value)) => dashboard.update(|state| {
                state
                    .get_or_insert_with(FanDashboardData::default)
                    .admission_pass = value;
            }),
            Ok(None) => {}
            // Silent: cached admission pass or empty state handles the UI.
            Err(_) => {}
        }
        if completed {
            loading.update(|state| state.admission_pass = false);
        }
    });
}

fn refresh_fan_area(
    area: RwSignal<Option<AreaWallet>>,
    loading: RwSignal<FanLoadingState>,
    _error: RwSignal<Option<String>>,
) {
    blank_only_when_empty(loading, area.with_untracked(|value| value.is_some()), |state, value| state.area = value);
    spawn_local(async move {
        let result = bridge::invoke_latest::<AreaWallet, _>(
            "fan_area_wallet",
            &EmptyArgs {},
            15_000,
            "fan:area",
        )
        .await;
        let completed = latest_request_completed(&result);
        match result {
            Ok(Some(value)) => area.set(Some(value)),
            Ok(None) => {}
            // Silent: cached AREA wallet or empty state handles the UI.
            Err(_) => {}
        }
        if completed {
            loading.update(|state| state.area = false);
        }
    });
}

fn refresh_wallets(
    wallets: RwSignal<Vec<TicketWallet>>,
    loading: Option<RwSignal<FanLoadingState>>,
    _error: RwSignal<Option<String>>,
) {
    if let Some(loading) = loading {
        blank_only_when_empty(
            loading,
            wallets.with_untracked(|value| !value.is_empty()),
            |state, value| state.wallets = value,
        );
    }
    // One live request per stored order under a 35 s budget. The last public
    // snapshot is already in the vault, so paint it beside the refresh instead
    // of holding a skeleton for the whole fan-out.
    if bridge::native_available() && wallets.with_untracked(|value| value.is_empty()) {
        spawn_local(async move {
            if let Ok(value) =
                bridge::invoke_timeout::<Vec<TicketWallet>, _>("fan_cached_wallets", &EmptyArgs {}, 2_000)
                    .await
                && !value.is_empty()
                && wallets.with_untracked(|state| state.is_empty())
            {
                wallets.set(stable_wallets(value));
                if let Some(loading) = loading {
                    loading.update(|state| state.wallets = false);
                }
            }
        });
    }
    spawn_local(async move {
        let result = bridge::invoke_latest::<WalletBatch, _>(
            "fan_wallets",
            &EmptyArgs {},
            35_000,
            "fan:wallets",
        )
        .await;
        let completed = latest_request_completed(&result);
        match result {
            Ok(Some(value)) => {
                wallets.set(stable_wallets(value.wallets));
                // Partial wallet refresh failures are silent: the cached
                // wallets are already on screen and the next refresh
                // retries. The fan never sees a "could not refresh" toast.
            }
            Ok(None) => {}
            // Silent: cached wallets or empty state handles the UI.
            Err(_) => {}
        }
        if completed
            && let Some(loading) = loading
        {
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
