#[component]
fn BackButton(mode: RwSignal<RootMode>) -> impl IntoView {
    view! { <button class="back-button" on:click=move |_| mode.set(RootMode::Fan)>{tr("back_signal")}</button> }
}

#[component]
fn StaffEntryButton(mode: RwSignal<RootMode>) -> impl IntoView {
    view! {
        <button
            type="button"
            class="staff-entry-button"
            on:click=move |_| mode.set(RootMode::StaffGate)
        >
            <span aria-hidden="true">"⌁"</span>
            {tr("are_you_on_the_staff")}
        </button>
    }
}

#[component]
fn StaffGate(mode: RwSignal<RootMode>, error: RwSignal<Option<String>>) -> impl IntoView {
    let password = RwSignal::new(String::new());
    let busy = RwSignal::new(false);

    let submit = move |_| {
        if busy.get_untracked() {
            return;
        }
        let current_password = password.get_untracked();
        if current_password.is_empty() {
            error.set(Some(tr("enter_the_staff_password_used_in_the").to_owned()));
            return;
        }
        busy.set(true);
        spawn_local(async move {
            let result = bridge::invoke_unit(
                "verify_staff_access",
                &StaffGateArgs {
                    password: &current_password,
                },
            )
            .await;
            password.set(String::new());
            busy.set(false);
            match result {
                Ok(()) => mode.set(RootMode::Team),
                Err(message) => error.set(Some(message)),
            }
        });
    };

    view! {
        <section class="access-screen staff-gate-screen">
            <button
                class="back-button"
                disabled=move || busy.get()
                on:click=move |_| mode.set(RootMode::Fan)
            >
                {tr("back_signal")}
            </button>
            <header class="hero compact staff-gate-hero">
                <p class="eyebrow">{tr("virya_staff_eyebrow")}</p>
                <h1>{tr("zone_prefix")}<em>{tr("team_zone_suffix")}</em></h1>
                <p>{tr("gate_sales_and_show_operations_access_is")}</p>
            </header>
            <div class="access-card staff-gate-card">
                <div class="staff-gate-lock" aria-hidden="true">"⌁"</div>
                <div>
                    <p class="eyebrow">{tr("staff_verification")}</p>
                    <h2>{tr("virya_panel_password")}</h2>
                    <p>{tr("use_the_same_password_as_in_qr")}</p>
                </div>
                <label>
                    {tr("staff_password")}
                    <input
                        type="password"
                        autocomplete="current-password"
                        maxlength="256"
                        prop:value=move || password.get()
                        on:input=move |event| password.set(event_target_value(&event))
                    />
                </label>
                <button
                    class="primary"
                    disabled=move || busy.get() || password.get().is_empty()
                    on:click=submit
                >
                    {move || if busy.get() { tr("checking") } else { tr("open_staff_zone") }}
                </button>
                <p class="staff-gate-note">{tr("password_is_verified_by_virya_music_and")}</p>
            </div>
        </section>
    }
}

#[component]
fn OperatorPortal(
    mode: RwSignal<RootMode>,
    status: RwSignal<SessionStatus>,
    dashboard: RwSignal<Option<DashboardData>>,
    tab: RwSignal<OperatorTab>,
    status_loading: RwSignal<bool>,
    status_failed: RwSignal<bool>,
    status_refresh: RwSignal<u32>,
    push_target: RwSignal<Option<String>>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    view! {
        {move || if status.get().unlocked {
            // Auth is the only blocking step. Once the native session is
            // confirmed, render the operator shell immediately; individual
            // sections can continue loading behind their own skeletons.
            view! { <OperatorApp mode=mode status=status dashboard=dashboard tab=tab push_target=push_target error=error /> }.into_any()
        } else if status_failed.get() {
            view! { <StatusFailure mode=mode status_refresh=status_refresh label=tr("failed_to_read_the_staff_vault") show_back=true /> }.into_any()
        } else if status_loading.get() {
            view! { <AccessLoader mode=mode label=tr("checking_the_secure_vault") show_back=true /> }.into_any()
        } else {
            view! { <OperatorAccess mode=mode status=status error=error /> }.into_any()
        }}
    }
}

