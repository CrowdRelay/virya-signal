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

    /// The KDF cost is a security parameter, so a change to it has to be a
    /// decision rather than a dependency bump. `vault_params` pins the values
    /// explicitly; this pins them again from the outside so a future default
    /// change in the `argon2` crate cannot move them quietly.
    ///
    /// m = 19 MiB, t = 2, p = 1 is the OWASP interactive-authentication floor.
    /// Raising it is not free — it is paid on every unlock, on the weakest
    /// Android device the app supports — and lowering it must never happen by
    /// accident, which is what this asserts.
    #[test]
    fn the_key_derivation_cost_cannot_drift_without_a_decision() {
        let params = vault_params().expect("vault params must build");
        assert_eq!(params.m_cost(), 19 * 1024, "Argon2id memory cost in KiB");
        assert_eq!(params.t_cost(), 2, "Argon2id iterations");
        assert_eq!(params.p_cost(), 1, "Argon2id parallelism");
        assert_eq!(params.output_len(), Some(PASSWORD_BYTES));
        // The snapshot's second stretch is deliberately cheap because the key
        // handed to it is already Argon2id output. Anything below this stops
        // being a stretch at all; anything near Stronghold's default of 19
        // brings back the 512 MiB arena that the low-memory killer noticed.
        assert!(
            (8..=12).contains(&SNAPSHOT_WORK_FACTOR),
            "snapshot work factor {SNAPSHOT_WORK_FACTOR} outside the reasoned range"
        );
    }

    /// What one guess costs an offline attacker who has the snapshot.
    ///
    /// Ignored because it measures rather than asserts: the number depends on
    /// the machine, and a threshold here would fail on a busy CI runner for
    /// reasons that say nothing about the code. Run it deliberately:
    ///
    ///   cargo test -p virya-signal --release -- --ignored --nocapture kdf_cost
    ///
    /// Read the output against the PIN space it defends. `validate_new_operator_pin`
    /// accepts 4 to 6 ASCII digits, so the space is 10^4 to 10^6 — 13.3 to 19.9
    /// bits. Multiply the per-guess cost below by 10_000 to get the wall time
    /// of an exhaustive 4-digit search on one core, then divide by however many
    /// cores or GPU lanes the attacker brings.
    #[test]
    #[ignore = "benchmark: run deliberately with --ignored --nocapture"]
    fn kdf_cost_per_guess() {
        let salt = [7_u8; SALT_BYTES];
        let started = std::time::Instant::now();
        const ROUNDS: u32 = 8;
        for index in 0..ROUNDS {
            let pin = format!("{:04}", index);
            password(&pin, &salt).expect("derivation must succeed");
        }
        let per_guess = started.elapsed() / ROUNDS;
        let four_digit = per_guess * 10_000;
        let six_digit = per_guess * 1_000_000;
        println!(
            "VAULT_KDF_COST per_guess={per_guess:?} \
             exhaustive_4_digit_single_core={four_digit:?} \
             exhaustive_6_digit_single_core={six_digit:?} \
             m_cost_kib={} t_cost={}",
            19 * 1024,
            2
        );
    }
}
