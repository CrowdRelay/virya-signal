#[component]
fn AutopilotPanel(
    overview: RwSignal<Option<OperatorAutopilotOverview>>,
    loading: RwSignal<bool>,
    chief: RwSignal<Option<AutopilotChiefOfStaff>>,
    chief_loading: RwSignal<bool>,
    refresh_requested: RwSignal<u32>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let refresh = move |_| {
        refresh_requested.update(|value| *value = value.wrapping_add(1).max(1));
    };
    view! {
        <section class="ops-panel autopilot-panel">
            <div class="section-head">
                <div><p class="eyebrow">VIRYAOS AUTOPILOT</p><h3>{tr("autopilot_control")}</h3></div>
                <button class="text-button" on:click=refresh disabled=move || loading.get()>{tr("refresh_2")}</button>
            </div>
            <Show when=move || !chief_loading.get() fallback=move || view! { <Skeleton rows=2 /> }>
                {move || chief.get().map(|brief| {
                    let attention = brief.attention_items.into_iter().take(6).collect::<Vec<_>>();
                    let opportunities = brief.top_opportunities.into_iter().take(5).collect::<Vec<_>>();
                    let show_tasks = brief.show_tasks.into_iter().take(5).collect::<Vec<_>>();
                    let attention_view = (!attention.is_empty()).then(|| view! {
                        <div class="section-head"><h3>{tr("autopilot_deadline_radar")}</h3></div>
                        <div class="ops-list"><For each=move || attention.clone() key=|item| format!("{}:{}:{}", item.kind, item.subject_kind, item.subject_id) children=move |item| {
                            let title = if item.kind == "approval" { autopilot_action_kind_label(&item.title).to_string() } else { item.title.clone() };
                            let detail = if item.kind == "approval" { autopilot_context_label(&item.detail).to_string() } else { item.detail.clone() };
                            let due_at = human_time(&item.due_at);
                            view! { <article class=format!("ops-item autopilot-attention attention-{}", item.urgency)><div><strong>{title}</strong><p>{detail}</p><small>{format!("{} · {} · {}", autopilot_attention_label(&item.kind), autopilot_urgency_label(&item.urgency), due_at)}</small></div></article> }
                        } /></div>
                    });
                    let opportunities_view = (!opportunities.is_empty()).then(|| view! {
                        <div class="section-head"><h3>{tr("autopilot_opportunities")}</h3></div>
                        <div class="ops-list"><For each=move || opportunities.clone() key=|item| format!("{}:{}:{}", item.context, item.subject_kind, item.subject_id) children=move |item| view! {
                            <article class="ops-item"><div><strong>{autopilot_context_label(&item.context)}</strong><p>{item.reason}</p><small>{format!("{}% · {}", item.confidence / 100, if item.needs_approval { tr("autopilot_approval") } else { item.decision_kind.as_str() })}</small></div></article>
                        } /></div>
                    });
                    let show_tasks_view = (!show_tasks.is_empty()).then(|| view! {
                        <div class="section-head"><h3>{tr("autopilot_show_tasks")}</h3></div>
                        <div class="ops-list"><For each=move || show_tasks.clone() key=|item| format!("{}:{}", item.event_id, item.task_key) children=move |item| view! {
                            <article class="ops-item"><div><strong>{item.event_title}</strong><p>{autopilot_show_task_label(&item.task_key)}</p><small>{item.status}</small></div></article>
                        } /></div>
                    });
                    view! {
                        <div class="autopilot-chief">
                            <div class="section-head"><h3>{tr("autopilot_chief")}</h3><span>{format!("{}", brief.needs_you)}</span></div>
                            <div class="ops-metrics">
                                <Metric value=brief.executed_24h.to_string() label=tr("autopilot_actions_24h") />
                                <Metric value=format!("~{}m", brief.estimated_minutes_saved_24h) label=tr("autopilot_time_saved") />
                                <Metric value=brief.measured_improved_7d.to_string() label=tr("autopilot_improved_7d") />
                                <Metric value=brief.needs_you.to_string() label=tr("autopilot_needs_you") />
                            </div>
                            {attention_view}
                            {opportunities_view}
                            {show_tasks_view}
                        </div>
                    }
                })}
            </Show>
            <Show when=move || !loading.get() fallback=move || view! { <Skeleton rows=3 /> }>
                {move || overview.get().map(|data| {
                    let runtime_enabled = data.runtime_enabled;
                    let policies = data.policies;
                    let promotion_budget_guardrails = data.promotion_budget_guardrails;
                    let needs_you = data.needs_you;
                    let available_assignees = data.available_assignees;
                    let recent_actions = data.recent_actions.into_iter().take(10).collect::<Vec<_>>();
                    let recent_effects = data.recent_effects.into_iter().take(6).collect::<Vec<_>>();
                    let queue = data.queued_actions.saturating_add(data.processing_actions);
                    let release_ledger = data.release_ledger;
                    let rum_metrics = data.rum_metrics_24h;
                    let release_drift = release_ledger.backend_sha_drift
                        || !release_ledger.missing_components.is_empty()
                        || release_ledger.active_executor_count == 0
                        || release_ledger.guarded_executor_count > 0
                        || release_ledger.active_executor_manifest_shas.len() > 1
                        || release_ledger.components.iter().any(|component| component.stale);
                    let release_components = release_ledger.components.clone();
                    let release_missing = release_ledger.missing_components.join(", ");
                    let release_view = view! {
                        <div class="section-head"><h3>{tr("autopilot_release_ledger")}</h3><span>{if release_drift { tr("autopilot_release_drift") } else { tr("autopilot_release_sync") }}</span></div>
                        <div class="ops-metrics">
                            <Metric value=release_ledger.active_executor_count.to_string() label=tr("autopilot_n8n_executors") />
                            <Metric value=release_ledger.guarded_executor_count.to_string() label=tr("autopilot_executor_guards") />
                            <Metric value=release_missing.clone() label=tr("autopilot_release_missing") />
                        </div>
                        <div class="ops-list"><For each=move || release_components.clone() key=|component| component.component_key.clone() children=move |component| view! {
                            <article class="ops-item"><div>
                                <strong>{component.component_key}</strong>
                                <p>{component.version.unwrap_or_else(|| component.source_sha.chars().take(12).collect())}</p>
                                <small>{if component.stale { tr("autopilot_release_stale") } else { tr("autopilot_release_production") }}</small>
                            </div></article>
                        } /></div>
                    };
                    let rum_view = (!rum_metrics.is_empty()).then(|| view! {
                        <div class="section-head"><h3>{tr("autopilot_rum_24h")}</h3><span>{rum_metrics.len()}</span></div>
                        <div class="ops-list"><For each=move || rum_metrics.clone() key=|metric| format!("{}:{}", metric.surface, metric.metric_key) children=move |metric| view! {
                            <article class="ops-item"><div>
                                <strong>{autopilot_rum_metric_label(&metric.surface, &metric.metric_key)}</strong>
                                <p>{autopilot_rum_value(&metric.metric_key, metric.p75, metric.p95)}</p>
                                <small>{format!("{} {}", metric.samples_24h, tr("autopilot_samples"))}</small>
                            </div></article>
                        } /></div>
                    });
                    let runtime_view = (!runtime_enabled).then(|| view! {
                        <p class="security-note warning">{tr("autopilot_runtime_disabled")}</p>
                    });
                    let needs_view = if needs_you.is_empty() {
                        view! { <p class="ops-healthy">{tr("autopilot_nothing_needs_you")}</p> }.into_any()
                    } else {
                        view! {
                            <div class="section-head"><h3>{tr("autopilot_needs_you")}</h3><span>{needs_you.len()}</span></div>
                            <div class="ops-list">
                                <For each=move || needs_you.clone() key=|action| action.id.clone() children={
                                    let available_assignees = available_assignees.clone();
                                    move |action| view! {
                                        <AutopilotPendingCard action=action available_assignees=available_assignees.clone() loading=loading refresh_requested=refresh_requested error=error />
                                    }
                                } />
                            </div>
                        }.into_any()
                    };
                    let effects_view = (!recent_effects.is_empty()).then(|| view! {
                        <div class="section-head"><h3>{tr("autopilot_measured_effects")}</h3></div>
                        <div class="ops-list"><For each=move || recent_effects.clone() key=|effect| effect.measurement_id.clone() children=move |effect| {
                            let delta = effect.delta_basis_points as f64 / 100.0;
                            view! {
                                <article class="ops-item"><div>
                                    <strong>{autopilot_context_label(&effect.context)}</strong>
                                    <p>{autopilot_measurement_kind_label(&effect.measurement_kind)}</p>
                                    <small>{format!("{} · {delta:+.1}%", autopilot_effect_label(&effect.assessment))}</small>
                                </div></article>
                            }
                        } /></div>
                    });
                    let recent_view = (!recent_actions.is_empty()).then(|| view! {
                        <div class="section-head"><h3>{tr("autopilot_recent_actions")}</h3></div>
                        <div class="ops-list"><For each=move || recent_actions.clone() key=|action| action.id.clone() children=move |action| {
                            let manual = action.manual_steps.iter().take(3)
                                .map(|step| format!("{}: {}", step.destination, step.what_to_do))
                                .collect::<Vec<_>>()
                                .join(" · ");
                            view! {
                                <article class="ops-item"><div>
                                    <strong>{autopilot_context_label(&action.context)}</strong>
                                    <p>{autopilot_action_kind_label(&action.action_kind)}</p>
                                    <small>{format!("{} · #{} · executor:{}", action.status, action.attempt_count, action.executor_status.as_deref().unwrap_or("pending"))}</small>
                                    {(!manual.is_empty()).then(|| view! { <small class="ops-note">{i18n::format("autopilot_manual_steps", std::slice::from_ref(&manual))}</small> })}
                                </div></article>
                            }
                        } /></div>
                    });
                    let guardrails_view = (!promotion_budget_guardrails.is_empty()).then(|| view! {
                        <div class="section-head"><h3>{tr("autopilot_financial_guardrails")}</h3></div>
                        <div class="ops-list"><For each=move || promotion_budget_guardrails.clone() key=|guardrail| guardrail.currency.clone() children=move |guardrail| view! {
                            <article class="ops-item"><div><strong>{guardrail.currency}</strong><p>{format!("≤{:.2}/day · ≤{:.2}/month", guardrail.maximum_total_daily_budget_minor as f64 / 100.0, guardrail.maximum_monthly_spend_minor as f64 / 100.0)}</p><small>{format!("v{}", guardrail.version)}</small></div></article>
                        } /></div>
                    });
                    view! {
                        <div class="ops-metrics">
                            <Metric value=data.succeeded_24h.to_string() label=tr("autopilot_actions_24h") />
                            <Metric value=queue.to_string() label=tr("autopilot_queue") />
                            <Metric value=data.executor_confirmed_24h.to_string() label=tr("autopilot_executor_confirmed") />
                            <Metric value=data.executor_failed_24h.to_string() label=tr("autopilot_executor_failed") />
                            <Metric value=data.failed_24h.to_string() label=tr("autopilot_failed_24h") />
                            <Metric value=if runtime_enabled { "ON".to_owned() } else { "OFF".to_owned() } label="runtime" />
                        </div>
                        {runtime_view}
                        <div class="section-head"><h3>{tr("autopilot_authority")}</h3></div>
                        <div class="ops-list autopilot-policies"><For each=move || policies.clone() key=|policy| format!("{}:{}", policy.context, policy.version) children=move |policy| view! {
                            <AutopilotPolicyCard policy=policy loading=loading refresh_requested=refresh_requested error=error />
                        } /></div>
                        {guardrails_view}
                        {release_view}
                        {rum_view}
                        {needs_view}
                        {effects_view}
                        {recent_view}
                    }
                })}
            </Show>
        </section>
    }
}

