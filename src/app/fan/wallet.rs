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
    // Inline status for form actions — the fan sees the result on the form,
    // not a global toast. Success still uses the shared error signal for the
    // positive toast.
    let import_status = RwSignal::new(None::<String>);
    let claim_status = RwSignal::new(None::<String>);

    let import = move |_| {
        let order = order_id.get().trim().to_owned();
        let token = checkout_token.get().trim().to_owned();
        if order.is_empty() || token.is_empty() {
            import_status.set(Some(tr("enter_the_order_id_and_private_token").to_owned()));
            return;
        }
        import_status.set(None);
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
                Err(message) => import_status.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let claim = move |_| {
        let token = claim_token.get().trim().to_owned();
        if token.is_empty() {
            claim_status.set(Some(tr("paste_the_admission_pass_token").to_owned()));
            return;
        }
        claim_status.set(None);
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
                Err(message) => claim_status.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let qr = move |_| {
        busy.set(true);
        spawn_local(async move {
            // Silent: the button re-enables and the fan can tap again.
            if let Ok(value) = bridge::invoke::<AdmissionQr, _>("fan_admission_qr", &EmptyArgs {}).await {
                admission_qr.set(Some(value));
            }
            busy.set(false);
        });
    };

    view! {
        <section class="screen"><header class="screen-title"><p class="eyebrow">{tr("mobile_wallet")}</p><h2>{tr("tickets_and_entry")}</h2></header><Show when=move || !loading.get().admission_pass fallback=move || view! { <Skeleton rows=1 height=140 /> }>{move || dashboard.with(|state| state.as_ref().and_then(|d| d.admission_pass.clone())).map(|pass| view! { <article class="admission-card"><p class="eyebrow">{tr("virya_admission_pass")}</p><h3>{pass.event_title}</h3><p>{event_time_location(&pass.starts_at, pass.venue.as_deref())}</p><strong>{pass.public_reference}</strong><span>{pass.status}</span><button class="primary" on:click=qr disabled=move || busy.get()>{tr("show_entry_qr")}</button>{move || admission_qr.get().map(|value| view! { <QrPanel svg=value.qr_svg token=value.token expires=value.expires_at /> })}</article> })}
        <Show when=move || dashboard.with(|state| state.as_ref().is_none_or(|d| d.admission_pass.is_none()))><div class="claim-box"><p class="eyebrow">{tr("did_you_win_an_admission_pass")}</p><h3>{tr("assign_it_to_your_phone")}</h3><p class="field-hint">{tr("claim_pass_hint")}</p><textarea rows="3" placeholder=tr("token_from_the_message") prop:value=move || claim_token.get() on:input=move |e| claim_token.set(event_target_value(&e))></textarea><button class="primary" on:click=claim disabled=move || busy.get()>{tr("claim_admission_pass")}</button>{move || claim_status.get().map(|msg| view! { <p class="inline-form-error">{error_message(&msg).to_owned()}</p> })}</div></Show></Show>
        <div class="section-head"><h3>{tr("ticket_wallet")}</h3><span>{move || wallets.get().len()}</span></div><Show when=move || !loading.get().wallets fallback=move || view! { <Skeleton rows=2 height=110 /> }><div class="wallet-stack"><For each=move || wallets.get() key=|wallet| wallet.order.order_id.clone() let:wallet>
            <WalletCard wallet=wallet error=error />
        </For></div></Show><details class="import-box">
            <summary>
                <span aria-hidden="true">"⤓"</span>
                <strong>{tr("add_an_existing_order")}</strong>
                <small>{tr("import_summary_hint")}</small>
            </summary>
            <div class="form-grid">
                <p class="field-hint">{tr("import_where_to_find")}</p>
                <label>"Order ID"<input placeholder=tr("order_uuid") prop:value=move || order_id.get() on:input=move |e| order_id.set(event_target_value(&e))/><small class="field-hint">{tr("import_order_id_hint")}</small></label>
                <label>{tr("private_checkout_token")}<textarea rows="3" autocomplete="off" autocapitalize="none" spellcheck="false" prop:value=move || checkout_token.get() on:input=move |e| checkout_token.set(event_target_value(&e))></textarea><small class="field-hint">{tr("import_token_hint")}</small></label>
                <button class="primary" on:click=import disabled=move || busy.get()>{tr("add_to_wallet")}</button>
                {move || import_status.get().map(|msg| view! { <p class="inline-form-error">{error_message(&msg).to_owned()}</p> })}
            </div>
        </details></section>
    }
}

#[component]
fn WalletCard(wallet: TicketWallet, error: RwSignal<Option<String>>) -> impl IntoView {
    let cached = wallet.cached;
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
            // Silent: the button re-enables and the fan can tap again.
            if let Ok(()) = bridge::invoke_unit("fan_request_delivery", &OrderArgs { order_id: &order }).await
            {
                error.set(Some(tr("we_resent_the_wallet_by_email").to_owned()));
            }
            busy.set(false);
        });
    };
    view! {
        <article class="wallet-card" class:cached=cached><header><div><p class="eyebrow">{order_status_label(&wallet.order.status)}</p><h3>{wallet.order.event_title}</h3><p>{event_time_location(&wallet.order.starts_at, wallet.order.venue.as_deref())}</p><Show when=move || cached><span class="cache-badge">{tr("wallet_cached_offline")}</span></Show></div><strong>{wallet.order.public_reference}</strong></header><div class="ticket-stack">{wallet.tickets.into_iter().map(|ticket| view! { <WalletTicketCard order_id=order_id.clone() ticket=ticket error=error /> }).collect_view()}</div><button class="text-button" on:click=resend disabled=move || busy.get()>{move || if busy.get() { tr("sending") } else { tr("resend_tickets_by_email") }}</button></article>
    }
}

