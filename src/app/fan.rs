#[component]
fn FanPortal(
    mode: RwSignal<RootMode>,
    status: RwSignal<FanSessionStatus>,
    status_loading: RwSignal<bool>,
    status_failed: RwSignal<bool>,
    status_refresh: RwSignal<u32>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    // Entering the fan zone must be a local-only transition. City data is
    // fetched only when the user explicitly asks for the canonical list.
    let public = RwSignal::new(Some(PublicHomeData::default()));

    view! {
        <Show when=move || !status.get().unlocked>
            <StaffEntryButton mode=mode />
        </Show>
        {move || if status_failed.get() {
            view! { <StatusFailure mode=mode status_refresh=status_refresh label=tr("failed_to_read_the_fan_profile") show_back=false /> }.into_any()
        } else if status_loading.get() {
            view! { <AccessLoader mode=mode label=tr("checking_your_signal") show_back=false /> }.into_any()
        } else if status.get().unlocked {
            view! { <FanApp mode=mode status=status public=public error=error /> }.into_any()
        } else {
            view! { <FanAccess status=status error=error /> }.into_any()
        }}
    }
}

#[component]
fn StatusFailure(
    mode: RwSignal<RootMode>,
    status_refresh: RwSignal<u32>,
    label: &'static str,
    show_back: bool,
) -> impl IntoView {
    view! {
        <section class="access-screen status-failure">
            <Show when=move || show_back>
                <BackButton mode=mode />
            </Show>
            <div class="access-card">
                <p class="eyebrow">{label}</p>
                <h2>{tr("your_profile_remains_untouched")}</h2>
                <p>{tr("app_will_not_continue_to_signup_or")}</p>
                <button
                    class="primary"
                    on:click=move |_| status_refresh.update(|value| *value = value.wrapping_add(1))
                >
                    {tr("try_again")}
                </button>
            </div>
        </section>
    }
}

#[component]
fn AccessLoader(mode: RwSignal<RootMode>, label: &'static str, show_back: bool) -> impl IntoView {
    view! {
        <section class="access-screen status-loader">
            <Show when=move || show_back>
                <BackButton mode=mode />
            </Show>
            <div class="access-card">
                <p class="eyebrow">{label}</p>
                <Skeleton rows=2 />
            </div>
        </section>
    }
}

// VIRYA SIGNAL FAN CONFIRM UX V1
fn submit_fan_confirmation(
    email: RwSignal<String>,
    name: RwSignal<String>,
    token: RwSignal<String>,
    pin: RwSignal<String>,
    busy: RwSignal<bool>,
    status: RwSignal<FanSessionStatus>,
    error: RwSignal<Option<String>>,
) {
    if busy.get_untracked() {
        return;
    }
    let input_email = email.get_untracked().trim().to_owned();
    let current_token = token.get_untracked().trim().to_owned();
    let current_pin = pin.get_untracked();
    if input_email.is_empty() {
        error.set(Some(tr("enter_the_email_used_to_join_signal").to_owned()));
        return;
    }
    if current_token.is_empty() {
        error.set(Some(tr("paste_the_code_or_full_link_or").to_owned()));
        return;
    }
    if current_pin.chars().count() < 4 {
        error.set(Some(tr("enter_4_6_digits_for_this_fan_profile").to_owned()));
        return;
    }
    let input = FanConfirmationInput {
        api_base_url: API_BASE.to_owned(),
        email: input_email,
        display_name: optional(name.get_untracked().trim().to_owned()),
        token: current_token,
    };
    busy.set(true);
    spawn_local(async move {
        match bridge::invoke::<FanAuthResult, _>(
            "fan_confirm",
            &FanConfirmArgs {
                input: &input,
                pin: &current_pin,
            },
        )
        .await
        {
            Ok(_) => {
                pin.set(String::new());
                token.set(String::new());
                refresh_fan_status(status, error);
            }
            Err(message) => error.set(Some(message)),
        }
        busy.set(false);
    });
}

