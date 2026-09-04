//! Wire contract for [`AutopilotActionPayload`].
//!
//! `AutopilotActionPayload` keeps a derived `Serialize` but a hand-written
//! `Deserialize` (see `autopilot.rs`) because the derived internally tagged
//! deserialiser cost ~84 KiB in the release WASM bundle. These tests are the
//! guard rail for that trade: the hand-written implementation must stay the
//! exact inverse of the derived serialiser for every variant, and must keep
//! ignoring unknown provider-only fields.

use serde_json::{Value, json};
use virya_signal_contracts::autopilot::{AutopilotActionPayload, ExperimentAllocation};

fn round_trip(value: &AutopilotActionPayload) -> (Value, Value) {
    let encoded = serde_json::to_value(value).expect("payload serialises");
    let decoded: AutopilotActionPayload =
        serde_json::from_value(encoded.clone()).expect("payload deserialises");
    let re_encoded = serde_json::to_value(&decoded).expect("payload re-serialises");
    (encoded, re_encoded)
}

#[test]
fn every_variant_round_trips_through_the_hand_written_deserializer() {
    let samples = vec![
        AutopilotActionPayload::ChangeTicketPrice {
            ticket_type_id: "ticket_type_id-0".to_owned(),
            from_minor: 1000,
            to_minor: 1000,
        },
        AutopilotActionPayload::ChangeTicketCapacity {
            ticket_type_id: "ticket_type_id-1".to_owned(),
            from_capacity: 11,
            to_capacity: 11,
            guardrail_version: 1001,
        },
        AutopilotActionPayload::RequestFanLifecycleMessage {
            fan_id: "fan_id-2".to_owned(),
            template_key: "template_key-2".to_owned(),
        },
        AutopilotActionPayload::RequestMerchReorder {
            variant_id: "variant_id-3".to_owned(),
            quantity: 13,
        },
        AutopilotActionPayload::ChangeMerchPrice {
            product_id: "product_id-4".to_owned(),
            from_minor: 1004,
            to_minor: 1004,
            economics_version: 1004,
        },
        AutopilotActionPayload::RequestBookingOutreach {
            city_id: "city_id-5".to_owned(),
            target_id: "target_id-5".to_owned(),
            target_version: 1005,
            target_name: "target_name-5".to_owned(),
            score: 25,
            phase: "phase-5".to_owned(),
        },
        AutopilotActionPayload::RequestAudienceCampaign {
            event_id: "event_id-6".to_owned(),
            phase: "phase-6".to_owned(),
            template_key: "template_key-6".to_owned(),
        },
        AutopilotActionPayload::RequestMerchBundle {
            product_a: "product_a-7".to_owned(),
            product_b: "product_b-7".to_owned(),
            bundle_price_minor: 1007,
            affinity_basis_points: 27,
        },
        AutopilotActionPayload::RequestOutreach {
            opportunity_id: "opportunity_id-8".to_owned(),
            target_id: "target_id-8".to_owned(),
            target_version: 1008,
            target_name: "target_name-8".to_owned(),
            phase: "phase-8".to_owned(),
            template_key: "template_key-8".to_owned(),
        },
        AutopilotActionPayload::RequestBeaconDiscovery {
            event_id: "event_id-9".to_owned(),
            target_count: 29,
        },
        AutopilotActionPayload::RequestBeaconOutreach {
            beacon_id: "beacon_id-10".to_owned(),
            event_id: "event_id-10".to_owned(),
            beacon_version: 1010,
            phase: "phase-10".to_owned(),
            template_key: "template_key-10".to_owned(),
        },
        AutopilotActionPayload::RequestShowGrowth {
            event_id: "event_id-11".to_owned(),
            lever: "lever-11".to_owned(),
            template_key: "template_key-11".to_owned(),
        },
        AutopilotActionPayload::RequestContentArtifact {
            source_id: "source_id-12".to_owned(),
            source_version: 1012,
            artifact: "artifact-12".to_owned(),
            template_key: "template_key-12".to_owned(),
        },
        AutopilotActionPayload::AdjustExperiment {
            experiment_id: "experiment_id-13".to_owned(),
            expected_version: 1013,
            winner_variant_id: "winner_variant_id-13".to_owned(),
            allocations: vec![ExperimentAllocation {
                variant_id: "v1".to_owned(),
                allocation_basis_points: 5000,
            }],
            complete: true,
        },
        AutopilotActionPayload::CompleteShowTask {
            event_id: "event_id-14".to_owned(),
            task: "task-14".to_owned(),
        },
        AutopilotActionPayload::EscalateShowTask {
            event_id: "event_id-15".to_owned(),
            task: "task-15".to_owned(),
        },
        AutopilotActionPayload::RequestPromotionBudgetChange {
            campaign_id: "campaign_id-16".to_owned(),
            from_minor: 1016,
            to_minor: 1016,
            roas_basis_points: 26,
        },
        AutopilotActionPayload::ExecuteReleaseMilestone {
            release_id: "release_id-17".to_owned(),
            title: "title-17".to_owned(),
            release_at: "release_at-17".to_owned(),
            milestone: "milestone-17".to_owned(),
        },
        AutopilotActionPayload::ApplyLiveOpportunity {
            opportunity_id: "opportunity_id-18".to_owned(),
            opportunity_kind: "opportunity_kind-18".to_owned(),
            score: 38,
        },
        AutopilotActionPayload::PrepareFundingPackage {
            opportunity_id: "opportunity_id-19".to_owned(),
        },
        AutopilotActionPayload::SubmitFundingApplication {
            opportunity_id: "opportunity_id-20".to_owned(),
        },
        AutopilotActionPayload::SendTeamAssignmentEmail {
            assignment_id: "assignment_id-21".to_owned(),
            recipient_name: "recipient_name-21".to_owned(),
            task_title: "task_title-21".to_owned(),
            task_detail: "task_detail-21".to_owned(),
            due_at: Some("due_at-21".to_owned()),
            action_url_path: "action_url_path-21".to_owned(),
            reminder_number: 24,
        },
        AutopilotActionPayload::VerifyPlaylistPlacement {
            opportunity_id: "opportunity_id-22".to_owned(),
            playlist_external_id: "playlist_external_id-22".to_owned(),
            track_external_id: "track_external_id-22".to_owned(),
            checkpoint: 3,
        },
        AutopilotActionPayload::RequestBeaconInviteBatch {
            beacon_id: "beacon_id-23".to_owned(),
            beacon_version: 1023,
            event_id: "event_id-23".to_owned(),
            requested_count: 50,
        },
        AutopilotActionPayload::RequestOutreachDiscovery {
            requested_candidates: 25,
        },
        AutopilotActionPayload::RequestBookingTargetDiscovery {
            requested_count: 15,
        },
        AutopilotActionPayload::EscalateEditorialPitch {
            release_id: "release_id-26".to_owned(),
            title: "title-26".to_owned(),
            due_at: "due_at-26".to_owned(),
        },
        AutopilotActionPayload::CounterLiveOpportunityTerms {
            opportunity_id: "opportunity_id-27".to_owned(),
            ask_minor: 50_000,
            currency: "PLN".to_owned(),
            round: 2,
        },
        AutopilotActionPayload::AcceptLiveOpportunityTerms {
            opportunity_id: "opportunity_id-28".to_owned(),
            fee_minor: 45_000,
            currency: "PLN".to_owned(),
        },
        AutopilotActionPayload::RaiseGrowthOpportunity {
            series_id: "series_id-29".to_owned(),
            platform: "platform-29".to_owned(),
            metric_key: "metric_key-29".to_owned(),
            signal: "signal-29".to_owned(),
            recommended_action: "recommended_action-29".to_owned(),
            deviation_basis_points: 1200,
            priority: 5,
            template_key: "template_key-29".to_owned(),
        },
        AutopilotActionPayload::RaiseGrowthDebt {
            subject_kind: "subject_kind-30".to_owned(),
            subject_id: "subject_id-30".to_owned(),
            debt_kind: "debt_kind-30".to_owned(),
            recommended_action: "recommended_action-30".to_owned(),
            overdue_basis_points: 800,
            outstanding_items: 3,
            tracked_items: 10,
            priority: 7,
            template_key: "template_key-30".to_owned(),
        },
        AutopilotActionPayload::IssueReferralCode {
            fan_id: "fan_id-31".to_owned(),
        },
        AutopilotActionPayload::RunPlayStep {
            play_id: "play_id-32".to_owned(),
            play_kind: "play_kind-32".to_owned(),
            step_index: 1,
            step_kind: "step_kind-32".to_owned(),
            event_id: Some("event_id-32".to_owned()),
            fan_id: None,
            template_key: "template_key-32".to_owned(),
        },
        AutopilotActionPayload::RequestAgentContent {
            template_id: Some("template_id-33".to_owned()),
            task_id: "task_id-33".to_owned(),
            draft: json!({"subject": "draft subject", "body": "draft body"}),
        },
        AutopilotActionPayload::RequestOutreachTarget {
            task_id: "task_id-34".to_owned(),
            target_kind: "press".to_owned(),
            display_name: "Metal Hammer PL".to_owned(),
            contact_email: Some("editor@metalhammer.example".to_owned()),
            contact_domain: Some("metalhammer.example".to_owned()),
            why_fit: "Covers Polish metal scene".to_owned(),
            evidence_urls: json!(["https://example.invalid/article-1"]),
            subreddit: Some("r/metalpoland".to_owned()),
        },
        AutopilotActionPayload::RequestAgentRun {
            template_id: "template_id-35".to_owned(),
            prompt: "Find press contacts for Polish metal scene".to_owned(),
            priority: 2,
            tier: "standard".to_owned(),
        },
        AutopilotActionPayload::RequestCommunityEngagement {
            target_id: "target_id-36".to_owned(),
            platform: "reddit".to_owned(),
            subreddit: Some("r/metalpoland".to_owned()),
            title: "New Virya show announced".to_owned(),
            body: "Share your thoughts".to_owned(),
            smart_link: Some("https://virya.music/s/show-36".to_owned()),
        },
        AutopilotActionPayload::RequestSignalPush {
            task_id: "task_id-37".to_owned(),
            title: "Show near you".to_owned(),
            body: "Virya is playing in your city".to_owned(),
            target_path: Some("/events/show-37".to_owned()),
            event_id: Some("event_id-37".to_owned()),
            segment: Some("nearby_warsaw".to_owned()),
        },
    ];
    assert_eq!(samples.len(), 38, "all payload variants must be covered");
    for sample in &samples {
        let (encoded, re_encoded) = round_trip(sample);
        assert_eq!(
            encoded, re_encoded,
            "hand-written Deserialize must be the exact inverse of derived Serialize"
        );
        assert!(
            encoded.get("kind").and_then(Value::as_str).is_some(),
            "every payload must carry its internal tag"
        );
    }
}

