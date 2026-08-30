// Focused Autopilot operator cards kept out of the main staff screen.

#[component]
fn AutopilotPendingCard(
    action: crate::models::PendingAutopilotAction,
    available_assignees: Vec<crate::models::TeamAssigneeSummary>,
    loading: RwSignal<bool>,
    refresh_requested: RwSignal<u32>,
    error: RwSignal<Option<String>>,
    detail_action: RwSignal<Option<crate::models::PendingAutopilotAction>>,
) -> impl IntoView {
    let approve_id = action.id.clone();
    let cancel_id = action.id.clone();
    let assign_id = action.id.clone();
    let busy = RwSignal::new(false);
    let selected_assignee = RwSignal::new(
        action.assignee.as_ref().map(|assignee| assignee.member_key.clone()).unwrap_or_default()
    );
    let assignee_options = available_assignees.clone();
    let detail = autopilot_payload_detail(&action.payload);
    let owner = action.assignee.as_ref().map(|assignee| assignee.display_name.clone());
    let assignment_due = action.assignment_due_at.clone();
    let kind_label = autopilot_action_kind_label(&action.action_kind).to_owned();
    let approval_detail = action.approval_expires_at.as_ref()
        .map(|value| format!("{}: {value}", tr("autopilot_task_expires_label")))
        .unwrap_or_else(|| kind_label.clone());
    let owner_detail = match (owner, assignment_due) {
        (Some(owner), Some(due)) => format!("{} · {} {}", i18n::format("autopilot_task_assigned_to", &[owner]), tr("autopilot_task_due_label"), due),
        (Some(owner), None) => i18n::format("autopilot_task_assigned_to", &[owner]),
        (None, _) => tr("autopilot_task_team_queue").to_owned(),
    };
    let action_for_detail = action.clone();
    view! {
        <article class="ops-item autopilot-pending-card">
            <button class="ops-item-info" type="button" on:click=move |_| detail_action.set(Some(action_for_detail.clone()))>
                <strong>{autopilot_context_label(&action.context)}</strong>
                <p>{detail}</p>
                <small>{owner_detail}</small>
                <small>{approval_detail}</small>
            </button>
            <div class="autopilot-policy-actions">
                <select
                    aria-label=tr("autopilot_assign_to")
                    prop:value=move || selected_assignee.get()
                    on:change=move |event| selected_assignee.set(event_target_value(&event))
                    disabled=move || loading.get() || busy.get()
                >
                    <option value="">{tr("autopilot_assign_to")}</option>
                    <For each=move || assignee_options.clone() key=|assignee| assignee.member_key.clone() children=move |assignee| view! {
                        <option value=assignee.member_key>{assignee.display_name}</option>
                    } />
                </select>
                <button class="text-button" on:click=move |_| assign_autopilot_action(assign_id.clone(), selected_assignee.get_untracked(), busy, refresh_requested, error) disabled=move || loading.get() || busy.get() || selected_assignee.get().is_empty()>{tr("autopilot_assign")}</button>
                <button class="primary" on:click=move |_| mutate_autopilot_action("operator_autopilot_approve", approve_id.clone(), busy, refresh_requested, error) disabled=move || loading.get() || busy.get()>{tr("autopilot_approve")}</button>
                <button class="danger ghost" on:click=move |_| mutate_autopilot_action("operator_autopilot_cancel", cancel_id.clone(), busy, refresh_requested, error) disabled=move || loading.get() || busy.get()>{tr("autopilot_cancel")}</button>
            </div>
        </article>
    }
}