/// The order status arrives as the raw CrowdRelay enum and was printed
/// straight into the card, so a Polish wallet was headed "PAID". The eight
/// values are the `ticket_orders.status` CHECK constraint; anything outside it
/// still shows through rather than being hidden behind a wrong label.
fn order_status_label(status: &str) -> String {
    match status {
        "reserved" => tr("order_status_reserved").to_owned(),
        "checkout_created" => tr("order_status_checkout_created").to_owned(),
        "paid" => tr("order_status_paid").to_owned(),
        "partially_refunded" => tr("order_status_partially_refunded").to_owned(),
        "refunded" => tr("order_status_refunded").to_owned(),
        "expired" => tr("order_status_expired").to_owned(),
        "cancelled" => tr("order_status_cancelled").to_owned(),
        "payment_failed" => tr("order_status_payment_failed").to_owned(),
        other => other.to_owned(),
    }
}

fn wallet_ticket_state(ticket: &WalletTicket) -> String {
    match ticket.status.as_str() {
        "redeemed" => ticket.redeemed_at.as_deref().map_or_else(
            || tr("wallet_ticket_used").to_owned(),
            |redeemed_at| {
                i18n::format(
                    "wallet_ticket_used_at",
                    &[human_time(redeemed_at).to_string()],
                )
            },
        ),
        "revoked" => tr("wallet_ticket_revoked").to_owned(),
        "expired" => tr("wallet_ticket_expired").to_owned(),
        "issued" => tr("wallet_ticket_not_claimed").to_owned(),
        "claimed" if ticket.qr_available => tr("wallet_ticket_ready").to_owned(),
        _ if ticket.qr_available => tr("wallet_ticket_ready").to_owned(),
        _ => tr("qr_unavailable").to_owned(),
    }
}

