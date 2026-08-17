#[cfg(test)]
mod tests {
    use super::*;
    use crate::models;

    fn test_value<T, E>(result: Result<T, E>) -> T
    where
        E: std::fmt::Debug,
    {
        match result {
            Ok(value) => value,
            Err(error) => panic!("test setup failed: {error:?}"),
        }
    }

    #[test]
    fn qr_render_is_bounded_and_produces_svg() {
        let svg = test_value(render_qr("v1.test-token"));
        assert!(svg.starts_with("<svg"));
        assert!(render_qr("").is_err());
        assert!(render_qr(&"x".repeat(MAX_SECRET_BYTES + 1)).is_err());
    }

    #[test]
    fn wallet_tokens_are_split_from_the_webview_payload() {
        let wallet = TicketWalletApi {
            order: models::WalletOrder {
                order_id: "01234567-89ab-cdef-0123-456789abcdef".into(),
                public_reference: "VRY-ORDER".into(),
                event_title: "Virya Live".into(),
                venue: Some("Club".into()),
                starts_at: "2026-08-01T20:00:00Z".into(),
                status: "paid".into(),
            },
            tickets: vec![models::WalletTicketApi {
                ticket_type_name: "Regular".into(),
                public_reference: "VRY-TICKET".into(),
                holder_name: Some("Fan".into()),
                holder_email_masked: "f***@example.com".into(),
                status: "claimed".into(),
                redeemed_at: None,
                qr_token: Some("v1.private-token".into()),
                qr_expires_at: "2099-08-01T21:00:00Z".into(),
            }],
        };
        let (public, tokens, cached_qr) = prepare_wallet(wallet);
        assert!(public.tickets[0].qr_available);
        assert_eq!(public.tickets[0].status, "claimed");
        assert_eq!(public.tickets[0].redeemed_at, None);
        assert_eq!(tokens["VRY-TICKET"].as_str(), "v1.private-token");
        assert_eq!(cached_qr.len(), 1);
    }

    #[test]
    fn invalid_wallet_qr_tokens_are_not_cached() {
        let wallet = TicketWalletApi {
            order: models::WalletOrder {
                order_id: "01234567-89ab-cdef-0123-456789abcdef".into(),
                public_reference: "VRY-ORDER".into(),
                event_title: "Virya Live".into(),
                venue: None,
                starts_at: "2026-08-01T20:00:00Z".into(),
                status: "paid".into(),
            },
            tickets: vec![models::WalletTicketApi {
                ticket_type_name: "Regular".into(),
                public_reference: "VRY-TICKET".into(),
                holder_name: None,
                holder_email_masked: "f***@example.com".into(),
                status: "claimed".into(),
                redeemed_at: None,
                qr_token: Some(String::new()),
                qr_expires_at: "2026-08-01T21:00:00Z".into(),
            }],
        };
        let (public, tokens, cached_qr) = prepare_wallet(wallet);
        assert!(!public.tickets[0].qr_available);
        assert!(tokens.is_empty());
        assert!(cached_qr.is_empty());
    }
}