#[test]
fn every_variant_tag_is_accepted() {
    let kinds = [
        "change_ticket_price",
        "change_ticket_capacity",
        "request_fan_lifecycle_message",
        "request_merch_reorder",
        "change_merch_price",
        "request_booking_outreach",
        "request_audience_campaign",
        "request_merch_bundle",
        "request_outreach",
        "request_beacon_discovery",
        "request_beacon_outreach",
        "request_show_growth",
        "request_content_artifact",
        "adjust_experiment",
        "complete_show_task",
        "escalate_show_task",
        "request_promotion_budget_change",
        "execute_release_milestone",
        "apply_live_opportunity",
        "prepare_funding_package",
        "submit_funding_application",
        "send_team_assignment_email",
        "verify_playlist_placement",
        "request_beacon_invite_batch",
        "request_outreach_discovery",
        "request_booking_target_discovery",
        "escalate_editorial_pitch",
        "counter_live_opportunity_terms",
        "accept_live_opportunity_terms",
        "raise_growth_opportunity",
        "raise_growth_debt",
        "issue_referral_code",
        "run_play_step",
        "request_agent_content",
        "request_outreach_target",
        "request_agent_run",
        "request_community_engagement",
        "request_signal_push",
    ];
    // A recognised tag with no fields decodes to `Unrecognized` carrying that
    // same tag — it is modellable in principle, just not with this body. A tag
    // this build has never heard of is indistinguishable at this level, which
    // is the point: neither may fail the decode. What separates them is the
    // ecosystem contract test, which compares the two repositories directly.
    for kind in kinds {
        let payload = serde_json::from_value::<AutopilotActionPayload>(json!({ "kind": kind }))
            .expect("a payload must never fail to decode; see the Deserialize impl");
        match payload {
            AutopilotActionPayload::Unrecognized { wire_kind } => {
                assert_eq!(wire_kind, kind, "the failing tag must be preserved");
            }
            other => panic!("`{kind}` decoded with no fields: {other:?}"),
        }
    }
}

