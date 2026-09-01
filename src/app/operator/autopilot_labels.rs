fn autopilot_context_label(context: &str) -> &'static str {
    match context {
        "ticket_yield" => tr("autopilot_context_ticket_yield"),
        "fan_lifecycle" => tr("autopilot_context_fan_lifecycle"),
        "campaign_lifecycle" => tr("autopilot_context_campaign_lifecycle"),
        "merchandising" => tr("autopilot_context_merchandising"),
        "merch_pricing" => tr("autopilot_context_merch_pricing"),
        "merch_bundle" => tr("autopilot_context_merch_bundle"),
        "booking_opportunity" => tr("autopilot_context_booking_opportunity"),
        "outreach" => tr("autopilot_context_outreach"),
        "beacon" => tr("autopilot_context_beacon"),
        "show_growth" => tr("autopilot_context_show_growth"),
        "content_supply" => tr("autopilot_context_content_supply"),
        "promotion_budget" => tr("autopilot_context_promotion_budget"),
        "experimentation" => tr("autopilot_context_experimentation"),
        "show_operations" => tr("autopilot_context_show_operations"),
        "release" => tr("autopilot_context_release"),
        "live_opportunity" => tr("autopilot_context_live_opportunity"),
        "funding" => tr("autopilot_context_funding"),
        _ => tr("autopilot_context_default"),
    }
}

fn autopilot_action_kind_label(kind: &str) -> &'static str {
    match kind {
        "ticket.price.change" => tr("autopilot_action_ticket_price_change"),
        "ticket.capacity.change" => tr("autopilot_action_ticket_capacity_change"),
        "fan.lifecycle.message.request" => tr("autopilot_action_fan_lifecycle_message"),
        "audience.campaign.request" => tr("autopilot_action_audience_campaign"),
        "merch.reorder.request" => tr("autopilot_action_merch_reorder"),
        "merch.price.change" => tr("autopilot_action_merch_price_change"),
        "merch.bundle.request" => tr("autopilot_action_merch_bundle"),
        "booking.outreach.request" => tr("autopilot_action_booking_outreach"),
        "outreach.request" => tr("autopilot_action_outreach"),
        "beacon.discovery.request" => tr("autopilot_action_beacon_discovery"),
        "beacon.outreach.request" => tr("autopilot_action_beacon_outreach"),
        "show.growth.request" => tr("autopilot_action_show_growth"),
        "content.artifact.request" => tr("autopilot_action_content_artifact"),
        "experiment.allocation.change" => tr("autopilot_action_experiment_reallocation"),
        "experiment.complete" => tr("autopilot_action_experiment_winner"),
        "show.task.complete" => tr("autopilot_action_show_task_complete"),
        "show.task.escalate" => tr("autopilot_action_show_task_escalate"),
        "promotion.budget_change.request" => tr("autopilot_action_promotion_budget_change"),
        "release.milestone.execute" => tr("autopilot_action_release_milestone"),
        "opportunity.live.apply" => tr("autopilot_action_live_opportunity_apply"),
        "funding.package.prepare" => tr("autopilot_action_funding_package_prepare"),
        "funding.application.submit" => tr("autopilot_action_funding_application_submit"),
        "team.assignment.email" => tr("autopilot_action_team_assignment_email"),
        "playlist.placement.verify" => tr("autopilot_action_playlist_placement_verify"),
        "beacon.invite_batch.request" => tr("autopilot_action_beacon_invite_batch"),
        "outreach.discovery.request" => tr("autopilot_action_outreach_discovery"),
        "booking.target_discovery.request" => tr("autopilot_action_booking_target_discovery"),
        "release.editorial_pitch.escalate" => tr("autopilot_action_editorial_pitch_escalate"),
        "opportunity.terms.counter" => tr("autopilot_action_opportunity_terms_counter"),
        "opportunity.terms.accept" => tr("autopilot_action_opportunity_terms_accept"),
        "growth.opportunity.raise" => tr("autopilot_action_growth_opportunity_raise"),
        "growth.debt.raise" => tr("autopilot_action_growth_debt_raise"),
        "referral.code.issue" => tr("autopilot_action_referral_code_issue"),
        "play.step.run" => tr("autopilot_action_play_step_run"),
        "agent.content.request" => tr("autopilot_action_agent_content_request"),
        "agent.run.request" => tr("autopilot_action_agent_run_request"),
        "community.engage.request" => tr("autopilot_action_community_engage_request"),
        "signal.push.request" => tr("autopilot_action_signal_push_request"),
        _ => tr("autopilot_action_default"),
    }
}

