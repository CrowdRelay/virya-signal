use reqwest::Method;

use crate::{
    AppError,
    models::{
        AudienceRevenueSummary, AudienceSummary, AutopilotAssignRequest, AutopilotAuthorityRequest,
        AutopilotChiefOfStaff, AutopilotMutation, ConcertQrOverview, CreateQrCampaignInput,
        IssuePassInput, OperatorAutopilotOverview, OperatorOpsOverview, OperatorProfile,
        OperatorRole, OperatorSignalOverview, OpsDeliveryItem, OpsOutboxItem, OpsRetryResult,
        OpsSummary, PublicEvent, ShowChecklist, ShowModeSnapshot, StaffEventDashboard,
        TicketingOverview,
    },
};

use super::http::{require_owner, segment, uuid_segment};

const AUTOPILOT_DATE_FIELDS: &[&str] = &[
    "guarded_until",
    "created_at",
    "approval_expires_at",
    "assignment_due_at",
    "evaluated_at",
    "finished_at",
    "executor_reported_at",
    "observed_at",
    "synced_at",
    "occurred_at",
    "expires_at",
    "workflow_attested_at",
    "starts_at",
    "due_at",
    "release_at",
];

fn legacy_offset_datetime_tuple(value: &serde_json::Value) -> Option<String> {
    let parts = value.as_array()?;
    if parts.len() != 9 {
        return None;
    }
    let year = i32::try_from(parts[0].as_i64()?).ok()?;
    let ordinal = u16::try_from(parts[1].as_u64()?).ok()?;
    let hour = u8::try_from(parts[2].as_u64()?).ok()?;
    let minute = u8::try_from(parts[3].as_u64()?).ok()?;
    let second = u8::try_from(parts[4].as_u64()?).ok()?;
    let nanosecond = u32::try_from(parts[5].as_u64()?).ok()?;
    let offset_hour = i8::try_from(parts[6].as_i64()?).ok()?;
    let offset_minute = i8::try_from(parts[7].as_i64()?).ok()?;
    let offset_second = i8::try_from(parts[8].as_i64()?).ok()?;

    let date = time::Date::from_ordinal_date(year, ordinal).ok()?;
    let local = date.with_hms_nano(hour, minute, second, nanosecond).ok()?;
    let offset = time::UtcOffset::from_hms(offset_hour, offset_minute, offset_second).ok()?;
    local
        .assume_offset(offset)
        .format(&time::format_description::well_known::Rfc3339)
        .ok()
}

fn normalize_legacy_autopilot_dates(value: &mut serde_json::Value) {
    match value {
        serde_json::Value::Object(object) => {
            for (key, nested) in object.iter_mut() {
                if AUTOPILOT_DATE_FIELDS.contains(&key.as_str())
                    && let Some(timestamp) = legacy_offset_datetime_tuple(nested)
                {
                    *nested = serde_json::Value::String(timestamp);
                    continue;
                }
                normalize_legacy_autopilot_dates(nested);
            }
        }
        serde_json::Value::Array(values) => {
            for nested in values {
                normalize_legacy_autopilot_dates(nested);
            }
        }
        _ => {}
    }
}

fn decode_autopilot_wire<T: serde::de::DeserializeOwned>(
    mut value: serde_json::Value,
) -> Result<T, AppError> {
    normalize_legacy_autopilot_dates(&mut value);
    serde_json::from_value(value).map_err(AppError::from)
}

impl super::CrowdRelayClient {
    pub async fn validate(&self, profile: &OperatorProfile) -> Result<(), AppError> {
        let path = match profile.role {
            OperatorRole::Owner => "admin/event-qr/overview",
            OperatorRole::Staff => "staff/event-qr/overview",
        };
        let _: serde_json::Value = self
            .auth_json(profile, Method::GET, path, Option::<&()>::None)
            .await?;
        Ok(())
    }

    pub async fn operator_events(
        &self,
        profile: &OperatorProfile,
    ) -> Result<Vec<PublicEvent>, AppError> {
        self.public_events(&profile.api_base_url).await
    }