#[component]
fn AutopilotPolicyCard(
    policy: AutopilotPolicySummary,
    loading: RwSignal<bool>,
    refresh_requested: RwSignal<u32>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let busy = RwSignal::new(false);
    let current = if policy.enabled { policy.autonomy_level.as_str() } else { "off" };
    let observe_policy = policy.clone();
    let recommend_policy = policy.clone();
    let approval_policy = policy.clone();
    let auto_policy = policy.clone();
    let off_policy = policy.clone();
    view! {
        <article class="ops-item autopilot-policy-card">
            <div>
                <strong>{autopilot_context_label(&policy.context)}</strong>
                <p>{format!("{} · {}% · ≤{}/24h · v{}", current, policy.minimum_confidence / 100, policy.max_actions_24h, policy.version)}</p>
                {policy.guarded_until.as_ref().map(|until| view! {
                    <small class="warning">{format!("{} · {}", tr("autopilot_guarded"), until)}</small>
                })}
            </div>
            <div class="autopilot-policy-actions" role="group" aria-label=tr("autopilot_authority")>
                <button type="button" class="text-button" class:active=current == "off" on:click=move |_| set_autopilot_policy(off_policy.clone(), false, "observe", busy, refresh_requested, error) disabled=move || busy.get() || loading.get()>{tr("autopilot_off")}</button>
                <button type="button" class="text-button" class:active=current == "observe" on:click=move |_| set_autopilot_policy(observe_policy.clone(), true, "observe", busy, refresh_requested, error) disabled=move || busy.get() || loading.get()>{tr("autopilot_observe")}</button>
                <button type="button" class="text-button" class:active=current == "recommend" on:click=move |_| set_autopilot_policy(recommend_policy.clone(), true, "recommend", busy, refresh_requested, error) disabled=move || busy.get() || loading.get()>{tr("autopilot_recommend")}</button>
                <button type="button" class="text-button" class:active=current == "require_approval" on:click=move |_| set_autopilot_policy(approval_policy.clone(), true, "require_approval", busy, refresh_requested, error) disabled=move || busy.get() || loading.get()>{tr("autopilot_approval")}</button>
                <button type="button" class="text-button" class:active=current == "bounded_auto" on:click=move |_| set_autopilot_policy(auto_policy.clone(), true, "bounded_auto", busy, refresh_requested, error) disabled=move || busy.get() || loading.get()>{tr("autopilot_auto")}</button>
            </div>
        </article>
    }
}

