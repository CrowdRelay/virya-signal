fn persisted_fan_tab() -> FanTab {
    match bridge::fan_tab_state().as_str() {
        "events" => FanTab::Events,
        "merch" => FanTab::Merch,
        "game" => FanTab::Game,
        "wallet" => FanTab::Wallet,
        "profile" => FanTab::Profile,
        _ => FanTab::Signal,
    }
}

fn persist_fan_tab(tab: FanTab) {
    let value = match tab {
        FanTab::Signal => "signal",
        FanTab::Events => "events",
        FanTab::Merch => "merch",
        FanTab::Game => "game",
        FanTab::Wallet => "wallet",
        FanTab::Profile => "profile",
    };
    bridge::set_fan_tab_state(value);
}

fn fan_tab_for_target(target: &FanTarget) -> FanTab {
    match target {
        FanTarget::Area => FanTab::Game,
        FanTarget::Merch => FanTab::Merch,
        FanTarget::Event(_) => FanTab::Events,
        FanTarget::Wallet => FanTab::Wallet,
        FanTarget::Profile => FanTab::Profile,
        FanTarget::Signal => FanTab::Signal,
    }
}

#[component]
fn FanApp(
    mode: RwSignal<RootMode>,
    status: RwSignal<FanSessionStatus>,
    status_refresh: RwSignal<u32>,
    public: RwSignal<Option<PublicHomeData>>,
    push_target: RwSignal<Option<String>>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let tab = RwSignal::new(persisted_fan_tab());
    let home = RwSignal::new(None::<FanHomeData>);
    let dashboard = RwSignal::new(None::<FanDashboardData>);
    let merch = RwSignal::new(None::<MerchCatalog>);
    let merch_stale = RwSignal::new(false);
    let merch_bundles = RwSignal::new(None::<FanMerchBundleCatalog>);
    let wallets = RwSignal::new(Vec::<TicketWallet>::new());
    let checkout_event = RwSignal::new(None::<PublicEvent>);
    let focused_event_slug = RwSignal::new(None::<String>);
    let focused_event_preview = RwSignal::new(None::<PublicEvent>);
    let admission_qr = RwSignal::new(None::<AdmissionQr>);
    let area = RwSignal::new(None::<AreaWallet>);
    let loading = RwSignal::new(FanLoadingState::all());

    // The unlock gate reports its own failures inline. Anything it left behind
    // must not surface as the first thing a fan sees after a successful unlock.
    error.set(None);

    // A push target belongs to the fan session that received it. When the
    // session locks — whether the fan tapped lock, forgot the profile, or
    // deleted the account — any target still in the signal is stale and must
    // not be consumed by the next fan who unlocks on this device.
    Effect::new(move |_| {
        if !status.get().unlocked {
            push_target.set(None);
        }
    });

    let loaded = RwSignal::new(FanLoadedState::default());
    let menu_open = RwSignal::new(false);
    let refresh_requested = RwSignal::new(0_u32);

    let content_ref = NodeRef::<leptos::html::Div>::new();
    Effect::new(move |_| {
        persist_fan_tab(tab.get());
        // The old per-tab remount landed every switch at the top of the page.
        // Keep-alive preserves the DOM, so restore that expectation explicitly
        // instead of letting a tall Merch scroll carry into Shows.
        reset_content_scroll(content_ref);
    });

    Effect::new(move |_| {
        let generation = status_refresh.get();
        if generation == 0 || !status.get_untracked().unlocked {
            return;
        }
        bridge::invalidate_latest("fan:home");
        loaded.update(|state| state.home = true);
        refresh_fan_home(home, loading, error);
    });

    Effect::new(move |_| {
        let Some(target) = push_target.get() else {
            return;
        };
        push_target.set(None);
        let target = FanTarget::parse(&target);
        // The tab moves first: the shell clears a focused show whenever the tab
        // is not Events, and setting the slug before the tab would hand that
        // rule a slug to throw away.
        tab.set(fan_tab_for_target(&target));
        // `FanTarget::Event` carries the slug the notification was about, and
        // it used to be dropped on the floor: a push saying tickets for one
        // show had gone on sale opened the list of every show instead. The
        // preview is cleared so the detail view resolves against the real
        // event once the list lands, rather than painting a stale card.
        if let FanTarget::Event(Some(slug)) = target {
            focused_event_preview.set(None);
            focused_event_slug.set(Some(slug));
        }
    });

    Effect::new(move |_| {
        if !status.get().unlocked {
            return;
        }
        if dashboard.get_untracked().is_none() {
            dashboard.set(Some(FanDashboardData::default()));
        }

        match tab.get() {
            FanTab::Signal => {
                if !loaded.get_untracked().home {
                    loaded.update(|state| state.home = true);
                    refresh_fan_home(home, loading, error);
                }
                if !loaded.get_untracked().referral {
                    loaded.update(|state| state.referral = true);
                    refresh_fan_referral(dashboard, loading, error);
                }
            }
            FanTab::Events => {
                let state = loaded.get_untracked();
                if !state.events {
                    loaded.update(|value| value.events = true);
                    refresh_fan_events(dashboard, loading, error);
                }
                if !state.interests {
                    loaded.update(|value| value.interests = true);
                    refresh_fan_interests(dashboard, loading, error);
                }
            }
            FanTab::Merch => {
                if !loaded.get_untracked().merch {
                    loaded.update(|state| state.merch = true);
                    refresh_fan_merch(merch, merch_stale, loading, error);
                    refresh_fan_merch_bundles(merch_bundles);
                }
            }
            FanTab::Game => {
                if !loaded.get_untracked().area {
                    loaded.update(|state| state.area = true);
                    refresh_fan_area(area, loading, error);
                }
            }
            FanTab::Wallet => {
                let state = loaded.get_untracked();
                if !state.admission_pass {
                    loaded.update(|value| value.admission_pass = true);
                    refresh_fan_admission_pass(dashboard, loading, error);
                }
                if !state.wallets {
                    loaded.update(|value| value.wallets = true);
                    refresh_wallets(wallets, Some(loading), error);
                }
            }
            FanTab::Profile => {
                let state = loaded.get_untracked();
                if !state.referral {
                    loaded.update(|value| value.referral = true);
                    refresh_fan_referral(dashboard, loading, error);
                }
                if !state.events {
                    loaded.update(|value| value.events = true);
                    refresh_fan_events(dashboard, loading, error);
                }
                if !state.interests {
                    loaded.update(|value| value.interests = true);
                    refresh_fan_interests(dashboard, loading, error);
                }
                if !state.admission_pass {
                    loaded.update(|value| value.admission_pass = true);
                    refresh_fan_admission_pass(dashboard, loading, error);
                }
                if !state.wallets {
                    loaded.update(|value| value.wallets = true);
                    refresh_wallets(wallets, Some(loading), error);
                }
                if !state.area {
                    loaded.update(|value| value.area = true);
                    refresh_fan_area(area, loading, error);
                }
            }
        }
    });
    // Sections used to wait for the first visit to their own tab, so the first
    // tap on Bilety, Merch or AREA always cost a full round trip with a
    // skeleton on screen. The warm-up now starts as soon as the fan is
    // unlocked, in two waves: the dashboard fragments the fan is most likely to
    // open next, then the heavier catalogs a moment later. It used to wait for
    // Home to land first, which meant one slow home request — up to its 12 s
    // timeout — held back sections that already had a snapshot to paint from.
    let warmed = RwSignal::new(false);
    Effect::new(move |_| {
        if warmed.get_untracked() || !status.get().unlocked {
            return;
        }
        warmed.set(true);
        // Disk first, network behind it. The snapshot read is issued before the
        // live requests so the panels that have one paint immediately instead
        // of showing a skeleton until their own round trip answers.
        prime_fan_sections(dashboard, area, loading);
        if claim_fan_section(loaded, |state| &mut state.events) {
            refresh_fan_events(dashboard, loading, error);
        }
        if claim_fan_section(loaded, |state| &mut state.referral) {
            refresh_fan_referral(dashboard, loading, error);
        }
        if claim_fan_section(loaded, |state| &mut state.interests) {
            refresh_fan_interests(dashboard, loading, error);
        }
        set_timeout(
            move || {
                // The fan can lock the app or leave the portal inside the delay.
                // A disposed owner reads as None, a locked one as !unlocked.
                if status
                    .try_get_untracked()
                    .is_none_or(|value| !value.unlocked)
                {
                    return;
                }
                if claim_fan_section(loaded, |state| &mut state.admission_pass) {
                    refresh_fan_admission_pass(dashboard, loading, error);
                }
                if claim_fan_section(loaded, |state| &mut state.merch) {
                    refresh_fan_merch(merch, merch_stale, loading, error);
                    refresh_fan_merch_bundles(merch_bundles);
                }
                if claim_fan_section(loaded, |state| &mut state.area) {
                    refresh_fan_area(area, loading, error);
                }
                // One request per stored order, so warm it only when the
                // profile says there is something to warm.
                let has_orders = status
                    .try_get_untracked()
                    .and_then(|value| value.session)
                    .is_some_and(|profile| profile.wallet_count > 0);
                if has_orders && claim_fan_section(loaded, |state| &mut state.wallets) {
                    refresh_wallets(wallets, Some(loading), error);
                }
            },
            std::time::Duration::from_millis(900),
        );
    });

    Effect::new(move |_| {
        let generation = refresh_requested.get();
        if generation == 0 {
            return;
        }
        // Every section below is refreshed in this generation. Mark the request
        // ownership up front, just like tab-scoped loading does, so switching
        // tabs while/after the full refresh cannot immediately issue duplicates.
        loaded.set(FanLoadedState {
            home: true,
            referral: true,
            events: true,
            interests: true,
            merch: true,
            admission_pass: true,
            wallets: true,
            area: true,
        });
        refresh_fan_home(home, loading, error);
        refresh_fan_parts(dashboard, loading, error);
        refresh_fan_merch(merch, merch_stale, loading, error);
        refresh_fan_merch_bundles(merch_bundles);
        refresh_wallets(wallets, Some(loading), error);
        refresh_fan_area(area, loading, error);
    });

    Effect::new(move |_| {
        if tab.get() != FanTab::Events {
            if checkout_event.get_untracked().is_some() {
                checkout_event.set(None);
            }
            if focused_event_slug.get_untracked().is_some() {
                focused_event_slug.set(None);
            }
            if focused_event_preview.get_untracked().is_some() {
                focused_event_preview.set(None);
            }
        }
    });
    on_cleanup(move || bridge::invalidate_latest("fan:"));

    let close = move |_| {
        bridge::invalidate_latest("fan:");
        status.set(FanSessionStatus {
            // Locking changes nothing about how this device opens, so the
            // unlock modes are carried over rather than reset — dropping them
            // would send the gate to a PIN prompt a device-sealed vault has no
            // answer for, until the native status caught up.
            ..FanSessionStatus {
                configured: status.get_untracked().configured,
                unlocked: false,
                session: None,
                ..status.get_untracked()
            }
        });
        persist_fan_tab(FanTab::Signal);
        spawn_local(async move {
            // Native state stays authoritative: adopt whatever it reports.
            // Silent: the optimistic UI already locked the session.
            if let Ok(value) = bridge::invoke::<FanSessionStatus, _>("fan_lock", &EmptyArgs {}).await {
                let _ = status.try_set(value);
            }
        });
    };

    let open_latarnik = move |_| {
        menu_open.set(false);
        // Lock the fan session before leaving Fan mode so the session
        // does not stay unlocked in memory while the user is in Latarnik.
        spawn_local(async move {
            if let Ok(value) = bridge::invoke::<FanSessionStatus, _>("fan_lock", &EmptyArgs {}).await {
                let _ = status.try_set(value);
            }
        });
        mode.set(RootMode::Latarnik);
    };

    // System back on Android used to close the app from anywhere — mid-checkout,
    // with the menu open, on the Merch tab. Every dismissible layer now costs
    // one back press before the app itself does. One guard entry is held while
    // any layer is open; consuming it closes the topmost layer, and the effect
    // re-arms for whatever is still open underneath.
    let back_guard_armed = RwSignal::new(false);
    let close_top_layer = move || {
        if menu_open.get_untracked() {
            menu_open.set(false);
        } else if checkout_event.get_untracked().is_some() {
            checkout_event.set(None);
        } else if focused_event_slug.get_untracked().is_some() {
            focused_event_slug.set(None);
            focused_event_preview.set(None);
        } else if tab.get_untracked() != FanTab::Signal {
            tab.set(FanTab::Signal);
        }
    };
    Effect::new(move |_| {
        let open = menu_open.get()
            || checkout_event.with(Option::is_some)
            || focused_event_slug.with(Option::is_some)
            || tab.get() != FanTab::Signal;
        if open && !back_guard_armed.get_untracked() {
            back_guard_armed.set(true);
            bridge::push_back_guard();
        } else if !open {
            // The guard entry is already gone once back consumed it. Nothing to
            // unwind here: dropping the flag is what lets the next layer re-arm.
            back_guard_armed.set(false);
        }
    });
    let back_handler_id = bridge::install_back_handler(move || {
        if back_guard_armed.try_get_untracked() != Some(true) {
            return;
        }
        let _ = back_guard_armed.try_set(false);
        close_top_layer();
    });
    on_cleanup(move || {
        bridge::uninstall_back_handler(back_handler_id);
        // Leaving Fan mode with a layer open would strand its guard entry, and
        // the next back press in Latarnik or Staff would be swallowed by it.
        if back_guard_armed.try_get_untracked() == Some(true) {
            bridge::go_back();
        }
    });
    // Keyboard parity for the same layers, for anyone on a desktop WebView.
    // It goes through history rather than closing the layer directly, so the
    // guard entry is consumed and the two paths can never disagree about how
    // many back presses are left.
    let dismiss_on_escape = move |event: leptos::ev::KeyboardEvent| {
        if event.key() == "Escape" && back_guard_armed.get_untracked() {
            bridge::go_back();
        }
    };

    let request_full_refresh = move || {
        bridge::invalidate_latest("fan:");
        refresh_requested.update(|value| *value = value.wrapping_add(1).max(1));
    };
    let refresh_all = move |_| {
        menu_open.set(false);
        request_full_refresh();
    };
    // A thin progress bar at the top of the content column tracks the full
    // refresh. It shows while any section is still loading after the
    // refresh_requested generation bumps, and hides once they all settle.
    let refresh_active = Signal::derive(move || {
        let state = loading.get();
        refresh_requested.get() > 0
            && (state.home
                || state.events
                || state.referral
                || state.interests
                || state.merch
                || state.admission_pass
                || state.wallets
                || state.area)
    });

    // Pull-to-refresh on the content column. Armed only while the page sits at
    // the very top, so a pull inside a scrolled list never hijacks the gesture,
    // and horizontal drags are ignored. The indicator is a decorative glyph
    // with no new i18n strings; releasing past the threshold reuses the same
    // refresh generation as the menu button.
    const PULL_TRIGGER_PX: f64 = 70.0;
    const PULL_FIRE_PX: f64 = 56.0;
    // The window never scrolls. The shell is `height: 100dvh; overflow: hidden`
    // and `.content` is the scroll container, so `window.scrollY` is always 0
    // and the top check above was always true — a pull anywhere down a long
    // list fired a full refresh, which is what this was written to prevent.
    // The handlers are bound to `.content`, so its own `scrollTop` is the
    // honest answer and needs no DOM lookup.
    let near_top = |event: &leptos::ev::TouchEvent| {
        event
            .current_target()
            .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
            .map(|element| element.scroll_top() <= 1)
            .value_or(true)
    };
    let pull_origin = RwSignal::new(None::<(i32, i32)>);
    let pull_px = RwSignal::new(0_f64);
    let ptr_start = move |event: leptos::ev::TouchEvent| {
        if !near_top(&event) {
            return;
        }
        if let Some(touch) = event.touches().item(0) {
            pull_origin.set(Some((touch.client_x(), touch.client_y())));
        }
    };
    let ptr_move = move |event: leptos::ev::TouchEvent| {
        let Some((origin_x, origin_y)) = pull_origin.get_untracked() else {
            return;
        };
        let Some(touch) = event.touches().item(0) else {
            return;
        };
        let dx = (touch.client_x() - origin_x) as f64;
        let dy = (touch.client_y() - origin_y) as f64;
        if dy <= 0.0 || dx.abs() > dy.abs() || !near_top(&event) {
            pull_origin.set(None);
            pull_px.set(0.0);
            return;
        }
        pull_px.set((dy * 0.4).min(96.0));
    };
    let ptr_end = move |_| {
        pull_origin.set(None);
        let pulled = pull_px.get_untracked();
        pull_px.set(0.0);
        if pulled >= PULL_FIRE_PX {
            bridge::haptic("tap");
            request_full_refresh();
        }
    };

    // Keep-alive tab pages: every tab stays mounted once unlocked and inactive
    // ones collapse, so a switch preserves scroll position and decoded imagery
    // instead of rebuilding the DOM on every visit. All per-tab data lives in
    // shell-owned signals gated by `loaded`, so mounting early changes when
    // nothing but the first paint of rarely visited tabs happens.

    view! {
        <section class="authenticated fan-authenticated" tabindex="-1" on:keydown=dismiss_on_escape>
            <header class="topbar fan-topbar">
                <div><p class="eyebrow">{tr("virya_signal")}</p><strong>{move || status.get().session.and_then(|s| s.display_name).value_or_else(|| tr("my_signal").to_owned())}</strong></div>
                <div class="topbar-actions"><button class="menu-trigger" aria-label=tr("open_menu") aria-expanded=move || menu_open.get() on:click=move |_| menu_open.update(|value| *value = !*value)><i></i><i></i><i></i></button></div>
            </header>
            <Show when=move || menu_open.get()>
                <div class="overflow-backdrop" on:click=move |_| menu_open.set(false)></div>
                <nav class="overflow-menu">
                    <button class:active=move || tab.get() == FanTab::Game on:click=move |_| { tab.set(FanTab::Game); menu_open.set(false); }><span>"◇"</span>{tr("area_game_tab")}</button>
                    <button on:click=refresh_all><span>"↻"</span>{tr("refresh_all_data")}</button>
                    <button on:click=open_latarnik><span>"◉"</span>{tr("latarnik_zone")}</button>
                    <button on:click=move |_| {
                        menu_open.set(false);
                        // Lock the fan session before entering Staff mode.
                        spawn_local(async move {
                            if let Ok(value) = bridge::invoke::<FanSessionStatus, _>("fan_lock", &EmptyArgs {}).await {
                                let _ = status.try_set(value);
                            }
                        });
                        mode.set(RootMode::StaffGate);
                    }><span>"⌁"</span>{tr("staff_zone")}</button>
                    <button on:click=close><span>"×"</span>{tr("close_and_lock_signal")}</button>
                </nav>
            </Show>
            <div class="content" node_ref=content_ref on:touchstart=ptr_start on:touchmove=ptr_move on:touchend=ptr_end>
                <div class="content-refresh-bar" class:active=move || refresh_active.get() aria-hidden="true"></div>
                <div
                    class="ptr-hint"
                    style=move || {
                        let pulled = pull_px.get();
                        format!(
                            "transform:translateY({:.1}px) rotate({:.0}deg);opacity:{:.2}{}",
                            pulled,
                            pulled * 4.0,
                            (pulled / PULL_TRIGGER_PX).min(1.0),
                            // Mid-drag the glyph must track the finger exactly;
                            // the stylesheet transition applies to release only.
                            if pulled > 2.0 { ";transition:none" } else { "" },
                        )
                    }
                    aria-hidden="true"
                ><span>"↻"</span></div>
                <div class="tab-page" class:hidden=move || tab.get() != FanTab::Signal class:tab-active=move || tab.get() == FanTab::Signal>
                    <FanSignal home=home dashboard=dashboard tab=tab focused_event_slug=focused_event_slug focused_event_preview=focused_event_preview loading=loading error=error />
                </div>
                <div class="tab-page" class:hidden=move || tab.get() != FanTab::Events class:tab-active=move || tab.get() == FanTab::Events>
                    {move || checkout_event.get().map(|event| view! {
                        <FanTicketCheckout
                            event=event
                            status=status
                            tab=tab
                            checkout_event=checkout_event
                            wallets=wallets
                            loading=loading
                            error=error
                        />
                    }.into_any()).value_or_else(|| view! {
                        <FanEvents dashboard=dashboard public=public focused_event_slug=focused_event_slug focused_event_preview=focused_event_preview checkout_event=checkout_event loading=loading error=error status=status />
                    }.into_any())}
                </div>
                <div class="tab-page" class:hidden=move || tab.get() != FanTab::Merch class:tab-active=move || tab.get() == FanTab::Merch>
                    <FanMerch merch=merch merch_stale=merch_stale bundles=merch_bundles loading=loading error=error />
                </div>
                <div class="tab-page" class:hidden=move || tab.get() != FanTab::Game class:tab-active=move || tab.get() == FanTab::Game>
                    <AreaGameScreen area=area loading=loading error=error />
                </div>
                <div class="tab-page" class:hidden=move || tab.get() != FanTab::Wallet class:tab-active=move || tab.get() == FanTab::Wallet>
                    <FanWallet dashboard=dashboard wallets=wallets admission_qr=admission_qr loading=loading error=error />
                </div>
                <div class="tab-page" class:hidden=move || tab.get() != FanTab::Profile class:tab-active=move || tab.get() == FanTab::Profile>
                    <FanProfileScreen status=status dashboard=dashboard wallets=wallets area=area loading=loading error=error />
                </div>
            </div>
            // Confirmations for what the fan just did — saved a show, imported
            // a wallet, opened Stripe, set a city — and the real failures of
            // those same actions. Transient background noise stays suppressed:
            // refreshes keep the last good snapshot and retry on their own.
            <Toast error=error suppress_transient=true />
            // Profile sat behind the hamburger, which is where a fan looks last
            // and where the account, notification and language settings are not
            // expected to live. AREA stays in the menu — it is an opt-in side
            // game, and the Profile screen links to it as well.
            <nav class="bottom-nav five" aria-label=tr("primary_navigation")>
                <FanNavButton tab=tab own=FanTab::Signal icon="signal" label=tr("signal_tab")/>
                <FanNavButton tab=tab own=FanTab::Events icon="events" label=tr("shows_tab")/>
                <FanNavButton tab=tab own=FanTab::Merch icon="shop" label=tr("store_tab")/>
                <FanNavButton tab=tab own=FanTab::Wallet icon="ticket" label=tr("tickets_tab")/>
                <FanNavButton tab=tab own=FanTab::Profile icon="profile" label=tr("profile_tab")/>
            </nav>
        </section>
    }
}

