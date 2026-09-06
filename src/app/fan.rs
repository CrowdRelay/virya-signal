#[component]
fn FanPortal(
    mode: RwSignal<RootMode>,
    status: RwSignal<FanSessionStatus>,
    status_loading: RwSignal<bool>,
    status_failed: RwSignal<bool>,
    status_refresh: RwSignal<u32>,
    push_target: RwSignal<Option<String>>,
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
            view! { <FanApp mode=mode status=status status_refresh=status_refresh public=public push_target=push_target error=error /> }.into_any()
        } else {
            view! { <FanAccess mode=mode status=status status_refresh=status_refresh error=error /> }.into_any()
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
                <Skeleton rows=2 height=120 />
            </div>
        </section>
    }
}

#[derive(Clone, Copy)]
struct FanConfirmationSession {
    status: RwSignal<FanSessionStatus>,
    status_refresh: RwSignal<u32>,
}

#[derive(Clone, Debug)]
struct FanConfirmationValues {
    email: String,
    name: Option<String>,
    token: String,
    pin: String,
}

async fn exchange_fan_confirmation(
    values: FanConfirmationValues,
) -> Result<FanSessionStatus, String> {
    let FanConfirmationValues {
        email: input_email,
        name: input_name,
        token: current_token,
        pin: current_pin,
    } = values;
    if current_token.trim().is_empty() {
        return Err(tr("paste_the_code_or_full_link_or").to_owned());
    }
    if !new_operator_pin_is_valid(&current_pin) {
        return Err(tr("enter_4_6_digits_for_this_fan_profile").to_owned());
    }
    let input = FanConfirmationInput {
        api_base_url: API_BASE.to_owned(),
        // Email/display name are optional hints only. CrowdRelay resolves and
        // returns the canonical identity from the one-time token.
        email: input_email.trim().to_owned(),
        display_name: input_name,
        token: current_token.trim().to_owned(),
    };
    let status = bridge::invoke::<FanSessionStatus, _>(
        "fan_confirm",
        &FanConfirmArgs {
            input: &input,
            pin: &current_pin,
        },
    )
    .await?;
    Ok(status)
}

/// Adopt an authoritative unlocked session into the UI.
///
/// Camera transitions can temporarily dispose/remount the WebView, so every
/// write here is best-effort: the native command has already persisted the
/// session and UI reconciliation must never turn a successful login into a
/// client-side failure.
fn adopt_fan_session(
    value: FanSessionStatus,
    token: RwSignal<String>,
    pin: RwSignal<String>,
    session: FanConfirmationSession,
) {
    // The identity this process reports has just changed. A `launcher_status`
    // that is still on the wire was asked before the change, and the refresh
    // below would otherwise join it — same command, same args, same scope — and
    // adopt its pre-login answer, dropping the fan straight back onto the lock
    // screen. Every adoption path funnels through here, so this is the one
    // place the stale scope has to be retired.
    bridge::invalidate_latest("launcher:");
    let _ = pin.try_set(String::new());
    let _ = token.try_set(String::new());
    // A confirmation (email code, deep link, or QR scan) is always a
    // deliberate "come back to Signal" moment. Land the fan on the
    // Signal tab even if a stale tab preference (e.g. Merch) was
    // persisted from a session that never explicitly logged out.
    persist_fan_tab(FanTab::Signal);
    let _ = session.status.try_set(value);
    let _ = session
        .status_refresh
        .try_update(|generation| *generation = generation.wrapping_add(1));
}

async fn run_fan_confirmation(
    values: FanConfirmationValues,
    token: RwSignal<String>,
    pin: RwSignal<String>,
    session: FanConfirmationSession,
    error: RwSignal<Option<String>>,
) {
    match exchange_fan_confirmation(values).await {
        Ok(value) => adopt_fan_session(value, token, pin, session),
        Err(message) => {
            // The mail token is one-time: by the time fan_confirm can fail on the
            // client, the native side may already have consumed it, written the
            // vault and unlocked the session. Losing that response (a WebView
            // disposed across the camera transition, a dropped IPC reply) would
            // otherwise strand the fan on the login screen holding a token that
            // can never be redeemed again. Native state is authoritative, so ask
            // it before reporting a failure and adopt an already-unlocked
            // session instead of surfacing an error for a login that succeeded.
            if let Ok(status) =
                bridge::invoke_timeout::<FanSessionStatus, _>("fan_status", &EmptyArgs {}, 5_000)
                    .await
                && status.unlocked
            {
                adopt_fan_session(status, token, pin, session);
                return;
            }
            let _ = error.try_set(Some(message));
        }
    }
}

