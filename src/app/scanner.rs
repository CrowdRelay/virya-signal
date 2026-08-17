// Live gate scanner and offline Show Mode. Kept separate from the broader operator cockpit.

// CrowdRelay's AdmissionRedemptionStatus can also return "revoked", "expired"
// and "not_claimed" (a QR that was never claimed by its winner). Those must
// read as an explicit, unambiguous door decision rather than the raw enum
// string, since a misread here is the difference between admitting or
// turning away a real person.
fn scan_status_label(status: &str) -> String {
    match status {
        "redeemed" => tr("scan_status_redeemed").to_owned(),
        "already_redeemed" => tr("scan_status_already_redeemed").to_owned(),
        "revoked" => tr("scan_status_revoked").to_owned(),
        "expired" => tr("scan_status_expired").to_owned(),
        "not_claimed" => tr("scan_status_not_claimed").to_owned(),
        "offline_queued" => tr("scan_status_offline_queued").to_owned(),
        "offline_duplicate" => tr("scan_status_offline_duplicate").to_owned(),
        other => other.to_uppercase(),
    }
}

#[component]
fn Scanner(
    dashboard: RwSignal<Option<DashboardData>>,
    loading: RwSignal<OperatorLoadingState>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let event_slug = RwSignal::new(String::new());
    let manual = RwSignal::new(String::new());
    let result = RwSignal::new(None::<AdmissionRedemption>);
    let busy = RwSignal::new(false);
    let offline = RwSignal::new(false);
    let show_mode = RwSignal::new(ShowModeStatus::default());
    let show_message = RwSignal::new(String::new());

    let refresh_show_status = move |slug: String| {
        if slug.is_empty() {
            show_mode.set(ShowModeStatus::default());
            offline.set(false);
            return;
        }
        spawn_local(async move {
            match bridge::invoke::<ShowModeStatus, _>(
                "show_mode_status",
                &EventArgs { event_slug: &slug },
            )
            .await
            {
                Ok(value) => {
                    if !value.prepared {
                        offline.set(false);
                    }
                    show_mode.set(value);
                }
                Err(message) => error.set(Some(message)),
            }
        });
    };

    let select_event = move |event| {
        let slug = event_target_value(&event);
        event_slug.set(slug.clone());
        result.set(None);
        show_message.set(String::new());
        refresh_show_status(slug);
    };

    let redeem_code = move |code: String| {
        let slug = event_slug.get_untracked();
        if slug.is_empty() {
            error.set(Some(tr("select_a_show_first").to_owned()));
            return;
        }
        busy.set(true);
        let use_offline = offline.get_untracked();
        spawn_local(async move {
            if use_offline {
                match bridge::invoke::<ShowModeScanResult, _>(
                    "show_mode_scan",
                    &RedeemArgs {
                        event_slug: &slug,
                        code: &code,
                    },
                )
                .await
                {
                    Ok(value) => {
                        let status = if value.duplicate {
                            "offline_duplicate"
                        } else {
                            "offline_queued"
                        };
                        result.set(Some(AdmissionRedemption {
                            public_reference: value.public_reference,
                            holder_name: value.holder_name,
                            holder_email_masked: value.holder_email_masked,
                            status: status.to_owned(),
                        }));
                        refresh_show_status(slug.clone());
                    }
                    Err(message) => error.set(Some(message)),
                }
            } else {
                match bridge::invoke::<AdmissionRedemption, _>(
                    "redeem_admission",
                    &RedeemArgs {
                        event_slug: &slug,
                        code: &code,
                    },
                )
                .await
                {
                    Ok(value) => result.set(Some(value)),
                    Err(message) => error.set(Some(message)),
                }
            }
            busy.set(false);
        });
    };

    let scan = move |_| {
        spawn_local(async move {
            match bridge::scan_qr().await {
                Ok(Some(code)) => redeem_code(code),
                Ok(None) => {}
                Err(message) => error.set(Some(message)),
            }
        });
    };
    let manual_submit = move |_| redeem_code(manual.get().trim().to_owned());

    let prepare = move |_| {
        let slug = event_slug.get();
        if slug.is_empty() {
            error.set(Some(tr("select_a_show").to_owned()));
            return;
        }
        busy.set(true);
        show_message.set(String::new());
        spawn_local(async move {
            match bridge::invoke::<ShowModeStatus, _>(
                "show_mode_prepare",
                &EventArgs { event_slug: &slug },
            )
            .await
            {
                Ok(value) => {
                    show_message.set(i18n::format(
                        "snapshot_gotowy_value_trwaych_biletow",
                        &[value.eligible_passes.to_string()],
                    ));
                    offline.set(true);
                    show_mode.set(value);
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let sync = move |_| {
        let slug = event_slug.get();
        if slug.is_empty() {
            return;
        }
        busy.set(true);
        show_message.set(String::new());
        spawn_local(async move {
            match bridge::invoke::<ShowModeSyncResult, _>(
                "show_mode_sync",
                &EventArgs { event_slug: &slug },
            )
            .await
            {
                Ok(value) => {
                    show_message.set(i18n::format(
                        "sync_value_zapisane_value_konfliktow_value_nadal_czeka",
                        &[
                            value.synced.to_string(),
                            value.conflicts.to_string(),
                            value.pending.to_string(),
                        ],
                    ));
                    refresh_show_status(slug.clone());
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    let clear = move |_| {
        let slug = event_slug.get();
        if slug.is_empty() {
            return;
        }
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<ShowModeStatus, _>(
                "show_mode_clear",
                &EventArgs { event_slug: &slug },
            )
            .await
            {
                Ok(value) => {
                    show_mode.set(value);
                    offline.set(false);
                    show_message.set(tr("show_data_removed_from_the_device").to_owned());
                }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    view! {
        <section class="screen">
            <header class="screen-title"><p class="eyebrow">GATE MODE</p><h2>{tr("scan_entry")}</h2></header>
            <label class="select-label">{tr("show")}<select disabled=move || loading.get().events prop:value=move || event_slug.get() on:change=select_event><option value="">{move || if loading.get().events { tr("loading_shows") } else { tr("select_an_event") }}</option><For each=move || operator_events(dashboard) key=|event| event.slug.clone() children=move |event| view! { <option value=event.slug.clone()>{event.title}</option> } /></select></label>
            <article class:show-mode-active=move || offline.get() class="show-mode-card">
                <div class="section-head"><div><p class="eyebrow">OFFLINE SHOW MODE</p><h3>{move || if offline.get() { tr("gate_works_locally") } else { tr("works_without_lte") }}</h3></div><button type="button" class:active=move || offline.get() disabled=move || !show_mode.get().prepared || busy.get() on:click=move |_| offline.update(|value| *value = !*value)>{move || if offline.get() { tr("offline_on") } else { tr("offline_off") }}</button></div>
                <p>{move || if show_mode.get().prepared { i18n::format("tickets_pending_conflicts", &[show_mode.get().eligible_passes.to_string(), show_mode.get().pending.to_string(), show_mode.get().conflicts.to_string()]) } else { tr("download_a_secure_snapshot_before_opening_the").to_owned() }}</p>
                <Show when=move || show_mode.get().prepared>
                    <div class="show-mode-status-grid" aria-label=tr("offline_show_mode_status")>
                        <article><strong>{move || show_mode.get().eligible_passes}</strong><span>{tr("eligible_tickets")}</span></article>
                        <article><strong>{move || show_mode.get().pending}</strong><span>{tr("pending_scans")}</span></article>
                        <article><strong>{move || show_mode.get().synced}</strong><span>{tr("synced_scans")}</span></article>
                        <article><strong>{move || show_mode.get().conflicts}</strong><span>{tr("scan_conflicts")}</span></article>
                    </div>
                </Show>
                <div class="show-mode-actions"><button type="button" on:click=prepare disabled=move || busy.get() || event_slug.get().is_empty()>{tr("prepare_offline")}</button><button type="button" on:click=sync disabled=move || busy.get() || !show_mode.get().prepared>{tr("sync")}</button><button type="button" class="danger ghost" on:click=clear disabled=move || busy.get() || !show_mode.get().prepared>{tr("clear")}</button></div>
                <Show when=move || !show_message.get().is_empty()><small>{move || show_message.get()}</small></Show>
            </article>
            <button class="scanner-button" on:click=scan disabled=move || busy.get()><span class="scanner-frame"></span><strong>{move || if busy.get() { tr("verifying") } else if offline.get() { tr("scan_locally") } else { tr("open_camera") }}</strong><small>{move || if offline.get() { tr("durable_t1_ticket_qr_only") } else { tr("ticket_or_admission_pass_qr") }}</small></button>
            <Show when=move || !offline.get()><div class="manual-row"><input placeholder=tr("qr_code_or_admission_pass_number") prop:value=move || manual.get() on:input=move |e| manual.set(event_target_value(&e))/><button on:click=manual_submit disabled=move || busy.get()>{tr("check")}</button></div></Show>
            {move || result.get().map(|entry| {
                let success = matches!(
                    entry.status.as_str(),
                    "redeemed" | "already_redeemed" | "offline_queued" | "offline_duplicate"
                );
                let denied = matches!(entry.status.as_str(), "revoked" | "expired" | "not_claimed");
                view! { <article class:scan-success=success class:scan-warning=!success && !denied class:scan-denied=denied class="scan-result"><strong>{scan_status_label(&entry.status)}</strong><span>{entry.public_reference}</span><p>{entry.holder_name.value_or(entry.holder_email_masked)}</p></article> }
            })}
        </section>
    }
}
