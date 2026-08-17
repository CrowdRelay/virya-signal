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
            match bridge::invoke_unit("fan_request_delivery", &OrderArgs { order_id: &order }).await
            {
                Ok(_) => error.set(Some(tr("we_resent_the_wallet_by_email").to_owned())),
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };
    view! {
        <article class="wallet-card" class:cached=cached><header><div><p class="eyebrow">{wallet.order.status}</p><h3>{wallet.order.event_title}</h3><p>{event_time_location(&wallet.order.starts_at, wallet.order.venue.as_deref())}</p><Show when=move || cached><span class="cache-badge">{tr("wallet_cached_offline")}</span></Show></div><strong>{wallet.order.public_reference}</strong></header><div class="ticket-stack">{wallet.tickets.into_iter().map(|ticket| view! { <WalletTicketCard order_id=order_id.clone() ticket=ticket error=error /> }).collect_view()}</div><button class="text-button" on:click=resend disabled=move || busy.get()>{move || if busy.get() { tr("sending") } else { tr("resend_tickets_by_email") }}</button></article>
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
                <NativePushControl error=error />
                <LanguageSwitch />
                <ExternalLink url="https://virya.music/?source=signal-app-settings".to_owned() label="Virya.music" error=error />
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