    pub async fn operator_show_checklist(
        &self,
        profile: &OperatorProfile,
        event_slug: &str,
    ) -> Result<ShowChecklist, AppError> {
        let path = format!("staff/ecosystem/checklists/{}", segment(event_slug)?);
        self.auth_json::<ShowChecklist, ()>(profile, Method::GET, &path, None)
            .await
    }

    pub async fn operator_update_show_checklist(
        &self,
        profile: &OperatorProfile,
        event_slug: &str,
        item_key: &str,
        status: &str,
    ) -> Result<ShowChecklist, AppError> {
        let path = format!(
            "staff/ecosystem/checklists/{}/{}",
            segment(event_slug)?,
            segment(item_key)?
        );
        let body = serde_json::json!({ "status": status, "note": null });
        self.auth_json(profile, Method::POST, &path, Some(&body))
            .await
    }

    pub async fn operator_push_config(
        &self,
        profile: &OperatorProfile,
    ) -> Result<super::fan::FanPushConfigApi, AppError> {
        self.require_capability(&profile.api_base_url, "staff_show_checklist_push_v1")
            .await?;
        let response = self
            .http
            .get(super::http::endpoint(
                &profile.api_base_url,
                "public/push/config",
            )?)
            .header(reqwest::header::ACCEPT, "application/json")
            .send()
            .await?;
        super::http::decode(response).await
    }

    pub async fn operator_register_android_push(
        &self,
        profile: &OperatorProfile,
        installation_id: &str,
        fcm_token: &str,
    ) -> Result<super::fan::FanPushMutationApi, AppError> {
        self.require_capability(&profile.api_base_url, "staff_show_checklist_push_v1")
            .await?;
        let body = serde_json::json!({
            "installation_id": installation_id,
            "transport": "android_fcm",
            "endpoint": fcm_token,
            "p256dh": null,
            "auth": null,
        });
        self.auth_json(profile, Method::POST, "staff/push/endpoints", Some(&body))
            .await
    }

    pub async fn operator_disable_android_push(
        &self,
        profile: &OperatorProfile,
        installation_id: &str,
    ) -> Result<super::fan::FanPushMutationApi, AppError> {
        self.require_capability(&profile.api_base_url, "staff_show_checklist_push_v1")
            .await?;
        let body = serde_json::json!({
            "installation_id": installation_id,
            "transport": "android_fcm",
        });
        self.auth_json(
            profile,
            Method::POST,
            "staff/push/endpoints/disable",
            Some(&body),
        )
        .await
    }

    pub async fn operator_qr(
        &self,
        profile: &OperatorProfile,
    ) -> Result<ConcertQrOverview, AppError> {
        let qr_path = match profile.role {
            OperatorRole::Owner => "admin/event-qr/overview",
            OperatorRole::Staff => "staff/event-qr/overview",
        };
        self.auth_json::<ConcertQrOverview, ()>(profile, Method::GET, qr_path, None)
            .await
    }

    pub async fn staff_event_dashboard(
        &self,
        profile: &OperatorProfile,
        event_slug: &str,
    ) -> Result<StaffEventDashboard, AppError> {
        self.auth_json(
            profile,
            Method::GET,
            &format!("staff/events/{}/dashboard", segment(event_slug)?),
            Option::<&()>::None,
        )
        .await
    }

    pub async fn ticketing_overview(
        &self,
        profile: &OperatorProfile,
        event_slug: &str,
    ) -> Result<TicketingOverview, AppError> {
        let prefix = match profile.role {
            OperatorRole::Owner => "admin",
            OperatorRole::Staff => "staff",
        };
        self.auth_json(
            profile,
            Method::GET,
            &format!("{prefix}/events/{}/ticketing", segment(event_slug)?),
            Option::<&()>::None,
        )
        .await
    }

