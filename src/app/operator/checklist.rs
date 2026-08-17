fn checklist_event_from_target(target: &str) -> Option<String> {
    let (path, query) = target.split_once('?')?;
    if path != "/staff/checklist" {
        return None;
    }
    query.split('&').find_map(|part| {
        let (key, value) = part.split_once('=')?;
        if key != "event"
            || value.is_empty()
            || value.len() > 128
            || !value
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
        {
            return None;
        }
        Some(value.to_owned())
    })
}

fn checklist_section_label(section: &str) -> &'static str {
    match section {
        "show_files" => tr("checklist_section_show_files"),
        "gear" => tr("checklist_section_gear"),
        "media" => tr("checklist_section_media"),
        "logistics" => tr("checklist_section_logistics"),
        "gate" => tr("checklist_section_gate"),
        "post_show" => tr("checklist_section_post_show"),
        _ => tr("gig_checklist"),
    }
}

fn checklist_item_label(item_key: &str) -> &'static str {
    match item_key {
        "laptop_charged_packed" => tr("checklist_laptop_charged_packed"),
        "setlist_ready" => tr("checklist_setlist_ready"),
        "show_files_backup_ready" => tr("checklist_show_files_backup_ready"),
        "merch_packed" => tr("checklist_merch_packed"),
        "rack_cables_instruments_packed" => tr("checklist_rack_cables_instruments_packed"),
        "instrument_spares_packed" => tr("checklist_instrument_spares_packed"),
        "stage_outfit_packed" => tr("checklist_stage_outfit_packed"),
        "wireless_checked" => tr("checklist_wireless_checked"),
        "power_and_chargers_packed" => tr("checklist_power_and_chargers_packed"),
        "camera_handoff_ready" => tr("checklist_camera_handoff_ready"),
        "venue_schedule_confirmed" => tr("checklist_venue_schedule_confirmed"),
        "tech_rider_confirmed" => tr("checklist_tech_rider_confirmed"),
        "staff_assigned" => tr("checklist_staff_assigned"),
        "guestlist_checked" => tr("checklist_guestlist_checked"),
        "offline_snapshot_ready" => tr("checklist_offline_snapshot_ready"),
        "gate_device_charged" => tr("checklist_gate_device_charged"),
        "backup_device_ready" => tr("checklist_backup_device_ready"),
        "network_tested" => tr("checklist_network_tested"),
        "post_show_reconciliation" => tr("checklist_post_show_reconciliation"),
        "post_show_report" => tr("checklist_post_show_report"),
        _ => tr("checklist_unknown_item"),
    }
}

fn refresh_operator_checklist(
    event_slug: String,
    checklist: RwSignal<Option<ShowChecklist>>,
    loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) {
    if event_slug.is_empty() {
        checklist.set(None);
        loading.set(false);
        return;
    }
    loading.set(true);
    spawn_local(async move {
        let result = bridge::invoke_latest::<ShowChecklist, _>(
            "operator_show_checklist",
            &EventArgs {
                event_slug: &event_slug,
            },
            15_000,
            "operator:checklist",
        )
        .await;
        let completed = latest_request_completed(&result);
        match result {
            Ok(Some(value)) => checklist.set(Some(value)),
            Ok(None) => {}
            Err(message) => error.set(Some(message)),
        }
        if completed {
            loading.set(false);
        }
    });
}

