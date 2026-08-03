use std::{
    fs::OpenOptions,
    io::Write,
    path::{Path, PathBuf},
};

use argon2::Argon2;
use iota_stronghold::{KeyProvider, SnapshotPath, Stronghold};
use rand::Rng;
use serde::{Serialize, de::DeserializeOwned};
use zeroize::Zeroizing;

use crate::{
    AppError,
    models::{FanProfile, OperatorProfile, ShowModeStore},
};

const OPERATOR_CLIENT_PATH: &[u8] = b"virya-control-device";
const OPERATOR_PROFILE_KEY: &[u8] = b"operator-profile-v1";
const SHOW_MODE_STORE_KEY: &[u8] = b"show-mode-store-v1";
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

/// Derives the Stronghold password once per unlocked operator session. The
/// caller keeps it in a Zeroizing buffer and drops it on lock.
pub fn operator_password(app_data_dir: &Path, pin: &str) -> Result<Zeroizing<Vec<u8>>, AppError> {
    ensure_pin(pin)?;
    let salt = read_salt(&operator_salt_path(app_data_dir))?;
    Ok(Zeroizing::new(password(pin, &salt)?))
}

pub fn remove(app_data_dir: &Path) -> Result<(), AppError> {
    remove_pair(
        &operator_vault_path(app_data_dir),
        &operator_salt_path(app_data_dir),
    )
}

/// Saves the show-mode store with a password derived at operator unlock.
pub fn save_show_mode_bytes_with_password(
    app_data_dir: &Path,
    password: &[u8],
    bytes: &[u8],
) -> Result<(), AppError> {
    save_bytes_with_password_at(
        &operator_vault_path(app_data_dir),
        OPERATOR_CLIENT_PATH,
        SHOW_MODE_STORE_KEY,
        password,
        bytes,
    )
}

/// Loads the show-mode store without repeating Argon2 for every gate scan.
pub fn load_show_mode_with_password(
    app_data_dir: &Path,
    password: &[u8],
) -> Result<ShowModeStore, AppError> {
    load_optional_with_password_at(
        &operator_vault_path(app_data_dir),
        OPERATOR_CLIENT_PATH,
        SHOW_MODE_STORE_KEY,
        password,
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
    let bytes = Zeroizing::new(serde_json::to_vec(profile)?);
    save_bytes_at(
        vault_path,
        salt_path,
        client_path,
        profile_key,
        pin,
        bytes.as_ref(),
    )
}

fn save_bytes_at(
    vault_path: &Path,
    salt_path: &Path,
    client_path: &[u8],
    profile_key: &[u8],
    pin: &str,
    bytes: &[u8],
) -> Result<(), AppError> {
    ensure_pin(pin)?;
    let salt = load_or_create_salt(salt_path)?;
    let password = Zeroizing::new(password(pin, &salt)?);
    save_bytes_with_password_at(
        vault_path,
        client_path,
        profile_key,
        password.as_ref(),
        bytes,
    )
}

fn save_bytes_with_password_at(
    vault_path: &Path,
    client_path: &[u8],
    profile_key: &[u8],
    password: &[u8],
    bytes: &[u8],
) -> Result<(), AppError> {
    if password.len() != PASSWORD_BYTES {
        return Err(AppError::StrongholdClient);
    }
    if let Some(parent) = vault_path.parent() {
        std::fs::create_dir_all(parent)?;
    }

    let snapshot_path = SnapshotPath::from_path(vault_path);
    let key_provider = KeyProvider::try_from(Zeroizing::new(password.to_vec()))
        .map_err(|_| AppError::StrongholdClient)?;
    let stronghold = Stronghold::default();
    let client = if vault_path.exists() {
        stronghold
            .load_client_from_snapshot(client_path, &key_provider, &snapshot_path)
            .map_err(|_| AppError::StrongholdClient)?
    } else {
        stronghold
            .create_client(client_path)
            .map_err(|_| AppError::StrongholdClient)?
    };
    client
        .store()
        .insert(profile_key.to_vec(), bytes.to_vec(), None)
        .map_err(|_| AppError::StrongholdClient)?;
    stronghold
        .write_client(client_path)
        .map_err(|_| AppError::StrongholdClient)?;
    stronghold
        .commit_with_keyprovider(&snapshot_path, &key_provider)
        .map_err(|_| AppError::StrongholdClient)?;
    set_private_permissions(vault_path)?;
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
    let key_provider = KeyProvider::try_from(Zeroizing::new(password(pin, &salt)?))
        .map_err(|_| AppError::InvalidPin)?;
    let snapshot_path = SnapshotPath::from_path(vault_path);
    let stronghold = Stronghold::default();
    let client = stronghold
        .load_client_from_snapshot(client_path, &key_provider, &snapshot_path)
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

fn load_optional_with_password_at<T: DeserializeOwned + Default>(
    vault_path: &Path,
    client_path: &[u8],
    profile_key: &[u8],
    password: &[u8],
) -> Result<T, AppError> {
    if !vault_path.exists() {
        return Ok(T::default());
    }
    if password.len() != PASSWORD_BYTES {
        return Err(AppError::InvalidPin);
    }
    let key_provider = KeyProvider::try_from(Zeroizing::new(password.to_vec()))
        .map_err(|_| AppError::InvalidPin)?;
    let snapshot_path = SnapshotPath::from_path(vault_path);
    let stronghold = Stronghold::default();
    let client = stronghold
        .load_client_from_snapshot(client_path, &key_provider, &snapshot_path)
        .map_err(|_| AppError::InvalidPin)?;
    let Some(bytes) = client
        .store()
        .get(profile_key)
        .map_err(|_| AppError::StrongholdClient)?
    else {
        return Ok(T::default());
    };
    let bytes = Zeroizing::new(bytes);
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
    match create_private_file(path, &salt) {
        Ok(()) => Ok(salt),
        Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => read_salt(path),
        Err(error) => Err(error.into()),
    }
}

fn read_salt(path: &Path) -> Result<[u8; SALT_BYTES], AppError> {
    if std::fs::metadata(path)?.len() != SALT_BYTES as u64 {
        return Err(AppError::InvalidInput(
            "Uszkodzony profil urządzenia".into(),
        ));
    }
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

fn create_private_file(path: &Path, contents: &[u8]) -> Result<(), std::io::Error> {
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let mut file = options.open(path)?;
    if let Err(error) = file.write_all(contents).and_then(|()| file.sync_all()) {
        drop(file);
        let _ = std::fs::remove_file(path);
        return Err(error);
    }
    Ok(())
}

fn set_private_permissions(path: &Path) -> Result<(), AppError> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o600))?;
    }
    #[cfg(not(unix))]
    let _ = path;
    Ok(())
}