#[component]
fn FanNavButton(
    tab: RwSignal<FanTab>,
    own: FanTab,
    icon: &'static str,
    label: &'static str,
) -> impl IntoView {
    view! { <button class:active=move || tab.get() == own aria-current=move || (tab.get() == own).then_some("page") on:click=move |_| { bridge::haptic("tap"); tab.set(own); }><NavGlyph icon=icon/><small>{label}</small></button> }
}

/// Asks for notifications in our own words before Android asks in its.
///
/// Android 13 grants exactly one `POST_NOTIFICATIONS` dialog per install: a
/// denial there is permanent short of a trip to system settings. Spending it
/// unprompted, on a fan who has just arrived and does not yet know what Signal
/// would send, is how an app loses the channel it needs to reach the people who
/// drift away — which is the whole reason for asking. So the system dialog is
/// only reached by a fan who has already said yes to this modal.
///
/// Shown once, as a centered modal overlay, after the home data has painted.
/// The fan sees the app first — what Signal actually gives them — and then the
/// primer appears. "Not now" is remembered the same as "yes", because a modal
/// that returns every launch is the same nag by another name; the switch in
/// Profile stays available for anyone who changes their mind.
#[component]
fn PushPrimer(loading: RwSignal<FanLoadingState>) -> impl IntoView {
    let visible = RwSignal::new(false);
    let busy = RwSignal::new(false);
    let eligible = RwSignal::new(false);

    // Check push status on mount — this only sets eligibility, it does not
    // show the modal. The modal waits for the home data to paint.
    Effect::new(move |_| {
        if !bridge::native_available() || bridge::push_primer_seen() {
            return;
        }
        // `invoke_timeout`, not `invoke_latest`. This is a one-shot probe, not
        // a latest-wins read, and the "fan:push-primer" scope shared the
        // "fan:" prefix that a pull-to-refresh or a lock invalidates — which
        // resolved the probe as superseded and abandoned the ask for the whole
        // session, silently, because the effect has nothing to re-run on.
        spawn_lifecycle_task(async move {
            // Nothing to ask for if the fan already answered Android, either
            // way, or if this build cannot deliver a notification at all.
            let Ok(status) =
                bridge::invoke_timeout::<FanPushStatus, _>("fan_push_sync", &EmptyArgs {}, 15_000)
                    .await
            else {
                return;
            };
            if status.supported && !status.enabled && status.permission != "denied" {
                let _ = eligible.try_set(true);
            }
        });
    });

    // Show the modal only after the home section has painted. This is the
    // "polite" part: the fan sees the app's value first, then the ask.
    //
    // Both inputs are tracked. `eligible` used to be read untracked, so the
    // effect only re-ran when `loading.home` changed — and the two settle in
    // the opposite order to the one that assumed. Home paints from the
    // encrypted snapshot in about a hundred milliseconds; the probe above is a
    // capability check plus a push-config round trip and lands seconds later.
    // So `loading.home` was already false by the time `eligible` turned true,
    // nothing re-ran, and the ask never appeared on any real launch.
    Effect::new(move |_| {
        if eligible.get() && !loading.get().home {
            let _ = visible.try_set(true);
        }
    });

    let dismiss = move || {
        bridge::mark_push_primer_seen();
        visible.set(false);
    };
    let allow = move |_| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        // Marked before the request, not after: the system dialog takes the
        // window away, and a fan who answers it and never comes back must not
        // be asked again on the next launch.
        bridge::mark_push_primer_seen();
        spawn_lifecycle_task(async move {
            // Silent: the Profile switch shows the outcome, and a failed
            // enable is not something the fan can act on from here.
            let _ = bridge::invoke_timeout::<FanPushStatus, _>(
                "fan_push_enable",
                &EmptyArgs {},
                45_000,
            )
            .await;
            let _ = busy.try_set(false);
            let _ = visible.try_set(false);
        });
    };

    view! {
        <Show when=move || visible.get()>
            <div class="modal-backdrop" />
            <div class="push-primer-modal" role="dialog" aria-modal="true" aria-label=tr("push_primer_title")>
                <div class="push-primer-modal-body">
                    <span class="push-primer-icon" aria-hidden="true">"◉"</span>
                    <strong>{tr("push_primer_title")}</strong>
                    <p>{tr("push_primer_body")}</p>
                    <div class="push-primer-actions">
                        <button type="button" class="primary" disabled=move || busy.get() on:click=allow>
                            {tr("push_primer_allow")}
                        </button>
                        <button type="button" class="text-button" on:click=move |_| dismiss()>
                            {tr("push_primer_later")}
                        </button>
                    </div>
                </div>
            </div>
        </Show>
    }
}