#[component]
fn WalletTicketCard(
    order_id: String,
    ticket: WalletTicket,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let _ = error;
    let public_reference = ticket.public_reference.clone();
    let qr_available = ticket.qr_available;
    let ticket_state = wallet_ticket_state(&ticket);
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
            // Silent: the button re-enables and the fan can tap again.
            if let Ok(svg) = bridge::invoke::<String, _>(
                "render_wallet_qr",
                &WalletQrArgs {
                    order_id: &order_id,
                    public_reference: &public_reference,
                },
            )
            .await
            {
                qr_svg.set(Some(svg));
                qr_visible.set(true);
            }
            busy.set(false);
        });
    };
    view! {
        <article class="ticket-card"><div><p class="eyebrow">{ticket.ticket_type_name}</p><strong>{ticket.public_reference}</strong><span>{ticket.holder_name.value_or(ticket.holder_email_masked)}</span></div><button class="ticket-qr-button" on:click=toggle_qr disabled=move || busy.get() || !qr_available>{move || if busy.get() { tr("generating") } else if qr_visible.get() { tr("hide_qr") } else if qr_available { tr("show_qr") } else { tr("qr_unavailable") }}</button><Show when=move || qr_visible.get()>{move || qr_svg.get().map(|svg| view! { <div class="mini-qr" inner_html=svg></div> })}</Show><small>{ticket_state}</small><Show when=move || qr_available><small>{i18n::format("qr_valid_until", &[human_time(&ticket.qr_expires_at).to_string()])}</small></Show></article>
    }
}

#[component]
fn QrPanel(svg: Option<String>, token: String, expires: String) -> impl IntoView {
    view! { <div class="qr-panel">{svg.map(|markup| view! { <div class="qr-svg" inner_html=markup></div> })}<code>{token}</code><small>{i18n::format("valid_until_2", &[human_time(&expires).to_string()])}</small></div> }
}