fn submit_fan_confirmation_values(
    values: FanConfirmationValues,
    token: RwSignal<String>,
    pin: RwSignal<String>,
    busy: RwSignal<bool>,
    session: FanConfirmationSession,
    error: RwSignal<Option<String>>,
) {
    if busy.get_untracked() {
        return;
    }
    if values.token.trim().is_empty() {
        error.set(Some(tr("paste_the_code_or_full_link_or").to_owned()));
        return;
    }
    if !new_operator_pin_is_valid(&values.pin) {
        error.set(Some(tr("enter_4_6_digits_for_this_fan_profile").to_owned()));
        return;
    }
    busy.set(true);
    spawn_local(async move {
        run_fan_confirmation(values, token, pin, session, error).await;
        let _ = busy.try_set(false);
    });
}

/// Spends a confirmation link Android handed to the app. The token stayed
/// native, so the only thing this sends is how the fan wants the vault sealed.
///
/// `None` means device unlock: the native side generates the password and hands
/// it to the keystore, and the fan is never asked for anything.
fn submit_fan_confirm_link(
    token: RwSignal<String>,
    pin: RwSignal<String>,
    busy: RwSignal<bool>,
    session: FanConfirmationSession,
    error: RwSignal<Option<String>>,
    with_pin: bool,
) {
    if busy.get_untracked() {
        return;
    }
    let current_pin = pin.get_untracked();
    if with_pin && !new_operator_pin_is_valid(&current_pin) {
        error.set(Some(tr("enter_4_6_digits_for_this_fan_profile").to_owned()));
        return;
    }
    busy.set(true);
    spawn_local(async move {
        match bridge::invoke::<FanSessionStatus, _>(
            "fan_confirm_link",
            &FanConfirmLinkArgs {
                api_base_url: API_BASE,
                pin: with_pin.then_some(current_pin.as_str()),
            },
        )
        .await
        {
            Ok(value) => adopt_fan_session(value, token, pin, session),
            Err(message) => {
                // Same reasoning as run_fan_confirmation: the capability is
                // one-time, so native state decides whether this failed.
                if let Ok(status) = bridge::invoke_timeout::<FanSessionStatus, _>(
                    "fan_status",
                    &EmptyArgs {},
                    5_000,
                )
                .await
                    && status.unlocked
                {
                    adopt_fan_session(status, token, pin, session);
                } else {
                    let _ = error.try_set(Some(message));
                }
            }
        }
        let _ = busy.try_set(false);
    });
}

fn submit_fan_confirmation(
    email: RwSignal<String>,
    name: RwSignal<String>,
    token: RwSignal<String>,
    pin: RwSignal<String>,
    busy: RwSignal<bool>,
    session: FanConfirmationSession,
    error: RwSignal<Option<String>>,
) {
    submit_fan_confirmation_values(
        FanConfirmationValues {
            email: email.get_untracked(),
            name: optional(name.get_untracked().trim().to_owned()),
            token: token.get_untracked(),
            pin: pin.get_untracked(),
        },
        token,
        pin,
        busy,
        session,
        error,
    );
}

