// Focused Autopilot operator cards kept out of the main staff screen.

#[component]
fn AutopilotPendingCard(
    action: crate::models::PendingAutopilotAction,
    available_assignees: Vec<crate::models::TeamAssigneeSummary>,
    overview: RwSignal<Option<OperatorAutopilotOverview>>,
    loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) -> impl IntoView {
    let approve_id = action.id.clone();
    let cancel_id = action.id.clone();
    let assign_id = action.id.clone();
    let selected_assignee = RwSignal::new(
        action.assignee.as_ref().map(|assignee| assignee.member_key.clone()).unwrap_or_default()
    );
    let assignee_options = available_assignees.clone();
    let detail = autopilot_payload_detail(&action.payload);
    let owner = action.assignee.as_ref().map(|assignee| assignee.display_name.clone());
    let assignment_due = action.assignment_due_at.clone();
    let approval_detail = action.approval_expires_at.as_ref()
        .map(|value| format!("{}: {value}", tr("autopilot_expires")))
        .unwrap_or(action.action_kind.clone());
    let owner_detail = match (owner, assignment_due) {
        (Some(owner), Some(due)) => format!("Owner: {owner} · due {due}"),
        (Some(owner), None) => format!("Owner: {owner}"),
        (None, _) => "Owner: team queue".to_owned(),
    };
    view! {
        <article class="ops-item autopilot-pending-card">
            <div><strong>{autopilot_context_label(&action.context)}</strong><p>{detail}</p><small>{owner_detail}</small><small>{approval_detail}</small></div>
            <div class="autopilot-policy-actions">
                <select
                    aria-label=tr("autopilot_assign_to")
                    prop:value=move || selected_assignee.get()
                    on:change=move |event| selected_assignee.set(event_target_value(&event))
                    disabled=move || loading.get()
                >
                    <option value="">{tr("autopilot_assign_to")}</option>
                    <For each=move || assignee_options.clone() key=|assignee| assignee.member_key.clone() children=move |assignee| view! {
                        <option value=assignee.member_key>{assignee.display_name}</option>
                    } />
                </select>
                <button class="text-button" on:click=move |_| assign_autopilot_action(assign_id.clone(), selected_assignee.get_untracked(), overview, loading, error) disabled=move || loading.get() || selected_assignee.get().is_empty()>{tr("autopilot_assign")}</button>
                <button class="primary" on:click=move |_| mutate_autopilot_action("operator_autopilot_approve", approve_id.clone(), overview, loading, error) disabled=move || loading.get()>{tr("autopilot_approve")}</button>
                <button class="danger ghost" on:click=move |_| mutate_autopilot_action("operator_autopilot_cancel", cancel_id.clone(), overview, loading, error) disabled=move || loading.get()>{tr("autopilot_cancel")}</button>
            </div>
        </article>
    }
}
