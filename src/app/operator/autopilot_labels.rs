fn autopilot_context_label(context: &str) -> &'static str {
    match context {
        "ticket_yield" => "Ticket Yield",
        "fan_lifecycle" => "Fan Lifecycle",
        "campaign_lifecycle" => "Campaign Lifecycle",
        "merchandising" => "Merch Stock",
        "merch_pricing" => "Merch Yield",
        "merch_bundle" => "Merch Bundles",
        "booking_opportunity" => "Gig Opportunity",
        "outreach" => "Relationship Outreach",
        "beacon" => "Local Beacon",
        "show_growth" => "Attendance Growth",
        "content_supply" => "Content Supply",
        "promotion_budget" => "Promotion Yield",
        "experimentation" => "Experiments",
        "show_operations" => "Show Operations",
        "release" => "Release Autopilot",
        "live_opportunity" => "Festival & Opportunity",
        "funding" => "Funding Autopilot",
        _ => "Autopilot",
    }
}

fn autopilot_action_kind_label(kind: &str) -> &'static str {
    match kind {
        "ticket.price.change" => "Ticket price changed",
        "ticket.capacity.change" => "Ticket pool expanded",
        "fan.lifecycle.message.request" => "Fan lifecycle message",
        "audience.campaign.request" => "Audience campaign",
        "merch.reorder.request" => "Merch reorder request",
        "merch.price.change" => "Merch price changed",
        "merch.bundle.request" => "Merch bundle request",
        "booking.outreach.request" => "Booking outreach",
        "outreach.request" => "Relationship outreach",
        "beacon.discovery.request" => "Local beacon discovery",
        "beacon.outreach.request" => "Local beacon outreach",
        "show.growth.request" => "Attendance / merch growth lever",
        "content.artifact.request" => "Content artifact",
        "experiment.allocation.change" => "Experiment reallocation",
        "experiment.complete" => "Experiment winner",
        "show.task.complete" => "Show task completed",
        "show.task.escalate" => "Show task needs human",
        "promotion.budget_change.request" => "Promotion budget change",
        "release.milestone.execute" => "Release milestone",
        "opportunity.live.apply" => "Festival / opportunity application",
        "funding.package.prepare" => "Funding package prepared",
        "funding.application.submit" => "Funding application",
        "team.assignment.email" => "Team assignment email",
        _ => "Autopilot action",
    }
}

fn autopilot_measurement_kind_label(kind: &str) -> &'static str {
    match kind {
        "ticket_revenue_72h" => "Ticket revenue · 72h",
        "merch_gross_proxy_7d" => "Merch gross proxy · 7d",
        "promotion_roas_7d" => "Promotion ROAS · 7d",
        "booking_reply_7d" => "Booking reply · 7d",
        "outreach_reply_7d" => "Outreach reply · 7d",
        "audience_ticket_revenue_72h" => "Audience ticket revenue · 72h",
        "show_ticket_revenue_7d" => "Show ticket revenue · 7d",
        "show_growth_surface_clicks_7d" => "Free distribution clicks · 7d",
        "show_growth_attributed_ticket_orders_7d" => "Attributed ticket orders · 7d",
        "grassroots_activation_replies_14d" => "Grassroots replies · 14d",
        _ => "Measured effect",
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
            format!("Wzrost koncertu · {} · {template_key}", lever.replace('_', " "))
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
        } => format!("{recipient_name} · {task_title} · reminder {reminder_number}"),
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
        "announcement_published" => "Announcement published",
        "ticketing_verified" => "Ticketing verified",
        "staff_assigned" => "Staff assigned",
        "offline_snapshot_ready" => "Offline snapshot ready",
        "gate_device_charged" => "Gate device charged",
        "backup_device_ready" => "Backup device ready",
        "network_tested" => "Network tested",
        "guestlist_checked" => "Guest list checked",
        "post_show_reconciliation" => "Post-show reconciliation",
        "post_show_report" => "Post-show report",
        _ => "Show task",
    }
}

fn autopilot_rum_metric_label(surface: &str, metric: &str) -> String {
    let surface_label = match surface {
        "virya_www" => "Virya WWW",
        "synesthesia" => "Synesthesia",
        "virya_signal" => "Virya Signal",
        _ => surface,
    };
    let metric_label = match metric {
        "lcp_ms" => "LCP",
        "inp_ms" => "INP",
        "cls_milli" => "CLS ×1000",
        "ttfb_ms" => "TTFB",
        "boot_interactive_ms" => "boot→interactive",
        "room_load_ms" => "room load",
        "transition_ms" => "transition",
        "frame_hitch_ms" => "frame hitch",
        "cold_start_ms" => "cold start",
        "api_latency_ms" => "API latency",
        "screen_transition_ms" => "screen transition",
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
