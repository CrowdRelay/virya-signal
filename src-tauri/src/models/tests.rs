#[cfg(test)]
mod tests {
    use super::*;

    fn test_value<T, E>(result: Result<T, E>) -> T
    where
        E: std::fmt::Debug,
    {
        match result {
            Ok(value) => value,
            Err(error) => panic!("test setup failed: {error:?}"),
        }
    }

    fn test_some<T>(value: Option<T>) -> T {
        match value {
            Some(value) => value,
            None => panic!("test setup expected Some value"),
        }
    }

    #[test]
    fn public_event_ignores_backend_fields_outside_the_webview_contract() {
        let event: PublicEvent = test_value(serde_json::from_value(serde_json::json!({
            "id": "backend-only",
            "slug": "virya-live",
            "title": "Virya Live",
            "description": "Concert",
            "city": {"id": "backend-only", "name": "Wrocław"},
            "venue": "Club",
            "starts_at": "2026-08-01T20:00:00Z",
            "ticket_url": "https://virya.music/tickets",
            "image_url": null,
            "large_backend_object": {"unused": true}
        })));
        assert_eq!(event.slug, "virya-live");
        assert_eq!(test_some(event.city).name, "Wrocław");
    }

    #[test]
    fn ticketing_overview_keeps_only_rendered_fields() {
        let overview: TicketingOverview = test_value(serde_json::from_value(serde_json::json!({
            "sale": {
                "currency": "PLN",
                "reserved": 3,
                "available": 97,
                "ticket_types": [{"id": "unused"}]
            },
            "paid_tickets": 42,
            "gross_sales_minor": 123400,
            "refunded_minor": 0,
            "recent_orders": [{
                "public_reference": "VRY-ORDER",
                "buyer_email_masked": "f***@example.com",
                "buyer_name": "Fan",
                "currency": "PLN",
                "amount_gross_minor": 9900,
                "tickets": [{"private": "unused"}]
            }]
        })));
        assert_eq!(overview.sale.available, 97);
        assert_eq!(overview.recent_orders.len(), 1);
    }

    #[test]
    fn referral_payload_accepts_extra_reward_metadata() {
        let referral: ReferralProgress = test_value(serde_json::from_value(serde_json::json!({
            "referral_code": "VIRYA",
            "qualified_referrals": 2,
            "pending_referrals": 1,
            "draw_entries": [{
                "name": "Backstage",
                "prize_kind": "pass",
                "draw_at": "2026-08-02T20:00:00Z",
                "total_entries": 4,
                "max_entries": 99
            }],
            "coupons": [],
            "physical_rewards": []
        })));
        assert_eq!(referral.draw_entries[0].total_entries, 4);
    }

    #[test]
    fn referral_payload_accepts_legacy_byte_sequences_for_text_fields() {
        let referral: ReferralProgress = test_value(serde_json::from_value(serde_json::json!({
            "referral_code": b"VIRYA".to_vec(),
            "qualified_referrals": 2,
            "pending_referrals": 1,
            "draw_entries": [{
                "name": b"Backstage".to_vec(),
                "prize_kind": b"pass".to_vec(),
                "draw_at": b"2026-08-02T20:00:00Z".to_vec(),
                "total_entries": 4
            }],
            "coupons": [{
                "code": b"VIRYA10".to_vec(),
                "discount_percent": 10,
                "status": b"active".to_vec()
            }],
            "physical_rewards": [{
                "item_name": b"Album".to_vec(),
                "sku": b"ALBUM-01".to_vec(),
                "status": b"granted".to_vec()
            }]
        })));
        assert_eq!(referral.referral_code, "VIRYA");
        assert_eq!(referral.draw_entries[0].name, "Backstage");
        assert_eq!(referral.coupons[0].code, "VIRYA10");
        assert_eq!(referral.physical_rewards[0].sku, "ALBUM-01");
    }
}

#[cfg(test)]
mod fan_profile_compat_tests {
    use super::*;

    #[test]
    fn legacy_byte_sequences_are_migrated_to_strings() {
        let payload = serde_json::json!({
            "api_base_url": b"https://signal-api.virya.music/v1/".to_vec(),
            "email": b"fan@example.com".to_vec(),
            "display_name": b"Fan".to_vec(),
            "fan_session_token": b"session-token".to_vec(),
            "pass_session_token": null,
            "wallets": [{
                "order_id": b"01234567-89ab-cdef-0123-456789abcdef".to_vec(),
                "checkout_token": b"checkout-token".to_vec()
            }]
        });
        let profile: FanProfile = match serde_json::from_value(payload) {
            Ok(profile) => profile,
            Err(error) => panic!("legacy profile migration failed: {error}"),
        };
        assert_eq!(profile.email, "fan@example.com");
        assert_eq!(profile.fan_session_token, "session-token");
        assert_eq!(profile.wallets[0].checkout_token, "checkout-token");
        assert!(!profile.area_wallet_id.is_empty());
    }
}

#[cfg(test)]
mod compat_string_shape_tests {
    use super::*;

    fn normalized(value: Value) -> String {
        match normalize_compat_string(value) {
            Ok(value) => value,
            Err(error) => panic!("compatibility normalization failed: {error}"),
        }
    }

    #[test]
    fn accepts_node_buffer_objects() {
        assert_eq!(
            normalized(serde_json::json!({
                "type": "Buffer",
                "data": [86, 73, 82, 89, 65]
            })),
            "VIRYA"
        );
    }

    #[test]
    fn accepts_indexed_byte_objects() {
        assert_eq!(
            normalized(serde_json::json!({
                "0": 86,
                "1": 73,
                "2": 82,
                "3": 89,
                "4": 65
            })),
            "VIRYA"
        );
    }

    #[test]
    fn accepts_character_sequences() {
        assert_eq!(
            normalized(serde_json::json!(["V", "I", "R", "Y", "A"])),
            "VIRYA"
        );
    }

    #[test]
    fn accepts_signed_utf8_byte_sequences() {
        assert_eq!(normalized(serde_json::json!([-59, -68])), "ż");
    }

    #[test]
    fn accepts_utf16_code_units() {
        assert_eq!(
            normalized(serde_json::json!([86, 105, 114, 121, 97, 32, 281])),
            "Virya ę"
        );
    }

    #[test]
    fn accepts_wrapped_compatibility_values() {
        assert_eq!(
            normalized(serde_json::json!({
                "value": {"bytes": [86, 73, 82, 89, 65]}
            })),
            "VIRYA"
        );
    }

    #[test]
    fn null_area_wallet_id_is_regenerated() {
        let payload = serde_json::json!({
            "api_base_url": "https://signal-api.virya.music/v1/",
            "area_wallet_id": null,
            "email": "fan@example.com",
            "display_name": null,
            "fan_session_token": "session-token",
            "pass_session_token": null,
            "wallets": []
        });
        let profile: FanProfile = match serde_json::from_value(payload) {
            Ok(profile) => profile,
            Err(error) => panic!("null wallet id migration failed: {error}"),
        };
        assert!(!profile.area_wallet_id.is_empty());
    }

    #[test]
    fn null_referral_code_uses_empty_compatibility_value() {
        let payload = serde_json::json!({
            "referral_code": null,
            "qualified_referrals": 0,
            "pending_referrals": 0,
            "draw_entries": [],
            "coupons": [],
            "physical_rewards": []
        });
        let referral: ReferralProgress = match serde_json::from_value(payload) {
            Ok(referral) => referral,
            Err(error) => panic!("null referral migration failed: {error}"),
        };
        assert!(referral.referral_code.is_empty());
    }
}

pub use virya_signal_contracts::push::*;
