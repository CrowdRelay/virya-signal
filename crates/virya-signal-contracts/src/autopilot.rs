use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AutopilotPolicySummary {
    pub context: String,
    #[serde(default)]
    pub enabled: bool,
    #[serde(default)]
    pub autonomy_level: String,
    #[serde(default)]
    pub minimum_confidence: u16,
    #[serde(default)]
    pub max_actions_24h: u32,
    #[serde(default)]
    pub version: i64,
    #[serde(default)]
    pub guarded_until: Option<String>,
    #[serde(default)]
    pub guardrail_reason: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PromotionBudgetGuardrailSummary {
    pub currency: String,
    pub maximum_total_daily_budget_minor: i64,
    pub maximum_monthly_spend_minor: i64,
    pub version: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct ExperimentAllocation {
    pub variant_id: String,
    pub allocation_basis_points: u16,
}

#[derive(Clone, Debug, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AutopilotActionPayload {
    /// An action kind this build does not know.
    ///
    /// CrowdRelay ships action kinds on its own cadence; a store-distributed
    /// client runs whatever version the user last installed, so the two are
    /// never in lockstep. This variant used to be an error, and because
    /// `payload` is a required field inside `PendingAutopilotAction`, that
    /// error failed the whole element — and with it the entire pending-actions
    /// list. One unknown kind blanked the operator's Autopilot screen rather
    /// than one row of it. Keeping the raw kind lets the UI show the action,
    /// label it generically, and leave it for a build that understands it.
    Unrecognized {
        /// The wire kind as received. Named apart from the `kind` tag serde
        /// already uses to discriminate this enum.
        wire_kind: String,
    },
    ChangeTicketPrice {
        ticket_type_id: String,
        from_minor: i64,
        to_minor: i64,
    },
    ChangeTicketCapacity {
        ticket_type_id: String,
        from_capacity: u32,
        to_capacity: u32,
        guardrail_version: i64,
    },
    RequestFanLifecycleMessage {
        fan_id: String,
        template_key: String,
    },
    RequestMerchReorder {
        variant_id: String,
        quantity: u32,
    },
    ChangeMerchPrice {
        product_id: String,
        from_minor: i64,
        to_minor: i64,
        economics_version: i64,
    },
    RequestBookingOutreach {
        city_id: String,
        target_id: String,
        target_version: i64,
        target_name: String,
        score: u16,
        phase: String,
    },
    RequestAudienceCampaign {
        event_id: String,
        phase: String,
        template_key: String,
    },
    RequestMerchBundle {
        product_a: String,
        product_b: String,
        bundle_price_minor: i64,
        affinity_basis_points: u16,
    },
    RequestOutreach {
        opportunity_id: String,
        target_id: String,
        target_version: i64,
        target_name: String,
        phase: String,
        template_key: String,
    },
    RequestBeaconDiscovery {
        event_id: String,
        target_count: u16,
    },
    RequestBeaconOutreach {
        beacon_id: String,
        event_id: String,
        beacon_version: i64,
        phase: String,
        template_key: String,
    },
    RequestShowGrowth {
        event_id: String,
        lever: String,
        template_key: String,
    },
    RequestContentArtifact {
        source_id: String,
        source_version: i64,
        artifact: String,
        template_key: String,
    },
    AdjustExperiment {
        experiment_id: String,
        expected_version: i64,
        winner_variant_id: String,
        allocations: Vec<ExperimentAllocation>,
        complete: bool,
    },
    CompleteShowTask {
        event_id: String,
        task: String,
    },
    EscalateShowTask {
        event_id: String,
        task: String,
    },
    RequestPromotionBudgetChange {
        campaign_id: String,
        from_minor: i64,
        to_minor: i64,
        roas_basis_points: u32,
    },
    ExecuteReleaseMilestone {
        release_id: String,
        title: String,
        release_at: String,
        milestone: String,
    },
    ApplyLiveOpportunity {
        opportunity_id: String,
        opportunity_kind: String,
        score: u16,
    },
    PrepareFundingPackage {
        opportunity_id: String,
    },
    SubmitFundingApplication {
        opportunity_id: String,
    },
    // Internal executor actions can appear in historical/control-plane payloads.
    // Keep recipient_email intentionally absent: unknown provider-only fields are
    // ignored by Serde and must never be surfaced into the Signal UI contract.
    SendTeamAssignmentEmail {
        assignment_id: String,
        recipient_name: String,
        task_title: String,
        task_detail: String,
        due_at: Option<String>,
        action_url_path: String,
        reminder_number: u8,
    },
    VerifyPlaylistPlacement {
        opportunity_id: String,
        playlist_external_id: String,
        track_external_id: String,
        checkpoint: u8,
    },
    RequestBeaconInviteBatch {
        beacon_id: String,
        beacon_version: i64,
        event_id: String,
        requested_count: u16,
    },
    RequestOutreachDiscovery {
        requested_candidates: u16,
    },
    RequestBookingTargetDiscovery {
        requested_count: u16,
    },
    EscalateEditorialPitch {
        release_id: String,
        title: String,
        due_at: String,
    },
    CounterLiveOpportunityTerms {
        opportunity_id: String,
        ask_minor: i64,
        currency: String,
        round: u8,
    },
    AcceptLiveOpportunityTerms {
        opportunity_id: String,
        fee_minor: i64,
        currency: String,
    },
    RaiseGrowthOpportunity {
        series_id: String,
        platform: String,
        metric_key: String,
        signal: String,
        recommended_action: String,
        deviation_basis_points: u32,
        priority: u16,
        template_key: String,
    },
    RaiseGrowthDebt {
        subject_kind: String,
        subject_id: String,
        debt_kind: String,
        recommended_action: String,
        overdue_basis_points: u32,
        outstanding_items: u32,
        tracked_items: u32,
        priority: u16,
        template_key: String,
    },
    IssueReferralCode {
        fan_id: String,
    },
    RunPlayStep {
        play_id: String,
        play_kind: String,
        step_index: u16,
        step_kind: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        fan_id: Option<String>,
        template_key: String,
    },
    RequestAgentContent {
        #[serde(default, skip_serializing_if = "Option::is_none")]
        template_id: Option<String>,
        task_id: String,
        draft: JsonValue,
    },
    /// An agent found an outreach target -- press contact, station, community
    /// -- that a human must verify before the growth loop uses it. Approving it
    /// only flips a staging row from proposed to promoted, so nothing leaves
    /// the system on approval and the decision is reversible.
    RequestOutreachTarget {
        task_id: String,
        target_kind: String,
        display_name: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        contact_email: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        contact_domain: Option<String>,
        #[serde(default)]
        why_fit: String,
        #[serde(default)]
        evidence_urls: JsonValue,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        subreddit: Option<String>,
    },
    RequestAgentRun {
        template_id: String,
        prompt: String,
        priority: u8,
        tier: String,
    },
    RequestCommunityEngagement {
        target_id: String,
        platform: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        subreddit: Option<String>,
        title: String,
        body: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        smart_link: Option<String>,
    },
    RequestSignalPush {
        task_id: String,
        title: String,
        body: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        target_path: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        event_id: Option<String>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        segment: Option<String>,
    },
}

/// Flat wire projection of [`AutopilotActionPayload`].
///
/// The payload is internally tagged, and `#[derive(Deserialize)]` on an
/// internally tagged enum forces Serde's `Content` buffering path: the whole map
/// is buffered, then each of the 22 variants is monomorphised again over
/// `ContentDeserializer`. In the WASM bundle that single generated `visit_map`
/// measured ~84 KiB. Deserialising one flat struct instead keeps exactly one
/// streaming `visit_map`, and the variant reconstruction below is a plain
/// non-generic function that is compiled once.
///
/// Unknown fields stay ignored (provider-only keys must never reach the UI), and
/// the accepted wire shape is byte-identical to the derived implementation.
#[derive(Deserialize)]
struct WireAutopilotActionPayload {
    kind: String,
    #[serde(default)]
    action_url_path: Option<String>,
    #[serde(default)]
    affinity_basis_points: Option<u16>,
    #[serde(default)]
    allocations: Option<Vec<ExperimentAllocation>>,
    #[serde(default)]
    artifact: Option<String>,
    #[serde(default)]
    assignment_id: Option<String>,
    #[serde(default)]
    beacon_id: Option<String>,
    #[serde(default)]
    beacon_version: Option<i64>,
    #[serde(default)]
    bundle_price_minor: Option<i64>,
    #[serde(default)]
    campaign_id: Option<String>,
    #[serde(default)]
    city_id: Option<String>,
    #[serde(default)]
    complete: Option<bool>,
    #[serde(default)]
    due_at: Option<String>,
    #[serde(default)]
    economics_version: Option<i64>,
    #[serde(default)]
    event_id: Option<String>,
    #[serde(default)]
    expected_version: Option<i64>,
    #[serde(default)]
    experiment_id: Option<String>,
    #[serde(default)]
    fan_id: Option<String>,
    #[serde(default)]
    from_capacity: Option<u32>,
    #[serde(default)]
    from_minor: Option<i64>,
    #[serde(default)]
    guardrail_version: Option<i64>,
    #[serde(default)]
    lever: Option<String>,
    #[serde(default)]
    milestone: Option<String>,
    #[serde(default)]
    opportunity_id: Option<String>,
    #[serde(default)]
    opportunity_kind: Option<String>,
    #[serde(default)]
    phase: Option<String>,
    #[serde(default)]
    product_a: Option<String>,
    #[serde(default)]
    product_b: Option<String>,
    #[serde(default)]
    product_id: Option<String>,
    #[serde(default)]
    quantity: Option<u32>,
    #[serde(default)]
    recipient_name: Option<String>,
    #[serde(default)]
    release_at: Option<String>,
    #[serde(default)]
    release_id: Option<String>,
    #[serde(default)]
    reminder_number: Option<u8>,
    #[serde(default)]
    roas_basis_points: Option<u32>,
    #[serde(default)]
    score: Option<u16>,
    #[serde(default)]
    source_id: Option<String>,
    #[serde(default)]
    source_version: Option<i64>,
    #[serde(default)]
    target_count: Option<u16>,
    #[serde(default)]
    target_id: Option<String>,
    #[serde(default)]
    target_name: Option<String>,
    #[serde(default)]
    target_version: Option<i64>,
    #[serde(default)]
    task: Option<String>,
    #[serde(default)]
    task_detail: Option<String>,
    #[serde(default)]
    task_title: Option<String>,
    #[serde(default)]
    template_key: Option<String>,
    #[serde(default)]
    ticket_type_id: Option<String>,
    #[serde(default)]
    title: Option<String>,
    #[serde(default)]
    to_capacity: Option<u32>,
    #[serde(default)]
    to_minor: Option<i64>,
    #[serde(default)]
    variant_id: Option<String>,
    #[serde(default)]
    winner_variant_id: Option<String>,
    // ── Fields for the 15 action kinds synced with CrowdRelay ──
    #[serde(default)]
    ask_minor: Option<i64>,
    #[serde(default)]
    fee_minor: Option<i64>,
    #[serde(default)]
    body: Option<String>,
    #[serde(default)]
    checkpoint: Option<u8>,
    #[serde(default)]
    currency: Option<String>,
    #[serde(default)]
    debt_kind: Option<String>,
    #[serde(default)]
    deviation_basis_points: Option<u32>,
    #[serde(default)]
    draft: Option<JsonValue>,
    #[serde(default)]
    metric_key: Option<String>,
    #[serde(default)]
    outstanding_items: Option<u32>,
    #[serde(default)]
    platform: Option<String>,
    #[serde(default)]
    play_id: Option<String>,
    #[serde(default)]
    play_kind: Option<String>,
    #[serde(default)]
    playlist_external_id: Option<String>,
    #[serde(default)]
    priority: Option<u16>,
    #[serde(default)]
    prompt: Option<String>,
    #[serde(default)]
    recommended_action: Option<String>,
    #[serde(default)]
    requested_candidates: Option<u16>,
    #[serde(default)]
    requested_count: Option<u16>,
    #[serde(default)]
    round: Option<u8>,
    #[serde(default)]
    series_id: Option<String>,
    #[serde(default)]
    signal: Option<String>,
    #[serde(default)]
    smart_link: Option<String>,
    #[serde(default)]
    step_index: Option<u16>,
    #[serde(default)]
    step_kind: Option<String>,
    #[serde(default)]
    subreddit: Option<String>,
    #[serde(default)]
    target_kind: Option<String>,
    #[serde(default)]
    display_name: Option<String>,
    #[serde(default)]
    contact_email: Option<String>,
    #[serde(default)]
    contact_domain: Option<String>,
    #[serde(default)]
    why_fit: Option<String>,
    #[serde(default)]
    evidence_urls: Option<JsonValue>,
    #[serde(default)]
    subject_id: Option<String>,
    #[serde(default)]
    subject_kind: Option<String>,
    #[serde(default)]
    target_path: Option<String>,
    #[serde(default)]
    task_id: Option<String>,
    #[serde(default)]
    template_id: Option<String>,
    #[serde(default)]
    tier: Option<String>,
    #[serde(default)]
    track_external_id: Option<String>,
    #[serde(default)]
    tracked_items: Option<u32>,
    #[serde(default)]
    overdue_basis_points: Option<u32>,
    #[serde(default)]
    segment: Option<String>,
}

/// Why a wire payload could not be modelled by this build.
///
/// Both outcomes are handled, never returned to serde: see the `Deserialize`
/// impl for why an error here would cost the whole pending-actions list.
enum WirePayloadError {
    /// A kind this build knows, missing a field this build requires.
    MissingField(&'static str),
}

#[inline]
fn required<T>(value: Option<T>, field: &'static str) -> Result<T, WirePayloadError> {
    value.ok_or(WirePayloadError::MissingField(field))
}

impl WireAutopilotActionPayload {
    fn into_payload(self) -> Result<AutopilotActionPayload, WirePayloadError> {
        Ok(match self.kind.as_str() {
            "change_ticket_price" => AutopilotActionPayload::ChangeTicketPrice {
                ticket_type_id: required(self.ticket_type_id, "ticket_type_id")?,
                from_minor: required(self.from_minor, "from_minor")?,
                to_minor: required(self.to_minor, "to_minor")?,
            },
            "change_ticket_capacity" => AutopilotActionPayload::ChangeTicketCapacity {
                ticket_type_id: required(self.ticket_type_id, "ticket_type_id")?,
                from_capacity: required(self.from_capacity, "from_capacity")?,
                to_capacity: required(self.to_capacity, "to_capacity")?,
                guardrail_version: required(self.guardrail_version, "guardrail_version")?,
            },
            "request_fan_lifecycle_message" => AutopilotActionPayload::RequestFanLifecycleMessage {
                fan_id: required(self.fan_id, "fan_id")?,
                template_key: required(self.template_key, "template_key")?,
            },
            "request_merch_reorder" => AutopilotActionPayload::RequestMerchReorder {
                variant_id: required(self.variant_id, "variant_id")?,
                quantity: required(self.quantity, "quantity")?,
            },
            "change_merch_price" => AutopilotActionPayload::ChangeMerchPrice {
                product_id: required(self.product_id, "product_id")?,
                from_minor: required(self.from_minor, "from_minor")?,
                to_minor: required(self.to_minor, "to_minor")?,
                economics_version: required(self.economics_version, "economics_version")?,
            },
            "request_booking_outreach" => AutopilotActionPayload::RequestBookingOutreach {
                city_id: required(self.city_id, "city_id")?,
                target_id: required(self.target_id, "target_id")?,
                target_version: required(self.target_version, "target_version")?,
                target_name: required(self.target_name, "target_name")?,
                score: required(self.score, "score")?,
                phase: required(self.phase, "phase")?,
            },
            "request_audience_campaign" => AutopilotActionPayload::RequestAudienceCampaign {
                event_id: required(self.event_id, "event_id")?,
                phase: required(self.phase, "phase")?,
                template_key: required(self.template_key, "template_key")?,
            },
            "request_merch_bundle" => AutopilotActionPayload::RequestMerchBundle {
                product_a: required(self.product_a, "product_a")?,
                product_b: required(self.product_b, "product_b")?,
                bundle_price_minor: required(self.bundle_price_minor, "bundle_price_minor")?,
                affinity_basis_points: required(
                    self.affinity_basis_points,
                    "affinity_basis_points",
                )?,
            },
            "request_outreach" => AutopilotActionPayload::RequestOutreach {
                opportunity_id: required(self.opportunity_id, "opportunity_id")?,
                target_id: required(self.target_id, "target_id")?,
                target_version: required(self.target_version, "target_version")?,
                target_name: required(self.target_name, "target_name")?,
                phase: required(self.phase, "phase")?,
                template_key: required(self.template_key, "template_key")?,
            },
            "request_beacon_discovery" => AutopilotActionPayload::RequestBeaconDiscovery {
                event_id: required(self.event_id, "event_id")?,
                target_count: required(self.target_count, "target_count")?,
            },
            "request_beacon_outreach" => AutopilotActionPayload::RequestBeaconOutreach {
                beacon_id: required(self.beacon_id, "beacon_id")?,
                event_id: required(self.event_id, "event_id")?,
                beacon_version: required(self.beacon_version, "beacon_version")?,
                phase: required(self.phase, "phase")?,
                template_key: required(self.template_key, "template_key")?,
            },
            "request_show_growth" => AutopilotActionPayload::RequestShowGrowth {
                event_id: required(self.event_id, "event_id")?,
                lever: required(self.lever, "lever")?,
                template_key: required(self.template_key, "template_key")?,
            },
            "request_content_artifact" => AutopilotActionPayload::RequestContentArtifact {
                source_id: required(self.source_id, "source_id")?,
                source_version: required(self.source_version, "source_version")?,
                artifact: required(self.artifact, "artifact")?,
                template_key: required(self.template_key, "template_key")?,
            },
            "adjust_experiment" => AutopilotActionPayload::AdjustExperiment {
                experiment_id: required(self.experiment_id, "experiment_id")?,
                expected_version: required(self.expected_version, "expected_version")?,
                winner_variant_id: required(self.winner_variant_id, "winner_variant_id")?,
                allocations: required(self.allocations, "allocations")?,
                complete: required(self.complete, "complete")?,
            },
            "complete_show_task" => AutopilotActionPayload::CompleteShowTask {
                event_id: required(self.event_id, "event_id")?,
                task: required(self.task, "task")?,
            },
            "escalate_show_task" => AutopilotActionPayload::EscalateShowTask {
                event_id: required(self.event_id, "event_id")?,
                task: required(self.task, "task")?,
            },
            "request_promotion_budget_change" => {
                AutopilotActionPayload::RequestPromotionBudgetChange {
                    campaign_id: required(self.campaign_id, "campaign_id")?,
                    from_minor: required(self.from_minor, "from_minor")?,
                    to_minor: required(self.to_minor, "to_minor")?,
                    roas_basis_points: required(self.roas_basis_points, "roas_basis_points")?,
                }
            }
            "execute_release_milestone" => AutopilotActionPayload::ExecuteReleaseMilestone {
                release_id: required(self.release_id, "release_id")?,
                title: required(self.title, "title")?,
                release_at: required(self.release_at, "release_at")?,
                milestone: required(self.milestone, "milestone")?,
            },
            "apply_live_opportunity" => AutopilotActionPayload::ApplyLiveOpportunity {
                opportunity_id: required(self.opportunity_id, "opportunity_id")?,
                opportunity_kind: required(self.opportunity_kind, "opportunity_kind")?,
                score: required(self.score, "score")?,
            },
            "prepare_funding_package" => AutopilotActionPayload::PrepareFundingPackage {
                opportunity_id: required(self.opportunity_id, "opportunity_id")?,
            },
            "submit_funding_application" => AutopilotActionPayload::SubmitFundingApplication {
                opportunity_id: required(self.opportunity_id, "opportunity_id")?,
            },
            "send_team_assignment_email" => AutopilotActionPayload::SendTeamAssignmentEmail {
                assignment_id: required(self.assignment_id, "assignment_id")?,
                recipient_name: required(self.recipient_name, "recipient_name")?,
                task_title: required(self.task_title, "task_title")?,
                task_detail: required(self.task_detail, "task_detail")?,
                due_at: self.due_at,
                action_url_path: required(self.action_url_path, "action_url_path")?,
                reminder_number: required(self.reminder_number, "reminder_number")?,
            },
            "verify_playlist_placement" => AutopilotActionPayload::VerifyPlaylistPlacement {
                opportunity_id: required(self.opportunity_id, "opportunity_id")?,
                playlist_external_id: required(self.playlist_external_id, "playlist_external_id")?,
                track_external_id: required(self.track_external_id, "track_external_id")?,
                checkpoint: required(self.checkpoint, "checkpoint")?,
            },
            "request_beacon_invite_batch" => AutopilotActionPayload::RequestBeaconInviteBatch {
                beacon_id: required(self.beacon_id, "beacon_id")?,
                beacon_version: required(self.beacon_version, "beacon_version")?,
                event_id: required(self.event_id, "event_id")?,
                requested_count: required(self.requested_count, "requested_count")?,
            },
            "request_outreach_discovery" => AutopilotActionPayload::RequestOutreachDiscovery {
                requested_candidates: required(self.requested_candidates, "requested_candidates")?,
            },
            "request_booking_target_discovery" => {
                AutopilotActionPayload::RequestBookingTargetDiscovery {
                    requested_count: required(self.requested_count, "requested_count")?,
                }
            }
            "escalate_editorial_pitch" => AutopilotActionPayload::EscalateEditorialPitch {
                release_id: required(self.release_id, "release_id")?,
                title: required(self.title, "title")?,
                due_at: required(self.due_at, "due_at")?,
            },
            "counter_live_opportunity_terms" => {
                AutopilotActionPayload::CounterLiveOpportunityTerms {
                    opportunity_id: required(self.opportunity_id, "opportunity_id")?,
                    ask_minor: required(self.ask_minor, "ask_minor")?,
                    currency: required(self.currency, "currency")?,
                    round: required(self.round, "round")?,
                }
            }
            "accept_live_opportunity_terms" => AutopilotActionPayload::AcceptLiveOpportunityTerms {
                opportunity_id: required(self.opportunity_id, "opportunity_id")?,
                fee_minor: required(self.fee_minor, "fee_minor")?,
                currency: required(self.currency, "currency")?,
            },
            "raise_growth_opportunity" => AutopilotActionPayload::RaiseGrowthOpportunity {
                series_id: required(self.series_id, "series_id")?,
                platform: required(self.platform, "platform")?,
                metric_key: required(self.metric_key, "metric_key")?,
                signal: required(self.signal, "signal")?,
                recommended_action: required(self.recommended_action, "recommended_action")?,
                deviation_basis_points: required(
                    self.deviation_basis_points,
                    "deviation_basis_points",
                )?,
                priority: required(self.priority, "priority")?,
                template_key: required(self.template_key, "template_key")?,
            },
            "raise_growth_debt" => AutopilotActionPayload::RaiseGrowthDebt {
                subject_kind: required(self.subject_kind, "subject_kind")?,
                subject_id: required(self.subject_id, "subject_id")?,
                debt_kind: required(self.debt_kind, "debt_kind")?,
                recommended_action: required(self.recommended_action, "recommended_action")?,
                overdue_basis_points: required(self.overdue_basis_points, "overdue_basis_points")?,
                outstanding_items: required(self.outstanding_items, "outstanding_items")?,
                tracked_items: required(self.tracked_items, "tracked_items")?,
                priority: required(self.priority, "priority")?,
                template_key: required(self.template_key, "template_key")?,
            },
            "issue_referral_code" => AutopilotActionPayload::IssueReferralCode {
                fan_id: required(self.fan_id, "fan_id")?,
            },
            "run_play_step" => AutopilotActionPayload::RunPlayStep {
                play_id: required(self.play_id, "play_id")?,
                play_kind: required(self.play_kind, "play_kind")?,
                step_index: required(self.step_index, "step_index")?,
                step_kind: required(self.step_kind, "step_kind")?,
                event_id: self.event_id,
                fan_id: self.fan_id,
                template_key: required(self.template_key, "template_key")?,
            },
            "request_agent_content" => AutopilotActionPayload::RequestAgentContent {
                template_id: self.template_id,
                task_id: required(self.task_id, "task_id")?,
                draft: required(self.draft, "draft")?,
            },
            "request_outreach_target" => AutopilotActionPayload::RequestOutreachTarget {
                task_id: required(self.task_id, "task_id")?,
                target_kind: required(self.target_kind, "target_kind")?,
                display_name: required(self.display_name, "display_name")?,
                contact_email: self.contact_email,
                contact_domain: self.contact_domain,
                // Optional on the wire: an agent that found a target without a
                // rationale still produces an actionable row.
                why_fit: self.why_fit.unwrap_or_default(),
                evidence_urls: self.evidence_urls.unwrap_or(JsonValue::Null),
                subreddit: self.subreddit,
            },
            "request_agent_run" => AutopilotActionPayload::RequestAgentRun {
                template_id: required(self.template_id, "template_id")?,
                prompt: required(self.prompt, "prompt")?,
                priority: required(self.priority, "priority")? as u8,
                tier: required(self.tier, "tier")?,
            },
            "request_community_engagement" => AutopilotActionPayload::RequestCommunityEngagement {
                target_id: required(self.target_id, "target_id")?,
                platform: required(self.platform, "platform")?,
                subreddit: self.subreddit,
                title: required(self.title, "title")?,
                body: required(self.body, "body")?,
                smart_link: self.smart_link,
            },
            "request_signal_push" => AutopilotActionPayload::RequestSignalPush {
                task_id: required(self.task_id, "task_id")?,
                title: required(self.title, "title")?,
                body: required(self.body, "body")?,
                target_path: self.target_path,
                event_id: self.event_id,
                segment: self.segment,
            },
            other => AutopilotActionPayload::Unrecognized {
                wire_kind: other.to_owned(),
            },
        })
    }
}

impl<'de> Deserialize<'de> for AutopilotActionPayload {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let wire = WireAutopilotActionPayload::deserialize(deserializer)?;
        let kind = wire.kind.clone();
        // Neither failure mode may propagate. `payload` is a required field of
        // `PendingAutopilotAction`, so an error here fails that element, and
        // failing one element of the pending-actions list fails the list — the
        // operator loses the whole Autopilot screen, not the one row nobody
        // could render. A store-distributed client cannot be updated in step
        // with the backend, so both an unknown kind and a known kind whose
        // shape has moved degrade to one generic row carrying the wire kind.
        Ok(wire.into_payload().unwrap_or_else(|error| {
            // A missing field means the two sides genuinely disagree about a
            // kind both claim to know. The contract crate has no logger — it is
            // shared by the wasm UI and the native shell — so the signal is the
            // generic row itself, carrying the wire kind that failed.
            let WirePayloadError::MissingField(_field) = error;
            AutopilotActionPayload::Unrecognized {
                wire_kind: kind.clone(),
            }
        }))
    }
}

impl AutopilotActionPayload {
    /// Stable CrowdRelay wire action kind. Kept in the Signal contract crate so
    /// standalone virya-signal CI never depends on a sibling repository checkout.
    #[must_use]
    pub const fn action_kind(&self) -> &'static str {
        match self {
            Self::ChangeTicketPrice { .. } => "ticket.price.change",
            Self::ChangeTicketCapacity { .. } => "ticket.capacity.change",
            Self::RequestFanLifecycleMessage { .. } => "fan.lifecycle.message.request",
            Self::RequestMerchReorder { .. } => "merch.reorder.request",
            Self::ChangeMerchPrice { .. } => "merch.price.change",
            Self::RequestBookingOutreach { .. } => "booking.outreach.request",
            Self::RequestAudienceCampaign { .. } => "audience.campaign.request",
            Self::RequestMerchBundle { .. } => "merch.bundle.request",
            Self::RequestOutreach { .. } => "outreach.request",
            Self::RequestBeaconDiscovery { .. } => "beacon.discovery.request",
            Self::RequestBeaconOutreach { .. } => "beacon.outreach.request",
            Self::RequestShowGrowth { .. } => "show.growth.request",
            Self::RequestContentArtifact { .. } => "content.artifact.request",
            Self::AdjustExperiment {
                complete: false, ..
            } => "experiment.allocation.change",
            Self::AdjustExperiment { complete: true, .. } => "experiment.complete",
            Self::CompleteShowTask { .. } => "show.task.complete",
            Self::EscalateShowTask { .. } => "show.task.escalate",
            Self::RequestPromotionBudgetChange { .. } => "promotion.budget_change.request",
            Self::ExecuteReleaseMilestone { .. } => "release.milestone.execute",
            Self::ApplyLiveOpportunity { .. } => "opportunity.live.apply",
            Self::PrepareFundingPackage { .. } => "funding.package.prepare",
            Self::SubmitFundingApplication { .. } => "funding.application.submit",
            Self::SendTeamAssignmentEmail { .. } => "team.assignment.email",
            Self::VerifyPlaylistPlacement { .. } => "playlist.placement.verify",
            Self::RequestBeaconInviteBatch { .. } => "beacon.invite_batch.request",
            Self::RequestOutreachDiscovery { .. } => "outreach.discovery.request",
            Self::RequestBookingTargetDiscovery { .. } => "booking.target_discovery.request",
            Self::EscalateEditorialPitch { .. } => "release.editorial_pitch.escalate",
            Self::CounterLiveOpportunityTerms { .. } => "opportunity.terms.counter",
            Self::AcceptLiveOpportunityTerms { .. } => "opportunity.terms.accept",
            Self::RaiseGrowthOpportunity { .. } => "growth.opportunity.raise",
            Self::RaiseGrowthDebt { .. } => "growth.debt.raise",
            Self::IssueReferralCode { .. } => "referral.code.issue",
            Self::RunPlayStep { .. } => "play.step.run",
            Self::RequestAgentContent { .. } => "agent.content.request",
            Self::RequestOutreachTarget { .. } => "outreach.target.request",
            Self::RequestAgentRun { .. } => "agent.run.request",
            Self::RequestCommunityEngagement { .. } => "community.engage.request",
            Self::RequestSignalPush { .. } => "signal.push.request",
            // Not a CrowdRelay kind: this build did not recognise the one it
            // was sent. `scripts/test_autopilot_wire_contract.py` excludes it
            // when comparing against the backend.
            Self::Unrecognized { .. } => "unrecognized",
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct TeamAssigneeSummary {
    pub member_id: String,
    pub member_key: String,
    pub display_name: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct PendingAutopilotAction {
    pub id: String,
    pub context: String,
    pub action_kind: String,
    pub subject_kind: String,
    pub subject_id: String,
    pub payload: AutopilotActionPayload,
    pub created_at: String,
    #[serde(default)]
    pub approval_expires_at: Option<String>,
    #[serde(default)]
    pub assignee: Option<TeamAssigneeSummary>,
    #[serde(default)]
    pub assignment_due_at: Option<String>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RecentAutopilotDecision {
    pub id: String,
    pub context: String,
    pub decision_kind: String,
    pub confidence: u16,
    pub disposition: String,
    pub reason: String,
    pub evaluated_at: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AutopilotManualStep {
    pub destination: String,
    pub url: String,
    pub what_to_do: String,
    pub why_it_matters: String,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RecentAutopilotAction {
    pub id: String,
    pub context: String,
    pub action_kind: String,
    pub subject_kind: String,
    pub subject_id: String,
    pub status: String,
    pub attempt_count: u32,
    pub created_at: String,
    pub finished_at: Option<String>,
    pub last_error_kind: Option<String>,
    #[serde(default)]
    pub executor_status: Option<String>,
    #[serde(default)]
    pub executor_id: Option<String>,
    #[serde(default)]
    pub provider_reference: Option<String>,
    #[serde(default)]
    pub executor_reported_at: Option<String>,
    #[serde(default)]
    pub manual_steps: Vec<AutopilotManualStep>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RecentAutopilotEffect {
    pub measurement_id: String,
    pub action_id: String,
    pub context: String,
    pub measurement_kind: String,
    pub assessment: String,
    pub delta_basis_points: i32,
    pub baseline_value: f64,
    pub observed_value: f64,
    pub observed_at: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ReleaseComponentSummary {
    #[serde(default)]
    pub component_key: String,
    #[serde(default)]
    pub environment: String,
    #[serde(default)]
    pub source_sha: String,
    #[serde(default)]
    pub artifact_digest: Option<String>,
    #[serde(default)]
    pub deploy_ref: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub manifest_sha: Option<String>,
    #[serde(default)]
    pub dependency_lock_sha256: Option<String>,
    #[serde(default)]
    pub artifact_manifest_sha256: Option<String>,
    #[serde(default)]
    pub workflow_attestation_sha: Option<String>,
    #[serde(default)]
    pub workflow_attested_at: Option<String>,
    #[serde(default)]
    pub observed_at: String,
    #[serde(default)]
    pub stale: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ReleaseLedgerOverview {
    #[serde(default)]
    pub components: Vec<ReleaseComponentSummary>,
    #[serde(default)]
    pub missing_components: Vec<String>,
    #[serde(default)]
    pub backend_sha_drift: bool,
    #[serde(default)]
    pub executor_manifest_drift: bool,
    #[serde(default)]
    pub active_executor_count: i64,
    #[serde(default)]
    pub guarded_executor_count: i64,
    #[serde(default)]
    pub active_executor_manifest_shas: Vec<String>,
    #[serde(default)]
    pub active_team_email_executor_count: i64,
    #[serde(default)]
    pub n8n_attestation_ready: bool,
    #[serde(default)]
    pub team_email_live: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct RumMetricSummary {
    #[serde(default)]
    pub surface: String,
    #[serde(default)]
    pub metric_key: String,
    #[serde(default)]
    pub samples_24h: i64,
    #[serde(default)]
    pub p75: f64,
    #[serde(default)]
    pub p95: f64,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct OperatorAutopilotOverview {
    #[serde(default)]
    pub runtime_enabled: bool,
    #[serde(default)]
    pub policies: Vec<AutopilotPolicySummary>,
    #[serde(default)]
    pub promotion_budget_guardrails: Vec<PromotionBudgetGuardrailSummary>,
    #[serde(default)]
    pub needs_you: Vec<PendingAutopilotAction>,
    #[serde(default)]
    pub available_assignees: Vec<TeamAssigneeSummary>,
    #[serde(default)]
    pub recent_decisions: Vec<RecentAutopilotDecision>,
    #[serde(default)]
    pub recent_actions: Vec<RecentAutopilotAction>,
    #[serde(default)]
    pub recent_effects: Vec<RecentAutopilotEffect>,
    #[serde(default)]
    pub queued_actions: i64,
    #[serde(default)]
    pub processing_actions: i64,
    #[serde(default)]
    pub succeeded_24h: i64,
    #[serde(default)]
    pub failed_24h: i64,
    #[serde(default)]
    pub executor_confirmed_24h: i64,
    #[serde(default)]
    pub executor_failed_24h: i64,
    #[serde(default)]
    pub awaiting_executor: i64,
    #[serde(default)]
    pub release_ledger: ReleaseLedgerOverview,
    #[serde(default)]
    pub rum_metrics_24h: Vec<RumMetricSummary>,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ChiefOfStaffOpportunity {
    pub context: String,
    pub decision_kind: String,
    pub subject_kind: String,
    pub subject_id: String,
    pub confidence: u16,
    pub reason: String,
    pub needs_approval: bool,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ChiefOfStaffShowTask {
    pub event_id: String,
    pub event_title: String,
    pub task_key: String,
    pub status: String,
    pub starts_at: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ChiefOfStaffAttentionItem {
    pub kind: String,
    pub subject_kind: String,
    pub subject_id: String,
    pub title: String,
    pub detail: String,
    pub due_at: String,
    pub urgency: String,
}

#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct AutopilotChiefOfStaff {
    #[serde(default)]
    pub executed_24h: i64,
    #[serde(default)]
    pub failed_24h: i64,
    #[serde(default)]
    pub needs_you: i64,
    #[serde(default)]
    pub estimated_minutes_saved_24h: i64,
    #[serde(default)]
    pub measured_improved_7d: i64,
    #[serde(default)]
    pub measured_neutral_7d: i64,
    #[serde(default)]
    pub measured_worsened_7d: i64,
    #[serde(default)]
    pub emitted_24h: i64,
    #[serde(default)]
    pub executor_confirmed_24h: i64,
    #[serde(default)]
    pub executor_failed_24h: i64,
    #[serde(default)]
    pub attention_items: Vec<ChiefOfStaffAttentionItem>,
    #[serde(default)]
    pub top_opportunities: Vec<ChiefOfStaffOpportunity>,
    #[serde(default)]
    pub show_tasks: Vec<ChiefOfStaffShowTask>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AutopilotMutation {
    pub operation_id: String,
    pub target_id: String,
    pub status: String,
    #[serde(default)]
    pub replayed: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AutopilotAuthorityRequest {
    pub enabled: bool,
    pub autonomy_level: String,
    pub minimum_confidence_basis_points: u16,
    pub max_actions_24h: u32,
    pub expected_version: i64,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct AutopilotAssignRequest {
    pub member_key: String,
}

#[cfg(test)]
mod forward_compatibility_tests {
    use super::*;

    /// The shape the operator screen actually receives: a list. One element it
    /// cannot model must not cost the other elements.
    ///
    /// Note the payload `kind` tag is the snake_case variant name
    /// (`change_ticket_price`), not the dotted wire action kind
    /// (`ticket.price.change`). The two are separate vocabularies.
    fn pending(kind: &str, extra: &str) -> String {
        format!(
            r#"{{"id":"a","context":"growth_intelligence","action_kind":"{kind}",
                 "subject_kind":"workspace","subject_id":"w",
                 "payload":{{"kind":"{kind}"{extra}}},"created_at":"2026-01-01T00:00:00Z"}}"#
        )
    }

    #[test]
    fn an_action_kind_this_build_never_heard_of_is_not_fatal() {
        // CrowdRelay ships action kinds on its own cadence; at the time of
        // writing it had 15 this crate cannot model. Each one used to fail the
        // element, and one failed element fails the whole list.
        let payload: AutopilotActionPayload =
            serde_json::from_str(r#"{"kind":"issue_referral_code"}"#).expect("must not error");
        match payload {
            AutopilotActionPayload::Unrecognized { wire_kind } => {
                assert_eq!(wire_kind, "issue_referral_code");
            }
            other => panic!("expected Unrecognized, got {other:?}"),
        }
    }

    #[test]
    fn a_known_kind_missing_a_required_field_is_not_fatal_either() {
        // The other half of the same failure: the backend renames or adds a
        // required field on a kind both sides claim to know.
        let payload: AutopilotActionPayload =
            serde_json::from_str(r#"{"kind":"change_ticket_price"}"#).expect("must not error");
        assert!(matches!(
            payload,
            AutopilotActionPayload::Unrecognized { .. }
        ));
    }

    #[test]
    fn one_unmodellable_action_does_not_cost_the_rest_of_the_list() {
        let list = format!(
            "[{},{}]",
            pending("some_future_kind", ""),
            pending(
                "change_ticket_price",
                r#","ticket_type_id":"t","from_minor":1000,"to_minor":1200"#
            ),
        );
        let actions: Vec<PendingAutopilotAction> =
            serde_json::from_str(&list).expect("the list must survive an unknown element");
        assert_eq!(actions.len(), 2, "the known action must still arrive");
        assert!(matches!(
            actions[1].payload,
            AutopilotActionPayload::ChangeTicketPrice { .. }
        ));
    }
}