/// Task detail modal — a glassmorphism bottom sheet that shows the full
/// details of a pending autopilot action, including an email preview when
/// the payload is `SendTeamAssignmentEmail`. Triggered by tapping the
/// info area of an `AutopilotPendingCard`.
#[component]
pub fn TaskDetailModal(
    action: Option<crate::models::PendingAutopilotAction>,
    on_close: impl Fn() + Copy + 'static,
) -> impl IntoView {
    let action = match action {
        Some(a) => a,
        None => return view! { <div></div> }.into_any(),
    };
    let context_label = autopilot_context_label(&action.context).to_owned();
    let kind_label = autopilot_action_kind_label(&action.action_kind).to_owned();
    let detail = autopilot_payload_detail(&action.payload);
    let owner_detail = action.assignee.as_ref().map(|a| a.display_name.clone());
    let due_detail = action.assignment_due_at.clone();
    let approval_detail = action.approval_expires_at.clone();

    // Extract email preview data if the payload is SendTeamAssignmentEmail.
    let email_preview = match &action.payload {
        crate::models::AutopilotActionPayload::SendTeamAssignmentEmail {
            recipient_name,
            task_title,
            task_detail,
            due_at,
            action_url_path,
            reminder_number,
            ..
        } => Some(EmailPreviewData {
            recipient_name: recipient_name.clone(),
            subject: task_title.clone(),
            body: task_detail.clone(),
            due_at: due_at.clone(),
            action_url: action_url_path.clone(),
            reminder_number: *reminder_number,
        }),
        _ => None,
    };

    view! {
        <div class="modal-backdrop" on:click=move |_| on_close()></div>
        <div class="modal-sheet" role="dialog" aria-modal="true">
            <div class="modal-sheet-head">
                <div>
                    <p class="modal-eyebrow">{kind_label}</p>
                    <h3>{context_label}</h3>
                </div>
                <button class="modal-close" aria-label=tr("close") on:click=move |_| on_close()>"×"</button>
            </div>
            <div class="modal-sheet-body">
                <p class="modal-detail">{detail}</p>

                <div class="modal-meta">
                    {owner_detail.map(|owner| view! {
                        <div class="modal-meta-row">
                            <span class="modal-meta-icon">"👤"</span>
                            <span>{i18n::format("autopilot_task_assigned_to", &[owner])}</span>
                        </div>
                    })}
                    {due_detail.map(|due| view! {
                        <div class="modal-meta-row">
                            <span class="modal-meta-icon">"📅"</span>
                            <span>{format!("{}: {}", tr("autopilot_task_due_label"), due)}</span>
                        </div>
                    })}
                    {approval_detail.map(|exp| view! {
                        <div class="modal-meta-row">
                            <span class="modal-meta-icon">"⏳"</span>
                            <span>{format!("{}: {}", tr("autopilot_task_expires_label"), exp)}</span>
                        </div>
                    })}
                </div>

                // Email preview section — only shown for SendTeamAssignmentEmail.
                {email_preview.map(|email| {
                    let recipient = email.recipient_name.clone();
                    let subject = email.subject.clone();
                    let body = email.body.clone();
                    let due = email.due_at.clone();
                    let url = email.action_url.clone();
                    let reminder = email.reminder_number;
                    let reminder_label = i18n::format("autopilot_task_reminder_of", &[reminder.to_string(), "3".to_owned()]);
                    view! {
                        <div class="email-preview">
                            <div class="email-preview-head">
                                <div class="email-preview-row">
                                    <span class="email-preview-label">{tr("to")}</span>
                                    <span>{recipient}</span>
                                </div>
                                <div class="email-preview-row">
                                    <span class="email-preview-label">{tr("subject")}</span>
                                    <strong>{subject}</strong>
                                </div>
                                {due.map(|d| view! {
                                    <div class="email-preview-row">
                                        <span class="email-preview-label">{tr("due")}</span>
                                        <span>{d}</span>
                                    </div>
                                })}
                            </div>
                            <div class="email-preview-body">{body}</div>
                            <div class="email-preview-meta">
                                <span class="email-preview-reminder">{reminder_label}</span>
                                <a class="email-preview-link" href={url}>{tr("autopilot_open_in_panel")}" →"</a>
                            </div>
                        </div>
                    }
                })}
            </div>
        </div>
    }.into_any()
}

struct EmailPreviewData {
    recipient_name: String,
    subject: String,
    body: String,
    due_at: Option<String>,
    action_url: String,
    reminder_number: u8,
}