    pub async fn redeem_admission(
        &self,
        profile: &OperatorProfile,
        event_slug: &str,
        raw_code: &str,
    ) -> Result<serde_json::Value, AppError> {
        let token = super::http::normalize_scanned_code(raw_code)?;
        let body = if token.starts_with("v1.") || token.starts_with("t1.") {
            serde_json::json!({"event_slug": event_slug, "qr_token": token, "public_reference": null})
        } else {
            serde_json::json!({"event_slug": event_slug, "qr_token": null, "public_reference": token})
        };
        self.auth_json(profile, Method::POST, "staff/admission/redeem", Some(&body))
            .await
    }

    pub async fn redeem_coupon(
        &self,
        profile: &OperatorProfile,
        code: &str,
        order_reference: &str,
    ) -> Result<serde_json::Value, AppError> {
        let code =
            super::http::bounded_required(code, crate::i18n::tr("native_coupon_code_label"), 128)?;
        let order_reference = super::http::bounded_required(
            order_reference,
            crate::i18n::tr("native_sale_number_label"),
            200,
        )?;
        let body = serde_json::json!({"code": code.to_ascii_uppercase(), "order_reference": order_reference});
        self.auth_json(profile, Method::POST, "staff/coupons/redeem", Some(&body))
            .await
    }

    pub async fn issue_pass(
        &self,
        profile: &OperatorProfile,
        input: &IssuePassInput,
    ) -> Result<serde_json::Value, AppError> {
        require_owner(profile)?;
        self.auth_json(profile, Method::POST, "admin/admission/passes", Some(input))
            .await
    }

    pub async fn revoke_pass(
        &self,
        profile: &OperatorProfile,
        reference: &str,
    ) -> Result<serde_json::Value, AppError> {
        require_owner(profile)?;
        self.auth_json(
            profile,
            Method::POST,
            &format!("admin/admission/passes/{}/revoke", segment(reference)?),
            Option::<&()>::None,
        )
        .await
    }

    pub async fn create_qr_campaign(
        &self,
        profile: &OperatorProfile,
        input: &CreateQrCampaignInput,
    ) -> Result<serde_json::Value, AppError> {
        self.auth_json(
            profile,
            Method::POST,
            "staff/event-qr/campaigns",
            Some(input),
        )
        .await
    }

    pub async fn revoke_qr_campaign(
        &self,
        profile: &OperatorProfile,
        campaign_id: &str,
    ) -> Result<serde_json::Value, AppError> {
        self.auth_json(
            profile,
            Method::POST,
            &format!("staff/event-qr/campaigns/{}/revoke", segment(campaign_id)?),
            Option::<&()>::None,
        )
        .await
    }

