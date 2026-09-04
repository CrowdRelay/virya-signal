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
