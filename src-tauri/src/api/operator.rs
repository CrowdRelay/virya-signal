use reqwest::Method;

use crate::{
    AppError,
    models::{
        AudienceRevenueSummary, AudienceSummary, ConcertQrOverview, CreateQrCampaignInput,
        IssuePassInput, OperatorProfile, OperatorRole, OperatorSignalOverview, PublicEventsResult,
        ShowChecklist, ShowModeSnapshot, StaffEventDashboard, TicketingOverview,
    },
};

use super::http::{require_owner, segment};

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
    ) -> Result<PublicEventsResult, AppError> {
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

    pub async fn operator_event_merch_summary(
        &self,
        profile: &OperatorProfile,
        event_id: &str,
    ) -> Result<crate::models::EventMerchSummary, AppError> {
        let path = format!("staff/events/{}/commerce-summary", segment(event_id)?);
        self.auth_json::<crate::models::EventMerchSummary, ()>(profile, Method::GET, &path, None)
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