    pub async fn operator_signal_overview(
        &self,
        profile: &OperatorProfile,
    ) -> Result<OperatorSignalOverview, AppError> {
        require_owner(profile)?;
        let signal_request = self.auth_json::<OperatorSignalOverview, ()>(
            profile,
            Method::GET,
            "admin/signal/overview",
            None,
        );
        let audience_request = self.auth_json::<AudienceSummary, ()>(
            profile,
            Method::GET,
            "admin/audience/overview",
            None,
        );
        let revenue_request = self.auth_json::<Vec<AudienceRevenueSummary>, ()>(
            profile,
            Method::GET,
            "admin/analytics/revenue",
            None,
        );
        let (signal_result, audience_result, revenue_result) =
            futures_util::future::join3(signal_request, audience_request, revenue_request).await;
        let mut overview = signal_result?;

        match audience_result {
            Ok(mut audience) => {
                audience.active_fans = audience.active_fans.max(0);
                audience.marketing_consented_fans = audience.marketing_consented_fans.max(0);
                audience.ticket_buyers = audience.ticket_buyers.max(0);
                audience.attendees = audience.attendees.max(0);
                audience.synesthesia_participants = audience.synesthesia_participants.max(0);
                audience.qualified_referrals = audience.qualified_referrals.max(0);
                audience.paid_ticket_orders = audience.paid_ticket_orders.max(0);
                overview.audience = audience;
            }
            Err(_) => overview.unavailable_sources.push("audience".to_owned()),
        }
        match revenue_result {
            Ok(mut revenue) => {
                revenue.retain(|row| {
                    row.currency.len() == 3
                        && row.currency.bytes().all(|byte| byte.is_ascii_uppercase())
                        && row.paid_orders >= 0
                        && row.gross_paid_minor >= 0
                        && row.refunded_minor >= 0
                        && row.refunded_minor <= row.gross_paid_minor
                        && row.after_refunds_minor == row.gross_paid_minor - row.refunded_minor
                });
                revenue.truncate(8);
                overview.ticket_revenue = revenue;
            }
            Err(_) => overview.unavailable_sources.push("revenue".to_owned()),
        }

        let summary = &mut overview.summary;
        summary.total_fans = summary.total_fans.max(0);
        summary.active_fans = summary.active_fans.max(0);
        summary.pending_fans = summary.pending_fans.max(0);
        summary.unsubscribed_fans = summary.unsubscribed_fans.max(0);
        summary.suppressed_fans = summary.suppressed_fans.max(0);
        summary.marketing_opted_in = summary.marketing_opted_in.max(0);
        summary.nearby_enabled = summary.nearby_enabled.max(0);

        let activity = &mut overview.activity;
        activity.new_fans_7d = activity.new_fans_7d.max(0);
        activity.new_fans_30d = activity.new_fans_30d.max(0);
        activity.referral_attributions_total = activity.referral_attributions_total.max(0);
        activity.referral_attributions_30d = activity.referral_attributions_30d.max(0);
        activity.event_interests_total = activity.event_interests_total.max(0);
        activity.event_interests_30d = activity.event_interests_30d.max(0);
        activity.nearby_notifications_30d = activity.nearby_notifications_30d.max(0);
        activity.pending_city_requests = activity.pending_city_requests.max(0);

        overview.top_cities.retain(|city| {
            !city.name.trim().is_empty()
                && city.name.chars().count() <= 160
                && city.country_code.len() == 2
                && city
                    .country_code
                    .bytes()
                    .all(|byte| byte.is_ascii_uppercase())
                && city.active_fans >= 0
        });
        overview.top_cities.truncate(10);
        overview.unavailable_sources.retain(|source| {
            !source.trim().is_empty()
                && source.len() <= 64
                && source
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
        });
        overview.unavailable_sources.sort_unstable();
        overview.unavailable_sources.dedup();
        overview.unavailable_sources.truncate(8);
        overview.generated_at = overview.generated_at.chars().take(64).collect();

        Ok(overview)
    }

    pub async fn operator_ops_overview(
        &self,
        profile: &OperatorProfile,
    ) -> Result<OperatorOpsOverview, AppError> {
        require_owner(profile)?;
        let summary_request =
            self.auth_json::<OpsSummary, ()>(profile, Method::GET, "admin/ops/summary", None);
        let deliveries_request = self.auth_json::<Vec<OpsDeliveryItem>, ()>(
            profile,
            Method::GET,
            "admin/ops/deliveries?status=dead&limit=25",
            None,
        );
        let outbox_request = self.auth_json::<Vec<OpsOutboxItem>, ()>(
            profile,
            Method::GET,
            "admin/ops/outbox?status=dead&limit=25",
            None,
        );
        let (summary_result, deliveries_result, outbox_result) =
            futures_util::future::join3(summary_request, deliveries_request, outbox_request).await;
        if summary_result.is_err() && deliveries_result.is_err() && outbox_result.is_err() {
            return match summary_result {
                Err(error) => Err(error),
                Ok(_) => Err(AppError::BackgroundTask),
            };
        }
        let mut unavailable_sources = Vec::new();
        let summary = match summary_result {
            Ok(value) => value,
            Err(_) => {
                unavailable_sources.push("summary".to_owned());
                OpsSummary::default()
            }
        };
        let dead_deliveries = match deliveries_result {
            Ok(value) => value,
            Err(_) => {
                unavailable_sources.push("deliveries".to_owned());
                Vec::new()
            }
        };
        let dead_outbox = match outbox_result {
            Ok(value) => value,
            Err(_) => {
                unavailable_sources.push("outbox".to_owned());
                Vec::new()
            }
        };
        Ok(OperatorOpsOverview {
            summary,
            dead_deliveries,
            dead_outbox,
            unavailable_sources,
        })
    }