#[component]
fn OperatorChecklist(
    dashboard: RwSignal<Option<DashboardData>>,
    loading: RwSignal<OperatorLoadingState>,
    push_target: RwSignal<Option<String>>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let selected_slug = RwSignal::new(String::new());
    let checklist = RwSignal::new(None::<ShowChecklist>);
    let checklist_loading = RwSignal::new(false);
    let busy_item = RwSignal::new(None::<String>);
    let push_status = RwSignal::new(None::<FanPushStatus>);
    let push_loading = RwSignal::new(false);
    let push_resume_refresh = RwSignal::new(0_u32);
    let push_resume_pending = RwSignal::new(false);
    let push_enable_after_settings = RwSignal::new(false);
    install_resume_refresh(push_resume_refresh);

    Effect::new(move |_| {
        if let Some(target) = push_target.get()
            && let Some(event_slug) = checklist_event_from_target(&target)
            && selected_slug.get_untracked() != event_slug
        {
            selected_slug.set(event_slug);
            push_target.set(None);
            return;
        }
        if selected_slug.get().is_empty()
            && let Some(event) = operator_events(dashboard).into_iter().next()
        {
            selected_slug.set(event.slug);
        }
    });

    Effect::new(move |_| {
        let event_slug = selected_slug.get();
        if !event_slug.is_empty() {
            refresh_operator_checklist(event_slug, checklist, checklist_loading, error);
        }
    });

    Effect::new(move |_| {
        push_resume_refresh.get();
        if push_loading.get_untracked() {
            push_resume_pending.set(true);
            return;
        }
        push_resume_pending.set(false);
        push_loading.set(true);
        spawn_lifecycle_task(async move {
            let retry_after_settings =
                push_enable_after_settings.try_get_untracked().unwrap_or(false);
            match bridge::invoke_timeout::<FanPushStatus, _>(
                "operator_push_sync",
                &EmptyArgs {},
                15_000,
            )
            .await
            {
                Ok(value)
                    if retry_after_settings && value.permission != "denied" && !value.enabled =>
                {
                    let _ = push_status.try_set(Some(value));
                    match bridge::invoke_timeout::<FanPushStatus, _>(
                        "operator_push_enable",
                        &EmptyArgs {},
                        45_000,
                    )
                    .await
                    {
                        Ok(value) => {
                            let _ = push_enable_after_settings
                                .try_set(!value.enabled && value.permission != "denied");
                            let _ = push_status.try_set(Some(value));
                        }
                        Err(message) => {
                            let _ = error.try_set(Some(message));
                        }
                    }
                }
                Ok(value) => {
                    if value.enabled || value.permission == "denied" {
                        let _ = push_enable_after_settings.try_set(false);
                    }
                    let _ = push_status.try_set(Some(value));
                }
                Err(message) => {
                    let _ = error.try_set(Some(message));
                }
            }
            finish_resumable_ui_task(
                push_loading,
                push_resume_pending,
                push_resume_refresh,
            );
        });
    });

    let enable_push = move |_| {
        if push_loading.get_untracked() {
            return;
        }
        let opens_settings = push_status
            .get_untracked()
            .as_ref()
            .is_some_and(|status| status.permission == "denied");
        push_loading.set(true);
        if opens_settings {
            push_enable_after_settings.set(true);
        }
        spawn_lifecycle_task(async move {
            let result = if opens_settings {
                bridge::invoke_timeout::<FanPushStatus, _>(
                    "operator_push_open_settings",
                    &EmptyArgs {},
                    45_000,
                )
                .await
            } else {
                bridge::invoke_timeout::<FanPushStatus, _>(
                    "operator_push_enable",
                    &EmptyArgs {},
                    45_000,
                )
                .await
            };
            match result {
                Ok(value) => {
                    let _ = push_status.try_set(Some(value));
                }
                Err(message) => {
                    let _ = error.try_set(Some(message));
                }
            }
            finish_resumable_ui_task(
                push_loading,
                push_resume_pending,
                push_resume_refresh,
            );
        });
    };

    let disable_push = move |_| {
        if push_loading.get_untracked() {
            return;
        }
        push_loading.set(true);
        spawn_lifecycle_task(async move {
            match bridge::invoke_timeout::<FanPushStatus, _>(
                "operator_push_disable",
                &EmptyArgs {},
                30_000,
            )
            .await
            {
                Ok(value) => {
                    let _ = push_status.try_set(Some(value));
                }
                Err(message) => {
                    let _ = error.try_set(Some(message));
                }
            }
            finish_resumable_ui_task(
                push_loading,
                push_resume_pending,
                push_resume_refresh,
            );
        });
    };

    let toggle = move |item_key: String, current_status: String| {
        let event_slug = selected_slug.get();
        if event_slug.is_empty() || busy_item.get_untracked().is_some() {
            return;
        }
        let next_status = if current_status == "done" { "pending" } else { "done" };
        busy_item.set(Some(item_key.clone()));
        spawn_local(async move {
            let args = ChecklistUpdateArgs {
                event_slug: &event_slug,
                item_key: &item_key,
                status: next_status,
            };
            match bridge::invoke_timeout::<ShowChecklist, _>("operator_update_show_checklist", &args, 15_000).await {
                Ok(value) => checklist.set(Some(value)),
                Err(message) => error.set(Some(message)),
            }
            busy_item.set(None);
        });
    };

    view! {
        <section class="screen">
            <header class="screen-title">
                <p class="eyebrow">{tr("show_operations")}</p>
                <h2>{tr("gig_checklist")}</h2>
                <p>{tr("gig_checklist_hint")}</p>
            </header>

            <article class="panel">
                <label>{tr("show")}
                    <select
                        disabled=move || loading.get().events
                        prop:value=move || selected_slug.get()
                        on:change=move |event| selected_slug.set(event_target_value(&event))
                    >
                        <option value="">{tr("select_a_show")}</option>
                        <For
                            each=move || operator_events(dashboard)
                            key=|event| event.slug.clone()
                            children=move |event| view! { <option value=event.slug.clone()>{event.title}</option> }
                        />
                    </select>
                </label>
            </article>

            <article class="panel">
                <div class="section-head">
                    <div>
                        <h3>{tr("team_push_notifications")}</h3>
                        <p>{tr("team_push_notifications_hint")}</p>
                    </div>
                </div>
                {move || {
                    if push_loading.get() {
                        view! { <p>{tr("syncing_push_notifications")}</p> }.into_any()
                    } else if let Some(status) = push_status.get() {
                        if status.enabled {
                            view! {
                                <div class="form-grid">
                                    <p class="success-note">{tr("team_push_active")}</p>
                                    <button class="ghost" type="button" on:click=disable_push>{tr("disable_push_notifications")}</button>
                                </div>
                            }.into_any()
                        } else {
                            view! {
                                <div class="form-grid">
                                    <p>{tr("team_push_inactive")}</p>
                                    <button class="primary" type="button" on:click=enable_push>{if status.permission == "denied" { tr("open_notification_settings") } else { tr("enable_notifications") }}</button>
                                </div>
                            }.into_any()
                        }
                    } else {
                        view! { <p>{tr("team_push_status_unknown")}</p> }.into_any()
                    }
                }}
            </article>

            <Show when=move || !checklist_loading.get() fallback=move || view! { <Skeleton /> }>
                {move || checklist.get().map(|snapshot| {
                    let done = snapshot.items.iter().filter(|item| item.status == "done").count();
                    let total = snapshot.items.len();
                    let event_id = snapshot.event_id.clone();
                    let event_slug = snapshot.event_slug.clone();
                    let event_time = human_time(&snapshot.starts_at);
                    let mut items = snapshot.items;
                    items.sort_by(|left, right| {
                        left.sort_order
                            .cmp(&right.sort_order)
                            .then_with(|| left.item_key.cmp(&right.item_key))
                    });
                    view! {
                        <article class="panel" data-event-id=event_id data-event-slug=event_slug>
                            <div class="section-head">
                                <div>
                                    <h3>{snapshot.event_title.clone()}</h3>
                                    <p>{event_time}</p>
                                    <p>{i18n::format("checklist_progress", &[done.to_string(), total.to_string()])}</p>
                                </div>
                                <strong>{format!("{done}/{total}")}</strong>
                            </div>
                            <div class="card-list">
                                {items.into_iter().map(|item| {
                                    let key = item.item_key.clone();
                                    let key_for_click = key.clone();
                                    let key_for_busy = key.clone();
                                    let current = item.status.clone();
                                    let done = current == "done";
                                    let label = checklist_item_label(&item.item_key);
                                    let section = checklist_section_label(&item.section);
                                    let note = item.note.filter(|value| !value.trim().is_empty());
                                    let updated_at = human_time(&item.updated_at);
                                    view! {
                                        <button
                                            type="button"
                                            class="campaign-card checklist-row"
                                            class:checklist-done=done
                                            disabled=move || busy_item.get().is_some()
                                            on:click=move |_| toggle(key_for_click.clone(), current.clone())
                                        >
                                            <span class="checklist-mark">{if done { "✓" } else { "○" }}</span>
                                            <span class="checklist-copy">
                                                <small>{section}</small>
                                                <strong>{label}</strong>
                                                {note.map(|note| view! { <span>{note}</span> })}
                                                <small>{updated_at}</small>
                                            </span>
                                            <span>{move || if busy_item.get().as_deref() == Some(key_for_busy.as_str()) { "…" } else { "" }}</span>
                                        </button>
                                    }
                                }).collect_view()}
                            </div>
                        </article>
                    }
                })}
            </Show>
        </section>
    }
}