/// An unknown tag used to be a decode error. It cannot be.
///
/// `payload` is a required field of `PendingAutopilotAction` and those arrive
/// as a list, so a rejected payload rejected its element and a rejected element
/// rejected the whole list: the operator lost the entire Autopilot screen
/// because of one row nobody could render. CrowdRelay adds action kinds on its
/// own cadence and a store-distributed client is always some versions behind —
/// at the time this changed, 16 backend payload kinds were already unmodellable
/// here. Drift is still caught, by `scripts/test_autopilot_wire_contract.py`
/// comparing the repositories, and still visible, as a labelled row.
#[test]
fn an_unknown_tag_decodes_to_the_unrecognized_variant() {
    let payload =
        serde_json::from_value::<AutopilotActionPayload>(json!({ "kind": "not_a_real_kind" }))
            .expect("an unknown tag must not fail the decode");
    match payload {
        AutopilotActionPayload::Unrecognized { wire_kind } => {
            assert_eq!(wire_kind, "not_a_real_kind");
        }
        other => panic!("expected Unrecognized, got {other:?}"),
    }
}

/// The other half of the same failure: a kind both sides know, whose shape has
/// moved. It degrades identically, and for the same reason — an error here
/// would cost the list, not the row.
#[test]
fn a_known_tag_missing_a_field_decodes_to_the_unrecognized_variant() {
    let payload = serde_json::from_value::<AutopilotActionPayload>(json!({
        "kind": "change_ticket_price",
        "ticket_type_id": "tt-1",
        "from_minor": 1000
    }))
    .expect("a missing field must not fail the decode");
    match payload {
        AutopilotActionPayload::Unrecognized { wire_kind } => {
            assert_eq!(wire_kind, "change_ticket_price");
        }
        other => panic!("expected Unrecognized, got {other:?}"),
    }
}

#[test]
fn unknown_provider_only_fields_stay_ignored() {
    let decoded: AutopilotActionPayload = serde_json::from_value(json!({
        "kind": "send_team_assignment_email",
        "assignment_id": "a-1",
        "recipient_name": "Ada",
        "task_title": "Load in",
        "task_detail": "Backline",
        "action_url_path": "/staff/tasks/a-1",
        "reminder_number": 1,
        "recipient_email": "ada@example.invalid",
        "some_future_provider_field": { "nested": [1, 2, 3] }
    }))
    .expect("unknown provider-only fields must be ignored, not rejected");
    let encoded = serde_json::to_value(&decoded).expect("re-serialises");
    assert!(
        encoded.get("recipient_email").is_none(),
        "provider-only fields must never be surfaced into the Signal contract"
    );
    assert!(
        encoded.get("due_at").is_some(),
        "an absent optional field must still round-trip as an explicit null"
    );
}