    pub async fn operator_autopilot_overview(
        &self,
        profile: &OperatorProfile,
    ) -> Result<OperatorAutopilotOverview, AppError> {
        require_owner(profile)?;
        let value = self
            .auth_json::<serde_json::Value, ()>(
                profile,
                Method::GET,
                "admin/autopilot/overview",
                None,
            )
            .await?;
        decode_autopilot_wire(value)
    }

    pub async fn operator_autopilot_chief_of_staff(
        &self,
        profile: &OperatorProfile,
    ) -> Result<AutopilotChiefOfStaff, AppError> {
        require_owner(profile)?;
        let value = self
            .auth_json::<serde_json::Value, ()>(
                profile,
                Method::GET,
                "admin/autopilot/chief-of-staff",
                None,
            )
            .await?;
        decode_autopilot_wire(value)
    }

    pub async fn operator_autopilot_set_authority(
        &self,
        profile: &OperatorProfile,
        context: &str,
        mut body: AutopilotAuthorityRequest,
    ) -> Result<AutopilotMutation, AppError> {
        require_owner(profile)?;
        let context = match context {
            "ticket_yield"
            | "fan_lifecycle"
            | "campaign_lifecycle"
            | "merchandising"
            | "merch_pricing"
            | "merch_bundle"
            | "booking_opportunity"
            | "outreach"
            | "content_supply"
            | "promotion_budget"
            | "experimentation"
            | "show_operations"
            | "release"
            | "live_opportunity"
            | "funding"
            | "beacon"
            | "show_growth" => context,
            _ => return Err(AppError::InvalidInput("Invalid Autopilot context".into())),
        };
        let autonomy_level = match body.autonomy_level.trim() {
            "observe" | "recommend" | "require_approval" | "bounded_auto" => {
                body.autonomy_level.trim().to_owned()
            }
            _ => return Err(AppError::InvalidInput("Invalid Autopilot authority".into())),
        };
        if body.minimum_confidence_basis_points > 10_000
            || !(1..=1000).contains(&body.max_actions_24h)
            || body.expected_version <= 0
        {
            return Err(AppError::InvalidInput("Invalid Autopilot policy".into()));
        }
        body.autonomy_level = autonomy_level;
        self.auth_json(
            profile,
            Method::POST,
            &format!("admin/autopilot/policies/{context}"),
            Some(&body),
        )
        .await
    }

    pub async fn operator_autopilot_assign(
        &self,
        profile: &OperatorProfile,
        action_id: &str,
        member_key: &str,
    ) -> Result<AutopilotMutation, AppError> {
        require_owner(profile)?;
        let action_id = uuid_segment(action_id)?;
        let member_key = super::http::bounded_required(member_key, "member key", 48)?;
        self.auth_json(
            profile,
            Method::POST,
            &format!("admin/autopilot/actions/{action_id}/assign"),
            Some(&AutopilotAssignRequest {
                member_key: member_key.to_owned(),
            }),
        )
        .await
    }

    pub async fn operator_autopilot_approve(
        &self,
        profile: &OperatorProfile,
        action_id: &str,
    ) -> Result<AutopilotMutation, AppError> {
        require_owner(profile)?;
        let action_id = uuid_segment(action_id)?;
        self.auth_json::<AutopilotMutation, ()>(
            profile,
            Method::POST,
            &format!("admin/autopilot/actions/{action_id}/approve"),
            None,
        )
        .await
    }

    pub async fn operator_autopilot_cancel(
        &self,
        profile: &OperatorProfile,
        action_id: &str,
    ) -> Result<AutopilotMutation, AppError> {
        require_owner(profile)?;
        let action_id = uuid_segment(action_id)?;
        self.auth_json::<AutopilotMutation, ()>(
            profile,
            Method::POST,
            &format!("admin/autopilot/actions/{action_id}/cancel"),
            None,
        )
        .await
    }

