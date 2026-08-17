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
    ];
    assert_eq!(samples.len(), 22, "all payload variants must be covered");
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
    ];
    for kind in kinds {
        let error = serde_json::from_value::<AutopilotActionPayload>(json!({ "kind": kind }))
            .expect_err("a tag with no fields must fail on a missing field, not an unknown tag");
        let message = error.to_string();
        assert!(
            !message.contains("unknown variant"),
            "`{kind}` must be a recognised payload tag, got: {message}"
        );
    }
}

#[test]
fn unknown_tag_is_rejected_as_an_unknown_variant() {
    let error =
        serde_json::from_value::<AutopilotActionPayload>(json!({ "kind": "not_a_real_kind" }))
            .expect_err("unknown payload tags must not decode");
    assert!(
        error.to_string().contains("unknown variant"),
        "unknown tags must report an unknown variant: {error}"
    );
}

#[test]
fn missing_required_field_is_reported_by_name() {
    let error = serde_json::from_value::<AutopilotActionPayload>(json!({
        "kind": "change_ticket_price",
        "ticket_type_id": "tt-1",
        "from_minor": 1000
    }))
    .expect_err("a missing required field must not decode");
    assert!(
        error.to_string().contains("to_minor"),
        "the missing field must be named: {error}"
    );
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