include!("autopilot_cards.rs");

fn set_autopilot_policy(
    policy: AutopilotPolicySummary,
    enabled: bool,
    autonomy_level: &'static str,
    busy: RwSignal<bool>,
    refresh_requested: RwSignal<u32>,
    error: RwSignal<Option<String>>,
) {
    if busy.get_untracked() { return; }
    busy.set(true);
    spawn_local(async move {
        let result = bridge::invoke_timeout::<AutopilotMutation, _>(
            "operator_autopilot_set_authority",
            &AutopilotAuthorityArgs {
                context: &policy.context,
                enabled,
                autonomy_level,
                minimum_confidence_basis_points: policy.minimum_confidence,
                max_actions_24h: policy.max_actions_24h,
                expected_version: policy.version,
            },
            15_000,
        ).await;
        match result {
            Ok(_) => {
                busy.set(false);
                refresh_requested.update(|value| *value = value.wrapping_add(1).max(1));
            }
            Err(message) => { busy.set(false); error.set(Some(message)); }
        }
    });
}

fn assign_autopilot_action(
    action_id: String,
    member_key: String,
    busy: RwSignal<bool>,
    refresh_requested: RwSignal<u32>,
    error: RwSignal<Option<String>>,
) {
    if busy.get_untracked() || member_key.is_empty() { return; }
    busy.set(true);
    spawn_local(async move {
        let result = bridge::invoke::<AutopilotMutation, _>(
            "operator_autopilot_assign",
            &AutopilotAssignArgs { action_id: &action_id, member_key: &member_key },
        ).await;
        match result {
            Ok(_) => {
                busy.set(false);
                refresh_requested.update(|value| *value = value.wrapping_add(1).max(1));
            }
            Err(message) => { busy.set(false); error.set(Some(message)); }
        }
    });
}

fn mutate_autopilot_action(
    command: &'static str,
    action_id: String,
    busy: RwSignal<bool>,
    refresh_requested: RwSignal<u32>,
    error: RwSignal<Option<String>>,
) {
    if busy.get_untracked() { return; }
    busy.set(true);
    spawn_local(async move {
        let result = match command {
            "operator_autopilot_approve" => bridge::invoke::<AutopilotMutation, _>(
                "operator_autopilot_approve", &AutopilotActionArgs { action_id: &action_id }
            ).await,
            "operator_autopilot_cancel" => bridge::invoke::<AutopilotMutation, _>(
                "operator_autopilot_cancel", &AutopilotActionArgs { action_id: &action_id }
            ).await,
            _ => Err(tr("autopilot_invalid_action").to_owned()),
        };
        match result {
            Ok(_) => {
                busy.set(false);
                refresh_requested.update(|value| *value = value.wrapping_add(1).max(1));
            }
            Err(message) => { busy.set(false); error.set(Some(message)); }
        }
    });
}

include!("autopilot_labels.rs");