    pub async fn operator_retry(
        &self,
        profile: &OperatorProfile,
        target_kind: &str,
        target_id: &str,
    ) -> Result<OpsRetryResult, AppError> {
        require_owner(profile)?;
        let target_kind = match target_kind {
            "outbox" | "deliveries" => target_kind,
            _ => {
                return Err(AppError::InvalidInput(
                    crate::i18n::tr("native_queue_type_invalid").into(),
                ));
            }
        };
        let target_id = uuid_segment(target_id)?;
        self.auth_json::<OpsRetryResult, ()>(
            profile,
            Method::POST,
            &format!("admin/ops/{target_kind}/{target_id}/retry"),
            None,
        )
        .await
    }

    pub async fn show_mode_snapshot(
        &self,
        profile: &OperatorProfile,
        event_slug: &str,
    ) -> Result<ShowModeSnapshot, AppError> {
        self.auth_json::<ShowModeSnapshot, ()>(
            profile,
            Method::GET,
            &format!("staff/ops/show-snapshot/{}", segment(event_slug)?),
            None,
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use super::{
        decode_autopilot_wire, legacy_offset_datetime_tuple, normalize_legacy_autopilot_dates,
    };
    use crate::models::AutopilotChiefOfStaff;

    #[test]
    fn legacy_time_tuple_is_normalized_to_rfc3339() {
        let value = serde_json::json!([2026, 227, 6, 45, 12, 123_400_000, 0, 0, 0]);
        assert_eq!(
            legacy_offset_datetime_tuple(&value).as_deref(),
            Some("2026-08-15T06:45:12.1234Z")
        );
    }

    #[test]
    fn legacy_time_tuple_preserves_non_utc_offset() {
        let value = serde_json::json!([2026, 227, 8, 45, 12, 0, 2, 0, 0]);
        assert_eq!(
            legacy_offset_datetime_tuple(&value).as_deref(),
            Some("2026-08-15T08:45:12+02:00")
        );
    }

    #[test]
    fn nested_autopilot_dates_are_normalized_without_touching_other_arrays() {
        let mut value = serde_json::json!({
            "created_at": [2026, 227, 6, 45, 12, 0, 0, 0, 0],
            "payload": {
                "kind": "adjust_experiment",
                "allocations": [1000, 9000]
            }
        });
        normalize_legacy_autopilot_dates(&mut value);
        assert_eq!(value["created_at"].as_str(), Some("2026-08-15T06:45:12Z"));
        assert_eq!(
            value["payload"]["allocations"],
            serde_json::json!([1000, 9000])
        );
    }

    #[test]
    fn chief_of_staff_accepts_legacy_backend_time_sequences() {
        let value = serde_json::json!({
            "executed_24h": 1,
            "failed_24h": 0,
            "needs_you": 1,
            "estimated_minutes_saved_24h": 12,
            "measured_improved_7d": 1,
            "measured_neutral_7d": 0,
            "measured_worsened_7d": 0,
            "attention_items": [{
                "kind": "opportunity_deadline",
                "subject_kind": "team_opportunity",
                "subject_id": "00000000-0000-0000-0000-000000000001",
                "title": "Deadline",
                "detail": "Submit",
                "due_at": [2026, 227, 6, 45, 12, 0, 0, 0, 0],
                "urgency": "today"
            }],
            "top_opportunities": [],
            "show_tasks": [{
                "event_id": "00000000-0000-0000-0000-000000000002",
                "event_title": "Show",
                "task_key": "network_tested",
                "status": "pending",
                "starts_at": [2026, 228, 18, 30, 0, 0, 0, 0, 0]
            }]
        });
        let decoded: AutopilotChiefOfStaff = match decode_autopilot_wire(value) {
            Ok(decoded) => decoded,
            Err(error) => panic!("legacy payload should decode: {error:?}"),
        };
        assert_eq!(decoded.attention_items[0].due_at, "2026-08-15T06:45:12Z");
        assert_eq!(decoded.show_tasks[0].starts_at, "2026-08-16T18:30:00Z");
    }
}