/// First letter for the profile avatar. The card used to print a hardcoded "V"
/// for every fan, so two accounts on one device looked identical.
fn profile_initial(display_name: &str, email: &str) -> String {
    display_name
        .chars()
        .chain(email.chars())
        .find(|character| character.is_alphanumeric())
        .map(|character| character.to_uppercase().to_string())
        .value_or_else(|| "V".to_owned())
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
        // Optimistic on purpose: the vault lock has no remote leg, so drop the
        // session material here and reconcile with native state when it replies.
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
        spawn_local(async move {
            // Silent: the optimistic UI already locked the session.
            // If the native side disagrees, it reconciles on next status.
            if let Ok(value) = bridge::invoke::<FanSessionStatus, _>("fan_lock", &EmptyArgs {}).await {
                let _ = status.try_set(value);
            }
        })
    };
    let forget = move |_| {
        spawn_local(async move {
            // Silent: the fan can tap again. The button stays enabled.
            if let Ok(value) = bridge::invoke::<FanSessionStatus, _>("fan_forget", &EmptyArgs {}).await {
                dashboard.set(None);
                wallets.set(Vec::new());
                area.set(None);
                loading.set(FanLoadingState::all());
                status.set(value);
            }
        })
    };
    let delete_confirming = RwSignal::new(false);
    let delete_account = move |_| {
        if !delete_confirming.get_untracked() {
            delete_confirming.set(true);
            return;
        }
        spawn_local(async move {
            // Silent: the confirmation dialog stays open and the fan
            // can try again. A toast here would be alarming.
            if let Ok(value) = bridge::invoke::<FanSessionStatus, _>("fan_delete_account", &EmptyArgs {}).await {
                dashboard.set(None);
                wallets.set(Vec::new());
                area.set(None);
                loading.set(FanLoadingState::all());
                delete_confirming.set(false);
                status.set(value);
            }
        })
    };
    let cancel_delete = move |_| delete_confirming.set(false);
    view! {
        <section class="screen fan-profile-screen">
            <header class="screen-title"><p class="eyebrow">{tr("my_profile")}</p><h2>{tr("signal_settings")}</h2></header>
            <Show when=move || status.get().session.is_some() fallback=move || view! { <Skeleton rows=1 height=170 /> }>
            {move || status.get().session.map(|profile| {
                let display_name = profile.display_name.clone().value_or_else(|| tr("virya_fan").to_owned());
                let initial = profile_initial(&display_name, &profile.email);
                let has_pass = profile.has_admission_pass;
                view! {
                    // The old header was an avatar, a name and an email over a
                    // flat card, and the three metrics under it repeated
                    // numbers the fan had already read on Signal. The hero now
                    // answers the question this screen is opened with — is my
                    // Signal working, and what do I hold — before the settings.
                    <article class="profile-hero">
                        <div class="avatar" aria-hidden="true">{initial}</div>
                        <div class="profile-hero-copy">
                            <strong>{display_name}</strong>
                            <p>{profile.email}</p>
                            <div class="profile-hero-badges">
                                <span class="cache-badge is-live">{tr("profile_signal_active")}</span>
                                {has_pass.then(|| view! {
                                    <span class="cache-badge">{tr("profile_admission_ready")}</span>
                                })}
                            </div>
                        </div>
                    </article>
                    <div class="stats-grid four">
                        <Metric value=profile.wallet_count.to_string() label=tr("orders")/>
                        <Metric value=if has_pass { "1".to_owned() } else { "0".to_owned() } label=tr("admission_passes")/>
                        <Metric value=dashboard.with(|state| state.as_ref().map(|d| d.referral.qualified_referrals.to_string())).value_or_else(|| "—".to_owned()) label=tr("referrals")/>
                        <Metric value=area.with(|state| state.as_ref().map(|wallet| wallet.claims.len().to_string())).value_or_else(|| "—".to_owned()) label=tr("counts_claims")/>
                    </div>
                }
            })}
            </Show>
            <p class="settings-group-label">{tr("settings_group_app")}</p>
            <div class="settings-list">
                <NativePushControl error=error />
                <LanguageSwitch />
                <DeviceUnlockSetting status=status error=error />
                <FanLocationSetting error=error />
                // AREA lives in the overflow menu, which is the one place a fan
                // does not look. The side game gets a row here so it is
                // reachable without knowing the hamburger holds it.
                <article class="settings-row settings-row-static">
                    <span class="settings-row-icon" aria-hidden="true">"◇"</span>
                    <strong>{tr("area_game_tab")}</strong>
                    <small>{tr("settings_area_hint")}</small>
                    <button class="ticket-buy-button" type="button" on:click=move |_| open_area_game(error)>
                        {tr("settings_open_area")}
                    </button>
                </article>
                <article class="settings-row settings-row-static">
                    <span class="settings-row-icon" aria-hidden="true">"⤴"</span>
                    <strong>"Virya.music"</strong>
                    <small>{tr("settings_site_hint")}</small>
                    <ExternalLink url="https://virya.music/?source=signal-app-settings".to_owned() label=tr("settings_open_site") error=error />
                </article>
                <button
                    class="settings-row"
                    on:click=refresh
                    disabled=move || { let state = loading.get(); state.events || state.referral || state.interests || state.admission_pass || state.wallets }
                >
                    <span class="settings-row-icon" aria-hidden="true">"⟳"</span>
                    <strong>{move || { let state = loading.get(); if state.events || state.referral || state.interests || state.admission_pass || state.wallets { tr("refreshing_2") } else { tr("refresh_data") } }}</strong>
                    <small>{tr("settings_refresh_hint")}</small>
                    <span class="settings-row-chevron" aria-hidden="true">"›"</span>
                </button>
                <button class="settings-row" on:click=lock>
                    <span class="settings-row-icon" aria-hidden="true">"◫"</span>
                    <strong>{tr("lock_app")}</strong>
                    <small>{tr("settings_lock_hint")}</small>
                    <span class="settings-row-chevron" aria-hidden="true">"›"</span>
                </button>
            </div>
            <p class="settings-group-label">{tr("settings_group_account")}</p>
            // Both rows are one tap from wiping a wallet, and they used to sit
            // open at the bottom of a screen a fan scrolls to change the
            // language. Folded away, they cost a deliberate open first.
            <details class="settings-danger-zone">
                <summary>
                    <span class="settings-row-icon" aria-hidden="true">"⚠"</span>
                    <strong>{tr("profile_danger_zone")}</strong>
                    <small>{tr("profile_danger_zone_hint")}</small>
                </summary>
                <div class="settings-list">
                    <button class="settings-row danger ghost" on:click=forget>
                        <span class="settings-row-icon" aria-hidden="true">"▨"</span>
                        <strong>{tr("remove_profile_and_tickets_from_device")}</strong>
                        <small>{tr("settings_forget_hint")}</small>
                        <span class="settings-row-chevron" aria-hidden="true">"›"</span>
                    </button>
                    <button class="settings-row danger ghost" on:click=delete_account>
                        <span class="settings-row-icon" aria-hidden="true">"✕"</span>
                        <strong>{tr("delete_virya_account")}</strong>
                        <small>{tr("settings_delete_hint")}</small>
                        <span class="settings-row-chevron" aria-hidden="true">"›"</span>
                    </button>
                    <Show when=move || delete_confirming.get()>
                        <div class="security-note warning">
                            <p>{tr("delete_account_warning")}</p>
                            <div class="button-row">
                                <button class="danger" on:click=delete_account>{tr("confirm_delete_account")}</button>
                                <button class="ghost" on:click=cancel_delete>{tr("cancel_delete_account")}</button>
                            </div>
                        </div>
                    </Show>
                </div>
            </details>
            <AnonymousFeedback error=error />
            <p class="security-note">{tr("fan_session_admission_pass_and_private_wallet")}</p>
        </section>
    }
}