fn autopilot_measurement_kind_label(kind: &str) -> &'static str {
    match kind {
        "ticket_revenue_72h" => tr("autopilot_metric_ticket_revenue_72h"),
        "merch_gross_proxy_7d" => tr("autopilot_metric_merch_gross_proxy_7d"),
        "promotion_roas_7d" => tr("autopilot_metric_promotion_roas_7d"),
        "booking_reply_7d" => tr("autopilot_metric_booking_reply_7d"),
        "outreach_reply_7d" => tr("autopilot_metric_outreach_reply_7d"),
        "audience_ticket_revenue_72h" => tr("autopilot_metric_audience_ticket_revenue_72h"),
        "show_ticket_revenue_7d" => tr("autopilot_metric_show_ticket_revenue_7d"),
        "show_growth_surface_clicks_7d" => tr("autopilot_metric_show_growth_surface_clicks_7d"),
        "show_growth_attributed_ticket_orders_7d" => {
            tr("autopilot_metric_show_growth_attributed_ticket_orders_7d")
        }
        "grassroots_activation_replies_14d" => tr("autopilot_metric_grassroots_activation_replies_14d"),
        _ => tr("autopilot_metric_default"),
    }
}

fn autopilot_effect_label(assessment: &str) -> String {
    match assessment {
        "improved" => tr("autopilot_effect_improved").to_owned(),
        "neutral" => tr("autopilot_effect_neutral").to_owned(),
        "worsened" => tr("autopilot_effect_worsened").to_owned(),
        _ => assessment.to_owned(),
    }
}

