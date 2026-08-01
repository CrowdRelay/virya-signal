use std::path::{Path, PathBuf};

use argon2::Argon2;
use rand::RngCore;
use serde::{de::DeserializeOwned, Serialize};
use tauri_plugin_stronghold::stronghold::Stronghold;
use zeroize::Zeroizing;

use crate::{
    models::{FanProfile, OperatorProfile},
    AppError,
};

const OPERATOR_CLIENT_PATH: &[u8] = b"virya-control-device";
const OPERATOR_PROFILE_KEY: &[u8] = b"operator-profile-v1";
const FAN_CLIENT_PATH: &[u8] = b"virya-signal-fan";
const FAN_PROFILE_KEY: &[u8] = b"fan-profile-v1";
const SALT_BYTES: usize = 16;
const PASSWORD_BYTES: usize = 32;

pub fn exists(app_data_dir: &Path) -> bool {
    exists_at(
        &operator_vault_path(app_data_dir),
        &operator_salt_path(app_data_dir),
    )
}

pub fn save(app_data_dir: &Path, pin: &str, profile: &OperatorProfile) -> Result<(), AppError> {
    save_at(
        &operator_vault_path(app_data_dir),
        &operator_salt_path(app_data_dir),
        OPERATOR_CLIENT_PATH,
        OPERATOR_PROFILE_KEY,
        pin,
        profile,
    )
}

pub fn load(app_data_dir: &Path, pin: &str) -> Result<OperatorProfile, AppError> {
    load_at(
        &operator_vault_path(app_data_dir),
        &operator_salt_path(app_data_dir),
        OPERATOR_CLIENT_PATH,
        OPERATOR_PROFILE_KEY,
        pin,
    )
}

pub fn remove(app_data_dir: &Path) -> Result<(), AppError> {
    remove_pair(
        &operator_vault_path(app_data_dir),
        &operator_salt_path(app_data_dir),
    )
}

pub fn fan_exists(app_data_dir: &Path) -> bool {
    exists_at(&fan_vault_path(app_data_dir), &fan_salt_path(app_data_dir))
}

pub fn save_fan(app_data_dir: &Path, pin: &str, profile: &FanProfile) -> Result<(), AppError> {
    save_at(
        &fan_vault_path(app_data_dir),
        &fan_salt_path(app_data_dir),
        FAN_CLIENT_PATH,
        FAN_PROFILE_KEY,
        pin,
        profile,
    )
}

pub fn load_fan(app_data_dir: &Path, pin: &str) -> Result<FanProfile, AppError> {
    load_at(
        &fan_vault_path(app_data_dir),
        &fan_salt_path(app_data_dir),
        FAN_CLIENT_PATH,
        FAN_PROFILE_KEY,
        pin,
    )
}

pub fn remove_fan(app_data_dir: &Path) -> Result<(), AppError> {
    remove_pair(&fan_vault_path(app_data_dir), &fan_salt_path(app_data_dir))
}

fn operator_vault_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("operator.vault.hold")
}

fn operator_salt_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("operator.vault.salt")
}

fn fan_vault_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("fan.vault.hold")
}

fn fan_salt_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join("fan.vault.salt")
}

fn exists_at(vault_path: &Path, salt_path: &Path) -> bool {
    vault_path.exists() && salt_path.exists()
}

fn save_at<T: Serialize>(
    vault_path: &Path,
    salt_path: &Path,
    client_path: &[u8],
    profile_key: &[u8],
    pin: &str,
    profile: &T,
) -> Result<(), AppError> {
    ensure_pin(pin)?;
    if let Some(parent) = vault_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let salt = load_or_create_salt(salt_path)?;
    let stronghold = Stronghold::new(vault_path, password(pin, &salt)?)
        .map_err(|_| AppError::StrongholdClient)?;
    let client = stronghold
        .load_client(client_path)
        .or_else(|_| stronghold.create_client(client_path))
        .map_err(|_| AppError::StrongholdClient)?;
    let bytes = Zeroizing::new(serde_json::to_vec(profile)?);
    client
        .store()
        .insert(profile_key.to_vec(), bytes.to_vec(), None)
        .map_err(|_| AppError::StrongholdClient)?;
    stronghold.save().map_err(|_| AppError::StrongholdClient)?;
    Ok(())
}

fn load_at<T: DeserializeOwned>(
    vault_path: &Path,
    salt_path: &Path,
    client_path: &[u8],
    profile_key: &[u8],
    pin: &str,
) -> Result<T, AppError> {
    ensure_pin(pin)?;
    if !exists_at(vault_path, salt_path) {
        return Err(AppError::NotConfigured);
    }
    let salt = read_salt(salt_path)?;
    let stronghold = Stronghold::new(vault_path, password(pin, &salt)?)
        .map_err(|_| AppError::InvalidPin)?;
    let client = stronghold
        .load_client(client_path)
        .map_err(|_| AppError::InvalidPin)?;
    let bytes = Zeroizing::new(
        client
            .store()
            .get(profile_key)
            .map_err(|_| AppError::StrongholdClient)?
            .ok_or(AppError::NotConfigured)?,
    );
    serde_json::from_slice(bytes.as_ref()).map_err(AppError::from)
}

fn remove_pair(vault_path: &Path, salt_path: &Path) -> Result<(), AppError> {
    remove_if_present(vault_path)?;
    remove_if_present(salt_path)
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

fn load_or_create_salt(path: &Path) -> Result<[u8; SALT_BYTES], AppError> {
    if path.exists() {
        return read_salt(path);
    }
    let mut salt = [0_u8; SALT_BYTES];
    rand::rng().fill_bytes(&mut salt);
    std::fs::write(path, salt)?;
    Ok(salt)
}

fn read_salt(path: &Path) -> Result<[u8; SALT_BYTES], AppError> {
    let bytes = std::fs::read(path)?;
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