/// Turns entry-without-a-PIN on and off.
///
/// On is the phone's hardware key holding the vault password; off is Argon2
/// over a PIN the fan types. Turning it off is a re-key rather than a flag, so
/// it asks for the PIN it is about to key the vault to — which is also why the
/// field is only shown once the fan has said they want the change.
#[component]
fn DeviceUnlockSetting(
    status: RwSignal<FanSessionStatus>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let busy = RwSignal::new(false);
    let disabling = RwSignal::new(false);
    let pin = RwSignal::new(String::new());
    let inline_status = RwSignal::new(None::<String>);

    let enable = move |_| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        inline_status.set(None);
        spawn_local(async move {
            match bridge::invoke::<FanSessionStatus, _>("fan_enable_device_unlock", &EmptyArgs {})
                .await
            {
                Ok(value) => {
                    let _ = status.try_set(value);
                    error.set(Some(tr("device_unlock_is_on").to_owned()));
                }
                Err(message) => inline_status.set(Some(message)),
            }
            let _ = busy.try_set(false);
        });
    };

    let disable = move |_| {
        if busy.get_untracked() {
            return;
        }
        let current_pin = pin.get_untracked();
        if !new_operator_pin_is_valid(&current_pin) {
            inline_status.set(Some(tr("enter_4_6_digits_for_this_fan_profile").to_owned()));
            return;
        }
        busy.set(true);
        inline_status.set(None);
        spawn_local(async move {
            match bridge::invoke::<FanSessionStatus, _>(
                "fan_disable_device_unlock",
                &PinArgs { pin: &current_pin },
            )
            .await
            {
                Ok(value) => {
                    let _ = pin.try_set(String::new());
                    let _ = disabling.try_set(false);
                    let _ = status.try_set(value);
                    error.set(Some(tr("device_unlock_is_off").to_owned()));
                }
                Err(message) => inline_status.set(Some(message)),
            }
            let _ = busy.try_set(false);
        });
    };

    view! {
        <Show when=move || status.get().device_unlock_supported>
            <article class="settings-row settings-row-static">
                <span class="settings-row-icon" aria-hidden="true">"⚿"</span>
                <strong>{tr("device_unlock_row_title")}</strong>
                <small>{tr("device_unlock_row_hint")}</small>
                <p class="settings-row-value">
                    {move || if status.get().device_unlock {
                        tr("device_unlock_is_on")
                    } else {
                        tr("device_unlock_is_off")
                    }}
                </p>
                <Show when=move || status.get().device_unlock fallback=move || view! {
                    <button class="ticket-buy-button" type="button" disabled=move || busy.get() on:click=enable>
                        {tr("device_unlock_turn_on")}
                    </button>
                }>
                    <Show when=move || disabling.get() fallback=move || view! {
                        <button class="ghost" type="button" disabled=move || busy.get() on:click=move |_| disabling.set(true)>
                            {tr("device_unlock_turn_off")}
                        </button>
                    }>
                        <label class="pin-field">
                            <span class="pin-field-label">{tr("create_fan_unlock_pin")}</span>
                            <small id="fan-device-unlock-pin-help">{tr("enter_4_6_digits_for_this_fan_profile")}</small>
                            <input aria-label=tr("create_fan_unlock_pin") type="password" autocomplete="new-password" inputmode="numeric" pattern="[0-9]*" maxlength="6" placeholder=tr("pin_example") aria-describedby="fan-device-unlock-pin-help" prop:value=move || pin.get() on:input=move |e| pin.set(normalize_new_operator_pin(event_target_value(&e)))/>
                        </label>
                        <button class="primary" type="button" disabled=move || busy.get() on:click=disable>
                            {tr("device_unlock_turn_off")}
                        </button>
                    </Show>
                </Show>
                {move || inline_status.get().map(|msg| view! { <p class="inline-form-error">{error_message(&msg).to_owned()}</p> })}
            </article>
        </Show>
    }
}