fn autopilot_payload_detail(payload: &AutopilotActionPayload) -> String {
    match payload {
        // An action kind newer than this build. Showing the raw kind is more
        // use to an operator than hiding the row, and hiding it was never an
        // option anyway: the alternative this replaced failed the entire list.
        AutopilotActionPayload::Unrecognized { wire_kind } => {
            format!("{wire_kind} · needs a newer app version")
        }
        AutopilotActionPayload::ChangeTicketPrice { from_minor, to_minor, .. } => {
            format!("{:.2} → {:.2} PLN", *from_minor as f64 / 100.0, *to_minor as f64 / 100.0)
        }
        AutopilotActionPayload::ChangeTicketCapacity { from_capacity, to_capacity, .. } => {
            format!("ticket pool {from_capacity} → {to_capacity}")
        }
        AutopilotActionPayload::RequestFanLifecycleMessage { template_key, .. } => template_key.clone(),
        AutopilotActionPayload::RequestMerchReorder { quantity, .. } => format!("Reorder ×{quantity}"),
        AutopilotActionPayload::ChangeMerchPrice { from_minor, to_minor, .. } => {
            format!("{:.2} → {:.2} PLN", *from_minor as f64 / 100.0, *to_minor as f64 / 100.0)
        }
        AutopilotActionPayload::RequestBookingOutreach { target_name, score, phase, .. } => {
            format!("{target_name} · {phase} · opportunity {score}/100")
        }
        AutopilotActionPayload::RequestAudienceCampaign { phase, template_key, .. } => {
            format!("{phase} · {template_key}")
        }
        AutopilotActionPayload::RequestMerchBundle { bundle_price_minor, affinity_basis_points, .. } => {
            format!("{:.2} PLN · affinity {:.1}%", *bundle_price_minor as f64 / 100.0, *affinity_basis_points as f64 / 100.0)
        }
        AutopilotActionPayload::RequestOutreach { target_name, phase, template_key, .. } => {
            format!("{target_name} · {phase} · {template_key}")
        }
        AutopilotActionPayload::RequestBeaconDiscovery { target_count, .. } => {
            i18n::format("autopilot_beacon_discovery_detail", &[target_count.to_string()])
        }
        AutopilotActionPayload::RequestBeaconOutreach { phase, template_key, .. } => {
            format!("Beacon · {phase} · {template_key}")
        }
        AutopilotActionPayload::RequestShowGrowth { lever, template_key, .. } => {
            format!("{} · {} · {template_key}", tr("autopilot_show_growth_prefix"), lever.replace('_', " "))
        }
        AutopilotActionPayload::RequestContentArtifact { artifact, template_key, .. } => {
            format!("{artifact} · {template_key}")
        }
        AutopilotActionPayload::AdjustExperiment { complete, allocations, .. } => {
            let state = if *complete { "winner" } else { "reallocate" };
            format!("{state} · {} variants", allocations.len())
        }
        AutopilotActionPayload::CompleteShowTask { task, .. } => format!("✓ {}", autopilot_show_task_label(task)),
        AutopilotActionPayload::EscalateShowTask { task, .. } => format!("⚠ {}", autopilot_show_task_label(task)),
        AutopilotActionPayload::RequestPromotionBudgetChange { from_minor, to_minor, roas_basis_points, .. } => format!(
            "{:.2} → {:.2} PLN/day · ROAS {:.2}×",
            *from_minor as f64 / 100.0,
            *to_minor as f64 / 100.0,
            *roas_basis_points as f64 / 10_000.0,
        ),
        AutopilotActionPayload::ExecuteReleaseMilestone { title, milestone, .. } => {
            format!("{title} · {milestone}")
        }
        AutopilotActionPayload::ApplyLiveOpportunity { opportunity_kind, score, .. } => {
            format!("{opportunity_kind} · score {score}/100")
        }
        AutopilotActionPayload::PrepareFundingPackage { .. } => tr("autopilot_funding_package_detail").to_owned(),
        AutopilotActionPayload::SubmitFundingApplication { .. } => tr("autopilot_funding_submit_detail").to_owned(),
        AutopilotActionPayload::SendTeamAssignmentEmail {
            recipient_name,
            task_title,
            reminder_number,
            ..
        } => {
            if *reminder_number > 0 {
                i18n::format("autopilot_task_reminder_of", &[reminder_number.to_string(), "3".to_owned()])
                    + " — "
                    + task_title
            } else {
                format!("{recipient_name} — {task_title}")
            }
        }
        AutopilotActionPayload::VerifyPlaylistPlacement { playlist_external_id, track_external_id, checkpoint, .. } => {
            format!("playlist {playlist_external_id} · track {track_external_id} · checkpoint {checkpoint}")
        }
        AutopilotActionPayload::RequestBeaconInviteBatch { event_id, requested_count, .. } => {
            i18n::format("autopilot_beacon_invite_batch_detail", &[requested_count.to_string(), event_id.clone()])
        }
        AutopilotActionPayload::RequestOutreachDiscovery { requested_candidates } => {
            i18n::format("autopilot_outreach_discovery_detail", &[requested_candidates.to_string()])
        }
        AutopilotActionPayload::RequestBookingTargetDiscovery { requested_count } => {
            i18n::format("autopilot_booking_target_discovery_detail", &[requested_count.to_string()])
        }
        AutopilotActionPayload::EscalateEditorialPitch { title, due_at, .. } => {
            format!("{title} · {due_at}")
        }
        AutopilotActionPayload::CounterLiveOpportunityTerms { ask_minor, currency, round, .. } => {
            format!("{:.2} {} · round {round}", *ask_minor as f64 / 100.0, currency)
        }
        AutopilotActionPayload::AcceptLiveOpportunityTerms { fee_minor, currency, .. } => {
            format!("{:.2} {}", *fee_minor as f64 / 100.0, currency)
        }
        AutopilotActionPayload::RaiseGrowthOpportunity { platform, metric_key, signal, .. } => {
            format!("{platform} · {metric_key} · {signal}")
        }
        AutopilotActionPayload::RaiseGrowthDebt { debt_kind, outstanding_items, tracked_items, .. } => {
            format!("{debt_kind} · {outstanding_items}/{tracked_items}")
        }
        AutopilotActionPayload::IssueReferralCode { .. } => tr("autopilot_referral_code_detail").to_owned(),
        AutopilotActionPayload::RunPlayStep { play_kind, step_kind, step_index, .. } => {
            format!("{play_kind} · {step_kind} · step {step_index}")
        }
        AutopilotActionPayload::RequestAgentContent { template_id, .. } => {
            template_id.as_deref().unwrap_or("draft").to_owned()
        }
        AutopilotActionPayload::RequestAgentRun { template_id, tier, .. } => {
            format!("{template_id} · {tier}")
        }
        AutopilotActionPayload::RequestCommunityEngagement { platform, title, .. } => {
            format!("{platform} · {title}")
        }
        AutopilotActionPayload::RequestSignalPush { title, segment, .. } => {
            format!("{title} · {}", segment.as_deref().unwrap_or("all"))
        }
    }
}


