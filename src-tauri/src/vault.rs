use std::path::{Path, PathBuf};

use argon2::Argon2;
use rand::RngCore;
use tauri_plugin_stronghold::stronghold::Stronghold;
use zeroize::Zeroizing;

use crate::{models::OperatorProfile, AppError};

const CLIENT_PATH: &[u8] = b"virya-control-device";
const PROFILE_KEY: &[u8] = b"operator-profile-v1";
const SALT_BYTES: usize = 16;
const PASSWORD_BYTES: usize = 32;

pub fn vault_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("operator.vault.hold")
}

fn salt_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("operator.vault.salt")
}

pub fn exists(app_data_dir: &Path) -> bool {
    vault_path(app_data_dir).exists() && salt_path(app_data_dir).exists()
}

pub fn save(app_data_dir: &Path, pin: &str, profile: &OperatorProfile) -> Result<(), AppError> {
    ensure_pin(pin)?;
    std::fs::create_dir_all(app_data_dir)?;
    let salt = load_or_create_salt(app_data_dir)?;
    let stronghold = Stronghold::new(vault_path(app_data_dir), password(pin, &salt)?)
        .map_err(|_| AppError::StrongholdClient)?;
    let client = stronghold
        .load_client(CLIENT_PATH)
        .or_else(|_| stronghold.create_client(CLIENT_PATH))
        .map_err(|_| AppError::StrongholdClient)?;
    let bytes = Zeroizing::new(serde_json::to_vec(profile)?);
    client
        .store()
        .insert(PROFILE_KEY.to_vec(), bytes.to_vec(), None)
        .map_err(|_| AppError::StrongholdClient)?;
    stronghold.save().map_err(|_| AppError::StrongholdClient)?;
    Ok(())
}

pub fn load(app_data_dir: &Path, pin: &str) -> Result<OperatorProfile, AppError> {
    ensure_pin(pin)?;
    if !exists(app_data_dir) {
        return Err(AppError::NotConfigured);
    }
    let salt = read_salt(app_data_dir)?;
    let stronghold = Stronghold::new(vault_path(app_data_dir), password(pin, &salt)?)
        .map_err(|_| AppError::InvalidPin)?;
    let client = stronghold
        .load_client(CLIENT_PATH)
        .map_err(|_| AppError::InvalidPin)?;
    let bytes = Zeroizing::new(
        client
            .store()
            .get(PROFILE_KEY)
            .map_err(|_| AppError::StrongholdClient)?
            .ok_or(AppError::NotConfigured)?,
    );
    serde_json::from_slice(bytes.as_ref()).map_err(AppError::from)
}

pub fn remove(app_data_dir: &Path) -> Result<(), AppError> {
    remove_if_present(&vault_path(app_data_dir))?;
    remove_if_present(&salt_path(app_data_dir))
}

fn remove_if_present(path: &Path) -> Result<(), AppError> {
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

fn ensure_pin(pin: &str) -> Result<(), AppError> {
    if (6..=128).contains(&pin.chars().count()) {
        Ok(())
    } else {
        Err(AppError::InvalidInput("PIN musi mieć 6–128 znaków".into()))
    }
}

fn load_or_create_salt(app_data_dir: &Path) -> Result<[u8; SALT_BYTES], AppError> {
    if salt_path(app_data_dir).exists() {
        return read_salt(app_data_dir);
    }
    let mut salt = [0_u8; SALT_BYTES];
    rand::rng().fill_bytes(&mut salt);
    std::fs::write(salt_path(app_data_dir), salt)?;
    Ok(salt)
}

fn read_salt(app_data_dir: &Path) -> Result<[u8; SALT_BYTES], AppError> {
    let bytes = std::fs::read(salt_path(app_data_dir))?;
    bytes
        .try_into()
        .map_err(|_| AppError::InvalidInput("Uszkodzony profil urządzenia".into()))
}

fn password(pin: &str, salt: &[u8; SALT_BYTES]) -> Result<Vec<u8>, AppError> {
    let mut output = Zeroizing::new(vec![0_u8; PASSWORD_BYTES]);
    Argon2::default()
        .hash_password_into(pin.as_bytes(), salt, &mut output)
        .map_err(|_| AppError::StrongholdClient)?;
    Ok(output.to_vec())
}
