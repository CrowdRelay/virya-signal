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
        <Show when=move || dashboard.with(|state| state.as_ref().is_none_or(|d| d.admission_pass.is_none()))><div class="claim-box"><p class="eyebrow">{tr("did_you_win_an_admission_pass")}</p><h3>{tr("assign_it_to_your_phone")}</h3><p class="field-hint">{tr("claim_pass_hint")}</p><textarea rows="3" placeholder=tr("token_from_the_message") prop:value=move || claim_token.get() on:input=move |e| claim_token.set(event_target_value(&e))></textarea><button class="primary" on:click=claim disabled=move || busy.get()>{tr("claim_admission_pass")}</button>{move || claim_status.get().map(|msg| view! { <p class="inline-form-error">{msg}</p> })}</div></Show></Show>
        <div class="section-head"><h3>{tr("ticket_wallet")}</h3><span>{move || wallets.get().len()}</span></div><Show when=move || !loading.get().wallets fallback=move || view! { <Skeleton rows=2 height=110 /> }><div class="wallet-stack">{move || wallets.get().into_iter().map(|wallet| view! {
            <WalletCard wallet=wallet error=error />
        }).collect_view()}</div></Show><details class="import-box">
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
                {move || import_status.get().map(|msg| view! { <p class="inline-form-error">{msg}</p> })}
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
            configured: status.get_untracked().configured,
            unlocked: false,
            session: None,
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
        <section class="screen">
            <header class="screen-title"><p class="eyebrow">{tr("my_profile")}</p><h2>{tr("signal_settings")}</h2></header>
            <Show when=move || status.get().session.is_some() fallback=move || view! { <Skeleton rows=1 height=96 /> }>
            {move || status.get().session.map(|profile| view! {
                <div class="profile-card"><div class="avatar">"V"</div><div><strong>{profile.display_name.value_or_else(|| tr("virya_fan").to_owned())}</strong><p>{profile.email}</p></div></div>
                <div class="stats-grid"><Metric value=profile.wallet_count.to_string() label=tr("orders")/><Metric value=if profile.has_admission_pass { "1".to_owned() } else { "0".to_owned() } label=tr("admission_passes")/><Metric value=dashboard.with(|state| state.as_ref().map(|d| d.referral.qualified_referrals.to_string())).value_or_else(|| "—".to_owned()) label=tr("referrals")/></div>
            })}
            </Show>
            <p class="settings-group-label">{tr("settings_group_app")}</p>
            <div class="settings-list">
                <NativePushControl error=error />
                <LanguageSwitch />
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
            {move || inline_status.get().map(|msg| view! { <p class="inline-form-error">{msg}</p> })}
        </section>
    }
}