fn autopilot_attention_label(kind: &str) -> String {
    match kind {
        "approval" => tr("autopilot_attention_approval").to_owned(),
        "opportunity_deadline" => tr("autopilot_attention_opportunity").to_owned(),
        "funding_deadline" => tr("autopilot_attention_funding").to_owned(),
        _ => kind.to_owned(),
    }
}

fn autopilot_urgency_label(urgency: &str) -> String {
    match urgency {
        "overdue" => tr("autopilot_urgency_overdue").to_owned(),
        "critical" => tr("autopilot_urgency_critical").to_owned(),
        "today" => tr("autopilot_urgency_today").to_owned(),
        "soon" => tr("autopilot_urgency_soon").to_owned(),
        "upcoming" => tr("autopilot_urgency_upcoming").to_owned(),
        _ => urgency.to_owned(),
    }
}

fn autopilot_show_task_label(task: &str) -> &'static str {
    match task {
        "announcement_published" => tr("autopilot_show_task_announcement_published"),
        "ticketing_verified" => tr("autopilot_show_task_ticketing_verified"),
        "staff_assigned" => tr("autopilot_show_task_staff_assigned"),
        "offline_snapshot_ready" => tr("autopilot_show_task_offline_snapshot_ready"),
        "gate_device_charged" => tr("autopilot_show_task_gate_device_charged"),
        "backup_device_ready" => tr("autopilot_show_task_backup_device_ready"),
        "network_tested" => tr("autopilot_show_task_network_tested"),
        "guestlist_checked" => tr("autopilot_show_task_guestlist_checked"),
        "post_show_reconciliation" => tr("autopilot_show_task_post_show_reconciliation"),
        "post_show_report" => tr("autopilot_show_task_post_show_report"),
        _ => tr("autopilot_show_task_default"),
    }
}

fn autopilot_rum_metric_label(surface: &str, metric: &str) -> String {
    let surface_label = match surface {
        "virya_www" => tr("autopilot_rum_surface_virya_www"),
        "synesthesia" => tr("autopilot_rum_surface_synesthesia"),
        "virya_signal" => tr("autopilot_rum_surface_virya_signal"),
        _ => surface,
    };
    let metric_label = match metric {
        "lcp_ms" => tr("autopilot_rum_metric_lcp"),
        "inp_ms" => tr("autopilot_rum_metric_inp"),
        "cls_milli" => tr("autopilot_rum_metric_cls"),
        "ttfb_ms" => tr("autopilot_rum_metric_ttfb"),
        "boot_interactive_ms" => tr("autopilot_rum_metric_boot_interactive"),
        "room_load_ms" => tr("autopilot_rum_metric_room_load"),
        "transition_ms" => tr("autopilot_rum_metric_transition"),
        "frame_hitch_ms" => tr("autopilot_rum_metric_frame_hitch"),
        "cold_start_ms" => tr("autopilot_rum_metric_cold_start"),
        "api_latency_ms" => tr("autopilot_rum_metric_api_latency"),
        "screen_transition_ms" => tr("autopilot_rum_metric_screen_transition"),
        _ => metric,
    };
    format!("{surface_label} · {metric_label}")
}

fn autopilot_rum_value(metric: &str, p75: f64, p95: f64) -> String {
    if metric == "cls_milli" {
        format!("p75 {:.0} · p95 {:.0}", p75, p95)
    } else {
        format!("p75 {:.0}ms · p95 {:.0}ms", p75, p95)
    }
}
