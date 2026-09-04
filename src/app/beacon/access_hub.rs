#[component]
fn BeaconAccessHub(
    home: RwSignal<Option<BeaconHomeData>>,
    requests: RwSignal<Option<BeaconPressRequestsData>>,
    releases: RwSignal<Option<BeaconReleasesData>>,
    refresh: RwSignal<u32>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let campaign = RwSignal::new(None::<String>);
    let name = RwSignal::new(String::new());
    let phone = RwSignal::new(String::new());
    let locker = RwSignal::new(String::new());
    let decline_candidate = RwSignal::new(None::<(String, String)>);
    let busy = RwSignal::new(false);

    let confirm_release = move |_| {
        let Some(id) = campaign.get_untracked() else {
            return;
        };
        let recipient_name = name.get_untracked();
        let recipient_phone = phone.get_untracked();
        let parcel_locker = locker.get_untracked();
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<BeaconMutationResult, _>(
                "beacon_release_confirm",
                &BeaconReleaseConfirmArgs {
                    campaign_id: &id,
                    recipient_name: &recipient_name,
                    recipient_phone: &recipient_phone,
                    parcel_locker_code: &parcel_locker,
                },
            )
            .await
            {
                Ok(_) => {
                    campaign.set(None);
                    name.set(String::new());
                    phone.set(String::new());
                    locker.set(String::new());
                    refresh_beacon_releases(releases, error);
                    refresh.update(|value| *value = value.wrapping_add(1));
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let decline_release = move |id: String| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<BeaconMutationResult, _>(
                "beacon_release_decline",
                &BeaconCampaignArgs { campaign_id: &id },
            )
            .await
            {
                Ok(_) => {
                    refresh_beacon_releases(releases, error);
                    refresh.update(|value| *value = value.wrapping_add(1));
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let confirm_decline_release = move |_| {
        let Some((id, _)) = decline_candidate.get_untracked() else { return; };
        decline_candidate.set(None);
        decline_release(id);
    };

    view! {
        <section class="screen beacon-screen">
            <header class="screen-title">
                <p class="eyebrow">{tr("latarnik_access_label")}</p>
                <h2>{tr("latarnik_access_title")}</h2>
                <p>{tr("latarnik_access_subtitle")}</p>
            </header>
            <div class="section-head"><h3>{tr("latarnik_requests")}</h3></div>
            <div class="card-list">
                <Show when=move || requests.get().is_some() fallback=move || view! { <Skeleton rows=2 height=80/> }>
                <Show when=move || requests.get().is_some_and(|data| data.requests.is_empty())>
                    <div class="empty-state compact">
                        <strong>{tr("latarnik_requests_empty")}</strong>
                        <p>{tr("latarnik_requests_empty_hint")}</p>
                    </div>
                </Show>
                {move || requests.get().map(|data| data.requests.into_iter().map(|request| {
                    // The badge printed the raw `status` enum — a Polish
                    // journalist read OPEN and RESOLVED — and the card never
                    // said what had been asked for, only which show it was
                    // about, so an accreditation and a photo request looked
                    // identical. The status vocabulary is the CHECK constraint
                    // on viryaos_beacon_press_requests; an unknown value still
                    // shows through rather than hiding behind a wrong label.
                    let status_label = match request.status.as_str() {
                        "open" => tr("latarnik_request_status_open").to_owned(),
                        "resolved" => tr("latarnik_request_status_resolved").to_owned(),
                        "cancelled" => tr("latarnik_request_status_cancelled").to_owned(),
                        other => other.to_owned(),
                    };
                    let kind_label = beacon_request_kind_label(&request.request_kind);
                    let title = request.event_title.unwrap_or_else(|| kind_label.clone());
                    let note = request.resolution_note.unwrap_or_else(|| request.details.unwrap_or_default());
                    view! {
                        <article class="beacon-request-status">
                            <span class="cache-badge">{status_label}</span>
                            <strong>{title}</strong>
                            <p class="inline-note">{kind_label}</p>
                            <p>{note}</p>
                        </article>
                    }
                }).collect_view())}
                </Show>
            </div>
            <div class="section-head"><h3>{tr("latarnik_release_allocations")}</h3></div>
            <div class="card-list">
                <Show when=move || releases.get().is_some() fallback=move || view! { <Skeleton rows=2 height=80/> }>
                <Show when=move || releases.get().is_some_and(|data| data.campaigns.is_empty())>
                    <div class="empty-state compact">
                        <strong>{tr("latarnik_releases_empty")}</strong>
                        <p>{tr("latarnik_releases_empty_hint")}</p>
                    </div>
                </Show>
                {move || releases.get().map(|data| data.campaigns.into_iter().map(|release| {
                    let campaign_id = release.campaign_id.clone();
                    let decline_id = campaign_id.clone();
                    let status = release.recipient_status.clone();
                    let can_respond = matches!(status.as_str(), "eligible" | "notified");
                    // The raw recipient enum was uppercased and shown as-is,
                    // so the card said ELIGIBLE or NOTIFIED. Vocabulary is the
                    // CHECK constraint on the release-recipient table.
                    let status_label = match status.as_str() {
                        "eligible" => tr("latarnik_release_status_eligible").to_owned(),
                        "notified" => tr("latarnik_release_status_notified").to_owned(),
                        "confirmed" => tr("latarnik_release_status_confirmed").to_owned(),
                        "prepared" => tr("latarnik_release_status_prepared").to_owned(),
                        "sent" => tr("latarnik_release_status_sent").to_owned(),
                        "delivered" => tr("latarnik_release_status_delivered").to_owned(),
                        "declined" => tr("latarnik_release_status_declined").to_owned(),
                        "expired" => tr("latarnik_release_status_expired").to_owned(),
                        "cancelled" => tr("latarnik_release_status_cancelled").to_owned(),
                        other => other.to_owned(),
                    };
                    let title = release.title;
                    let decline_title = title.clone();
                    let product = format!("{} · {}", release.product_name, release.variant_label);
                    let deadline = format!("{} {}", tr("latarnik_claim_until"), human_time(&release.claim_deadline));
                    let actions = can_respond.then(|| {
                        let confirm_id = campaign_id.clone();
                        view! {
                            <div class="beacon-action-row">
                                <button class="primary" on:click=move |_| campaign.set(Some(confirm_id.clone()))>{tr("latarnik_confirm_delivery")}</button>
                                <button class="text-button" disabled=move || busy.get() on:click=move |_| decline_candidate.set(Some((decline_id.clone(), decline_title.clone())) )>{tr("latarnik_decline")}</button>
                            </div>
                        }
                    });
                    view! {
                        <article class="beacon-release-card">
                            <p class="eyebrow">{status_label}</p>
                            <h3>{title}</h3>
                            <p>{product}</p>
                            <small>{deadline}</small>
                            {actions}
                        </article>
                    }
                }).collect_view())}
                </Show>
            </div>
            <Show when=move || decline_candidate.get().is_some()>
                {move || decline_candidate.get().map(|(_, title)| view! {
                    <article class="beacon-delivery-form beacon-decline-confirm">
                        <h3>{tr("latarnik_decline_release_confirm_title")}</h3>
                        <p>{i18n::format("latarnik_decline_release_confirm_hint", &[title])}</p>
                        <button class="danger" disabled=move || busy.get() on:click=confirm_decline_release>{tr("latarnik_decline")}</button>
                        <button class="text-button" disabled=move || busy.get() on:click=move |_| decline_candidate.set(None)>{tr("cancel")}</button>
                    </article>
                })}
            </Show>
            <Show when=move || campaign.get().is_some()>
                <article class="beacon-delivery-form">
                    <h3>{tr("latarnik_delivery_details")}</h3>
                    <label>{tr("latarnik_recipient_name")}<input autocomplete="name" prop:value=move || name.get() on:input=move |e| name.set(event_target_value(&e))/></label>
                    <label>{tr("latarnik_phone")}<input type="tel" prop:value=move || phone.get() on:input=move |e| phone.set(event_target_value(&e))/></label>
                    <label>{tr("latarnik_parcel_locker")}<input prop:value=move || locker.get() on:input=move |e| locker.set(event_target_value(&e))/></label>
                    <button class="primary" disabled=move || busy.get() || name.get().trim().is_empty() || phone.get().trim().is_empty() || locker.get().trim().is_empty() on:click=confirm_release>{tr("latarnik_save_delivery")}</button>
                    <button class="text-button" on:click=move |_| campaign.set(None)>{tr("cancel")}</button>
                </article>
            </Show>
            <BeaconSettings home=home refresh=refresh error=error />
        </section>
    }
}
