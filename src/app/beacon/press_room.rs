#[component]
fn BeaconPressRoom(
    press_room: RwSignal<Option<BeaconPressRoomData>>,
    selected_event: RwSignal<Option<String>>,
    requests: RwSignal<Option<BeaconPressRequestsData>>,
    refresh: RwSignal<u32>,
    tab: RwSignal<BeaconTab>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let request_kind = RwSignal::new("press_photo".to_owned());
    let details = RwSignal::new(String::new());
    let coverage_kind = RwSignal::new("article".to_owned());
    let coverage_url = RwSignal::new(String::new());
    let coverage_title = RwSignal::new(String::new());
    let coverage_status = RwSignal::new(None::<String>);
    let busy = RwSignal::new(false);

    Effect::new(move |_| {
        refresh.get();
        // This screen is mounted from the moment the shell is, so without the
        // tab check it would fetch a press room for every event the fan picks
        // on the radar, whether or not they ever open this tab. The section
        // stays owned here rather than in the shell's loader because it is
        // keyed on the selected event, not only on the refresh generation.
        if tab.get() != BeaconTab::Press {
            return;
        }
        let event_selected = selected_event.get().is_some();
        let current_kind = request_kind.get_untracked();
        if event_selected && current_kind == "press_photo" {
            request_kind.set("accreditation".to_owned());
        } else if !event_selected && current_kind == "accreditation" {
            request_kind.set("press_photo".to_owned());
        }
        refresh_beacon_press_room(press_room, selected_event.get_untracked(), error);
    });

    let request = move |_| {
        if busy.get_untracked() {
            return;
        }
        busy.set(true);
        let kind = request_kind.get_untracked();
        let text = details.get_untracked();
        let event = selected_event.get_untracked();
        spawn_local(async move {
            match bridge::invoke::<BeaconMutationResult, _>(
                "beacon_press_request_create",
                &BeaconPressRequestArgs {
                    event_id: event.as_deref(),
                    request_kind: &kind,
                    details: (!text.trim().is_empty()).then_some(text.trim()),
                },
            )
            .await
            {
                Ok(_) => {
                    details.set(String::new());
                    refresh_beacon_requests(requests, error);
                    refresh.update(|value| *value = value.wrapping_add(1));
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let submit_coverage = move |_| {
        if busy.get_untracked() {
            return;
        }
        let Some(event_id) = selected_event.get_untracked() else {
            return;
        };
        let url = coverage_url.get_untracked();
        if url.trim().is_empty() {
            return;
        }
        let kind = coverage_kind.get_untracked();
        let title = coverage_title.get_untracked();
        busy.set(true);
        coverage_status.set(None);
        spawn_local(async move {
            match bridge::invoke::<BeaconMutationResult, _>(
                "beacon_coverage",
                &BeaconCoverageArgs {
                    event_id: &event_id,
                    coverage_kind: &kind,
                    url: url.trim(),
                    title: (!title.trim().is_empty()).then_some(title.trim()),
                },
            )
            .await
            {
                Ok(_) => {
                    coverage_url.set(String::new());
                    coverage_title.set(String::new());
                    coverage_status.set(Some(tr("latarnik_coverage_saved").to_owned()));
                    refresh.update(|value| *value = value.wrapping_add(1));
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    view! {
        <section class="screen beacon-screen">
            <header class="screen-title">
                <p class="eyebrow">{tr("latarnik_press_label")}</p>
                <h2>{tr("latarnik_press_title")}</h2>
                {move || press_room.get().and_then(|room| room.event).map(|event| {
                    view! { <p>{format!("{} · {}", event.title, event.city.unwrap_or_default())}</p> }
                })}
            </header>
            <div class="beacon-assets">
                <Show when=move || press_room.get().is_some() fallback=move || view! { <Skeleton rows=2 height=80/> }>
                    {move || press_room.get().map(|room| room.assets.into_iter().map(|asset| {
                        let label = if i18n::current() == Language::En { asset.label_en } else { asset.label_pl };
                        let url = asset.url;
                        view! {
                            // The raw `asset_kind` enum used to sit above the
                            // label as a badge — a journalist read PRESS_PHOTO
                            // over the localized asset name, a backend token
                            // saying nothing the label did not already say in
                            // their own language, and a new kind upstream would
                            // simply print itself.
                            <article class="beacon-asset-card">
                                <strong>{label}</strong>
                                <button class="ghost" on:click=move |_| beacon_open_url(url.clone(), error)>{tr("latarnik_open_asset")}</button>
                            </article>
                        }
                    }).collect_view())}
                </Show>
            </div>
            <article class="beacon-request-card">
                <p class="eyebrow">{tr("latarnik_need_something")}</p>
                <h3>{tr("latarnik_request_material")}</h3>
                <label>{tr("type")}<select prop:value=move || request_kind.get() on:change=move |e| request_kind.set(event_target_value(&e))><Show when=move || selected_event.get().is_some()><option value="accreditation">{tr("latarnik_request_accreditation")}</option></Show><option value="press_photo">{tr("latarnik_request_photos")}</option><option value="wav">WAV</option><option value="clean_version">{tr("latarnik_request_clean")}</option><option value="interview">{tr("latarnik_request_interview")}</option><option value="custom">{tr("latarnik_request_other")}</option></select></label>
                <label>{tr("latarnik_details_optional")}<textarea rows="3" prop:value=move || details.get() on:input=move |e| details.set(event_target_value(&e))></textarea></label>
                <button class="primary" disabled=move || busy.get() || (request_kind.get()=="custom" && details.get().trim().is_empty()) || (request_kind.get()=="accreditation" && selected_event.get().is_none()) on:click=request>{tr("latarnik_send_request")}</button>
                <p class="inline-note">{tr("latarnik_accreditation_note")}</p>
            </article>
            <Show when=move || selected_event.get().is_some()>
                <article class="beacon-request-card beacon-coverage-card">
                    <p class="eyebrow">{tr("latarnik_coverage_label")}</p>
                    <h3>{tr("latarnik_coverage_title")}</h3>
                    <p>{tr("latarnik_coverage_hint")}</p>
                    <label>{tr("latarnik_coverage_kind")}<select prop:value=move || coverage_kind.get() on:change=move |e| coverage_kind.set(event_target_value(&e))><option value="article">{tr("latarnik_help_article")}</option><option value="radio">{tr("latarnik_help_radio")}</option><option value="video">{tr("latarnik_coverage_video")}</option><option value="photo">{tr("latarnik_help_photos")}</option><option value="social">{tr("latarnik_coverage_social")}</option><option value="podcast">{tr("latarnik_help_podcast")}</option><option value="other">{tr("latarnik_help_other")}</option></select></label>
                    <label>{tr("latarnik_coverage_url")}<input type="url" inputmode="url" placeholder="https://" prop:value=move || coverage_url.get() on:input=move |e| coverage_url.set(event_target_value(&e))/></label>
                    <label>{tr("latarnik_coverage_title_optional")}<input prop:value=move || coverage_title.get() on:input=move |e| coverage_title.set(event_target_value(&e))/></label>
                    <button class="primary" disabled=move || busy.get() || coverage_url.get().trim().is_empty() on:click=submit_coverage>{tr("latarnik_coverage_submit")}</button>
                    {move || coverage_status.get().map(|message| view! { <small class="success">{message}</small> })}
                </article>
            </Show>
        </section>
    }
}