/// Lets a fan set their city and nearby-show preference after signup.
///
/// Signup was the only place this could ever be set, and the server refuses to
/// change it from an unauthenticated repeat submission -- so a fan who bought a
/// ticket, or one who moved, had no way to establish a location at all, and
/// nearby delivery is keyed on one. The reply carries the server's own verdict
/// on whether targeting can work: a city nobody has put on the map yet cannot
/// reach anybody, and saying "saved" without saying that is how a fan ends up
/// waiting for shows that were never coming.
#[component]
fn FanLocationSetting(error: RwSignal<Option<String>>) -> impl IntoView {
    let city = RwSignal::new(String::new());
    let region = RwSignal::new(String::new());
    let nearby = RwSignal::new(true);
    let radius = RwSignal::new(150_u16);
    let busy = RwSignal::new(false);
    let stored = RwSignal::new(None::<FanLocationState>);
    let inline_status = RwSignal::new(None::<String>);

    let save = move |_| {
        if busy.get_untracked() {
            return;
        }
        let name = city.get_untracked().trim().to_owned();
        if name.is_empty() {
            inline_status.set(Some(tr("location_city_required").to_owned()));
            return;
        }
        let requested = RequestedCityInput {
            name,
            region: optional(region.get_untracked().trim().to_owned()),
            country_code: "PL".to_owned(),
        };
        let enabled = nearby.get_untracked();
        let km = radius.get_untracked();
        inline_status.set(None);
        busy.set(true);
        spawn_local(async move {
            // Same two steps signup takes: name the city, then attach to the
            // slug it resolves to. A city nobody has approved yet still
            // resolves, which is exactly why the second call reports whether
            // it can be targeted.
            let slug = match bridge::invoke::<RequestedCityResult, _>(
                "request_city",
                &RequestedCityArgs { input: &requested },
            )
            .await
            {
                Ok(value) => value.city_slug,
                Err(message) => {
                    inline_status.set(Some(message));
                    busy.set(false);
                    return;
                }
            };
            match bridge::invoke::<FanLocationState, _>(
                "fan_set_location",
                &FanLocationArgs {
                    city_slug: &slug,
                    nearby_gigs_enabled: enabled,
                    radius_km: km,
                },
            )
            .await
            {
                Ok(value) => {
                    error.set(Some(tr("location_saved").to_owned()));
                    stored.set(Some(value));
                }
                Err(message) => inline_status.set(Some(message)),
            }
            busy.set(false);
        });
    };

    view! {
        <article class="settings-row settings-row-static location-setting">
            <span class="settings-row-icon" aria-hidden="true">"⌖"</span>
            <strong>{tr("location_heading")}</strong>
            <small>{tr("location_hint")}</small>
            <div class="form-grid location-fields">
                <label>{tr("city")}<input placeholder=tr("e_g_bielawa") prop:value=move || city.get() on:input=move |e| city.set(event_target_value(&e))/></label>
                <label>{tr("province_region_optional")}<input placeholder=tr("lower_silesia") prop:value=move || region.get() on:input=move |e| region.set(event_target_value(&e))/></label>
                <label class="pref-row pref-row-inline"><span class="pref-row-label">{tr("notify_me_about_nearby_shows")}</span><input type="checkbox" class="pref-switch" prop:checked=move || nearby.get() on:change=move |e| nearby.set(event_target_checked(&e))/></label>
                <Show when=move || nearby.get()>
                    <div class="radius-picker">
                        <button type="button" class:active=move || radius.get()==50 on:click=move |_| radius.set(50)>"50 km"</button>
                        <button type="button" class:active=move || radius.get()==100 on:click=move |_| radius.set(100)>"100 km"</button>
                        <button type="button" class:active=move || radius.get()==150 on:click=move |_| radius.set(150)>"150 km"</button>
                        <button type="button" class:active=move || radius.get()==250 on:click=move |_| radius.set(250)>"250 km"</button>
                    </div>
                </Show>
                <button type="button" class="primary" disabled=move || busy.get() on:click=save>
                    {move || if busy.get() { tr("sending_2") } else { tr("location_save") }}
                </button>
            </div>
            {move || stored.get().map(|value| {
                // The distinction the old UI could not draw: stored is not the
                // same as reachable.
                let (class, message) = if !value.nearby_gigs_enabled {
                    ("security-note", tr("location_muted").to_owned())
                } else if value.targeting_ready {
                    ("security-note", i18n::format("location_active", std::slice::from_ref(&value.city_name)))
                } else {
                    ("security-note warning", i18n::format("location_not_targetable", std::slice::from_ref(&value.city_name)))
                };
                view! { <p class=class>{message}</p> }
            })}
            {move || inline_status.get().map(|msg| view! { <p class="inline-form-error">{error_message(&msg).to_owned()}</p> })}
        </article>
    }
}

