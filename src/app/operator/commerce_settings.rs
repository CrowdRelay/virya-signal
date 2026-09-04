#[component]
fn Tickets(
    dashboard: RwSignal<Option<DashboardData>>,
    loading: RwSignal<OperatorLoadingState>,
    error: RwSignal<Option<String>>,
    owner: Signal<bool>,
) -> impl IntoView {
    let event_slug = RwSignal::new(String::new());
    let overview = RwSignal::new(None::<TicketingOverview>);
    let event_snapshot = RwSignal::new(None::<StaffEventDashboard>);
    let busy = RwSignal::new(false);
    let fan_email = RwSignal::new(String::new());
    let pool_slug = RwSignal::new("tickets".to_owned());
    let revoke_ref = RwSignal::new(String::new());
    let issued = RwSignal::new(None::<IssuedPass>);
    // Revoking a pass turns a real person away at the door. It used to be one
    // tap on a field that is easy to paste the wrong value into.
    let revoke_confirming = RwSignal::new(false);

    let load = move |_| {
        let slug = event_slug.get();
        if slug.is_empty() {
            error.set(Some(tr("select_a_show").to_owned()));
            return;
        }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<TicketingOverview, _>(
                "ticketing_overview",
                &EventArgs { event_slug: &slug },
            )
            .await
            {
                Ok(value) => overview.set(Some(value)),
                Err(message) => error.set(Some(message)),
            }
            match bridge::invoke::<StaffEventDashboard, _>(
                "staff_event_dashboard",
                &EventArgs { event_slug: &slug },
            )
            .await
            {
                Ok(value) if value.has_supported_schema() => event_snapshot.set(Some(value)),
                Ok(value) => error.set(Some(i18n::format(
                    "unsupported_staff_snapshot_version",
                    &[value.schema_version.to_string()],
                ))),
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let issue = move |_| {
        let input = IssuePassInput {
            event_slug: event_slug.get(),
            pool_slug: pool_slug.get().trim().to_owned(),
            fan_email: fan_email.get().trim().to_owned(),
            claim_expires_hours: 72,
        };
        if input.event_slug.is_empty() || input.fan_email.trim().is_empty() {
            error.set(Some(tr("select_a_show_and_enter_the_fan").to_owned()));
            return;
        }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<IssuedPass, _>("issue_pass", &IssueArgs { input: &input }).await
            {
                Ok(value) => issued.set(Some(value)),
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let revoke = move |_| {
        let reference = revoke_ref.get().trim().to_owned();
        if reference.is_empty() {
            error.set(Some(
                tr("enter_the_admission_pass_public_reference").to_owned(),
            ));
            return;
        }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke_unit(
                "revoke_pass",
                &ReferenceArgs {
                    public_reference: &reference,
                },
            )
            .await
            {
                Ok(_) => {
                    revoke_confirming.set(false);
                    revoke_ref.set(String::new());
                    error.set(Some(tr("admission_pass_has_been_revoked").to_owned()));
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    view! {
        <section class="screen">
            <header class="screen-title"><p class="eyebrow">{tr("ticketing_eyebrow")}</p><h2>{tr("tickets_and_admission_passes")}</h2></header>
            <div class="toolbar"><select disabled=move || loading.get().events prop:value=move || event_slug.get() on:change=move |e| event_slug.set(event_target_value(&e))><option value="">{move || if loading.get().events { tr("loading_shows") } else { tr("select_a_show_2") }}</option>{move || operator_events(dashboard).into_iter().map(|event| view! { <option value=event.slug.clone()>{event.title}</option> }).collect_view()}</select><button on:click=load disabled=move || busy.get() || loading.get().events>{tr("refresh")}</button></div>
            {move || event_snapshot.get().map(|snapshot| view! {
                <article class="event-snapshot-heading">
                    <p class="eyebrow">{snapshot.slug}</p>
                    <h3>{snapshot.title}</h3>
                    <p>{event_time_location(&snapshot.starts_at, snapshot.venue.as_deref())}</p>
                </article>
                <div class="stats-grid wide event-context-stats">
                    <Metric value=snapshot.interested_fans.to_string() label=tr("interested")/>
                    <Metric value=snapshot.paid_orders.to_string() label=tr("paid_orders")/>
                    <Metric value=snapshot.paid_tickets.to_string() label=tr("sold")/>
                    <Metric value=snapshot.passes_issued.to_string() label=tr("passes_issued")/>
                    <Metric value=snapshot.passes_claimed.to_string() label=tr("claimed")/>
                    <Metric value=snapshot.passes_redeemed.to_string() label=tr("redeemed")/>
                </div>
            })}
            {move || overview.get().map(|data| view! {
                <div class="stats-grid wide"><Metric value=data.paid_tickets.to_string() label=tr("sold")/><Metric value=data.sale.reserved.to_string() label=tr("in_checkout")/><Metric value=data.sale.available.to_string() label=tr("available_label")/></div>
                <div class="revenue-card"><p>{tr("gross_revenue")}</p><strong>{money(data.gross_sales_minor, &data.sale.currency)}</strong><span>{i18n::format("refunds", &[money(data.refunded_minor, &data.sale.currency)])}</span></div>
                <div class="section-head"><h3>{tr("recent_orders")}</h3><span>{data.recent_orders.len()}</span></div>
                <div class="card-list">{data.recent_orders.into_iter().map(|order| view! { <article class="order-card"><div><strong>{order.public_reference}</strong><p>{order.buyer_name.value_or(order.buyer_email_masked)}</p></div><span>{money(order.amount_gross_minor, &order.currency)}</span></article> }).collect_view()}</div>
            })}
            <Show when=move || !loading.get().events fallback=move || view! { <Skeleton rows=2 height=120 /> }>
                <Show when=move || event_slug.get().is_empty() && overview.get().is_none()>
                    <div class="empty-state">
                        <strong>{tr("ticketing_pick_show")}</strong>
                        <p>{tr("ticketing_pick_show_hint")}</p>
                    </div>
                </Show>
            </Show>
            <Show when=move || owner.get()>
                <div class="admin-box">
                    <p class="eyebrow">{tr("owner_only_eyebrow")}</p>
                    <h3>{tr("manual_admission_pass")}</h3>
                    <p class="inline-note">{tr("admission_pass_number_is_a_safe_public")}</p>
                    <div class="form-grid">
                        <label>{tr("pass_fan_email_label")}<input type="email" autocomplete="off" placeholder="fan@email.com" prop:value=move || fan_email.get() on:input=move |e| fan_email.set(event_target_value(&e))/><small class="field-hint">{tr("pass_fan_email_hint")}</small></label>
                        <label>{tr("pass_pool_label")}<input placeholder=tr("pool_slug_placeholder") prop:value=move || pool_slug.get() on:input=move |e| pool_slug.set(event_target_value(&e))/><small class="field-hint">{tr("pass_pool_hint")}</small></label>
                        <button class="primary" on:click=issue disabled=move || busy.get()>{tr("issue_pass")}</button>
                    </div>
                    {move || issued.get().map(|pass| view! {
                        <div class="token-box">
                            <p class="eyebrow">{tr("pass_issued_eyebrow")}</p>
                            <strong>{pass.public_reference}</strong>
                            <p>{tr("claim_token_label")}: {pass.claim_token}</p>
                            <small>{tr("pass_issued_hint")}</small>
                        </div>
                    })}
                    <hr/>
                    <h3>{tr("pass_revoke_heading")}</h3>
                    <div class="form-grid">
                        <label>{tr("pass_revoke_label")}<input placeholder=tr("admission_pass_number_e_g_vry") prop:value=move || revoke_ref.get() on:input=move |e| revoke_ref.set(event_target_value(&e))/><small class="field-hint">{tr("pass_revoke_hint")}</small></label>
                        <Show
                            when=move || revoke_confirming.get()
                            fallback=move || view! { <button class="danger ghost" on:click=move |_| revoke_confirming.set(true) disabled=move || busy.get() || revoke_ref.get().trim().is_empty()>{tr("revoke")}</button> }
                        >
                            <div class="security-note warning">
                                <p>{i18n::format("pass_revoke_confirm", std::slice::from_ref(&revoke_ref.get()))}</p>
                                <div class="button-row">
                                    <button class="danger" on:click=revoke disabled=move || busy.get()>{tr("pass_revoke_confirm_action")}</button>
                                    <button class="ghost" on:click=move |_| revoke_confirming.set(false)>{tr("cancel")}</button>
                                </div>
                            </div>
                        </Show>
                    </div>
                </div>
            </Show>
        </section>
    }
}

#[component]
fn Discounts(error: RwSignal<Option<String>>) -> impl IntoView {
    let code = RwSignal::new(String::new());
    let order = RwSignal::new(String::new());
    let result = RwSignal::new(None::<CouponEnvelope>);
    let busy = RwSignal::new(false);
    let redeem = move |_| {
        let c = code.get().trim().to_owned();
        let o = order.get().trim().to_owned();
        if c.is_empty() || o.is_empty() {
            error.set(Some(tr("enter_the_code_and_sale_number").to_owned()));
            return;
        }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<CouponEnvelope, _>(
                "redeem_coupon",
                &CouponArgs {
                    code: &c,
                    order_reference: &o,
                },
            )
            .await
            {
                Ok(value) => result.set(Some(value)),
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };
    view! {
        <section class="screen"><header class="screen-title"><p class="eyebrow">{tr("merch_desk_eyebrow")}</p><h2>{tr("redeem_a_discount")}</h2></header><div class="coupon-visual"><span>%</span><div><strong>{tr("virya_signal")}</strong><p>{tr("fan_coupon_controlled_use")}</p></div></div><div class="form-grid panel"><label>{tr("discount_code")}<input autocapitalize="characters" placeholder="VIRYA-…" prop:value=move || code.get() on:input=move |e| code.set(event_target_value(&e))/><small class="field-hint">{tr("discount_code_hint")}</small></label><label>{tr("sale_number")}<input placeholder="MERCH-WRO-001" prop:value=move || order.get() on:input=move |e| order.set(event_target_value(&e))/><small class="field-hint">{tr("sale_number_hint")}</small></label><button class="primary" on:click=redeem disabled=move || busy.get()>{tr("redeem_coupon")}</button></div>{move || result.get().map(|envelope| view! { <article class="scan-result scan-success"><strong>{tr("coupon_redeemed")}</strong><span>{envelope.result.status}</span><p>{i18n::format("usage", &[envelope.result.used_count.to_string(), envelope.result.max_uses.to_string()])}</p></article> })}</section>
    }
}

#[component]
fn Campaigns(
    dashboard: RwSignal<Option<DashboardData>>,
    loading: RwSignal<OperatorLoadingState>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let event_slug = RwSignal::new(String::new());
    let label = RwSignal::new(tr("main_entrance").to_owned());
    let valid_from = RwSignal::new(String::new());
    let valid_until = RwSignal::new(String::new());
    let max_checkins = RwSignal::new(String::new());
    let busy = RwSignal::new(false);

    let create = move |_| {
        let Some(from) = local_to_rfc3339(&valid_from.get()) else {
            error.set(Some(tr("enter_a_valid_start_date").to_owned()));
            return;
        };
        let Some(until) = local_to_rfc3339(&valid_until.get()) else {
            error.set(Some(tr("enter_a_valid_end_date").to_owned()));
            return;
        };
        let max_value = max_checkins.get();
        let max = if max_value.trim().is_empty() {
            None
        } else {
            match max_value.trim().parse::<u32>() {
                Ok(value) if value > 0 => Some(value),
                _ => {
                    error.set(Some(tr("limit_must_be_a_positive_number").to_owned()));
                    return;
                }
            }
        };
        if until <= from {
            error.set(Some(tr("campaign_end_must_be_after_its_start").to_owned()));
            return;
        }
        let input = CreateQrCampaignInput {
            event_slug: event_slug.get(),
            label: label.get().trim().to_owned(),
            valid_from: from,
            valid_until: until,
            max_checkins: max,
        };
        if input.event_slug.is_empty() || input.label.is_empty() {
            error.set(Some(tr("select_a_show_and_name_the_campaign").to_owned()));
            return;
        }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<QrCampaign, _>(
                "create_qr_campaign",
                &CampaignArgs { input: &input },
            )
            .await
            {
                Ok(_) => {
                    error.set(Some(tr("qr_campaign_created").to_owned()));
                    refresh_operator_qr(dashboard, loading, error);
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    view! {
        <section class="screen"><header class="screen-title"><p class="eyebrow">{tr("concert_signal_eyebrow")}</p><h2>{tr("qr_campaigns")}</h2><p class="screen-copy">{tr("qr_campaigns_purpose")}</p></header><div class="form-grid panel"><label>{tr("show")}<select disabled=move || loading.get().qr prop:value=move || event_slug.get() on:change=move |e| event_slug.set(event_target_value(&e))><option value="">{move || if loading.get().qr { tr("loading_campaigns") } else { tr("select_a_show_2") }}</option><For each=move || operator_qr_events(dashboard) key=|event| event.slug.clone() children=move |event| view! { <option value=event.slug.clone()>{event.title}</option> } /></select></label><label>{tr("point_campaign_name")}<input prop:value=move || label.get() on:input=move |e| label.set(event_target_value(&e))/></label><div class="two-cols"><label>{tr("valid_from")}<input type="datetime-local" prop:value=move || valid_from.get() on:input=move |e| valid_from.set(event_target_value(&e))/></label><label>{tr("valid_until")}<input type="datetime-local" prop:value=move || valid_until.get() on:input=move |e| valid_until.set(event_target_value(&e))/></label></div><label>{tr("check_in_limit_optional")}<input inputmode="numeric" prop:value=move || max_checkins.get() on:input=move |e| max_checkins.set(event_target_value(&e))/></label><button class="primary" on:click=create disabled=move || busy.get() || loading.get().qr>{tr("create_campaign")}</button></div><div class="section-head"><h3>{tr("active_and_historical")}</h3></div><Show when=move || !loading.get().qr fallback=move || view! { <Skeleton rows=2 height=96 /> }>
            <Show when=move || operator_campaigns(dashboard).is_empty()>
                <div class="empty-state">
                    <strong>{tr("qr_campaigns_empty")}</strong>
                    <p>{tr("qr_campaigns_empty_hint")}</p>
                </div>
            </Show>
            <div class="card-list"><For each=move || operator_campaigns(dashboard) key=|campaign| campaign.id.clone() children=move |campaign| view! { <CampaignCard campaign=campaign dashboard=dashboard loading=loading error=error /> } /></div></Show></section>
    }
}

#[component]
fn CampaignCard(
    campaign: QrCampaign,
    dashboard: RwSignal<Option<DashboardData>>,
    loading: RwSignal<OperatorLoadingState>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    // The id used to be a `String` cloned inside the handler, which made the
    // handler `FnOnce` — fine for one button, but the confirm step renders it
    // from a `Show` fallback that has to be callable more than once. A signal
    // is `Copy`, so the handler is too.
    let id = RwSignal::new(campaign.id.clone());
    let active = campaign.active;
    // Disabling a campaign closes a live entrance mid-show. It was one tap.
    let confirming = RwSignal::new(false);
    let revoke = move |_| {
        let campaign_id = id.get_untracked();
        spawn_local(async move {
            match bridge::invoke_unit(
                "revoke_qr_campaign",
                &CampaignIdArgs {
                    campaign_id: &campaign_id,
                },
            )
            .await
            {
                Ok(_) => {
                    confirming.set(false);
                    error.set(Some(tr("campaign_has_been_disabled").to_owned()));
                    refresh_operator_qr(dashboard, loading, error);
                }
                Err(message) => error.set(Some(message)),
            }
        });
    };
    let revoke_button = if active {
        Some(view! {
            <Show
                when=move || confirming.get()
                fallback=move || view! { <button class="danger ghost" on:click=move |_| confirming.set(true)>{tr("disable_campaign")}</button> }
            >
                <div class="security-note warning">
                    <p>{tr("disable_campaign_confirm")}</p>
                    <div class="button-row">
                        <button class="danger" on:click=revoke>{tr("disable_campaign_confirm_action")}</button>
                        <button class="ghost" on:click=move |_| confirming.set(false)>{tr("cancel")}</button>
                    </div>
                </div>
            </Show>
        })
    } else {
        None
    };
    view! {
        <article class="campaign-card"><div class="campaign-head"><div><strong>{campaign.label}</strong><p>{campaign.event_title}</p></div><span class:online=active class:offline=!active>{if active { tr("active_status") } else { tr("closed") }}</span></div><div class="campaign-stats"><span>{i18n::format("check_ins_2", &[campaign.checkin_count.to_string()])}</span><span>{campaign.max_checkins.map(|v| i18n::format("limit_v", &[v.to_string()])).value_or_else(|| tr("no_limit").to_owned())}</span></div>{campaign.token.map(|token| view! { <div class="campaign-token"><p class="eyebrow">{tr("campaign_token_label")}</p><code>{token}</code><small>{tr("campaign_token_hint")}</small></div> })}{revoke_button}</article>
    }
}

#[component]
fn LanguageSwitch() -> impl IntoView {
    let selected = i18n::current();
    view! {
        <article class="language-setting">
            <div>
                <strong>{tr("app_language")}</strong>
                <p>{tr("changing_the_language_reloads_the_interface_your")}</p>
            </div>
            <div class="language-switch" role="group" aria-label=tr("language")>
                <button type="button" class:active=selected == Language::Pl on:click=move |_| i18n::select(Language::Pl)>"PL"</button>
                <button type="button" class:active=selected == Language::En on:click=move |_| i18n::select(Language::En)>"EN"</button>
            </div>
        </article>
    }
}

#[component]
fn OperatorSettings(
    status: RwSignal<SessionStatus>,
    dashboard: RwSignal<Option<DashboardData>>,
    loading: RwSignal<OperatorLoadingState>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let refresh = move |_| {
        refresh_operator_parts(dashboard, loading, error);
    };
    let lock = move |_| {
        // Optimistic on purpose: the vault lock has no remote leg, so drop the
        // session material here and reconcile with native state when it replies.
        dashboard.set(None);
        loading.set(OperatorLoadingState::all());
        status.set(SessionStatus {
            configured: status.get_untracked().configured,
            unlocked: false,
            session: None,
        });
        spawn_local(async move {
            match bridge::invoke::<SessionStatus, _>("lock", &EmptyArgs {}).await {
                Ok(value) => {
                    let _ = status.try_set(value);
                }
                Err(message) => {
                    let _ = error.try_set(Some(message));
                }
            }
        })
    };
    let forget = move |_| {
        spawn_local(async move {
            match bridge::invoke::<SessionStatus, _>("forget_device", &EmptyArgs {}).await {
                Ok(value) => {
                    dashboard.set(None);
                    loading.set(OperatorLoadingState::all());
                    status.set(value);
                }
                Err(message) => error.set(Some(message)),
            }
        })
    };
    view! {
        <section class="screen">
            <header class="screen-title"><p class="eyebrow">{tr("device_label")}</p><h2>{tr("settings")}</h2></header>
            <p class="settings-group-label">{tr("settings_group_this_device")}</p>
            <div class="settings-list">
                <article class="settings-row settings-row-static">
                    <span class="settings-row-icon" aria-hidden="true">"⌁"</span>
                    <strong>{tr("connection")}</strong>
                    <small>{tr("staff_connection_hint")}</small>
                    <p class="settings-row-value">{move || status.get().session.map(|s| s.api_base_url).value_or_else(Default::default)}</p>
                    <span class="settings-row-state" class:online=move || !loading.get().events && !loading.get().qr>{move || if loading.get().events || loading.get().qr { tr("connecting_2") } else { tr("online") }}</span>
                </article>
                <article class="settings-row settings-row-static">
                    <span class="settings-row-icon" aria-hidden="true">"⊙"</span>
                    <strong>{tr("permissions")}</strong>
                    <small>{tr("staff_permissions_hint")}</small>
                    <p class="settings-row-value">{move || status.get().session.map(|s| s.role.label().to_owned()).value_or_else(Default::default)}</p>
                </article>
                {move || status.get().session.and_then(|session| {
                    let expires_at = session.session_expires_at?;
                    let now = (js_sys::Date::now() / 1_000.0).max(0.0) as u64;
                    let key = if expires_at <= now {
                        Some("staff_session_expired_pair_again")
                    } else if expires_at.saturating_sub(now) <= 86_400 {
                        Some("staff_session_expires_soon_pair_again")
                    } else {
                        None
                    };
                    key.map(|key| view! { <p class="security-note warning">{tr(key)}</p> })
                })}
                <LanguageSwitch />
            </div>
            <p class="settings-group-label">{tr("settings_group_shortcuts")}</p>
            <div class="settings-list">
                <button type="button" class="settings-row" on:click=move |_| {
                    spawn_local(async move {
                        if let Err(message) = bridge::invoke_unit(
                            "open_external_url",
                            &UrlArgs { url: "https://virya.music/?source=signal-staff-settings" },
                        ).await {
                            error.set(Some(message));
                        }
                    });
                }>
                    <span class="settings-row-icon" aria-hidden="true">"⤴"</span>
                    <strong>"Virya.music"</strong>
                    <small>{tr("staff_site_hint")}</small>
                    <span class="settings-row-chevron" aria-hidden="true">"›"</span>
                </button>
                <button type="button" class="settings-row" on:click=move |_| {
                    spawn_local(async move {
                        if let Err(message) = bridge::invoke_unit(
                            "open_external_url",
                            &UrlArgs { url: "https://virya.music/staff/commerce/?source=signal-staff-latarnik" },
                        ).await {
                            error.set(Some(message));
                        }
                    });
                }>
                    <span class="settings-row-icon" aria-hidden="true">"▤"</span>
                    <strong>{tr("staff_latarnik_panel")}</strong>
                    <small>{tr("staff_latarnik_panel_hint")}</small>
                    <span class="settings-row-chevron" aria-hidden="true">"›"</span>
                </button>
            </div>
            <p class="settings-group-label">{tr("settings_group_session")}</p>
            <div class="settings-list">
                <button class="settings-row" on:click=refresh disabled=move || loading.get().events || loading.get().qr>
                    <span class="settings-row-icon" aria-hidden="true">"⟳"</span>
                    <strong>{tr("refresh_all_data")}</strong>
                    <small>{tr("staff_refresh_hint")}</small>
                    <span class="settings-row-chevron" aria-hidden="true">"›"</span>
                </button>
                <button class="settings-row" on:click=lock>
                    <span class="settings-row-icon" aria-hidden="true">"◫"</span>
                    <strong>{tr("lock_panel")}</strong>
                    <small>{tr("staff_lock_hint")}</small>
                    <span class="settings-row-chevron" aria-hidden="true">"›"</span>
                </button>
                <button class="settings-row danger ghost" on:click=forget>
                    <span class="settings-row-icon" aria-hidden="true">"▨"</span>
                    <strong>{tr("remove_operator_profile")}</strong>
                    <small>{tr("staff_forget_hint")}</small>
                    <span class="settings-row-chevron" aria-hidden="true">"›"</span>
                </button>
            </div>
            <AnonymousFeedback error=error />
            <p class="security-note">{tr("operator_token_is_stored_in_an_encrypted")}</p>
        </section>
    }
}