#[component]
fn FanAccess(status: RwSignal<FanSessionStatus>, error: RwSignal<Option<String>>) -> impl IntoView {
    let access_mode = RwSignal::new(FanAccessMode::Signup);
    let email = RwSignal::new(String::new());
    let name = RwSignal::new(String::new());
    // City onboarding is deliberately local-first. The remote canonical list is
    // not placed in Leptos reactive state: this removes a repeatedly crashing
    // Android WebView/WASM path while preserving full signup functionality.
    let custom_city_name = RwSignal::new(String::new());
    let custom_region = RwSignal::new(String::new());
    let referral = RwSignal::new(String::new());
    let token = RwSignal::new(String::new());
    let pin = RwSignal::new(String::new());
    let consent = RwSignal::new(false);
    let nearby_enabled = RwSignal::new(true);
    let radius_km = RwSignal::new(150_u16);
    let busy = RwSignal::new(false);
    let recovery_open = RwSignal::new(false);

    let unlock = move |_| {
        let current_pin = pin.get();
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<FanSessionStatus, _>(
                "fan_unlock",
                &PinArgs { pin: &current_pin },
            )
            .await
            {
                Ok(value) => {
                    pin.set(String::new());
                    status.set(value);
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let signup = move |_| {
        if !consent.get() {
            error.set(Some(
                tr("marketing_consent_is_required_to_join_signal").to_owned(),
            ));
            return;
        }
        let current_pin = pin.get();
        let requested = RequestedCityInput {
            name: custom_city_name.get().trim().to_owned(),
            region: optional(custom_region.get().trim().to_owned()),
            country_code: "PL".to_owned(),
        };
        let input_email = email.get().trim().to_owned();
        let input_name = optional(name.get().trim().to_owned());
        let input_referral = optional(referral.get().trim().to_owned());
        let nearby = nearby_enabled.get();
        let radius = radius_km.get();
        busy.set(true);
        spawn_local(async move {
            let city_slug = match bridge::invoke::<RequestedCityResult, _>(
                "request_city",
                &RequestedCityArgs { input: &requested },
            )
            .await
            {
                Ok(value) => value.city_slug,
                Err(message) => {
                    error.set(Some(i18n::format(
                        "could_not_save_city_message",
                        std::slice::from_ref(&message),
                    )));
                    busy.set(false);
                    return;
                }
            };
            if city_slug.trim().is_empty() {
                error.set(Some(tr("select_a_city_or_enter_your_own").to_owned()));
                busy.set(false);
                return;
            }
            let input = FanSignupInput {
                api_base_url: API_BASE.to_owned(),
                email: input_email,
                display_name: input_name,
                city_slug,
                locale: i18n::current().code().to_owned(),
                referral_code: input_referral,
                policy_version: POLICY_VERSION.to_owned(),
                nearby_gigs_enabled: nearby,
                nearby_radius_km: radius,
            };
            match bridge::invoke::<FanAuthResult, _>(
                "fan_signup",
                &FanSignupArgs {
                    input: &input,
                    pin: &current_pin,
                },
            )
            .await
            {
                Ok(result) => {
                    if result.session_created {
                        pin.set(String::new());
                        refresh_fan_status(status, error);
                    } else {
                        access_mode.set(FanAccessMode::Confirm);
                        let message = match result.email_queued {
                            Some(true)
                                if result.email_kind.as_deref() == Some("session_recovery") =>
                            {
                                { tr("we_sent_a_secure_access_link_scan") }.to_owned()
                            }
                            Some(true) => { tr("we_sent_a_confirmation_code_scan_the") }.to_owned(),
                            Some(false) => {
                                let minutes = result
                                    .retry_after_seconds
                                    .map(|seconds| seconds.saturating_add(59) / 60)
                                    .unwrap_or(15)
                                    .max(1);
                                i18n::format(
                                    "new_message_not_sent_previous_code_still_valid_minutes",
                                    &[minutes.to_string()],
                                )
                            }
                            None => { tr("request_was_accepted_check_your_inbox_and") }.to_owned(),
                        };
                        error.set(Some(message));
                    }
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let confirm = move |_| {
        submit_fan_confirmation(email, name, token, pin, busy, status, error);
    };

    let request_access = move |_| {
        if busy.get_untracked() {
            return;
        }
        let current_email = email.get_untracked().trim().to_owned();
        if current_email.is_empty() {
            error.set(Some(tr("enter_the_email_used_in_virya_signal").to_owned()));
            return;
        }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke_unit(
                "fan_request_access",
                &FanAccessArgs {
                    api_base_url: API_BASE,
                    email: &current_email,
                    locale: i18n::current().code(),
                },
            )
            .await
            {
                Ok(()) => {
                    access_mode.set(FanAccessMode::Confirm);
                    error.set(Some(tr("if_this_email_is_registered_in_virya").to_owned()));
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let scan_confirmation = move |_| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        spawn_local(async move {
            match bridge::scan_qr().await {
                Ok(Some(value)) => {
                    token.set(value);
                    busy.set(false);
                    if !email.get_untracked().trim().is_empty()
                        && pin.get_untracked().chars().count() >= 4
                    {
                        submit_fan_confirmation(email, name, token, pin, busy, status, error);
                    } else {
                        error.set(Some(tr("qr_scanned_enter_your_email_and_local").to_owned()));
                    }
                    return;
                }
                Ok(None) => {}
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    view! {
        <section class="fan-access">
            <header class="fan-access-hero">
                <p class="eyebrow">{tr("virya_signal")}</p>
                <h1>{tr("shows_tickets")}<br/><em>{tr("and_rewards")}</em></h1>
                <p class="hero-subtitle">{tr("join_in_3_steps")}</p>
                <ol class="signal-steps" aria-label=tr("how_to_join")>
                    <li><span class="step-num">"1"</span>{tr("enter_your_email_and_city")}</li>
                    <li><span class="step-num">"2"</span>{tr("confirm_the_code_from_the_message")}</li>
                    <li><span class="step-num">"3"</span>{tr("discover_shows_near_you")}</li>
                </ol>
                <div class="signal-purpose-grid" aria-label=tr("what_virya_signal_gives_you")>
                    <span><b aria-hidden="true">"⌁"</b>{tr("shows_near_you")}</span>
                    <span><b aria-hidden="true">"▣"</b>{tr("tickets_and_qr_codes_on_your_phone")}</span>
                    <span><b aria-hidden="true">"✦"</b>{tr("rewards_for_simple_actions")}</span>
                </div>
            </header>
            <Show when=move || status.get().configured fallback=move || view! {
                <div class="access-card fan-card">
                    <div class="segmented">
                        <button class:active=move || access_mode.get() == FanAccessMode::Signup on:click=move |_| access_mode.set(FanAccessMode::Signup)>{tr("get_started")}</button>
                        <button class:active=move || access_mode.get() == FanAccessMode::Confirm on:click=move |_| access_mode.set(FanAccessMode::Confirm)>{tr("i_have_a_code")}</button>
                    </div>
                    <div class="form-grid fan-form">
                        <label>{tr("email")}<input type="email" autocomplete="email" prop:value=move || email.get() on:input=move |e| email.set(event_target_value(&e))/></label>
                        <label>{tr("name_optional")}<input prop:value=move || name.get() on:input=move |e| name.set(event_target_value(&e))/></label>
                        <Show when=move || access_mode.get() == FanAccessMode::Signup fallback=move || view! {
                            <>
                                <p class="confirm-hint"><strong>{tr("fastest_scan_the_qr_from_the_email")}</strong><br/>{tr("you_can_also_paste_the_full_link")}</p>
                                <label>{tr("email_link_or_code")}<textarea rows="3" autocomplete="one-time-code" spellcheck="false" autocapitalize="none" placeholder=tr("paste_a_link_or_code_or_use") prop:value=move || token.get() on:input=move |e| token.set(event_target_value(&e))></textarea></label>
                                <div class="confirmation-actions single">
                                    <button type="button" class="confirmation-action primary-scan" disabled=move || busy.get() on:click=scan_confirmation><span aria-hidden="true">"▦"</span><strong>{tr("scan_qr")}</strong><small>{tr("or_hold_the_field_above_and_choose")}</small></button>
                                </div>
                                <label class="pin-field">
                                    <span class="pin-field-label">{tr("create_fan_unlock_pin")}</span>
                                    <small id="fan-confirm-pin-help">{tr("enter_4_6_digits_for_this_fan_profile")}</small>
                                    <input type="password" autocomplete="new-password" inputmode="numeric" pattern="[0-9]*" maxlength="6" placeholder=tr("pin_example") aria-describedby="fan-confirm-pin-help" prop:value=move || pin.get() on:input=move |e| pin.set(normalize_new_operator_pin(event_target_value(&e)))/>
                                </label>
                                <p class="confirmation-note">{tr("pin_encrypts_your_profile_on_this_device")}</p>
                                <button class="primary" disabled=move || busy.get() || email.get().trim().is_empty() || token.get().trim().is_empty() || !new_operator_pin_is_valid(&pin.get()) on:click=confirm>{tr("confirm_and_enter")}</button>
                                <button type="button" class="text-button" disabled=move || busy.get() || email.get().trim().is_empty() on:click=request_access>{tr("i_already_have_an_account_send_login")}</button>
                                <p class="confirmation-resend">{tr("no_message_check_spam_after_15_minutes")}</p>
                            </>
                        }>
                            <>
                                <div class="custom-city-fields city-stable-entry">
                                    <label>{tr("city")}<input placeholder=tr("e_g_bielawa") prop:value=move || custom_city_name.get() on:input=move |e| custom_city_name.set(event_target_value(&e))/></label>
                                    <label>{tr("province_region_optional")}<input placeholder=tr("lower_silesia") prop:value=move || custom_region.get() on:input=move |e| custom_region.set(event_target_value(&e))/></label>
                                    <p class="inline-note">{tr("enter_your_city_manually_we_will_match")}</p>
                                </div>
                                <div class="nearby-pref">
                                    <label class="check-label"><input type="checkbox" prop:checked=move || nearby_enabled.get() on:change=move |e| nearby_enabled.set(event_target_checked(&e))/><span>{tr("notify_me_about_nearby_shows")}</span></label>
                                    <Show when=move || nearby_enabled.get()>
                                        <div class="radius-picker">
                                            <button type="button" class:active=move || radius_km.get()==50 on:click=move |_| radius_km.set(50)>"50 km"</button>
                                            <button type="button" class:active=move || radius_km.get()==100 on:click=move |_| radius_km.set(100)>"100 km"</button>
                                            <button type="button" class:active=move || radius_km.get()==150 on:click=move |_| radius_km.set(150)>"150 km"</button>
                                            <button type="button" class:active=move || radius_km.get()==250 on:click=move |_| radius_km.set(250)>"250 km"</button>
                                        </div>
                                    </Show>
                                </div>
                                <label>{tr("referral_code_optional")}<input prop:value=move || referral.get() on:input=move |e| referral.set(event_target_value(&e))/></label>
                                <label class="pin-field">
                                    <span class="pin-field-label">{tr("create_fan_unlock_pin")}</span>
                                    <small id="fan-signup-pin-help">{tr("enter_4_6_digits_for_this_fan_profile")}</small>
                                    <input type="password" autocomplete="new-password" inputmode="numeric" pattern="[0-9]*" maxlength="6" placeholder=tr("pin_example") aria-describedby="fan-signup-pin-help" prop:value=move || pin.get() on:input=move |e| pin.set(normalize_new_operator_pin(event_target_value(&e)))/>
                                </label>
                                <label class="check-label"><input type="checkbox" prop:checked=move || consent.get() on:change=move |e| consent.set(event_target_checked(&e))/><span>{tr("i_want_to_receive_information_about_virya")}</span></label>
                                <button class="primary" disabled=move || busy.get() || !new_operator_pin_is_valid(&pin.get()) on:click=signup>{tr("join_signal")}</button>
                            </>
                        </Show>
                    </div>
                </div>
            }>
                <div class="access-card fan-card">
                    <Show when=move || recovery_open.get() fallback=move || view! {
                        <>
                            <p class="lock-copy">{tr("your_profile_and_tickets_are_encrypted_on")}</p>
                            <div class="form-grid">
                                <label class="pin-field">
                                    <span class="pin-field-label">{tr("fan_app_unlock_pin")}</span>
                                    <small id="fan-unlock-pin-help">{tr("enter_the_pin_created_for_this_fan")}</small>
                                    <input type="password" autocomplete="current-password" placeholder=tr("your_pin") aria-describedby="fan-unlock-pin-help" prop:value=move || pin.get() on:input=move |e| pin.set(event_target_value(&e))/>
                                </label>
                                <button class="primary" disabled=move || busy.get() || pin.get().chars().count() < 4 on:click=unlock>{tr("open_my_signal")}</button>
                                <button type="button" class="text-button recovery-link" on:click=move |_| recovery_open.set(true)>{tr("i_forgot_my_pin_sign_in_again")}</button>
                            </div>
                        </>
                    }>
                        <div class="form-grid recovery-panel">
                            <div class="recovery-heading"><p class="eyebrow">{tr("access_recovery")}</p><h3>{tr("create_a_new_pin")}</h3><p>{tr("enter_your_email_request_a_fresh_link")}</p></div>
                            <label>{tr("email")}<input type="email" autocomplete="email" prop:value=move || email.get() on:input=move |e| email.set(event_target_value(&e))/></label>
                            <button type="button" class="ghost" disabled=move || busy.get() || email.get().trim().is_empty() on:click=request_access>{tr("send_login_link")}</button>
                            <label>{tr("email_link_or_code")}<textarea rows="3" autocomplete="one-time-code" spellcheck="false" autocapitalize="none" placeholder=tr("paste_link_or_code") prop:value=move || token.get() on:input=move |e| token.set(event_target_value(&e))></textarea></label>
                            <div class="confirmation-actions single">
                                <button type="button" class="confirmation-action primary-scan" disabled=move || busy.get() on:click=scan_confirmation><span aria-hidden="true">"▦"</span><strong>{tr("scan_qr")}</strong><small>{tr("or_hold_the_field_and_choose_paste")}</small></button>
                            </div>
                            <label class="pin-field">
                                <span class="pin-field-label">{tr("create_fan_unlock_pin")}</span>
                                <small id="fan-recovery-pin-help">{tr("enter_4_6_digits_for_this_fan_profile")}</small>
                                <input type="password" autocomplete="new-password" inputmode="numeric" pattern="[0-9]*" maxlength="6" placeholder=tr("pin_example") aria-describedby="fan-recovery-pin-help" prop:value=move || pin.get() on:input=move |e| pin.set(normalize_new_operator_pin(event_target_value(&e)))/>
                            </label>
                            <button class="primary" disabled=move || busy.get() || email.get().trim().is_empty() || token.get().trim().is_empty() || !new_operator_pin_is_valid(&pin.get()) on:click=confirm>{tr("confirm_and_set_new_pin")}</button>
                            <button type="button" class="text-button" on:click=move |_| recovery_open.set(false)>{tr("back_to_pin_login")}</button>
                        </div>
                    </Show>
                </div>
            </Show>
        </section>
    }
}

#[component]
fn FanApp(
    mode: RwSignal<RootMode>,
    status: RwSignal<FanSessionStatus>,
    public: RwSignal<Option<PublicHomeData>>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let tab = RwSignal::new(FanTab::Signal);
    let dashboard = RwSignal::new(None::<FanDashboardData>);
    let merch = RwSignal::new(None::<MerchCatalog>);
    let merch_bundles = RwSignal::new(None::<FanMerchBundleCatalog>);
    let wallets = RwSignal::new(Vec::<TicketWallet>::new());
    let checkout_event = RwSignal::new(None::<PublicEvent>);
    let admission_qr = RwSignal::new(None::<AdmissionQr>);
    let area = RwSignal::new(None::<AreaWallet>);
    let loading = RwSignal::new(FanLoadingState::all());

    let loaded = RwSignal::new(FanLoadedState::default());
    let menu_open = RwSignal::new(false);

    Effect::new(move |_| {
        if !status.get().unlocked {
            return;
        }
        if dashboard.get_untracked().is_none() {
            dashboard.set(Some(FanDashboardData::default()));
        }

        match tab.get() {
            FanTab::Signal => {
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
        if tab.get() != FanTab::Events && checkout_event.get_untracked().is_some() {
            checkout_event.set(None);
        }
    });
    on_cleanup(move || bridge::invalidate_latest("fan:"));

    let close = move |_| {
        bridge::invalidate_latest("fan:");
        spawn_local(async move {
            match bridge::invoke::<FanSessionStatus, _>("fan_lock", &EmptyArgs {}).await {
                Ok(value) => {
                    dashboard.set(None);
                    merch.set(None);
                    merch_bundles.set(None);
                    wallets.set(Vec::new());
                    checkout_event.set(None);
                    admission_qr.set(None);
                    area.set(None);
                    loading.set(FanLoadingState::all());
                    status.set(value);
                    mode.set(RootMode::Fan);
                }
                Err(message) => error.set(Some(message)),
            }
        });
    };

    view! {
        <section class="authenticated fan-authenticated">
            <header class="topbar fan-topbar">
                <div on:dblclick=move |_| { loaded.set(FanLoadedState::default()); refresh_fan_parts(dashboard, loading, error); refresh_fan_merch(merch, loading, error); refresh_fan_merch_bundles(merch_bundles); refresh_wallets(wallets, Some(loading), error); refresh_fan_area(area, loading, error); } style="cursor:pointer"><p class="eyebrow">{tr("virya_signal")}</p><strong>{move || status.get().session.and_then(|s| s.display_name).value_or_else(|| tr("my_signal").to_owned())}</strong></div>
                <div class="topbar-actions"><span class="live-dot"></span><button class="menu-trigger" aria-label=tr("open_menu") aria-expanded=move || menu_open.get() on:click=move |_| menu_open.update(|value| *value = !*value)><i></i><i></i><i></i></button><button aria-label=tr("close_and_lock_signal") on:click=close>"×"</button></div>
            </header>
            <Show when=move || menu_open.get()>
                <div class="overflow-backdrop" on:click=move |_| menu_open.set(false)></div>
                <nav class="overflow-menu">
                    <button class:active=move || tab.get() == FanTab::Game on:click=move |_| { tab.set(FanTab::Game); menu_open.set(false); }><span>"◇"</span>{tr("area_game_tab")}</button>
                    <button class:active=move || tab.get() == FanTab::Profile on:click=move |_| { tab.set(FanTab::Profile); menu_open.set(false); }><span>"◎"</span>{tr("profile_tab")}</button>
                    <button on:click=move |_| { menu_open.set(false); mode.set(RootMode::StaffGate); }><span>"⌁"</span>{tr("staff_zone")}</button>
                </nav>
            </Show>
            <div class="content">{move || match tab.get() {
                FanTab::Signal => view! { <FanSignal dashboard=dashboard loading=loading error=error /> }.into_any(),
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
                    <FanEvents dashboard=dashboard public=public checkout_event=checkout_event loading=loading error=error />
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
    dashboard: RwSignal<Option<FanDashboardData>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <section class="screen fan-screen">
            <header class="signal-dashboard-hero">
                <p class="eyebrow">{tr("your_impact")}</p>
                <h2>{move || dashboard.with(|state| state.as_ref().map(|d| d.referral.qualified_referrals.to_string())).value_or_else(|| "—".to_owned())}</h2>
                <strong>{tr("confirmed_referrals")}</strong>
                <p>{move || dashboard.with(|state| state.as_ref().map(|d| i18n::format("code", std::slice::from_ref(&d.referral.referral_code)))).value_or_else(|| tr("loading_signal").to_owned())}</p>
            </header>
            <Show when=move || !loading.get().referral fallback=move || view! { <Skeleton /> }>
            {move || dashboard.with(|state| state.as_ref().map(|data| data.referral.clone())).map(|referral| {
                let entries_total = referral.draw_entries.iter().map(|draw| draw.total_entries).sum::<u32>();
                let draw_count = referral.draw_entries.len();
                let coupon_count = referral.coupons.len();
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
                    <article class="draw-card synesthesia-entry-card">
                        <div>
                            <p class="eyebrow">{tr("album_experience")}</p>
                            <strong>"SYNESTHESIA"</strong>
                            <span>{tr("synesthesia_five_album_draw")}</span>
                        </div>
                        <div class="draw-actions">
                            <ExternalLink url="https://synesthesia.virya.music/?source=signal-app".to_owned() label=tr("enter_synesthesia") error=error />
                        </div>
                    </article>
                    <div class="stats-grid"><Metric value=referral.pending_referrals.to_string() label=tr("pending_2")/><Metric value=entries_total.to_string() label=tr("entries")/><Metric value=coupon_count.to_string() label=tr("coupons")/></div>
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

#[component]
fn FanMerch(
    merch: RwSignal<Option<MerchCatalog>>,
    bundles: RwSignal<Option<FanMerchBundleCatalog>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <section class="screen fan-screen merch-screen">
            <header class="screen-title">
                <p class="eyebrow">{tr("virya_store")}</p>
                <h2>{tr("merch")}</h2>
                <p>{tr("products_and_bundles_use_the_same_inventory")}</p>
            </header>
            <Show when=move || !loading.get().merch fallback=move || view! { <Skeleton rows=4 /> }>
                {move || merch.get().map(|catalog| {
                    let products = catalog.products.into_iter()
                        .filter(|product| product.active && product.public)
                        .collect::<Vec<_>>();
                    if products.is_empty() {
                        view! {
                            <div class="empty-state">
                                <strong>{tr("store_is_temporarily_unavailable")}</strong>
                                <p>{tr("rest_of_signal_is_working_normally_try")}</p>
                                <button class="ghost" on:click=move |_| {
                                    refresh_fan_merch(merch, loading, error);
                                    refresh_fan_merch_bundles(bundles);
                                }>{tr("refresh_merch")}</button>
                            </div>
                        }.into_any()
                    } else {
                        let bundle_catalog = bundles.get();
                        view! {
                            <div class="fan-merch-list">
                                <div class="merch-grid-action">
                                    <ExternalLink url="https://virya.music/pl/merch/?source=signal-app".to_owned() label=tr("open_full_store") error=error />
                                </div>
                                <div class="merch-grid-heading">
                                    <div><p class="eyebrow">{tr("bundles")}</p><h3>{tr("bundles_from_the_online_store")}</h3></div>
                                    <span>{tr("up_to_30")}</span>
                                </div>
                                {bundle_catalog.map(|catalog| {
                                    if catalog.bundles.is_empty() {
                                        view! {
                                            <div class="merch-grid-message">
                                                <p>{tr("bundles_are_currently_unavailable_in_live_inventory")}</p>
                                                <ExternalLink url="https://virya.music/pl/merch/?source=signal-app&product=bundle-stage-pack".to_owned() label=tr("view_bundles") error=error />
                                            </div>
                                        }.into_any()
                                    } else {
                                        catalog.bundles.into_iter().map(|bundle| {
                                            let availability_label = match bundle.availability.as_str() {
                                                "low_stock" => {tr("low_stock")},
                                                "available" => {tr("available_status")},
                                                _ => {tr("out_of_stock")},
                                            };
                                            let available = bundle.available;
                                            let product_url = bundle.product_url.clone();
                                            let bundle_name = bundle.name;
                                            let image_alt = i18n::format(
                                                "value_zestaw_merchu_virya",
                                                std::slice::from_ref(&bundle_name),
                                            );
                                            let original_price = (bundle.original_price_gross_minor > bundle.price_gross_minor)
                                                .then(|| money(bundle.original_price_gross_minor, &bundle.currency));
                                            let includes = bundle.includes;
                                            let includes_view = (!includes.is_empty()).then(|| view! {
                                                <ul class="fan-merch-includes">
                                                    {includes.into_iter().map(|item| view! { <li>{item}</li> }).collect_view()}
                                                </ul>
                                            });
                                            let variants = bundle.variants;
                                            let variants_view = (!variants.is_empty()).then(|| view! {
                                                <div class="fan-merch-variants">
                                                    {variants.into_iter().map(|variant| view! {
                                                        <span class:available=variant.available>{variant.label}</span>
                                                    }).collect_view()}
                                                </div>
                                            });
                                            view! {
                                                <article class="fan-merch-card fan-merch-bundle">
                                                    <div class="bundle-badge">"BUNDLE"</div>
                                                    {bundle.image_url.map(|url| view! {
                                                        <img src=url alt=image_alt width="720" height="720" loading="lazy" decoding="async" referrerpolicy="no-referrer" />
                                                    })}
                                                    <div class="fan-merch-body">
                                                        <div class="fan-merch-heading">
                                                            <div>
                                                                <h3>{bundle_name}</h3>
                                                                <div class="fan-merch-price">
                                                                    <strong>{money(bundle.price_gross_minor, &bundle.currency)}</strong>
                                                                    {original_price.map(|price| view! { <del>{price}</del> })}
                                                                </div>
                                                            </div>
                                                            <span class:available=available>{availability_label}</span>
                                                        </div>
                                                        {bundle.description.map(|description| view! { <p>{description}</p> })}
                                                        {includes_view}
                                                        {variants_view}
                                                        <Show when=move || available fallback=move || view! {
                                                            <button class="ghost" on:click=move |_| refresh_fan_merch_bundles(bundles)>{tr("check_again")}</button>
                                                        }>
                                                            <ExternalLink url=product_url.clone() label=tr("buy_in_store") error=error />
                                                        </Show>
                                                    </div>
                                                </article>
                                            }
                                        }).collect_view().into_any()
                                    }
                                }).value_or_else(|| view! {
                                    <div class="merch-grid-message">
                                        <p>{tr("bundles_load_independently_from_products")}</p>
                                        <ExternalLink url="https://virya.music/pl/merch/?source=signal-app&product=bundle-stage-pack".to_owned() label=tr("view_bundles") error=error />
                                    </div>
                                }.into_any())}
                                <div class="merch-grid-heading merch-products-heading">
                                    <div><p class="eyebrow">{tr("individual_products")}</p><h3>{tr("choose_your_merch")}</h3></div>
                                </div>
                                {products.into_iter().map(|product| {
                                    let available_variants = product.variants.iter()
                                        .filter(|variant| variant.active && variant.available)
                                        .cloned()
                                        .collect::<Vec<_>>();
                                    let has_stock = !available_variants.is_empty();
                                    let preorder = available_variants.iter()
                                        .any(|variant| variant.availability == "preorder");
                                    let low_stock = available_variants.iter()
                                        .any(|variant| variant.availability == "low_stock");
                                    let availability_label = if preorder {
                                        tr("pre_order")
                                    } else if low_stock {
                                        tr("low_stock")
                                    } else if has_stock {
                                        tr("available_status")
                                    } else {
                                        tr("out_of_stock")
                                    };
                                    let shop_url = format!(
                                        "https://virya.music/pl/merch/?source=signal-app&product={}",
                                        product.slug,
                                    );
                                    let product_name = product.name;
                                    let image_alt = i18n::format(
                                        "value_merch_virya",
                                        std::slice::from_ref(&product_name),
                                    );
                                    let variants = product.variants.into_iter()
                                        .filter(|variant| variant.active)
                                        .collect::<Vec<_>>();
                                    let variants_view = (!variants.is_empty()).then(|| view! {
                                        <div class="fan-merch-variants">
                                            {variants.into_iter().map(|variant| view! {
                                                <span class:available=variant.available>{variant.label}</span>
                                            }).collect_view()}
                                        </div>
                                    });
                                    view! {
                                        <article class="fan-merch-card">
                                            {product.image_url.map(|url| view! {
                                                <img src=url alt=image_alt width="720" height="720" loading="lazy" decoding="async" referrerpolicy="no-referrer" />
                                            })}
                                            <div class="fan-merch-body">
                                                <div class="fan-merch-heading">
                                                    <div><h3>{product_name}</h3><strong>{money(product.price_gross_minor, &product.currency)}</strong></div>
                                                    <span class:available=has_stock>{availability_label}</span>
                                                </div>
                                                {product.description.map(|description| view! { <p>{description}</p> })}
                                                {variants_view}
                                                <Show when=move || has_stock fallback=move || view! {
                                                    <button class="ghost" on:click=move |_| refresh_fan_merch(merch, loading, error)>{tr("check_again")}</button>
                                                }>
                                                    <ExternalLink url=shop_url.clone() label=tr("buy_in_store") error=error />
                                                </Show>
                                            </div>
                                        </article>
                                    }
                                }).collect_view()}
                            </div>
                        }.into_any()
                    }
                }).value_or_else(|| view! {
                    <div class="empty-state">
                        <strong>{tr("could_not_load_store_status")}</strong>
                        <p>{tr("shows_tickets_and_profile_remain_available")}</p>
                        <button class="ghost" on:click=move |_| {
                            refresh_fan_merch(merch, loading, error);
                            refresh_fan_merch_bundles(bundles);
                        }>{tr("try_again")}</button>
                    </div>
                }.into_any())}
            </Show>
        </section>
    }
}

#[component]
fn FanEvents(
    dashboard: RwSignal<Option<FanDashboardData>>,
    public: RwSignal<Option<PublicHomeData>>,
    checkout_event: RwSignal<Option<PublicEvent>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    view! {
        <section class="screen"><header class="screen-title"><p class="eyebrow">{tr("where_we_play")}</p><h2>{tr("shows_tab")}</h2></header><Show when=move || !loading.get().events fallback=move || view! { <Skeleton /> }>{move || { let events = fan_events(dashboard, public); if events.is_empty() { view! { <div class="empty-state"><strong>{tr("no_shows_in_the_calendar")}</strong><p>{tr("new_events_will_appear_here_2")}</p></div> }.into_any() } else { view! { <div class="card-list fan-event-list">{events.into_iter().map(|event| view! { <FanEventCard event=event checkout_event=checkout_event dashboard=dashboard loading=loading error=error /> }).collect_view()}</div> }.into_any() }}}</Show></section>
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TicketPoolAvailability {
    Checking,
    Available,
    Missing,
    Failed,
}

#[component]
fn FanEventCard(
    event: PublicEvent,
    checkout_event: RwSignal<Option<PublicEvent>>,
    dashboard: RwSignal<Option<FanDashboardData>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let should_probe_pool = event.ticket_url.is_none();
    let pool = RwSignal::new(if should_probe_pool {
        TicketPoolAvailability::Checking
    } else {
        TicketPoolAvailability::Available
    });
    let pool_slug = event.slug.clone();
    let pool_scope = format!("fan:ticket-pool:{pool_slug}");
    let request_scope = pool_scope.clone();
    Effect::new(move |_| {
        if !should_probe_pool {
            return;
        }
        let event_slug = pool_slug.clone();
        let request_scope = request_scope.clone();
        spawn_local(async move {
            match bridge::invoke_latest::<Option<TicketSaleOffer>, _>(
                "fan_ticket_sale",
                &EventArgs {
                    event_slug: &event_slug,
                },
                10_000,
                &request_scope,
            )
            .await
            {
                Ok(Some(Some(_))) => pool.set(TicketPoolAvailability::Available),
                Ok(Some(None)) => pool.set(TicketPoolAvailability::Missing),
                Ok(None) => {}
                Err(_) => pool.set(TicketPoolAvailability::Failed),
            }
        });
    });
    on_cleanup(move || bridge::invalidate_latest(&pool_scope));
    let checkout = event.clone();
    let event_slug = event.slug.clone();
    let interested = Signal::derive(move || {
        dashboard.with(|state| {
            state.as_ref().is_some_and(|data| {
                data.interests
                    .iter()
                    .any(|item| item.event.slug == event_slug)
            })
        })
    });
    let interest_slug = event.slug.clone();
    let busy = RwSignal::new(false);
    let interest = move |_| {
        if interested.get_untracked() || busy.get_untracked() {
            return;
        }
        let event_slug = interest_slug.clone();
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke_unit(
                "fan_register_interest",
                &EventArgs {
                    event_slug: &event_slug,
                },
            )
            .await
            {
                Ok(_) => {
                    error.set(Some(tr("show_saved_to_your_signal").to_owned()));
                    refresh_fan_interests(dashboard, loading, error);
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };
    let event_day = day(&event.starts_at);
    let event_month = month(&event.starts_at);
    let event_time = human_time(&event.starts_at);
    let location = event_location(&event);
    let image = event.image_thumbnail_url.or(event.image_url);
    let description = event.description;
    let title = event.title;
    let image_alt = i18n::format("virya_show", std::slice::from_ref(&title));
    view! {
        <article class="fan-event-card">
            {image.map(|url| view! {
                <img
                    src=url
                    alt=image_alt
                    width="720"
                    height="405"
                    loading="lazy"
                    decoding="async"
                    fetchpriority="low"
                    referrerpolicy="no-referrer"
                />
            })}
            <div class="fan-event-body">
                <div class="date-line"><span>{format!("{event_day} {event_month}")}</span><small>{event_time}</small></div>
                <h3>{title}</h3><p>{location}</p>
                {description.map(|text| view! { <p class="event-description">{text}</p> })}
                <div class="event-actions">
                    <button type="button" class:active=move || interested.get() on:click=interest disabled=move || busy.get() || interested.get()>{move || if busy.get() { tr("saving") } else if interested.get() { tr("saved") } else { tr("interested") }}</button>
                    {move || match pool.get() {
                        TicketPoolAvailability::Available => {
                            let checkout = checkout.clone();
                            view! {
                                <button type="button" class="ticket-buy-button" on:click=move |_| checkout_event.set(Some(checkout.clone()))>{tr("buy_ticket")}</button>
                            }.into_any()
                        },
                        TicketPoolAvailability::Checking => view! {
                            <div class="ticket-pool-status is-loading" role="status">{tr("ticket_pool_status_loading")}</div>
                        }.into_any(),
                        TicketPoolAvailability::Missing => view! {
                            <div class="ticket-pool-status" role="status">{tr("this_show_has_no_ticket_pool")}</div>
                        }.into_any(),
                        TicketPoolAvailability::Failed => view! {
                            <div class="ticket-pool-status is-warning" role="status">{tr("ticket_pool_temporarily_unavailable")}</div>
                        }.into_any(),
                    }}
                </div>
            </div>
        </article>
    }
}

#[component]
fn FanTicketCheckout(
    event: PublicEvent,
    status: RwSignal<FanSessionStatus>,
    tab: RwSignal<FanTab>,
    checkout_event: RwSignal<Option<PublicEvent>>,
    wallets: RwSignal<Vec<TicketWallet>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let sale = RwSignal::new(None::<TicketSaleOffer>);
    let sale_loading = RwSignal::new(true);
    let sale_failed = RwSignal::new(false);
    let sale_refresh = RwSignal::new(0_u32);
    let load_slug = event.slug.clone();

    Effect::new(move |_| {
        sale_refresh.get();
        sale_loading.set(true);
        let event_slug = load_slug.clone();
        spawn_local(async move {
            match bridge::invoke_latest::<Option<TicketSaleOffer>, _>(
                "fan_ticket_sale",
                &EventArgs {
                    event_slug: &event_slug,
                },
                15_000,
                "fan:ticket-sale",
            )
            .await
            {
                Ok(Some(Some(value))) => {
                    sale.set(Some(value));
                    sale_failed.set(false);
                }
                Ok(Some(None)) => {
                    sale.set(None);
                    sale_failed.set(false);
                }
                Ok(None) => return,
                Err(message) => {
                    sale.set(None);
                    sale_failed.set(true);
                    error.set(Some(message));
                }
            }
            sale_loading.set(false);
        });
    });
    on_cleanup(move || bridge::invalidate_latest("fan:ticket-sale"));

    let back = move |_| checkout_event.set(None);
    let event_title = event.title.clone();
    let event_meta = event_time_location(&event.starts_at, event.venue.as_deref());
    let event_slug = event.slug.clone();
    let fallback_url = event
        .ticket_url
        .clone()
        .value_or_else(|| format!("https://virya.music/pl/live/{}/#tickets", event.slug));
    let full_form_url = format!("https://virya.music/pl/live/{}/#tickets", event.slug);

    view! {
        <section class="screen fan-ticket-checkout-screen">
            <button class="checkout-back" on:click=back>{tr("back_back_to_shows")}</button>
            <header class="ticket-checkout-hero">
                <p class="eyebrow">{tr("virya_tickets")}</p>
                <h2>{event_title}</h2>
                <p>{event_meta}</p>
            </header>
            {
                let render_sale = move || {
                    let event_slug = event_slug.clone();
                    let fallback_url = fallback_url.clone();
                    let full_form_url = full_form_url.clone();
                    match sale.get() {
                        Some(offer) => view! {
                            <FanTicketSale
                                offer=offer
                                event_slug=event_slug
                                fallback_url=fallback_url
                                full_form_url=full_form_url
                                status=status
                                tab=tab
                                checkout_event=checkout_event
                                wallets=wallets
                                loading=loading
                                sale_refresh=sale_refresh
                                error=error
                            />
                        }
                        .into_any(),
                        None => view! {
                            <div class="empty-state">
                                <strong>{if sale_failed.get() { tr("could_not_check_ticket_sales") } else { tr("no_virya_ticket_pool") }}</strong>
                                <p>{tr("you_can_open_the_show_page_or")}</p>
                                <ExternalLink url=fallback_url label=tr("check_tickets") error=error />
                            </div>
                        }
                        .into_any(),
                    }
                };
                view! {
                    <Show when=move || !sale_loading.get() fallback=move || view! { <Skeleton rows=4 /> }>
                        {render_sale.clone()}
                    </Show>
                }
            }
        </section>
    }
}

#[component]
fn FanTicketSale(
    offer: TicketSaleOffer,
    event_slug: String,
    fallback_url: String,
    full_form_url: String,
    status: RwSignal<FanSessionStatus>,
    tab: RwSignal<FanTab>,
    checkout_event: RwSignal<Option<PublicEvent>>,
    wallets: RwSignal<Vec<TicketWallet>>,
    loading: RwSignal<FanLoadingState>,
    sale_refresh: RwSignal<u32>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let ticket_types = offer
        .ticket_types
        .iter()
        .filter(|ticket_type| ticket_type.active)
        .cloned()
        .collect::<Vec<_>>();
    let max_per_order = offer.max_per_order.max(0) as u32;
    let has_available_type = ticket_types
        .iter()
        .any(|ticket_type| ticket_type.available > 0);
    let is_open = offer.active
        && offer.sales_state == "open"
        && offer.available > 0
        && max_per_order > 0
        && has_available_type;
    let state_copy = match offer.sales_state.as_str() {
        "upcoming" => tr("ticket_sales_will_open_soon"),
        "closed" => tr("online_sales_have_ended"),
        "sold_out" => tr("this_ticket_pool_is_sold_out"),
        "inactive" => tr("ticket_sales_are_temporarily_disabled"),
        "event_unavailable" => tr("this_show_is_not_currently_on_sale"),
        _ if !is_open => tr("tickets_are_not_available_right_now"),
        _ => tr("select_tickets_places_will_be_reserved_while"),
    };
    let sale_available = offer.available.max(0);
    let sale_reserved = offer.reserved.max(0);
    let sale_sold = offer.sold.max(0);

    if !is_open {
        return view! {
            <div class="ticket-sale-summary">
                <div><strong>{sale_available}</strong><span>{tr("available_label")}</span></div>
                <div><strong>{sale_reserved}</strong><span>{tr("in_checkout_2")}</span></div>
                <div><strong>{sale_sold}</strong><span>{tr("sold")}</span></div>
            </div>
            <p class="checkout-state-copy">{state_copy}</p>
            <div class="empty-state compact">
                <strong>{tr("open_the_show_page")}</strong>
                <p>{tr("if_the_organiser_runs_a_separate_ticket")}</p>
                <ExternalLink url=fallback_url label=tr("check_tickets") error=error />
            </div>
        }
        .into_any();
    }

    let quantities = RwSignal::new(
        ticket_types
            .iter()
            .map(|ticket_type| TicketCheckoutItemInput {
                ticket_type_slug: ticket_type.slug.clone(),
                quantity: 0,
            })
            .collect::<Vec<_>>(),
    );
    let buyer_name = RwSignal::new(
        status
            .get_untracked()
            .session
            .and_then(|profile| profile.display_name)
            .unwrap_or_default(),
    );
    let busy = RwSignal::new(false);
    let pending_checkout = RwSignal::new(None::<TicketCheckoutStart>);
    let selected_count = Signal::derive(move || checkout_count(quantities));
    let gross_offer = offer.clone();
    let selected_gross = Signal::derive(move || checkout_gross(&gross_offer, quantities));
    let purchase_slug = event_slug.clone();

    let purchase = move |_| {
        if busy.get_untracked() || pending_checkout.get_untracked().is_some() {
            return;
        }
        let items = quantities
            .get_untracked()
            .into_iter()
            .filter(|item| item.quantity > 0)
            .collect::<Vec<_>>();
        if items.is_empty() {
            error.set(Some(tr("select_at_least_one_ticket").to_owned()));
            return;
        }
        let name = buyer_name.get_untracked().trim().to_owned();
        let input = TicketCheckoutInput {
            event_slug: purchase_slug.clone(),
            buyer_name: (!name.is_empty()).then_some(name),
            items,
        };
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke_timeout::<TicketCheckoutStart, _>(
                "fan_start_ticket_checkout",
                &TicketCheckoutArgs { input: &input },
                35_000,
            )
            .await
            {
                Ok(checkout) => {
                    pending_checkout.set(Some(checkout.clone()));
                    refresh_wallets(wallets, Some(loading), error);
                    let checkout_url = checkout.url.clone();
                    match bridge::invoke_unit("open_external_url", &UrlArgs { url: &checkout_url })
                        .await
                    {
                        Ok(_) => {
                            checkout_event.set(None);
                            tab.set(FanTab::Wallet);
                            error.set(Some(i18n::format(
                                "zamowienie_value_zapisane_dokoncz_bezpieczna_patnosc_stripe",
                                std::slice::from_ref(&checkout.order_reference),
                            )));
                        }
                        Err(message) => {
                            error.set(Some(i18n::format(
                                "message_zamowienie_value_jest_zapisane_uzyj_przycisku_ponownego",
                                &[message.to_string(), checkout.order_reference.to_string()],
                            )));
                        }
                    }
                }
                Err(message) => {
                    error.set(Some(message));
                    sale_refresh.update(|value| *value = value.wrapping_add(1));
                }
            }
            busy.set(false);
        });
    };

    let retry_payment = move |_| {
        let Some(checkout) = pending_checkout.get_untracked() else {
            return;
        };
        let checkout_url = checkout.url.clone();
        spawn_local(async move {
            match bridge::invoke_unit("open_external_url", &UrlArgs { url: &checkout_url }).await {
                Ok(_) => {
                    checkout_event.set(None);
                    tab.set(FanTab::Wallet);
                    error.set(Some(i18n::format(
                        "otworzono_patnosc_dla_zamowienia_value",
                        std::slice::from_ref(&checkout.order_reference),
                    )));
                }
                Err(message) => error.set(Some(message)),
            }
        });
    };

    let currency_for_total = offer.currency.clone();
    let purchase_disabled = Signal::derive(move || {
        busy.get() || selected_count.get() == 0 || pending_checkout.get().is_some()
    });
    view! {
        <div class="ticket-sale-summary">
            <div><strong>{sale_available}</strong><span>{tr("available_label")}</span></div>
            <div><strong>{sale_reserved}</strong><span>{tr("in_checkout_2")}</span></div>
            <div><strong>{sale_sold}</strong><span>{tr("sold")}</span></div>
        </div>
        <p class="checkout-state-copy">{state_copy}</p>
        <div class="ticket-type-list">
            {ticket_types.into_iter().map(|ticket_type| {
                let quantity_slug = ticket_type.slug.clone();
                let decrement_slug = ticket_type.slug.clone();
                let increment_slug = ticket_type.slug.clone();
                let available = ticket_type.available.max(0) as u32;
                let currency = offer.currency.clone();
                let quantity = Signal::derive(move || checkout_quantity(quantities, &quantity_slug));
                let decrement_disabled = Signal::derive(move || quantity.get() == 0);
                let increment_disabled = Signal::derive(move || {
                    quantity.get() >= available || selected_count.get() >= max_per_order
                });
                let decrement = move |_| {
                    let current = checkout_quantity(quantities, &decrement_slug);
                    set_checkout_quantity(
                        quantities,
                        &decrement_slug,
                        current.saturating_sub(1),
                        available,
                        max_per_order,
                    );
                };
                let increment = move |_| {
                    let current = checkout_quantity(quantities, &increment_slug);
                    set_checkout_quantity(
                        quantities,
                        &increment_slug,
                        current.saturating_add(1),
                        available,
                        max_per_order,
                    );
                };
                view! {
                    <article class="ticket-type-card">
                        <div>
                            <h3>{ticket_type.name}</h3>
                            {ticket_type.description.map(|description| view! { <p>{description}</p> })}
                            <strong>{money(ticket_type.price_gross_minor, &currency)}</strong>
                            <small>{i18n::format("available", &[ticket_type.available.max(0).to_string()])}</small>
                        </div>
                        <div class="ticket-stepper" aria-label=tr("ticket_quantity")>
                            <button type="button" aria-label=tr("decrease_ticket_quantity") on:click=decrement disabled=move || decrement_disabled.get()>"−"</button>
                            <output aria-live="polite">{move || quantity.get()}</output>
                            <button type="button" aria-label=tr("increase_ticket_quantity") on:click=increment disabled=move || increment_disabled.get()>"+"</button>
                        </div>
                    </article>
                }
            }).collect_view()}
        </div>
        <div class="ticket-buyer-panel">
            <label>{tr("name_on_the_order_optional")}<input autocomplete="name" maxlength="160" prop:value=move || buyer_name.get() on:input=move |event| buyer_name.set(event_target_value(&event))/></label>
            <p>{move || status.get().session.map(|profile| i18n::format("tickets_and_confirmation_will_be_sent_to", std::slice::from_ref(&profile.email))).value_or_else(|| tr("tickets_will_be_sent_to_the_fan").to_owned())}</p>
            <ExternalLink url=full_form_url label=tr("invoice_full_form") error=error />
        </div>
        <footer class="ticket-checkout-total">
            <div><span>{tr("selected_tickets")}</span><strong>{move || selected_count.get()}</strong></div>
            <div><span>{tr("gross_total")}</span><strong>{move || money(selected_gross.get(), &currency_for_total)}</strong></div>
            <button type="button" class="primary" on:click=purchase disabled=move || purchase_disabled.get()>{move || if busy.get() { tr("reserving") } else if pending_checkout.get().is_some() { tr("order_saved") } else { tr("continue_to_stripe_payment") }}</button>
            <Show when=move || pending_checkout.get().is_some()>
                <button type="button" class="ghost checkout-retry" on:click=retry_payment>{tr("reopen_payment")}</button>
            </Show>
            <small>{tr("card_details_never_reach_virya_signal_payment")}</small>
        </footer>
    }
    .into_any()
}

fn checkout_quantity(
    quantities: RwSignal<Vec<TicketCheckoutItemInput>>,
    ticket_type_slug: &str,
) -> u32 {
    quantities.with(|items| {
        items
            .iter()
            .find(|item| item.ticket_type_slug == ticket_type_slug)
            .map(|item| item.quantity)
            .unwrap_or_default()
    })
}

fn checkout_count(quantities: RwSignal<Vec<TicketCheckoutItemInput>>) -> u32 {
    quantities.with(|items| items.iter().map(|item| item.quantity).sum())
}

fn checkout_gross(
    sale: &TicketSaleOffer,
    quantities: RwSignal<Vec<TicketCheckoutItemInput>>,
) -> i64 {
    quantities.with(|items| {
        items
            .iter()
            .filter_map(|item| {
                sale.ticket_types
                    .iter()
                    .find(|ticket_type| ticket_type.slug == item.ticket_type_slug)
                    .map(|ticket_type| {
                        ticket_type
                            .price_gross_minor
                            .saturating_mul(i64::from(item.quantity))
                    })
            })
            .fold(0_i64, i64::saturating_add)
    })
}

fn set_checkout_quantity(
    quantities: RwSignal<Vec<TicketCheckoutItemInput>>,
    ticket_type_slug: &str,
    requested: u32,
    available: u32,
    max_per_order: u32,
) {
    quantities.update(|items| {
        let other = items
            .iter()
            .filter(|item| item.ticket_type_slug != ticket_type_slug)
            .map(|item| item.quantity)
            .sum::<u32>();
        let allowed = available.min(max_per_order.saturating_sub(other));
        if let Some(item) = items
            .iter_mut()
            .find(|item| item.ticket_type_slug == ticket_type_slug)
        {
            item.quantity = requested.min(allowed);
        }
    });
}

#[component]
fn ExternalLink(
    url: String,
    label: &'static str,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let open_url = url.clone();
    let open = move |_| {
        let current = open_url.clone();
        spawn_local(async move {
            if let Err(message) =
                bridge::invoke_unit("open_external_url", &UrlArgs { url: &current }).await
            {
                error.set(Some(message));
            }
        });
    };
    view! { <button type="button" class="ticket-buy-button" on:click=open>{label}</button> }
}

fn open_area_game(error: RwSignal<Option<String>>) {
    spawn_local(async move {
        let url = format!(
            "https://virya.music/{}/area/#area-map",
            i18n::current().code()
        );
        if let Err(message) = bridge::invoke_unit("open_external_url", &UrlArgs { url: &url }).await
        {
            error.set(Some(message));
        }
    });
}

#[component]
fn FanWallet(
    dashboard: RwSignal<Option<FanDashboardData>>,
    wallets: RwSignal<Vec<TicketWallet>>,
    admission_qr: RwSignal<Option<AdmissionQr>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let order_id = RwSignal::new(String::new());
    let checkout_token = RwSignal::new(String::new());
    let claim_token = RwSignal::new(String::new());
    let busy = RwSignal::new(false);

    let import = move |_| {
        let order = order_id.get().trim().to_owned();
        let token = checkout_token.get().trim().to_owned();
        if order.is_empty() || token.is_empty() {
            error.set(Some(tr("enter_the_order_id_and_private_token").to_owned()));
            return;
        }
        // The recovery token must not remain rendered in the WebView while IPC runs.
        checkout_token.set(String::new());
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<TicketWallet, _>(
                "fan_import_wallet",
                &ImportWalletArgs {
                    order_id: &order,
                    checkout_token: &token,
                },
            )
            .await
            {
                Ok(_) => {
                    error.set(Some(tr("tickets_saved_to_the_wallet").to_owned()));
                    refresh_wallets(wallets, Some(loading), error);
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let claim = move |_| {
        let token = claim_token.get().trim().to_owned();
        if token.is_empty() {
            error.set(Some(tr("paste_the_admission_pass_token").to_owned()));
            return;
        }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<AdmissionPass, _>(
                "fan_claim_pass",
                &ClaimArgs {
                    claim_token: &token,
                },
            )
            .await
            {
                Ok(_) => {
                    error.set(Some(
                        tr("admission_pass_assigned_to_this_device").to_owned(),
                    ));
                    refresh_fan_admission_pass(dashboard, loading, error);
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let qr = move |_| {
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<AdmissionQr, _>("fan_admission_qr", &EmptyArgs {}).await {
                Ok(value) => admission_qr.set(Some(value)),
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    view! {
        <section class="screen"><header class="screen-title"><p class="eyebrow">{tr("mobile_wallet")}</p><h2>{tr("tickets_and_entry")}</h2></header><Show when=move || !loading.get().admission_pass fallback=move || view! { <Skeleton rows=1 /> }>{move || dashboard.with(|state| state.as_ref().and_then(|d| d.admission_pass.clone())).map(|pass| view! { <article class="admission-card"><p class="eyebrow">{tr("virya_admission_pass")}</p><h3>{pass.event_title}</h3><p>{event_time_location(&pass.starts_at, pass.venue.as_deref())}</p><strong>{pass.public_reference}</strong><span>{pass.status}</span><button class="primary" on:click=qr disabled=move || busy.get()>{tr("show_entry_qr")}</button>{move || admission_qr.get().map(|value| view! { <QrPanel svg=value.qr_svg token=value.token expires=value.expires_at /> })}</article> })}
        <Show when=move || dashboard.with(|state| state.as_ref().is_none_or(|d| d.admission_pass.is_none()))><div class="claim-box"><p class="eyebrow">{tr("did_you_win_an_admission_pass")}</p><h3>{tr("assign_it_to_your_phone")}</h3><textarea rows="3" placeholder=tr("token_from_the_message") prop:value=move || claim_token.get() on:input=move |e| claim_token.set(event_target_value(&e))></textarea><button class="primary" on:click=claim disabled=move || busy.get()>{tr("claim_admission_pass")}</button></div></Show></Show>
        <div class="section-head"><h3>{tr("ticket_wallet")}</h3><span>{move || wallets.get().len()}</span></div><Show when=move || !loading.get().wallets fallback=move || view! { <Skeleton rows=2 /> }><div class="wallet-stack">{move || wallets.get().into_iter().map(|wallet| view! {
            <WalletCard wallet=wallet error=error />
        }).collect_view()}</div></Show><details class="import-box"><summary>{tr("add_an_existing_order")}</summary><div class="form-grid"><label>"Order ID"<input placeholder=tr("order_uuid") prop:value=move || order_id.get() on:input=move |e| order_id.set(event_target_value(&e))/></label><label>{tr("private_checkout_token")}<textarea rows="3" autocomplete="off" autocapitalize="none" spellcheck="false" prop:value=move || checkout_token.get() on:input=move |e| checkout_token.set(event_target_value(&e))></textarea></label><button class="primary" on:click=import disabled=move || busy.get()>{tr("add_to_wallet")}</button></div></details></section>
    }
}

#[component]
fn WalletCard(wallet: TicketWallet, error: RwSignal<Option<String>>) -> impl IntoView {
    let order_id = wallet.order.order_id.clone();
    let delivery_order_id = order_id.clone();
    let busy = RwSignal::new(false);
    let resend = move |_| {
        if busy.get_untracked() {
            return;
        }
        let order = delivery_order_id.clone();
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke_unit("fan_request_delivery", &OrderArgs { order_id: &order }).await
            {
                Ok(_) => error.set(Some(tr("we_resent_the_wallet_by_email").to_owned())),
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };
    view! {
        <article class="wallet-card"><header><div><p class="eyebrow">{wallet.order.status}</p><h3>{wallet.order.event_title}</h3><p>{event_time_location(&wallet.order.starts_at, wallet.order.venue.as_deref())}</p></div><strong>{wallet.order.public_reference}</strong></header><div class="ticket-stack">{wallet.tickets.into_iter().map(|ticket| view! { <WalletTicketCard order_id=order_id.clone() ticket=ticket error=error /> }).collect_view()}</div><button class="text-button" on:click=resend disabled=move || busy.get()>{move || if busy.get() { tr("sending") } else { tr("resend_tickets_by_email") }}</button></article>
    }
}

#[component]
fn WalletTicketCard(
    order_id: String,
    ticket: WalletTicket,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let public_reference = ticket.public_reference.clone();
    let qr_available = ticket.qr_available;
    let qr_svg = RwSignal::new(None::<String>);
    let qr_visible = RwSignal::new(false);
    let busy = RwSignal::new(false);
    let toggle_qr = move |_| {
        if busy.get_untracked() {
            return;
        }
        if qr_svg.get_untracked().is_some() {
            qr_visible.update(|visible| *visible = !*visible);
            return;
        }
        let order_id = order_id.clone();
        let public_reference = public_reference.clone();
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<String, _>(
                "render_wallet_qr",
                &WalletQrArgs {
                    order_id: &order_id,
                    public_reference: &public_reference,
                },
            )
            .await
            {
                Ok(svg) => {
                    qr_svg.set(Some(svg));
                    qr_visible.set(true);
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };
    view! {
        <article class="ticket-card"><div><p class="eyebrow">{ticket.ticket_type_name}</p><strong>{ticket.public_reference}</strong><span>{ticket.holder_name.value_or(ticket.holder_email_masked)}</span></div><button class="ticket-qr-button" on:click=toggle_qr disabled=move || busy.get() || !qr_available>{move || if busy.get() { tr("generating") } else if qr_visible.get() { tr("hide_qr") } else if qr_available { tr("show_qr") } else { tr("qr_unavailable") }}</button><Show when=move || qr_visible.get()>{move || qr_svg.get().map(|svg| view! { <div class="mini-qr" inner_html=svg></div> })}</Show><small>{i18n::format("qr_valid_until", &[human_time(&ticket.qr_expires_at).to_string()])}</small></article>
    }
}

#[component]
fn QrPanel(svg: Option<String>, token: String, expires: String) -> impl IntoView {
    view! { <div class="qr-panel">{svg.map(|markup| view! { <div class="qr-svg" inner_html=markup></div> })}<code>{token}</code><small>{i18n::format("valid_until_2", &[human_time(&expires).to_string()])}</small></div> }
}

#[component]
fn FanProfileScreen(
    status: RwSignal<FanSessionStatus>,
    dashboard: RwSignal<Option<FanDashboardData>>,
    wallets: RwSignal<Vec<TicketWallet>>,
    area: RwSignal<Option<AreaWallet>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let refresh = move |_| {
        refresh_fan_parts(dashboard, loading, error);
        refresh_wallets(wallets, Some(loading), error);
        refresh_fan_area(area, loading, error);
    };
    let lock = move |_| {
        spawn_local(async move {
            match bridge::invoke::<FanSessionStatus, _>("fan_lock", &EmptyArgs {}).await {
                Ok(value) => {
                    dashboard.set(None);
                    wallets.set(Vec::new());
                    area.set(None);
                    loading.set(FanLoadingState::all());
                    status.set(value);
                }
                Err(message) => error.set(Some(message)),
            }
        })
    };
    let forget = move |_| {
        spawn_local(async move {
            match bridge::invoke::<FanSessionStatus, _>("fan_forget", &EmptyArgs {}).await {
                Ok(value) => {
                    dashboard.set(None);
                    wallets.set(Vec::new());
                    area.set(None);
                    loading.set(FanLoadingState::all());
                    status.set(value);
                }
                Err(message) => error.set(Some(message)),
            }
        })
    };
    view! {
        <section class="screen">
            <header class="screen-title"><p class="eyebrow">{tr("my_profile")}</p><h2>{tr("signal_settings")}</h2></header>
            {move || status.get().session.map(|profile| view! {
                <div class="profile-card"><div class="avatar">"V"</div><div><strong>{profile.display_name.value_or_else(|| tr("virya_fan").to_owned())}</strong><p>{profile.email}</p></div></div>
                <div class="stats-grid"><Metric value=profile.wallet_count.to_string() label=tr("orders")/><Metric value=if profile.has_admission_pass { "1".to_owned() } else { "0".to_owned() } label=tr("admission_passes")/><Metric value=dashboard.with(|state| state.as_ref().map(|d| d.referral.qualified_referrals.to_string())).value_or_else(|| "—".to_owned()) label=tr("referrals")/></div>
            })}
            <div class="settings-list">
                <LanguageSwitch />
                <button on:click=refresh disabled=move || { let state = loading.get(); state.events || state.referral || state.interests || state.admission_pass || state.wallets }>{move || { let state = loading.get(); if state.events || state.referral || state.interests || state.admission_pass || state.wallets { tr("refreshing_2") } else { tr("refresh_data") } }}</button>
                <button on:click=lock>{tr("lock_app")}</button>
                <button class="danger ghost" on:click=forget>{tr("remove_profile_and_tickets_from_device")}</button>
            </div>
            <AnonymousFeedback error=error />
            <p class="security-note">{tr("fan_session_admission_pass_and_private_wallet")}</p>
        </section>
    }
}

#[component]
fn AnonymousFeedback(error: RwSignal<Option<String>>) -> impl IntoView {
    let category = RwSignal::new("idea".to_owned());
    let message = RwSignal::new(String::new());
    let busy = RwSignal::new(false);

    let submit = move |_| {
        if busy.get_untracked() {
            return;
        }
        let current_category = category.get_untracked();
        let current_message = message.get_untracked().trim().to_owned();
        let length = current_message.chars().count();
        if !(8..=2_000).contains(&length) {
            error.set(Some(
                tr("feedback_must_contain_between_8_and_2000").to_owned(),
            ));
            return;
        }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke_unit(
                "submit_anonymous_feedback",
                &AnonymousFeedbackArgs {
                    category: &current_category,
                    message: &current_message,
                },
            )
            .await
            {
                Ok(()) => {
                    message.set(String::new());
                    error.set(Some(
                        tr("feedback_was_sent_anonymously_thank_you").to_owned(),
                    ));
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    view! {
        <section class="feedback-card">
            <div class="feedback-heading">
                <div><p class="eyebrow">{tr("anonymous_feedback")}</p><h3>{tr("tell_us_what_to_improve")}</h3></div>
                <span aria-hidden="true">"◌"</span>
            </div>
            <p>{tr("app_sends_only_the_category_and_message")}</p>
            <label class="select-label">
                {tr("category")}
                <select prop:value=move || category.get() on:change=move |event| category.set(event_target_value(&event))>
                    <option value="idea">{tr("idea")}</option>
                    <option value="bug">{tr("bug_label")}</option>
                    <option value="concert">{tr("shows_and_tickets")}</option>
                    <option value="merch">{tr("merch")}</option>
                    <option value="other">{tr("other")}</option>
                </select>
            </label>
            <label>
                {tr("message")}
                <textarea
                    rows="6"
                    maxlength="2000"
                    placeholder=tr("tell_us_directly_what_is_broken_or")
                    prop:value=move || message.get()
                    on:input=move |event| message.set(event_target_value(&event))
                ></textarea>
            </label>
            <div class="feedback-submit-row">
                <small>{move || format!("{} / 2000", message.get().chars().count())}</small>
                <button type="button" class="primary" disabled=move || busy.get() || message.get().trim().chars().count() < 8 on:click=submit>
                    {move || if busy.get() { tr("sending_2") } else { tr("send_anonymously") }}
                </button>
            </div>
        </section>
    }
}
