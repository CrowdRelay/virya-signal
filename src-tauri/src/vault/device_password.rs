// The vault password without a PIN behind it.
//
// Split out of `vault.rs` so that file stays under the module limit; the
// snapshot layer below is shared with the PIN path and lives there.
//
// Included into `vault.rs`, so an inner doc comment is not available here.

/// Replaces the fan vault, sealed with a password the caller already holds.
///
/// The PIN is one way to produce that password, not the only one: a fan who
/// chose device unlock has a random password sealed in the platform keystore
/// instead, and it arrives here as bytes. Everything below this line is
/// identical either way, so the snapshot format does not fork.
///
/// A salt is still written even though nothing derives from it here. It is
/// what `fan_exists` looks for alongside the snapshot, and it is what a later
/// "add a PIN" re-key derives against, so a vault created without a PIN must
/// not be missing one.
pub fn replace_fan_with_password(
    app_data_dir: &Path,
    password: &[u8],
    profile: &FanProfile,
) -> Result<(), AppError> {
    let vault_path = fan_vault_path(app_data_dir);
    let salt_path = fan_salt_path(app_data_dir);
    let vault_backup = backup_path(&vault_path);
    let salt_backup = backup_path(&salt_path);

    remove_if_present(&vault_backup)?;
    remove_if_present(&salt_backup)?;
    move_if_present(&vault_path, &vault_backup)?;
    if let Err(error) = move_if_present(&salt_path, &salt_backup) {
        let _ = move_if_present(&vault_backup, &vault_path);
        return Err(error);
    }

    let written = (|| -> Result<(), AppError> {
        if let Some(parent) = vault_path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let _salt = load_or_create_salt(&salt_path)?;
        let bytes = Zeroizing::new(serde_json::to_vec(profile)?);
        save_bytes_with_password_at(
            &vault_path,
            FAN_CLIENT_PATH,
            FAN_PROFILE_KEY,
            password,
            bytes.as_ref(),
        )
    })();

    match written {
        Ok(()) => {
            let _ = remove_if_present(&vault_backup);
            let _ = remove_if_present(&salt_backup);
            Ok(())
        }
        Err(error) => {
            let _ = remove_pair(&vault_path, &salt_path);
            let _ = move_if_present(&vault_backup, &vault_path);
            let _ = move_if_present(&salt_backup, &salt_path);
            Err(error)
        }
    }
}

/// A vault password with no PIN behind it.
///
/// `PASSWORD_BYTES` of system randomness, which is what Argon2 produces for
/// the PIN path — the same length and the same use, so the snapshot layer
/// cannot tell the two apart.
pub fn random_vault_password() -> Result<Zeroizing<Vec<u8>>, AppError> {
    let mut password = Zeroizing::new(vec![0_u8; PASSWORD_BYTES]);
    getrandom::fill(password.as_mut()).map_err(|_| AppError::StrongholdClient)?;
    Ok(password)
}