#[component]
fn FanSignal(
    home: RwSignal<Option<FanHomeData>>,
    dashboard: RwSignal<Option<FanDashboardData>>,
    tab: RwSignal<FanTab>,
    focused_event_slug: RwSignal<Option<String>>,
    focused_event_preview: RwSignal<Option<PublicEvent>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let share_status = RwSignal::new(None::<String>);
    // Reading `dashboard` straight in the render closure tied this whole block —
    // benefits hero, draws, coupons, rewards — to every field of the dashboard.
    // Saving a show writes `dashboard.interests` from the Shows tab, and because
    // tabs stay mounted, that rebuilt this hidden screen's DOM on every tap.
    let referral_progress =
        Memo::new(move |_| dashboard.with(|state| state.as_ref().map(|data| data.referral.clone())));
    let copyable_referral_code = Memo::new(move |_| {
        dashboard.with(|state| {
            state
                .as_ref()
                .map(|data| data.referral.referral_code.trim().to_owned())
                .filter(|code| !code.is_empty())
        })
    });
    let copy_referral = move |_| {
        let Some(referral_code) = dashboard
            .get_untracked()
            .map(|data| data.referral.referral_code.trim().to_owned())
        else {
            return;
        };
        if referral_code.is_empty() {
            return;
        }
        let url = format!("https://www.virya.music/r/{referral_code}");
        share_status.set(None);
        spawn_local(async move {
            // Silent: the fan can tap the copy button again.
            if let Ok(()) = bridge::copy_text(&url).await {
                share_status.set(Some(tr("signal_link_copied").to_owned()));
            }
        });
    };
    view! {
        <section class="screen fan-screen">
            <PushPrimer loading=loading />
            <FanHomeOverview home=home loading=loading tab=tab focused_event_slug=focused_event_slug focused_event_preview=focused_event_preview error=error />
            <Show when=move || !loading.get().referral fallback=move || view! { <Skeleton rows=1 height=180 /> }>
            {move || referral_progress.get().map(|referral| {
                let draw_count = referral.draw_entries.len();
                let referral_code = referral.referral_code.clone();
                let share_url = (!referral_code.trim().is_empty()).then(|| {
                    format!("https://www.virya.music/r/{referral_code}")
                });
                let draws = referral.draw_entries;
                let coupons = referral.coupons;
                let rewards = referral.physical_rewards;
                let coupons_view = (!coupons.is_empty()).then(|| view! {
                    <div class="section-head"><h3>{tr("your_coupons")}</h3></div>
                    <div class="card-list">{coupons.into_iter().map(|coupon| view! {
                        <article class="fan-coupon"><div><span>{format!("-{}%", coupon.discount_percent)}</span><strong>{coupon.code}</strong></div><small>{coupon.status}</small></article>
                    }).collect_view()}</div>
                });
                let rewards_view = (!rewards.is_empty()).then(|| view! {
                    <div class="section-head"><h3>{tr("rewards")}</h3></div>
                    <div class="card-list">{rewards.into_iter().map(|reward| view! {
                        <article class="reward-card"><div><strong>{reward.item_name}</strong><p>{reward.sku}</p></div><span>{reward.status}</span></article>
                    }).collect_view()}</div>
                });
                view! {
                    // Premium benefits section — always visible, even with 0
                    // referrals or while the code is still loading. The ID is
                    // used by the ExploreSignal banner CTA to scroll here.
                    <article class="benefits-hero" id="signal-benefits-section">
                        <div class="benefits-hero-head">
                            <p class="eyebrow">{tr("carry_the_signal")}</p>
                            <strong>{tr("invite_real_metalheads")}</strong>
                            <p>{tr("referral_preview")}</p>
                        </div>
                        <div class="benefits-grid" role="group" aria-label=tr("what_virya_signal_gives_you")>
                            <div class="benefit-tile">
                                <span class="benefit-icon" aria-hidden="true">"⌁"</span>
                                <strong>{tr("benefit_shows_title")}</strong>
                                <p>{tr("benefit_shows_desc")}</p>
                            </div>
                            <div class="benefit-tile">
                                <span class="benefit-icon" aria-hidden="true">"▣"</span>
                                <strong>{tr("benefit_tickets_title")}</strong>
                                <p>{tr("benefit_tickets_desc")}</p>
                            </div>
                            <div class="benefit-tile">
                                <span class="benefit-icon" aria-hidden="true">"✦"</span>
                                <strong>{tr("benefit_rewards_title")}</strong>
                                <p>{tr("benefit_rewards_desc")}</p>
                            </div>
                            <div class="benefit-tile">
                                <span class="benefit-icon" aria-hidden="true">"⤴"</span>
                                <strong>{tr("benefit_referrals_title")}</strong>
                                <p>{tr("benefit_referrals_desc")}</p>
                            </div>
                        </div>
                        <div class="benefits-cta">
                            <button
                                class="referral-code-copy"
                                type="button"
                                on:click=copy_referral
                                disabled=move || copyable_referral_code.with(Option::is_none)
                            >
                                // A dashboard that has landed with a blank code
                                // rendered the label as a bare "Code:" followed
                                // by nothing — a button that looks broken rather
                                // than one still waiting. Blank is absent.
                                {move || copyable_referral_code.with(|code| code
                                    .as_ref()
                                    .map(|value| i18n::format("code", std::slice::from_ref(value)))
                                    .value_or_else(|| tr("referral_code_loading").to_owned()))}
                            </button>
                            {share_url.as_ref().map(|url| {
                                let share_url = url.clone();
                                view! {
                                    <button class="ghost" type="button" on:click=move |_| {
                                        let url = share_url.clone();
                                        share_status.set(None);
                                        spawn_local(async move {
                                            match bridge::share_text(
                                                tr("virya_signal"),
                                                tr("virya_signal_share_copy"),
                                                &url,
                                            ).await {
                                                Ok(result) if result == "shared" => share_status.set(Some(tr("signal_shared").to_owned())),
                                                Ok(result) if result == "copied" => share_status.set(Some(tr("signal_link_copied").to_owned())),
                                                Ok(_) => {},
                                                // Silent: the fan can tap share again.
                                                Err(_) => {}
                                            }
                                        });
                                    }>{tr("share_signal")}</button>
                                }
                            })}
                            {move || share_status.get().map(|message| view! { <small class="success">{message}</small> })}
                        </div>
                    </article>
                    {if draw_count > 0 {
                        view! {
                            <div class="section-head"><h3>{tr("active_draws")}</h3><span>{draw_count}</span></div>
                            <div class="card-list">{draws.into_iter().map(|draw| {
                                let proof_url = (!draw.slug.is_empty()).then(|| format!(
                                    "https://virya.music/{}/dowody/losowania/{}/?source=signal-app",
                                    i18n::current().code(),
                                    draw.slug,
                                ));
                                view! {
                                    <article class="draw-card">
                                        <div><p class="eyebrow">{draw.prize_kind}</p><strong>{draw.name}</strong><span>{i18n::format("draw", &[human_time(&draw.draw_at).to_string()])}</span></div>
                                        <div class="draw-actions">
                                            <div class="entry-count"><b>{draw.total_entries}</b><small>{tr("entries_2")}</small></div>
                                            {proof_url.map(|url| view! { <ExternalLink url=url label=tr("proof") error=error /> })}
                                        </div>
                                    </article>
                                }
                            }).collect_view()}</div>
                        }.into_any()
                    } else {
                        view! { <span></span> }.into_any()
                    }}
                    {coupons_view}
                    {rewards_view}
                }.into_any()
            }).value_or_else(|| view! { <Skeleton rows=1 height=96 /> }.into_any())}
            </Show>
            // Synesthesia sits below "carry the signal" so the primary CTA
            // (next show + referral benefits) comes first. It is an opt-in
            // side experience — only people who already engaged with it see
            // the progress card. Source order matches rendered order (WCAG
            // 1.3.2, 2.4.3).
            <SynesthesiaHomeCard home=home error=error />
        </section>
    }
}