#[component]
fn AnonymousFeedback(error: RwSignal<Option<String>>) -> impl IntoView {
    let category = RwSignal::new("idea".to_owned());
    let message = RwSignal::new(String::new());
    let busy = RwSignal::new(false);
    let inline_status = RwSignal::new(None::<String>);

    let submit = move |_| {
        if busy.get_untracked() {
            return;
        }
        let current_category = category.get_untracked();
        let current_message = message.get_untracked().trim().to_owned();
        let length = current_message.chars().count();
        if !(8..=2_000).contains(&length) {
            inline_status.set(Some(
                tr("feedback_must_contain_between_8_and_2000").to_owned(),
            ));
            return;
        }
        inline_status.set(None);
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<String, _>(
                "submit_anonymous_feedback",
                &AnonymousFeedbackArgs {
                    category: &current_category,
                    message: &current_message,
                },
            )
            .await
            {
                // "queued" means the outbox holds it and a later launch will
                // try again. Saying "sent" there would be the app answering a
                // question only the server can answer.
                Ok(outcome) => {
                    message.set(String::new());
                    error.set(Some(
                        if outcome == "queued" {
                            tr("feedback_queued_until_online")
                        } else {
                            tr("feedback_was_sent_anonymously_thank_you")
                        }
                        .to_owned(),
                    ));
                }
                Err(message) => inline_status.set(Some(message)),
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
            {move || inline_status.get().map(|msg| view! { <p class="inline-form-error">{error_message(&msg).to_owned()}</p> })}
        </section>
    }
}
