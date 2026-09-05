#[component]
fn FanEvents(
    dashboard: RwSignal<Option<FanDashboardData>>,
    public: RwSignal<Option<PublicHomeData>>,
    focused_event_slug: RwSignal<Option<String>>,
    focused_event_preview: RwSignal<Option<PublicEvent>>,
    checkout_event: RwSignal<Option<PublicEvent>>,
    loading: RwSignal<FanLoadingState>,
    error: RwSignal<Option<String>>,
    status: RwSignal<crate::models::FanSessionStatus>,
) -> impl IntoView {
    // The list has to come through a memo. Reading `dashboard` directly in the
    // render closure tied the whole list to every field of the dashboard, so
    // tapping "interested" — which writes `dashboard.interests` — tore down and
    // rebuilt every card. Each rebuilt card ran `on_cleanup` (invalidating its
    // ticket-pool scope) and then re-fired its probe, turning one tap into one
    // network round trip per show on screen. The memo compares the event list
    // itself, so an interests write no longer reaches it.
    let events = Memo::new(move |_| fan_events(dashboard, public));
    // `loading` is one struct for eight sections, so reading `.events` off it
    // raw made the warm-up's merch/wallet/area flags rebuild this list too.
    let events_loading = Memo::new(move |_| loading.get().events);
    view! {
        <section class="screen">
            <header class="screen-title"><p class="eyebrow">{tr("where_we_play")}</p><h2>{tr("shows_tab")}</h2></header>
            {move || {
                let events = events.get();
                if let Some(slug) = focused_event_slug.get() {
                    let resolved = events
                        .iter()
                        .find(|event| event.slug == slug)
                        .cloned()
                        .or_else(|| {
                            focused_event_preview
                                .get()
                                .filter(|event| event.slug == slug)
                        });
                    if let Some(event) = resolved {
                        return view! {
                            <div class="fan-event-detail">
                                <button class="checkout-back" type="button" on:click=move |_| {
                                    focused_event_slug.set(None);
                                    focused_event_preview.set(None);
                                }>{tr("back_back_to_shows")}</button>
                                <FanEventCard event=event checkout_event=checkout_event dashboard=dashboard error=error status=status />
                            </div>
                        }.into_any();
                    }
                }
                if events_loading.get() {
                    return view! { <Skeleton rows=3 height=280 /> }.into_any();
                }
                if events.is_empty() {
                    view! { <div class="empty-state"><strong>{tr("no_shows_in_the_calendar")}</strong><p>{tr("new_events_will_appear_here_2")}</p></div> }.into_any()
                } else {
                    // Keyed <For> preserves FanEventCard components across
                    // re-renders. Without keys, every dashboard update
                    // destroyed all cards and re-fired their ticket-pool
                    // probe effects, causing redundant network calls and
                    // DOM churn.
                    view! {
                        <div class="card-list fan-event-list">
                            <For each=move || events.clone() key=|event| event.slug.clone() let:event>
                                <FanEventCard event=event checkout_event=checkout_event dashboard=dashboard error=error status=status />
                            </For>
                        </div>
                    }.into_any()
                }
            }}
        </section>
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
    error: RwSignal<Option<String>>,
    status: RwSignal<crate::models::FanSessionStatus>,
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
        // Guard: don't probe the ticket pool until the fan session is
        // unlocked. After app restart, the session is locked and
        // fan_ticket_sale returns AppError::Locked. This would set
        // window.__VIRYA_LAST_OPERATION__ to "fan_ticket_sale", and any
        // concurrent window-error (e.g. a resource load failure during
        // startup) gets falsely attributed to fan_ticket_sale in the
        // crash report. Waiting for unlock avoids the false attribution
        // and the unnecessary IPC round-trip.
        if !status.get_untracked().unlocked {
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
    let interest_event = event.clone();
    let busy = RwSignal::new(false);
    // Saving used to take two round trips before the button changed: the write,
    // then a full re-read of the interest list. The row shows as saved while
    // the write happens behind it; a failed write takes the row back out, and
    // the next full refresh reconciles server ordering instead of paying a
    // second round trip on every tap.
    let interest = move |_| {
        if interested.get_untracked() || busy.get_untracked() {
            return;
        }
        let event_slug = interest_slug.clone();
        busy.set(true);
        dashboard.update(|state| {
            let data = state.get_or_insert_with(FanDashboardData::default);
            let mut next = std::mem::take(&mut data.interests);
            next.push(FanEventInterest {
                event: interest_event.clone(),
            });
            data.interests = stable_fan_interests(next);
        });
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
                }
                Err(_) => {
                    // Silent: the row reverts to unsaved state. The fan
                    // can tap again. No toast for a transient interest toggle.
                    dashboard.update(|state| {
                        if let Some(data) = state.as_mut() {
                            data.interests
                                .retain(|item| item.event.slug != event_slug);
                        }
                    });
                }
            }
            busy.set(false);
        });
    };
    let event_day = day(&event.starts_at);
    let event_month = month(&event.starts_at);
    let event_time = human_time(&event.starts_at);
    let location = event_location(&event);
    // The card carries a date, a title and a line of location, and that is all
    // Signal holds about a show. Everything else — the lineup, the doors, the
    // venue write-up — lives on the show page, and until now nothing on this
    // card led there. The locale segment follows the reader: /pl/live/<slug>
    // is a 404 for an English fan, and the page exists under both prefixes.
    let details_url = match i18n::current() {
        i18n::Language::Pl => {
            format!("https://virya.music/pl/live/{}/?source=signal-app", event.slug)
        }
        i18n::Language::En => {
            format!("https://virya.music/live/{}/?source=signal-app", event.slug)
        }
    };
    let open_details = move |_| {
        let url = details_url.clone();
        spawn_local(async move {
            if let Err(message) =
                bridge::invoke_unit("open_external_url", &UrlArgs { url: &url }).await
            {
                error.set(Some(message));
            }
        });
    };
    let image = event.image_thumbnail_url.or(event.image_url);
    let description = event.description;
    let title = event.title;
    let image_alt = i18n::format("virya_show", std::slice::from_ref(&title));
    let image_loaded = RwSignal::new(false);
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
                    class:loaded=move || image_loaded.get()
                    on:load=move |_| image_loaded.set(true)
                />
            })}
            <div class="fan-event-body">
                <div class="date-line"><span>{format!("{event_day} {event_month}")}</span><small>{event_time}</small></div>
                <h3>{title}</h3><p>{location}</p>
                {description.map(|text| view! { <p class="event-description">{text}</p> })}
                <div class="event-actions">
                    // Everything Signal holds about a show is already on this
                    // card; the lineup, the doors and the venue write-up are on
                    // the show page. A tappable heading was carrying that link
                    // with nothing to say it was one, so it is a labelled
                    // control in the row where a fan looks for actions.
                    <button type="button" class="event-details-action" on:click=open_details>{tr("show_details")}</button>
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
            let result = bridge::invoke_latest::<Option<TicketSaleOffer>, _>(
                "fan_ticket_sale",
                &EventArgs {
                    event_slug: &event_slug,
                },
                15_000,
                "fan:ticket-sale",
            )
            .await;
            let completed = latest_request_completed(&result);
            match result {
                Ok(Some(Some(value))) => {
                    sale.set(Some(value));
                    sale_failed.set(false);
                }
                Ok(Some(None)) => {
                    sale.set(None);
                    sale_failed.set(false);
                }
                Ok(None) => {}
                Err(_) => {
                    sale.set(None);
                    sale_failed.set(true);
                    // Silent: sale_failed shows the "temporarily unavailable"
                    // inline status. No toast needed.
                }
            }
            if completed {
                sale_loading.set(false);
            }
        });
    });
    on_cleanup(move || bridge::invalidate_latest("fan:ticket-sale"));

    let back = move |_| checkout_event.set(None);
    let event_title = event.title.clone();
    let event_meta = event_time_location(&event.starts_at, event.venue.as_deref());
    let event_slug = event.slug.clone();
    // Virya's own show page is the first-party ticket surface, so it wins over
    // whatever external promoter link the provider happened to attach. Falling
    // back to ticket_url only matters for shows we do not sell ourselves.
    // The locale segment has to follow the reader: /pl/live/<slug> is a 404 for
    // an English fan, and the page exists under both prefixes.
    let live_url = match i18n::current() {
        i18n::Language::Pl => format!("https://virya.music/pl/live/{}/#tickets", event.slug),
        i18n::Language::En => format!("https://virya.music/live/{}/#tickets", event.slug),
    };
    let fallback_url = live_url.clone();
    let full_form_url = live_url;

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
                    <Show when=move || !sale_loading.get() fallback=move || view! { <Skeleton rows=4 height=120 /> }>
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
    let purchase_slug = RwSignal::new(event_slug.clone());
    // Inline checkout error — shown on the checkout total bar, not a toast.
    let checkout_error = RwSignal::new(None::<String>);

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
            checkout_error.set(Some(tr("select_at_least_one_ticket").to_owned()));
            return;
        }
        checkout_error.set(None);
        let name = buyer_name.get_untracked().trim().to_owned();
        let input = TicketCheckoutInput {
            event_slug: purchase_slug.get_untracked(),
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
                    bridge::haptic("confirm");
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
                                "order_saved_complete_the_secure_stripe_payment",
                                std::slice::from_ref(&checkout.order_reference),
                            )));
                        }
                        Err(message) => {
                            error.set(Some(i18n::format(
                                "message_order_is_saved_use_the_reopen",
                                &[message.to_string(), checkout.order_reference.to_string()],
                            )));
                        }
                    }
                }
                Err(message) => {
                    // Inline: show on the checkout total bar, not a toast.
                    checkout_error.set(Some(message));
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
                        "payment_opened_for_order",
                        std::slice::from_ref(&checkout.order_reference),
                    )));
                }
                // Inline: show on the checkout bar, not a toast.
                Err(message) => checkout_error.set(Some(message)),
            }
        });
    };

    let purchase_disabled = Signal::derive(move || {
        busy.get() || selected_count.get() == 0 || pending_checkout.get().is_some()
    });
    // Progressive checkout: step 1 = tickets, step 2 = details, step 3 = payment.
    // A sticky summary bar shows the running total at every step.
    let checkout_step = RwSignal::new(1_u8);
    let can_continue_to_details = Signal::derive(move || selected_count.get() > 0 && !busy.get());
    // Currency travels in a RwSignal so multiple Show blocks can read it
    // without moving a String into their FnOnce children closures.
    let currency_signal = RwSignal::new(offer.currency.clone());
    view! {
        <div class="ticket-sale-summary">
            <div><strong>{sale_available}</strong><span>{tr("available_label")}</span></div>
            <div><strong>{sale_reserved}</strong><span>{tr("in_checkout_2")}</span></div>
            <div><strong>{sale_sold}</strong><span>{tr("sold")}</span></div>
        </div>
        <p class="checkout-state-copy">{state_copy}</p>
        // Step indicator
        <div class="checkout-steps">
            <span class:active=move || { checkout_step.get() == 1 }>{tr("checkout_step_tickets")}</span>
            <span class:active=move || { checkout_step.get() == 2 }>{tr("checkout_step_details")}</span>
            <span class:active=move || { checkout_step.get() == 3 }>{tr("checkout_step_payment")}</span>
        </div>
        // Step 1: Ticket selection
        <Show when=move || { checkout_step.get() == 1 }>
            <div class="ticket-type-list">
                {ticket_types.iter().map(|ticket_type| {
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
                    let name = ticket_type.name.clone();
                    let description = ticket_type.description.clone();
                    let price_gross_minor = ticket_type.price_gross_minor;
                    let available_count = ticket_type.available.max(0);
                    view! {
                        <article class="ticket-type-card">
                            <div>
                                <h3>{name}</h3>
                                {description.map(|description| view! { <p>{description}</p> })}
                                <strong>{money(price_gross_minor, &currency)}</strong>
                                <small>{i18n::format("available", &[available_count.to_string()])}</small>
                            </div>
                            <div class="ticket-stepper" role="group" aria-label=tr("ticket_quantity")>
                                <button type="button" aria-label=tr("decrease_ticket_quantity") on:click=decrement disabled=move || decrement_disabled.get()>"−"</button>
                                <output aria-live="polite">{move || quantity.get()}</output>
                                <button type="button" aria-label=tr("increase_ticket_quantity") on:click=increment disabled=move || increment_disabled.get()>"+"</button>
                            </div>
                        </article>
                    }
                }).collect_view()}
            </div>
        </Show>
        // Step 2: Buyer details
        <Show when=move || { checkout_step.get() == 2 }>
            <div class="ticket-buyer-panel">
                <label>{tr("name_on_the_order_optional")}<input autocomplete="name" maxlength="160" prop:value=move || buyer_name.get() on:input=move |event| buyer_name.set(event_target_value(&event))/></label>
                <p>{move || status.get().session.map(|profile| i18n::format("tickets_and_confirmation_will_be_sent_to", std::slice::from_ref(&profile.email))).value_or_else(|| tr("tickets_will_be_sent_to_the_fan").to_owned())}</p>
                <ExternalLink url=full_form_url.clone() label=tr("invoice_full_form") error=error />
            </div>
        </Show>
        // Step 3: Payment review
        <Show when=move || { checkout_step.get() == 3 }>
            <div class="checkout-review">
                <h3>{tr("checkout_review")}</h3>
                <div><span>{tr("selected_tickets")}</span><strong>{move || selected_count.get()}</strong></div>
                <div><span>{tr("gross_total")}</span><strong>{move || money(selected_gross.get(), &currency_signal.get())}</strong></div>
            </div>
        </Show>
        // Sticky summary bar — visible at all steps
        <footer class="ticket-checkout-total ticket-checkout-sticky">
            <div><span>{tr("selected_tickets")}</span><strong>{move || selected_count.get()}</strong></div>
            <div><span>{tr("gross_total")}</span><strong>{move || money(selected_gross.get(), &currency_signal.get())}</strong></div>
            // Step 1: Continue to details
            <Show when=move || { checkout_step.get() == 1 }>
                <button type="button" class="primary" on:click=move |_| checkout_step.set(2) disabled=move || !can_continue_to_details.get()>{tr("checkout_continue_to_details")}</button>
            </Show>
            // Step 2: Back + Continue to payment
            <Show when=move || { checkout_step.get() == 2 }>
                <button type="button" class="ghost" on:click=move |_| checkout_step.set(1)>{tr("checkout_back")}</button>
                <button type="button" class="primary" on:click=move |_| checkout_step.set(3)>{tr("checkout_continue_to_payment")}</button>
            </Show>
            // Step 3: Back + Pay
            <Show when=move || { checkout_step.get() == 3 }>
                <button type="button" class="ghost" on:click=move |_| checkout_step.set(2)>{tr("checkout_back")}</button>
                <button type="button" class="primary" on:click=purchase disabled=move || purchase_disabled.get()>{move || if busy.get() { tr("reserving") } else if pending_checkout.get().is_some() { tr("order_saved") } else { tr("continue_to_stripe_payment") }}</button>
            </Show>
            <Show when=move || pending_checkout.get().is_some()>
                <button type="button" class="ghost checkout-retry" on:click=retry_payment>{tr("reopen_payment")}</button>
            </Show>
            {move || checkout_error.get().map(|msg| view! { <small class="inline-form-error">{error_message(&msg).to_owned()}</small> })}
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
            if let Err(message) = bridge::invoke_unit("open_external_url", &UrlArgs { url: &current }).await {
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
        if let Err(message) = bridge::invoke_unit("open_external_url", &UrlArgs { url: &url }).await {
            error.set(Some(message));
        }
    });
}
