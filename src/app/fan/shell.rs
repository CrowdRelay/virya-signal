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

fn fan_tab_for_push_target(target: &str) -> FanTab {
    match FanTarget::parse(target) {
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
    let merch_bundles = RwSignal::new(None::<FanMerchBundleCatalog>);
    let wallets = RwSignal::new(Vec::<TicketWallet>::new());
    let checkout_event = RwSignal::new(None::<PublicEvent>);
    let focused_event_slug = RwSignal::new(None::<String>);
    let focused_event_preview = RwSignal::new(None::<PublicEvent>);
    let admission_qr = RwSignal::new(None::<AdmissionQr>);
    let area = RwSignal::new(None::<AreaWallet>);
    let loading = RwSignal::new(FanLoadingState::all());

    let loaded = RwSignal::new(FanLoadedState::default());
    let menu_open = RwSignal::new(false);
    let refresh_requested = RwSignal::new(0_u32);

    Effect::new(move |_| {
        persist_fan_tab(tab.get());
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
        tab.set(fan_tab_for_push_target(&target));
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
                    refresh_fan_merch(merch, loading, error);
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
    Effect::new(move |_| {
        let generation = refresh_requested.get();
        if generation == 0 {
            return;
        }
        loaded.set(FanLoadedState::default());
        refresh_fan_home(home, loading, error);
        refresh_fan_parts(dashboard, loading, error);
        refresh_fan_merch(merch, loading, error);
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
        spawn_local(async move {
            match bridge::invoke::<FanSessionStatus, _>("fan_lock", &EmptyArgs {}).await {
                Ok(value) => {
                    home.set(None);
                    dashboard.set(None);
                    merch.set(None);
                    merch_bundles.set(None);
                    wallets.set(Vec::new());
                    checkout_event.set(None);
                    focused_event_slug.set(None);
                    focused_event_preview.set(None);
                    admission_qr.set(None);
                    area.set(None);
                    loading.set(FanLoadingState::all());
                    status.set(value);
                    persist_fan_tab(FanTab::Signal);
                    mode.set(RootMode::Fan);
                }
                Err(message) => error.set(Some(message)),
            }
        });
    };

    let open_latarnik = move |_| {
        menu_open.set(false);
        mode.set(RootMode::Latarnik);
    };

    let refresh_all = move |_| {
        menu_open.set(false);
        bridge::invalidate_latest("fan:");
        refresh_requested.update(|value| *value = value.wrapping_add(1).max(1));
    };

    view! {
        <section class="authenticated fan-authenticated">
            <header class="topbar fan-topbar">
                <div><p class="eyebrow">{tr("virya_signal")}</p><strong>{move || status.get().session.and_then(|s| s.display_name).value_or_else(|| tr("my_signal").to_owned())}</strong></div>
                <div class="topbar-actions"><button class="menu-trigger" aria-label=tr("open_menu") aria-expanded=move || menu_open.get() on:click=move |_| menu_open.update(|value| *value = !*value)><i></i><i></i><i></i></button></div>
            </header>
            <Show when=move || menu_open.get()>
                <div class="overflow-backdrop" on:click=move |_| menu_open.set(false)></div>
                <nav class="overflow-menu">
                    <button class:active=move || tab.get() == FanTab::Game on:click=move |_| { tab.set(FanTab::Game); menu_open.set(false); }><span>"◇"</span>{tr("area_game_tab")}</button>
                    <button class:active=move || tab.get() == FanTab::Profile on:click=move |_| { tab.set(FanTab::Profile); menu_open.set(false); }><span>"◎"</span>{tr("profile_tab")}</button>
                    <button on:click=refresh_all><span>"↻"</span>{tr("refresh_all_data")}</button>
                    <button on:click=open_latarnik><span>"◉"</span>{tr("latarnik_zone")}</button>
                    <button on:click=move |_| { menu_open.set(false); mode.set(RootMode::StaffGate); }><span>"⌁"</span>{tr("staff_zone")}</button>
                    <button on:click=close><span>"×"</span>{tr("close_and_lock_signal")}</button>
                </nav>
            </Show>
            <div class="content">{move || match tab.get() {
                FanTab::Signal => view! { <FanSignal home=home dashboard=dashboard tab=tab focused_event_slug=focused_event_slug focused_event_preview=focused_event_preview loading=loading error=error /> }.into_any(),
                FanTab::Events => checkout_event.get().map(|event| view! {
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
                    <FanEvents dashboard=dashboard public=public focused_event_slug=focused_event_slug focused_event_preview=focused_event_preview checkout_event=checkout_event loading=loading error=error />
                }.into_any()),
                FanTab::Merch => view! { <FanMerch merch=merch bundles=merch_bundles loading=loading error=error /> }.into_any(),
                FanTab::Game => view! { <AreaGameScreen area=area loading=loading error=error /> }.into_any(),
                FanTab::Wallet => view! { <FanWallet dashboard=dashboard wallets=wallets admission_qr=admission_qr loading=loading error=error /> }.into_any(),
                FanTab::Profile => view! { <FanProfileScreen status=status dashboard=dashboard wallets=wallets area=area loading=loading error=error /> }.into_any(),
            }}</div>
            <nav class="bottom-nav four primary-four"><FanNavButton tab=tab own=FanTab::Signal icon="signal" label=tr("signal_tab")/><FanNavButton tab=tab own=FanTab::Events icon="events" label=tr("shows_tab")/><FanNavButton tab=tab own=FanTab::Merch icon="shop" label=tr("store_tab")/><FanNavButton tab=tab own=FanTab::Wallet icon="ticket" label=tr("tickets_tab")/></nav>
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
    view! { <button class:active=move || tab.get() == own on:click=move |_| tab.set(own)><NavGlyph icon=icon/><small>{label}</small></button> }
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
            match bridge::copy_text(&url).await {
                Ok(()) => share_status.set(Some(tr("signal_link_copied").to_owned())),
                Err(message) => error.set(Some(message)),
            }
        });
    };
    view! {
        <section class="screen fan-screen">
            <FanHomeOverview home=home loading=loading tab=tab focused_event_slug=focused_event_slug focused_event_preview=focused_event_preview error=error />
            <Show when=move || !loading.get().referral fallback=move || view! { <Skeleton /> }>
            {move || dashboard.with(|state| state.as_ref().map(|data| data.referral.clone())).map(|referral| {
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
                    {share_url.map(|url| {
                        let share_url = url.clone();
                        view! {
                            <article class="home-action-card signal-relay-card">
                                <p class="eyebrow">{tr("carry_the_signal")}</p>
                                <strong>{tr("invite_real_metalheads")}</strong>
                                <p>{tr("invite_one_to_three_people_you_really_think_would_care")}</p>
                                <button
                                    class="referral-code-copy"
                                    type="button"
                                    on:click=copy_referral
                                    disabled=move || dashboard.with(|state| state.as_ref().is_none_or(|data| data.referral.referral_code.trim().is_empty()))
                                >
                                    {move || dashboard.with(|state| state.as_ref().map(|d| i18n::format("code", std::slice::from_ref(&d.referral.referral_code))).value_or_else(|| tr("loading_signal").to_owned()))}
                                </button>
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
                                            Err(message) => error.set(Some(message)),
                                        }
                                    });
                                }>{tr("share_signal")}</button>
                                {move || share_status.get().map(|message| view! { <small class="success">{message}</small> })}
                            </article>
                        }
                    })}
                    <div class="section-head"><h3>{tr("active_draws")}</h3><span>{draw_count}</span></div>
                    <div class="card-list">{draws.into_iter().map(|draw| {
                        let proof_url = (!draw.slug.is_empty()).then(|| format!(
                            "https://virya.music/pl/dowody/losowania/{}/?source=signal-app",
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
                    {coupons_view}
                    {rewards_view}
                }.into_any()
            }).value_or_else(|| view! { <Skeleton /> }.into_any())}
            </Show>
        </section>
    }
}
