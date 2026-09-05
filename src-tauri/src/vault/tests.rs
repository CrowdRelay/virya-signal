#[cfg(test)]
#[allow(clippy::expect_used)]
mod tests {
    use super::*;
    use crate::models::OperatorRole;

    fn test_dir(name: &str) -> PathBuf {
        std::env::temp_dir().join(format!("virya-signal-{name}-{}", uuid::Uuid::new_v4()))
    }

    fn profile() -> OperatorProfile {
        OperatorProfile {
            display_name: "Bramka Virya".to_owned(),
            api_base_url: "https://signal-api.virya.music/v1/".to_owned(),
            role: OperatorRole::Staff,
            bearer_token: "staff-device-token-0123456789abcdef".to_owned(),
            session_expires_at: None,
        }
    }

    #[test]
    fn operator_pin_survives_a_fresh_vault_round_trip() {
        let directory = test_dir("operator-round-trip");
        let expected = profile();
        let persisted = save_verified(&directory, "1234", &expected)
            .expect("operator profile should persist and reopen");
        assert_eq!(persisted, expected);
        assert!(exists(&directory));
        let open = |pin: &str| {
            let password = operator_password(&directory, pin)?;
            load_operator_with_password(&directory, password.as_ref())
        };
        assert_eq!(open("1234").expect("same PIN should unlock"), expected);
        assert!(matches!(open("4321"), Err(AppError::InvalidPin)));
        let _ = std::fs::remove_dir_all(directory);
    }

    fn fan_profile() -> FanProfile {
        FanProfile {
            api_base_url: "https://signal-api.virya.music/v1/".to_owned(),
            area_wallet_id: "6f7c2f2a-0000-4000-8000-000000000001".to_owned(),
            email: "kasia@example.com".to_owned(),
            display_name: Some("Kasia".to_owned()),
            fan_session_token: "fan-session-token-0123456789abcdef".to_owned(),
            push_enabled: false,
            push_last_sync_ok: false,
            pass_session_token: None,
            wallets: Vec::new(),
            cached_wallets: Vec::new(),
            cached_wallet_qr: Vec::new(),
        }
    }

    /// A vault with no PIN behind it still has to look configured, or the gate
    /// reports "not set up" for a device the fan just signed into.
    #[test]
    fn a_device_sealed_fan_vault_reopens_and_reports_as_configured() {
        let directory = test_dir("fan-device-sealed");
        let expected = fan_profile();
        let password = random_vault_password().expect("password should be generated");
        replace_fan_with_password(&directory, password.as_ref(), &expected)
            .expect("device-sealed vault should persist");
        assert!(fan_exists(&directory));
        let reopened = load_fan_with_password(&directory, password.as_ref())
            .expect("device-sealed vault should reopen with the same password");
        assert_eq!(reopened.email, expected.email);
        assert_eq!(reopened.fan_session_token, expected.fan_session_token);
        let _ = std::fs::remove_dir_all(directory);
    }

    /// Two calls must not produce the same password: it is the whole secret for
    /// a vault that has no PIN.
    #[test]
    fn random_vault_passwords_do_not_repeat() {
        let first = random_vault_password().expect("password should be generated");
        let second = random_vault_password().expect("password should be generated");
        assert_eq!(first.len(), PASSWORD_BYTES);
        assert_ne!(first.as_slice(), second.as_slice());
    }

    /// Turning device unlock off re-keys the same profile to a PIN. The old
    /// password must stop working, or "off" would leave two ways in.
    #[test]
    fn re_keying_a_device_sealed_vault_to_a_pin_retires_the_old_password() {
        let directory = test_dir("fan-device-rekey");
        let profile = fan_profile();
        let sealed = random_vault_password().expect("password should be generated");
        replace_fan_with_password(&directory, sealed.as_ref(), &profile)
            .expect("device-sealed vault should persist");
        let from_pin =
            replace_fan(&directory, "2580", &profile).expect("vault should re-key to a PIN");
        assert!(load_fan_with_password(&directory, from_pin.as_ref()).is_ok());
        assert!(load_fan_with_password(&directory, sealed.as_ref()).is_err());
        let _ = std::fs::remove_dir_all(directory);
    }

    #[test]
    fn operator_save_creates_a_missing_app_data_directory() {
        let directory = test_dir("missing-directory");
        assert!(!directory.exists());
        save_verified(&directory, "9876", &profile())
            .expect("save should create the app data directory");
        assert!(exists(&directory));
        let _ = std::fs::remove_dir_all(directory);
    }
}