#[component]
fn FanAccess(
    mode: RwSignal<RootMode>,
    status: RwSignal<FanSessionStatus>,
    status_refresh: RwSignal<u32>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let access_mode = RwSignal::new(FanAccessMode::Signup);
    // A confirmation link Android routed to the app instead of the browser. The
    // token itself stays native; this only records that one is waiting so the
    // form can drop to a PIN prompt.
    let link_pending = RwSignal::new(false);
    Effect::new(move |_| {
        // A link tapped while Signal is already running arrives as a warm
        // intent, and the resume signal is what the shell bumps for exactly
        // that case. Without reading it here this runs once at mount and a
        // warm confirmation link is never collected.
        status_refresh.get();
        if !bridge::native_available() || link_pending.get_untracked() {
            return;
        }
        spawn_local(async move {
            if let Ok(true) =
                bridge::invoke::<bool, _>("fan_take_confirm_link", &EmptyArgs {}).await
            {
                let _ = link_pending.try_set(true);
                let _ = access_mode.try_set(FanAccessMode::Confirm);
            }
        });
    });
    let email = RwSignal::new(String::new());
    let name = RwSignal::new(String::new());
    // City onboarding is deliberately local-first. The remote canonical list is
    // not placed in Leptos reactive state: this removes a repeatedly crashing
    // Android WebView/WASM path while preserving full signup functionality.
    let custom_city_name = RwSignal::new(String::new());
    let custom_region = RwSignal::new(String::new());
    let referral = RwSignal::new(bridge::referral_code_from_location().unwrap_or_default());
    let token = RwSignal::new(String::new());
    let pin = RwSignal::new(String::new());
    let consent = RwSignal::new(true);
    let nearby_enabled = RwSignal::new(true);
    let radius_km = RwSignal::new(150_u16);
    let busy = RwSignal::new(false);
    let recovery_open = RwSignal::new(false);
    // Email-first signup: stage 1 collects only the email, stage 2 reveals
    // city, PIN and consent. Lower friction, same backend payload.
    let signup_stage = RwSignal::new(1_u8);

    // A fan who wants a secret in their head rather than one in the phone can
    // say so, and the PIN field comes back. The offer is only made where the
    // keystore can actually hold a password.
    let prefer_pin = RwSignal::new(false);

    let open_latarnik = move |_| mode.set(RootMode::Latarnik);

    let run_unlock = move || {
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
                    let _ = pin.try_set(String::new());
                    bridge::invalidate_latest("launcher:");
                    let _ = status.try_set(value);
                    status_refresh.update(|generation| *generation = generation.wrapping_add(1));
                }
                Err(message) => {
                    let _ = error.try_set(Some(message));
                }
            }
            let _ = busy.try_set(false);
        });
    };

    // Signup makes the same offer the mailed link does: on a phone that can
    // hold the password, a PIN is a choice rather than a step. The field is
    // only shown to a fan who asked for one, or to a device that has no
    // keystore to hold anything.
    let signup_with_pin =
        move || prefer_pin.get() || !status.get().device_unlock_supported;
    let run_signup = move || {
        if !consent.get() {
            error.set(Some(
                tr("marketing_consent_is_required_to_join_signal").to_owned(),
            ));
            return;
        }
        let current_pin = pin.get();
        let with_pin = signup_with_pin();
        if with_pin && !new_operator_pin_is_valid(&current_pin) {
            error.set(Some(tr("enter_4_6_digits_for_this_fan_profile").to_owned()));
            return;
        }
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
                    let _ = error.try_set(Some(i18n::format(
                        "could_not_save_city_message",
                        std::slice::from_ref(&message),
                    )));
                    let _ = busy.try_set(false);
                    return;
                }
            };
            if city_slug.trim().is_empty() {
                let _ = error.try_set(Some(tr("select_a_city_or_enter_your_own").to_owned()));
                let _ = busy.try_set(false);
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
                    pin: with_pin.then_some(current_pin.as_str()),
                },
            )
            .await
            {
                Ok(result) => {
                    if result.session_created {
                        let _ = pin.try_set(String::new());
                        bridge::invalidate_latest("launcher:");
                        match bridge::invoke_timeout::<FanSessionStatus, _>(
                            "fan_status",
                            &EmptyArgs {},
                            5_000,
                        )
                        .await
                        {
                            Ok(value) => {
                                let _ = status.try_set(value);
                            }
                            Err(message) => {
                                let _ = error.try_set(Some(message));
                            }
                        }
                        status_refresh.update(|value| *value = value.wrapping_add(1));
                    } else {
                        let _ = access_mode.try_set(FanAccessMode::Confirm);
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
                                    .map(|seconds| (seconds / 60).saturating_add(if seconds % 60 > 0 { 1 } else { 0 }))
                                    .unwrap_or(15)
                                    .clamp(1, 15);
                                i18n::format(
                                    "new_message_not_sent_previous_code_still_valid_minutes",
                                    &[minutes.to_string()],
                                )
                            }
                            None => { tr("request_was_accepted_check_your_inbox_and") }.to_owned(),
                        };
                        let _ = error.try_set(Some(message));
                    }
                }
                Err(message) => {
                    let _ = error.try_set(Some(message));
                }
            }
            let _ = busy.try_set(false);
        });
    };

    // Stage two opens below the fold, so advancing used to leave the fan
    // looking at the same email field with the next step off screen. Focusing
    // the first field of the new stage both continues the flow and scrolls it
    // into view.
    let city_ref = NodeRef::<leptos::html::Input>::new();
    Effect::new(move |_| {
        if signup_stage.get() >= 2
            && access_mode.get() == FanAccessMode::Signup
            && let Some(element) = city_ref.get()
        {
            let _ = element.focus();
        }
    });

    let unlock_pin_ref = NodeRef::<leptos::html::Input>::new();
    Effect::new(move |_| {
        // Only when the field is actually on screen: a configured fan that is
        // not being asked for a link is looking at the PIN prompt.
        if status.get().configured
            && !link_pending.get()
            && !recovery_open.get()
            && let Some(element) = unlock_pin_ref.get()
        {
            let _ = element.focus();
        }
    });

    let confirmation_session = FanConfirmationSession {
        status,
        status_refresh,
    };

    let run_confirm = move || {
        if link_pending.get_untracked() {
            submit_fan_confirm_link(token, pin, busy, confirmation_session, error, true);
            return;
        }
        submit_fan_confirmation(email, name, token, pin, busy, confirmation_session, error);
    };
    // The link is the credential and the fan already proved they hold it by
    // opening the mail. Nothing is asked for here: the device seals the vault
    // password itself.
    let run_confirm_without_pin =
        move || submit_fan_confirm_link(token, pin, busy, confirmation_session, error, false);
    let unlock = move |_| run_unlock();
    let signup = move |_| run_signup();
    let confirm = move |_| run_confirm();
    let confirm_without_pin = move |_: leptos::ev::MouseEvent| run_confirm_without_pin();
    let seal_without_pin = move || status.get().device_unlock_supported && !prefer_pin.get();

    // A device that has already sealed a password opens on its own. The gate
    // never shows a PIN field it does not need, and a seal the keystore
    // refuses falls through to the PIN prompt rather than to a dead screen.
    let device_unlock_attempted = RwSignal::new(false);
    Effect::new(move |_| {
        let current = status.get();
        if !current.configured
            || current.unlocked
            || !current.device_unlock
            || device_unlock_attempted.get_untracked()
        {
            return;
        }
        device_unlock_attempted.set(true);
        spawn_local(async move {
            if let Ok(value) =
                bridge::invoke::<FanSessionStatus, _>("fan_device_unlock", &EmptyArgs {}).await
                && value.unlocked
            {
                bridge::invalidate_latest("launcher:");
                let _ = status.try_set(value);
                let _ = status_refresh
                    .try_update(|generation| *generation = generation.wrapping_add(1));
            }
        });
    });

    // A mailed link that this device can seal for itself needs nothing from the
    // fan, so it is spent on arrival rather than behind a button. The tap it
    // replaces was asking permission to do the only thing the screen offered.
    //
    // One attempt, ever: the token is one-time, and a retry after a partial
    // failure would spend a credential that may already be gone. If it fails
    // the panel below is still there, with the same button and the PIN
    // alternative next to it.
    let auto_link_attempted = RwSignal::new(false);
    Effect::new(move |_| {
        // `busy` is read untracked on purpose. Submitting sets it, and an
        // effect that tracked it would re-run inside its own call — disposing
        // the owner that the spawned request is scoped to and cancelling the
        // exchange before it reached the native side.
        if !link_pending.get()
            || !seal_without_pin()
            || busy.get_untracked()
            || auto_link_attempted.get_untracked()
        {
            return;
        }
        auto_link_attempted.set(true);
        run_confirm_without_pin();
    });

    let run_request_access = move || {
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
                    let _ = access_mode.try_set(FanAccessMode::Confirm);
                    let _ = error.try_set(Some(tr("if_this_email_is_registered_in_virya").to_owned()));
                }
                Err(message) => {
                    let _ = error.try_set(Some(message));
                }
            }
            let _ = busy.try_set(false);
        });
    };
    let request_access = move |_| run_request_access();

    let scan_confirmation = move |_| {
        if busy.get_untracked() {
            return;
        }

        let scan_pin = pin.get_untracked();
        busy.set(true);
        spawn_local(async move {
            if let Err(message) = bridge::invoke_unit(
                "fan_prepare_confirmation",
                &FanPrepareConfirmationArgs {
                    api_base_url: API_BASE,
                    pin: &scan_pin,
                },
            )
            .await
            {
                let _ = error.try_set(Some(message));
                let _ = busy.try_set(false);
                return;
            }

            match bridge::scan_and_confirm_fan().await {
                Ok(Some(value)) => adopt_fan_session(value, token, pin, confirmation_session),
                Ok(None) => {
                    // A cancelled scan is a no-op, but a camera-resume race
                    // can report a cancellation while the native side actually
                    // completed a previous attempt. Check before doing nothing.
                    if let Ok(status) = bridge::invoke_timeout::<FanSessionStatus, _>(
                        "fan_status",
                        &EmptyArgs {},
                        3_000,
                    )
                    .await
                        && status.unlocked
                    {
                        adopt_fan_session(status, token, pin, confirmation_session);
                    }
                }
                Err(message) => {
                    if let Ok(status) = bridge::invoke_timeout::<FanSessionStatus, _>(
                        "fan_status",
                        &EmptyArgs {},
                        5_000,
                    )
                    .await
                        && status.unlocked
                    {
                        adopt_fan_session(status, token, pin, confirmation_session);
                    } else {
                        let _ = error.try_set(Some(message));
                    }
                }
            }
            let _ = busy.try_set(false);
        });
    };

    view! {
        <section class="fan-access">
            <header class="fan-access-hero">
                <p class="eyebrow">{tr("virya_signal")}</p>
                <h1>{tr("shows_tickets")}<br/><em>{tr("and_rewards")}</em></h1>
                <div class="signal-purpose-grid" role="group" aria-label=tr("what_virya_signal_gives_you")>
                    <span><b aria-hidden="true">"⌁"</b>{tr("shows_near_you")}</span>
                    <span><b aria-hidden="true">"▣"</b>{tr("tickets_and_qr_codes_on_your_phone")}</span>
                    <span><b aria-hidden="true">"✦"</b>{tr("rewards_for_simple_actions")}</span>
                </div>
            </header>
            <Show when=move || status.get().configured fallback=move || view! {
                <div class="access-card fan-card">
                    {move || error.get().map(|msg| view! { <p class="inline-form-error" role="alert">{msg}</p> })}
                    <div class="segmented">
                        <button class:active=move || access_mode.get() == FanAccessMode::Signup on:click=move |_| access_mode.set(FanAccessMode::Signup)>{tr("get_started")}</button>
                        <button class:active=move || access_mode.get() == FanAccessMode::Confirm on:click=move |_| access_mode.set(FanAccessMode::Confirm)>{tr("i_have_a_code")}</button>
                    </div>
                    <div class="form-grid fan-form">
                        // CrowdRelay resolves the identity from the token itself,
                        // so a capability that already arrived needs no address and
                        // no name. Asking for them anyway is what made the screen
                        // look like the link had done nothing.
                        <Show when=move || !link_pending.get()>
                            // The hint is a 56-character sentence — "we will send a
                            // confirmation link, the rest takes 30 seconds" — and it was the
                            // input's placeholder. At the field's width it clipped mid-word,
                            // and a placeholder disappears the moment anyone types, so the
                            // one line that tells a new fan this is a 30-second link signup
                            // was never readable. It sits under the field instead.
                            <label>{tr("enter_email_to_start")}<input aria-label=tr("email") type="email" autocomplete="email" inputmode="email" placeholder="you@example.com" prop:value=move || email.get() on:input=move |e| email.set(event_target_value(&e)) on:keydown=on_enter(move || { if !busy.get_untracked() && !email.get_untracked().trim().is_empty() && signup_stage.get_untracked() < 2 { signup_stage.set(2); } })/></label>
                            <p class="inline-note">{tr("email_first_hint")}</p>
                            <Show when=move || { signup_stage.get() >= 2 }>
                                <label>{tr("name_optional")}<input prop:value=move || name.get() on:input=move |e| name.set(event_target_value(&e))/></label>
                            </Show>
                        </Show>
                        <Show when=move || access_mode.get() == FanAccessMode::Signup fallback=move || view! {
                            <>
                                <Show when=move || link_pending.get() fallback=move || view! {
                                    <p class="confirm-hint"><strong>{tr("fastest_scan_the_qr_from_the_email")}</strong><br/>{tr("email_button_or_scan_hint")}</p>
                                }>
                                    <p class="confirm-hint"><strong>{tr("link_from_the_email_is_ready")}</strong><br/>{move || if seal_without_pin() { tr("enter_without_a_pin") } else { tr("set_a_pin_and_enter_signal") }}</p>
                                </Show>
                                // The link already proved the fan holds the
                                // address. Asking for a PIN on top of it buys
                                // nothing the device cannot hold itself, so
                                // where the keystore can hold it, that is the
                                // offer and the PIN is the alternative.
                                <Show when=seal_without_pin fallback=move || view! {
                                    <>
                                        <label class="pin-field">
                                            <span class="pin-field-label">{tr("create_fan_unlock_pin")}</span>
                                            <small id="fan-confirm-pin-help">{tr("enter_4_6_digits_for_this_fan_profile")}</small>
                                            <input aria-label=tr("create_fan_unlock_pin") type="password" autocomplete="new-password" inputmode="numeric" pattern="[0-9]*" maxlength="6" placeholder=tr("pin_example") aria-describedby="fan-confirm-pin-help" prop:value=move || pin.get() on:input=move |e| pin.set(normalize_new_operator_pin(event_target_value(&e))) on:keydown=on_enter(move || { if !busy.get_untracked() && new_operator_pin_is_valid(&pin.get_untracked()) && (link_pending.get_untracked() || !token.get_untracked().trim().is_empty()) { run_confirm(); } })/>
                                        </label>
                                        <p class="confirmation-note">{tr("pin_encrypts_your_profile_on_this_device")}</p>
                                        // A pending link is spent by the button;
                                        // without one the camera is the only
                                        // capability source, and it needs the PIN
                                        // first, so it follows the field rather
                                        // than preceding it.
                                        <Show when=move || link_pending.get() fallback=move || view! {
                                            <div class="confirmation-actions single">
                                                <button type="button" class="confirmation-action primary-scan" disabled=move || busy.get() || (signup_with_pin() && !new_operator_pin_is_valid(&pin.get())) on:click=scan_confirmation><span aria-hidden="true">"▦"</span><strong>{tr("scan_qr")}</strong><small>{tr("scan_the_code_from_the_email")}</small></button>
                                            </div>
                                        }>
                                            <button class="primary" disabled=move || busy.get() || !new_operator_pin_is_valid(&pin.get()) on:click=confirm>{tr("confirm_and_enter")}</button>
                                        </Show>
                                    </>
                                }>
                                    <p class="confirmation-note">{tr("device_unlock_explainer")}</p>
                                    // A link already in hand is spent by the
                                    // button; without one the camera is still
                                    // the capability source, and it no longer
                                    // needs a PIN to start.
                                    <Show when=move || link_pending.get() fallback=move || view! {
                                        <div class="confirmation-actions single">
                                            <button type="button" class="confirmation-action primary-scan" disabled=move || busy.get() on:click=scan_confirmation><span aria-hidden="true">"▦"</span><strong>{tr("scan_qr")}</strong><small>{tr("scan_the_code_from_the_email")}</small></button>
                                        </div>
                                    }>
                                        <button class="primary" disabled=move || busy.get() on:click=confirm_without_pin>{tr("enter_without_a_pin")}</button>
                                    </Show>
                                    <button type="button" class="text-button" on:click=move |_| prefer_pin.set(true)>{tr("prefer_a_pin_instead")}</button>
                                </Show>
                                <button type="button" class="text-button" disabled=move || busy.get() || email.get().trim().is_empty() on:click=request_access>{tr("i_already_have_an_account_send_login")}</button>
                                <p class="confirmation-resend">{tr("no_message_check_spam_after_15_minutes")}</p>
                            </>
                        }>
                            <>
                                <Show when=move || { signup_stage.get() < 2 } fallback=move || view! {
                                    <>
                                        // The hint named a PIN the step below no longer always asks for.
                                        <p class="signup-details-hint">{move || if signup_with_pin() { tr("signup_details_hint") } else { tr("signup_details_hint_no_pin") }}</p>
                                        <div class="custom-city-fields city-stable-entry">
                                            <label>{tr("city")}<input node_ref=city_ref placeholder=tr("e_g_bielawa") prop:value=move || custom_city_name.get() on:input=move |e| custom_city_name.set(event_target_value(&e))/></label>
                                            <label>{tr("province_region_optional")}<input placeholder=tr("lower_silesia") prop:value=move || custom_region.get() on:input=move |e| custom_region.set(event_target_value(&e))/></label>
                                            <p class="inline-note">{tr("enter_your_city_manually_we_will_match")}</p>
                                        </div>
                                        <div class="nearby-pref">
                                            <label class="pref-row pref-row-inline"><span class="pref-row-label">{tr("notify_me_about_nearby_shows")}</span><input type="checkbox" class="pref-switch" prop:checked=move || nearby_enabled.get() on:change=move |e| nearby_enabled.set(event_target_checked(&e))/></label>
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
                                        <Show when=signup_with_pin fallback=move || view! {
                                            <p class="confirmation-note">{tr("device_unlock_explainer")}</p>
                                            <button type="button" class="text-button" on:click=move |_| prefer_pin.set(true)>{tr("prefer_a_pin_instead")}</button>
                                        }>
                                            <label class="pin-field">
                                                <span class="pin-field-label">{tr("create_fan_unlock_pin")}</span>
                                                <small id="fan-signup-pin-help">{tr("enter_4_6_digits_for_this_fan_profile")}</small>
                                                <input type="password" autocomplete="new-password" inputmode="numeric" pattern="[0-9]*" maxlength="6" placeholder=tr("pin_example") aria-describedby="fan-signup-pin-help" prop:value=move || pin.get() on:input=move |e| pin.set(normalize_new_operator_pin(event_target_value(&e))) on:keydown=on_enter(move || { if !busy.get_untracked() && new_operator_pin_is_valid(&pin.get_untracked()) { run_signup(); } })/>
                                            </label>
                                        </Show>
                                        <div class="pref-list"><label class="pref-row"><span class="pref-row-label">{tr("i_want_to_receive_information_about_virya")}</span><input type="checkbox" class="pref-switch" prop:checked=move || consent.get() on:change=move |e| consent.set(event_target_checked(&e))/></label></div>
                                        <button class="primary" disabled=move || busy.get() || (signup_with_pin() && !new_operator_pin_is_valid(&pin.get())) on:click=signup>{tr("join_signal")}</button>
                                    </>
                                }>
                                    <button class="primary" disabled=move || busy.get() || email.get().trim().is_empty() on:click=move |_| signup_stage.set(2)>{tr("continue_to_details")}</button>
                                </Show>
                            </>
                        </Show>
                    </div>
                </div>
            }>
                <div class="access-card fan-card">
                    {move || error.get().map(|msg| view! { <p class="inline-form-error" role="alert">{msg}</p> })}
                    // A fan who already has a profile on this device and asked
                    // for a login link landed here: the link was collected, the
                    // token stayed native, and this branch never looked at
                    // `link_pending`. The screen said "enter your PIN" — the one
                    // thing they had just told us they could not do — and the
                    // recovery panel's confirm stayed disabled forever, because
                    // it required a token the link had deliberately kept out of
                    // the WebView. The arriving link now owns the screen.
                    <Show when=move || link_pending.get() && !recovery_open.get() fallback=move || view! {
                    <Show when=move || recovery_open.get() fallback=move || view! {
                        <>
                            <p class="lock-copy">{tr("your_profile_and_tickets_are_encrypted_on")}</p>
                            <div class="form-grid">
                                // A vault sealed by the device has no PIN behind
                                // it, so the field below cannot open it and any
                                // PIN typed into it would read as "wrong PIN".
                                // The way back in is a fresh link.
                                <Show when=move || !status.get().pin_unlock>
                                    <p class="inline-note">{tr("device_unlock_is_on")}</p>
                                </Show>
                                <label class="pin-field" class:hidden=move || !status.get().pin_unlock>
                                    <span class="pin-field-label">{tr("fan_app_unlock_pin")}</span>
                                    <small id="fan-unlock-pin-help">{tr("enter_the_pin_created_for_this_fan")}</small>
                                    // The lock screen has exactly one field and
                                    // is the first thing every launch shows.
                                    // Landing in it costs the fan a tap that has
                                    // no other possible target.
                                    <input node_ref=unlock_pin_ref type="password" autocomplete="current-password" inputmode="numeric" pattern="[0-9]*" placeholder=tr("your_pin") aria-describedby="fan-unlock-pin-help" prop:value=move || pin.get() on:input=move |e| pin.set(event_target_value(&e)) on:keydown=on_enter(move || { if !busy.get_untracked() && pin.get_untracked().chars().count() >= 4 { run_unlock(); } })/>
                                </label>
                                <Show when=move || status.get().pin_unlock>
                                    <button class="primary" disabled=move || busy.get() || pin.get().chars().count() < 4 on:click=unlock>{tr("open_my_signal")}</button>
                                </Show>
                                <button type="button" class="text-button recovery-link" on:click=move |_| recovery_open.set(true)>{tr("i_forgot_my_pin_sign_in_again")}</button>
                            </div>
                        </>
                    }>
                        <div class="form-grid recovery-panel">
                            <div class="recovery-heading"><p class="eyebrow">{tr("access_recovery")}</p><h3>{tr("create_a_new_pin")}</h3><p>{tr("enter_your_email_request_a_fresh_link")}</p></div>
                            <label>{tr("email")}<input aria-label=tr("email") type="email" autocomplete="email" prop:value=move || email.get() on:input=move |e| email.set(event_target_value(&e)) on:keydown=on_enter(move || { if !busy.get_untracked() && !email.get_untracked().trim().is_empty() { run_request_access(); } })/></label>
                            <button type="button" class="ghost" disabled=move || busy.get() || email.get().trim().is_empty() on:click=request_access>{tr("send_login_link")}</button>
                            <p class="confirm-hint">{tr("email_button_or_scan_hint")}</p>
                            <Show when=seal_without_pin fallback=move || view! {
                                <>
                                    <label class="pin-field">
                                        <span class="pin-field-label">{tr("create_fan_unlock_pin")}</span>
                                        <small id="fan-recovery-pin-help">{tr("enter_4_6_digits_for_this_fan_profile")}</small>
                                        <input aria-label=tr("create_fan_unlock_pin") type="password" autocomplete="new-password" inputmode="numeric" pattern="[0-9]*" maxlength="6" placeholder=tr("pin_example") aria-describedby="fan-recovery-pin-help" prop:value=move || pin.get() on:input=move |e| pin.set(normalize_new_operator_pin(event_target_value(&e))) on:keydown=on_enter(move || { if !busy.get_untracked() && new_operator_pin_is_valid(&pin.get_untracked()) && link_pending.get_untracked() { run_confirm(); } })/>
                                    </label>
                                    // The link Android routed here is spent by
                                    // the button. Without one the camera is the
                                    // capability source, and it needs the PIN
                                    // first.
                                    <Show when=move || link_pending.get() fallback=move || view! {
                                        <div class="confirmation-actions single">
                                            <button type="button" class="confirmation-action primary-scan" disabled=move || busy.get() || !new_operator_pin_is_valid(&pin.get()) on:click=scan_confirmation><span aria-hidden="true">"▦"</span><strong>{tr("scan_qr")}</strong><small>{tr("scan_the_code_from_the_email")}</small></button>
                                        </div>
                                    }>
                                        <button class="primary" disabled=move || busy.get() || !new_operator_pin_is_valid(&pin.get()) on:click=confirm>{tr("confirm_and_set_new_pin")}</button>
                                    </Show>
                                </>
                            }>
                                <p class="confirmation-note">{tr("device_unlock_explainer")}</p>
                                <Show when=move || link_pending.get() fallback=move || view! {
                                    <div class="confirmation-actions single">
                                        <button type="button" class="confirmation-action primary-scan" disabled=move || busy.get() on:click=scan_confirmation><span aria-hidden="true">"▦"</span><strong>{tr("scan_qr")}</strong><small>{tr("scan_the_code_from_the_email")}</small></button>
                                    </div>
                                }>
                                    <button class="primary" disabled=move || busy.get() on:click=confirm_without_pin>{tr("enter_without_a_pin")}</button>
                                </Show>
                                <button type="button" class="text-button" on:click=move |_| prefer_pin.set(true)>{tr("prefer_a_pin_instead")}</button>
                            </Show>
                            <button type="button" class="text-button" on:click=move |_| recovery_open.set(false)>{tr("back_to_pin_login")}</button>
                        </div>
                    </Show>
                    }>
                        <div class="form-grid recovery-panel">
                            <div class="recovery-heading">
                                <p class="eyebrow">{tr("access_recovery")}</p>
                                <h3>{tr("link_from_the_email_is_ready")}</h3>
                                // The subline used to promise a PIN step the
                                // panel below no longer takes.
                                <p>{move || if seal_without_pin() {
                                    tr("enter_without_a_pin")
                                } else {
                                    tr("set_a_pin_and_enter_signal")
                                }}</p>
                            </div>
                            <Show when=seal_without_pin fallback=move || view! {
                                <>
                                    <label class="pin-field">
                                        <span class="pin-field-label">{tr("create_fan_unlock_pin")}</span>
                                        <small id="fan-link-pin-help">{tr("enter_4_6_digits_for_this_fan_profile")}</small>
                                        <input aria-label=tr("create_fan_unlock_pin") type="password" autocomplete="new-password" inputmode="numeric" pattern="[0-9]*" maxlength="6" placeholder=tr("pin_example") aria-describedby="fan-link-pin-help" prop:value=move || pin.get() on:input=move |e| pin.set(normalize_new_operator_pin(event_target_value(&e))) on:keydown=on_enter(move || { if !busy.get_untracked() && new_operator_pin_is_valid(&pin.get_untracked()) { run_confirm(); } })/>
                                    </label>
                                    <p class="confirmation-note">{tr("pin_encrypts_your_profile_on_this_device")}</p>
                                    <button class="primary" disabled=move || busy.get() || !new_operator_pin_is_valid(&pin.get()) on:click=confirm>{tr("confirm_and_enter")}</button>
                                </>
                            }>
                                <p class="confirmation-note">{tr("device_unlock_explainer")}</p>
                                <button class="primary" disabled=move || busy.get() on:click=confirm_without_pin>{tr("enter_without_a_pin")}</button>
                                <button type="button" class="text-button" on:click=move |_| prefer_pin.set(true)>{tr("prefer_a_pin_instead")}</button>
                            </Show>
                            // The old PIN still works until the link is spent,
                            // so a fan who tapped the link by accident is not
                            // trapped in a screen with no way back. The native
                            // pending link is also cleared so the next resume
                            // tick does not re-offer it and reopen this panel.
                            // The panel closes only once native reports the link
                            // is actually gone. Closing first and clearing after
                            // would put the fan on the PIN screen while the next
                            // resume tick still had a link to re-offer.
                            <button type="button" class="text-button" on:click=move |_| {
                                spawn_local(async move {
                                    match bridge::invoke_unit("fan_clear_pending_confirm_link", &EmptyArgs {}).await {
                                        Ok(()) => { let _ = link_pending.try_set(false); }
                                        Err(message) => { let _ = error.try_set(Some(message)); }
                                    }
                                });
                            }>{tr("back_to_pin_login")}</button>
                        </div>
                    </Show>
                </div>
            </Show>
            // Latarnik is the press/partner path. It used to sit between the
            // headline and the sign-in card, so the first screen of a fan app
            // spent its best position pitching a zone almost no fan wants. It
            // follows the fan's own action now, in source order as well as on
            // screen (WCAG 1.3.2).
            <div class="latarnik-entry">
                <p class="eyebrow">{tr("latarnik_zone")}</p>
                <p>{tr("latarnik_short_pitch")}</p>
                <button type="button" class="text-button" on:click=open_latarnik>{tr("open_latarnik")}</button>
            </div>
        </section>
    }
}

include!("fan/shell.rs");
include!("fan/merch.rs");
include!("fan/events.rs");
include!("fan/wallet.rs");
