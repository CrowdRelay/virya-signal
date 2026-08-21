#[component]
fn BeaconSettings(
    home: RwSignal<Option<BeaconHomeData>>,
    refresh: RwSignal<u32>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let radius = RwSignal::new(100_i32);
    let nearby = RwSignal::new(true);
    let topics = RwSignal::new(default_beacon_topics());
    let initialized = RwSignal::new(false);
    let busy = RwSignal::new(false);
    let danger_action = RwSignal::new(None::<u8>);
    let push = RwSignal::new(None::<FanPushStatus>);
    let resume_refresh = RwSignal::new(0_u32);
    let resume_pending = RwSignal::new(false);
    install_resume_refresh(resume_refresh);

    Effect::new(move |_| {
        if !initialized.get()
            && let Some(data) = home.get()
        {
            radius.set(data.preferences.radius_km);
            nearby.set(data.preferences.nearby_gigs_enabled);
            topics.set(data.preferences.topics);
            initialized.set(true);
        }
    });

    Effect::new(move |_| {
        resume_refresh.get();
        if !bridge::native_available() || busy.get_untracked() {
            if busy.get_untracked() {
                resume_pending.set(true);
            }
            return;
        }
        resume_pending.set(false);
        busy.set(true);
        spawn_lifecycle_task(async move {
            match bridge::invoke_latest::<FanPushStatus, _>(
                "beacon_push_sync",
                &EmptyArgs {},
                15_000,
                "beacon:push-sync",
            )
            .await
            {
                Ok(Some(value)) => {
                    let _ = push.try_set(Some(value));
                }
                Ok(None) => {}
                Err(message) => {
                    let _ = error.try_set(Some(message));
                }
            }
            finish_resumable_ui_task(busy, resume_pending, resume_refresh);
        });
    });

    let toggle_topic = move |topic: &'static str| {
        topics.update(|values| {
            if let Some(index) = values.iter().position(|value| value == topic) {
                if values.len() > 1 {
                    values.remove(index);
                }
            } else {
                values.push(topic.to_owned());
            }
        });
    };

    let save = move |_| {
        let current_topics = topics.get_untracked();
        busy.set(true);
        spawn_local(async move {
            match bridge::invoke::<crate::models::BeaconPreferences, _>(
                "beacon_preferences_update",
                &BeaconPreferencesArgs {
                    radius_km: radius.get_untracked(),
                    locale: i18n::current().code(),
                    topics: &current_topics,
                    nearby_gigs_enabled: nearby.get_untracked(),
                },
            )
            .await
            {
                Ok(_) => refresh.update(|value| *value = value.wrapping_add(1)),
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    // Same rule as the fan card: the choice is local and instant, so the card
    // shows it now and the FCM/backend registration catches up in the task.
    let desired_push = RwSignal::new(None::<bool>);
    let shown_push_enabled = move || {
        desired_push
            .get()
            .unwrap_or_else(|| push.get().is_some_and(|value| value.enabled))
    };
    let toggle_push = move |_| {
        if busy.get_untracked() {
            return;
        }
        let current = push.get_untracked();
        let opens_settings = current
            .as_ref()
            .is_some_and(|value| !value.enabled && value.permission == "denied");
        let want_enabled = !current.as_ref().is_some_and(|value| value.enabled);
        let command = if opens_settings {
            "beacon_push_open_settings"
        } else if want_enabled {
            "beacon_push_enable"
        } else {
            "beacon_push_disable"
        };
        busy.set(true);
        if opens_settings {
            resume_pending.set(true);
        } else {
            desired_push.set(Some(want_enabled));
        }
        spawn_lifecycle_task(async move {
            match bridge::invoke_timeout::<FanPushStatus, _>(command, &EmptyArgs {}, 45_000).await {
                Ok(value) => {
                    let _ = push.try_set(Some(value));
                }
                Err(message) => {
                    let _ = error.try_set(Some(message));
                }
            }
            let _ = desired_push.try_set(None);
            finish_resumable_ui_task(busy, resume_pending, resume_refresh);
        });
    };

    let run_danger_action = move |action: u8| {
        if busy.get_untracked() { return; }
        busy.set(true);
        danger_action.set(None);
        spawn_local(async move {
            let result = match action {
                1 => bridge::invoke::<BeaconSessionStatus, _>("beacon_logout", &EmptyArgs {}).await,
                2 => bridge::invoke::<BeaconSessionStatus, _>("beacon_leave", &BeaconLeaveArgs { do_not_contact: false }).await,
                3 => bridge::invoke::<BeaconSessionStatus, _>("beacon_leave", &BeaconLeaveArgs { do_not_contact: true }).await,
                _ => { busy.set(false); return; }
            };
            match result {
                Ok(_) => { let _ = web_reload(); }
                Err(message) => error.set(Some(message)),
            }
            busy.set(false);
        });
    };

    view! {
        <article class="beacon-settings">
            <p class="eyebrow">{tr("latarnik_settings")}</p>
            <h3>{tr("latarnik_preferences")}</h3>
            <div class="radius-picker beacon-radius">
                <button class:active=move || radius.get() == 25 on:click=move |_| radius.set(25)>"25 km"</button>
                <button class:active=move || radius.get() == 50 on:click=move |_| radius.set(50)>"50 km"</button>
                <button class:active=move || radius.get() == 100 on:click=move |_| radius.set(100)>"100 km"</button>
                <button class:active=move || radius.get() == 200 on:click=move |_| radius.set(200)>"200 km"</button>
                <button class:active=move || radius.get() == 300 on:click=move |_| radius.set(300)>"300 km"</button>
            </div>
            <div class="beacon-topic-grid">
                <button type="button" class:active=move || topics.get().iter().any(|value| value == "shows") on:click=move |_| toggle_topic("shows")>{tr("latarnik_topic_shows")}</button>
                <button type="button" class:active=move || topics.get().iter().any(|value| value == "press_materials") on:click=move |_| toggle_topic("press_materials")>{tr("latarnik_topic_press")}</button>
                <button type="button" class:active=move || topics.get().iter().any(|value| value == "releases") on:click=move |_| toggle_topic("releases")>{tr("latarnik_topic_releases")}</button>
                <button type="button" class:active=move || topics.get().iter().any(|value| value == "interviews") on:click=move |_| toggle_topic("interviews")>{tr("latarnik_topic_interviews")}</button>
                <button type="button" class:active=move || topics.get().iter().any(|value| value == "accreditation") on:click=move |_| toggle_topic("accreditation")>{tr("latarnik_topic_accreditation")}</button>
            </div>
            <label class="check-label"><input type="checkbox" prop:checked=move || nearby.get() on:change=move |e| nearby.set(event_target_checked(&e))/><span>{tr("latarnik_nearby_push")}</span></label>
            <button class="ghost" disabled=move || busy.get() on:click=save>{tr("save")}</button>
            <Show when=move || bridge::native_available()>
                <div class="push-setting-card beacon-push-setting">
                    <div class="push-setting-copy">
                        <strong>{tr("push_notifications")}</strong>
                        {move || {
                            let on = shown_push_enabled();
                            let message = match push.get() {
                                _ if desired_push.get().is_some() => tr("syncing_push_notifications"),
                                Some(current) if !current.supported => tr("push_notifications_degraded"),
                                Some(current) if !current.backend_enabled => tr("push_notifications_waiting_backend"),
                                Some(current) if current.permission == "denied" => tr("push_notifications_blocked"),
                                Some(current) if current.enabled => tr("push_notifications_on"),
                                Some(_) => tr("push_notifications_off"),
                                None => tr("syncing_push_notifications"),
                            };
                            view! { <small class:success=on class:warning=!on>{message}</small> }
                        }}
                    </div>
                    <button class="ghost" disabled=move || busy.get() on:click=toggle_push>
                        {move || if push.get().as_ref().is_some_and(|value| value.permission == "denied")
                            && desired_push.get().is_none()
                        {
                            tr("open_notification_settings")
                        } else if shown_push_enabled() {
                            tr("disable_push_notifications")
                        } else {
                            tr("enable_push_notifications")
                        }}
                    </button>
                </div>
            </Show>
            <div class="beacon-danger-zone">
                <button class="text-button" disabled=move || busy.get() on:click=move |_| danger_action.set(Some(1))>{tr("latarnik_logout_device")}</button>
                <button class="text-button danger" disabled=move || busy.get() on:click=move |_| danger_action.set(Some(2))>{tr("latarnik_leave")}</button>
                <button class="text-button danger" disabled=move || busy.get() on:click=move |_| danger_action.set(Some(3))>{tr("latarnik_do_not_contact")}</button>
                <Show when=move || danger_action.get().is_some()>
                    <div class="beacon-delivery-form beacon-danger-confirm" role="alert">
                        <strong>{move || match danger_action.get() { Some(1) => tr("latarnik_logout_device"), Some(2) => tr("latarnik_leave"), _ => tr("latarnik_do_not_contact") }}</strong>
                        <p>{move || match danger_action.get() { Some(1) => tr("latarnik_logout_confirm_hint"), Some(2) => tr("latarnik_leave_confirm_hint"), _ => tr("latarnik_dnc_confirm_hint") }}</p>
                        <div class="confirmation-actions">
                            <button class="danger" disabled=move || busy.get() on:click=move |_| if let Some(action)=danger_action.get_untracked() { run_danger_action(action) }>{tr("latarnik_confirm_action")}</button>
                            <button class="ghost" disabled=move || busy.get() on:click=move |_| danger_action.set(None)>{tr("cancel")}</button>
                        </div>
                    </div>
                </Show>
            </div>
        </article>
    }
}

fn web_reload() -> Result<(), JsValue> {
    let global=js_sys::global(); let location=js_sys::Reflect::get(&global,&JsValue::from_str("location"))?; let reload=js_sys::Reflect::get(&location,&JsValue::from_str("reload"))?.dyn_into::<js_sys::Function>()?; reload.call0(&location).map(|_|())
}
