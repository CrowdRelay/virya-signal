use serde::{Deserialize, Serialize};

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

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum AutopilotActionPayload {
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