/// Adopts an authoritative unlocked staff session into the UI.
///
/// The `launcher:` scope is retired first. A `launcher_status` issued before
/// the unlock — by the root resume subscriber, for instance — carries the
/// answer for a locked device, and landing after this write would put the
/// portal straight back on the PIN screen.
fn adopt_operator_session(status: RwSignal<SessionStatus>, value: SessionStatus) {
    bridge::invalidate_latest("launcher:");
    status.set(value);
}

fn new_operator_pin_is_valid(pin: &str) -> bool {
    (4..=6).contains(&pin.len()) && pin.bytes().all(|byte| byte.is_ascii_digit())
}

fn normalize_new_operator_pin(value: String) -> String {
    value.chars().filter(char::is_ascii_digit).take(6).collect()
}

#[component]
fn OperatorAccess(
    mode: RwSignal<RootMode>,
    status: RwSignal<SessionStatus>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let pin = RwSignal::new(String::new());
    let pairing = RwSignal::new(String::new());
    let advanced = RwSignal::new(false);
    let name = RwSignal::new(tr("virya_staff").to_owned());
    let token = RwSignal::new(String::new());
    let api = RwSignal::new(API_BASE.to_owned());
    let role = RwSignal::new(OperatorRole::Staff);
    let busy = RwSignal::new(false);

    let unlock = move |_| {
        let current_pin = pin.get();
        if !new_operator_pin_is_valid(&current_pin) {
            error.set(Some(
                tr("pin_must_contain_at_least_4_characters").to_owned(),
            ));
            return;
        }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<SessionStatus, _>("unlock", &PinArgs { pin: &current_pin }).await
            {
                Ok(value) => {
                    pin.set(String::new());
                    adopt_operator_session(status, value);
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let pair = move |payload: String| {
        let current_pin = pin.get();
        if !new_operator_pin_is_valid(&current_pin) || payload.trim().is_empty() {
            error.set(Some(tr("enter_a_4_6_digit_pin_and").to_owned()));
            return;
        }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<SessionStatus, _>(
                "configure_from_pairing",
                &PairingArgs {
                    pin: &current_pin,
                    payload: &payload,
                },
            )
            .await
            {
                Ok(value) => {
                    pin.set(String::new());
                    pairing.set(String::new());
                    adopt_operator_session(status, value);
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let scan_pairing = move |_| {
        spawn_local(async move {
            match bridge::scan_qr().await {
                Ok(Some(value)) => {
                    pairing.set(value);
                    if new_operator_pin_is_valid(&pin.get()) {
                        pair(pairing.get());
                    }
                }
                Ok(None) => {}
                Err(message) => error.set(Some(message)),
            }
        });
    };
    let submit_pairing = move |_| pair(pairing.get());

    let configure_manual = move |_| {
        let current_pin = pin.get();
        let profile = OperatorProfileInput {
            display_name: name.get(),
            api_base_url: api.get(),
            role: role.get(),
            bearer_token: token.get(),
        };
        if !new_operator_pin_is_valid(&current_pin) || profile.bearer_token.trim().len() < 24 {
            error.set(Some(tr("enter_a_4_6_digit_pin_and_2").to_owned()));
            return;
        }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<SessionStatus, _>(
                "configure",
                &ConfigureArgs {
                    pin: &current_pin,
                    profile: &profile,
                },
            )
            .await
            {
                Ok(value) => {
                    pin.set(String::new());
                    token.set(String::new());
                    adopt_operator_session(status, value);
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    view! {
        <section class="access-screen">
            <BackButton mode=mode />
            <header class="hero compact">
                <p class="eyebrow">{tr("virya_control")}</p>
                <h1>{tr("pair")}<em>{tr("device_label")}</em></h1>
                <p>{tr("no_retyping_the_api_role_or_long")}</p>
            </header>
            <div class="access-card">
                <Show when=move || status.get().configured fallback=move || view! {
                    <div class="pairing-flow">
                        <Show when=move || pairing.get().trim().is_empty() fallback=move || view! {
                            <div class="pairing-scanned">
                                <span class="pairing-ok">"✓"</span>
                                <strong>{tr("code_scanned")}</strong>
                                <small>{tr("enter_the_pin_below_and_tap_pair")}</small>
                            </div>
                        }>
                            <button class="pairing-scan primary" on:click=scan_pairing disabled=move || busy.get()>
                                <span class="pairing-qr">"▦"</span>
                                <strong>{move || if busy.get() { tr("connecting") } else { tr("scan_qr_code") }}</strong>
                                <small>{tr("code_shown_in_the_virya_panel")}</small>
                            </button>
                        </Show>
                        <div class="pairing-divider"><span></span><small>{tr("or_label")}</small><span></span></div>
                        <label>{tr("pairing_code")}<textarea rows="2" placeholder="virya-signal://pair?…" prop:value=move || pairing.get() on:input=move |e| pairing.set(event_target_value(&e))></textarea></label>
                        <label class="pin-field">
                            <span class="pin-field-label">{tr("create_an_unlock_pin")}</span>
                            <small id="operator-new-pin-help">{tr("enter_4_6_digits_for_example_2580")}</small>
                            <input aria-label=tr("create_an_unlock_pin") type="password" autocomplete="new-password" inputmode="numeric" pattern="[0-9]*" maxlength="6" placeholder=tr("pin_example") aria-describedby="operator-new-pin-help" prop:value=move || pin.get() on:input=move |e| pin.set(normalize_new_operator_pin(event_target_value(&e))) />
                        </label>
                        <button class="primary" on:click=submit_pairing disabled=move || busy.get() || pairing.get().trim().is_empty() || !new_operator_pin_is_valid(&pin.get())>{tr("pair_2")}</button>
                        <button class="text-button" type="button" on:click=move |_| advanced.update(|v| *v = !*v)>
                            {move || if advanced.get() { tr("hide_manual_settings") } else { tr("advanced_settings") }}
                        </button>
                        <Show when=move || advanced.get()>
                            <div class="advanced-config">
                                <label>{tr("device_person_name")}<input aria-label=tr("device_person_name") prop:value=move || name.get() on:input=move |e| name.set(event_target_value(&e)) /></label>
                                <label>"API CrowdRelay"<input aria-label="API CrowdRelay" prop:value=move || api.get() on:input=move |e| api.set(event_target_value(&e)) /></label>
                                <div class="segmented">
                                    <button class:active=move || role.get() == OperatorRole::Owner on:click=move |_| role.set(OperatorRole::Owner)>{tr("owner")}</button>
                                    <button class:active=move || role.get() == OperatorRole::Staff on:click=move |_| role.set(OperatorRole::Staff)>{tr("staff")}</button>
                                </div>
                                <label>{tr("device_token")}<textarea aria-label=tr("device_token") rows="3" prop:value=move || token.get() on:input=move |e| token.set(event_target_value(&e))></textarea></label>
                                <button class="primary" on:click=configure_manual disabled=move || busy.get() || !new_operator_pin_is_valid(&pin.get())>{tr("save_manually")}</button>
                            </div>
                        </Show>
                    </div>
                }>
                    <div class="form-grid">
                        <p class="lock-copy">{tr("operator_profile_is_encrypted_locally")}</p>
                        <label class="pin-field">
                            <span class="pin-field-label">{tr("app_unlock_pin")}</span>
                            <small id="operator-unlock-pin-help">{tr("enter_the_pin_created_when_this_device")}</small>
                            <input type="password" autocomplete="current-password" inputmode="numeric" placeholder=tr("your_pin") aria-describedby="operator-unlock-pin-help" prop:value=move || pin.get() on:input=move |e| pin.set(event_target_value(&e)) />
                        </label>
                        <button class="primary" disabled=move || busy.get() || pin.get().chars().count() < 4 on:click=unlock>{tr("unlock")}</button>
                    </div>
                </Show>
            </div>
        </section>
    }
}

include!("operator/shell.rs");
include!("operator/signal.rs");
include!("operator/commerce_settings.rs");
include!("operator/checklist.rs");

// operator_push_open_settings is surfaced from OperatorChecklist when the team
// push permission is denied; settings continuation re-syncs via operator_push_sync.
