//! Device unlock for the fan vault.
//!
//! The fan vault is opened with a 32-byte password. Until now that password
//! could only come from Argon2 over a PIN, so every entry — including one that
//! started from a link the fan had already proven they control by receiving it
//! — ended in a PIN prompt.
//!
//! This module supplies the second source: a random password of the same
//! length, sealed by a hardware-backed key in the Android keystore and stored
//! beside the vault. Opening it needs no input from the fan, which is what
//! makes a mailed link land straight inside the app.
//!
//! What that trades away is stated plainly, because it is a real trade: a PIN
//! is something an attacker must know, and a sealed password is something the
//! device holds. The perimeter becomes the device's own lock screen and the app
//! sandbox rather than a secret in the fan's head. The keystore key is
//! non-exportable, so lifting the file off the device is not enough to open it,
//! but an attacker already inside an unlocked phone is not slowed down. That is
//! why the PIN never goes away: it stays available, and a fan who wants it can
//! turn device unlock off.
//!
//! Anything that is not Android has no keystore here, so `supported()` is false
//! and the PIN remains the only path — the code below never degrades to storing
//! a password in the clear.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

use crate::AppError;

const SEALED_FILE: &str = "fan.device-unlock.v1";
const MODE_FILE: &str = "fan.unlock-mode.v1.json";

/// Which ways into this device's vault currently work.
///
/// Recorded rather than inferred: a PIN tried against a vault that has no PIN
/// fails exactly like a wrong PIN does, and telling a fan their PIN is wrong
/// when the vault never had one is the kind of dead end that ends in a
/// reinstall.
#[derive(Clone, Copy, Debug, Default, Deserialize, Serialize, PartialEq, Eq)]
pub struct UnlockMode {
    #[serde(default)]
    pub pin: bool,
    #[serde(default)]
    pub device: bool,
}

fn sealed_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(SEALED_FILE)
}

fn mode_path(app_data_dir: &Path) -> PathBuf {
    app_data_dir.join(MODE_FILE)
}

/// Reads the recorded unlock mode.
///
/// A vault from a build that predates this file has no record, and every one of
/// those was created from a PIN — so an absent record reads as "PIN only"
/// rather than as "no way in".
pub fn read_mode(app_data_dir: &Path) -> UnlockMode {
    let path = mode_path(app_data_dir);
    let Ok(bytes) = std::fs::read(&path) else {
        return UnlockMode {
            pin: true,
            device: false,
        };
    };
    serde_json::from_slice(&bytes).unwrap_or(UnlockMode {
        pin: true,
        device: false,
    })
}

/// The unlock mode as the gate should see it, held in memory after the first
/// resolve.
///
/// "Effective" rather than "recorded": the record says the fan turned device
/// unlock on, and the sealed file says the password is still here. A restore
/// onto another device carries the record and not the keystore key, so both
/// have to agree before the gate offers an unlock it cannot perform — and
/// resolving that costs a read plus a `stat`, which is why it is not repeated
/// on every status call.
pub async fn effective_mode(state: &crate::AppState) -> UnlockMode {
    if let Some(mode) = *state.fan_unlock_mode.read().await {
        return mode;
    }
    let recorded = read_mode(&state.app_data_dir);
    let resolved = UnlockMode {
        pin: recorded.pin,
        device: recorded.device && has_sealed_password(&state.app_data_dir),
    };
    *state.fan_unlock_mode.write().await = Some(resolved);
    resolved
}

/// Drops the cache. Called wherever this process changes how the vault opens,
/// and after a keystore refusal, so a seal that stopped working is not offered
/// again for the life of the process.
pub async fn invalidate_cache(state: &crate::AppState) {
    *state.fan_unlock_mode.write().await = None;
}

/// Records the mode and updates the cache in the same step.
///
/// Taking the state rather than a path is deliberate: a writer that updates the
/// file without the cache leaves the gate answering from a stale value until
/// the process restarts, and that is the kind of miss that only shows up on a
/// device. One function, both effects.
pub async fn write_mode(state: &crate::AppState, mode: UnlockMode) -> Result<(), AppError> {
    std::fs::create_dir_all(&state.app_data_dir)?;
    let bytes = serde_json::to_vec(&mode)?;
    crate::vault::write_private_file(&mode_path(&state.app_data_dir), &bytes)?;
    // Every caller seals before recording `device: true`, so the file is
    // already there and the recorded mode is the effective one.
    *state.fan_unlock_mode.write().await = Some(mode);
    Ok(())
}

pub async fn clear_mode(state: &crate::AppState) -> Result<(), AppError> {
    *state.fan_unlock_mode.write().await = None;
    match std::fs::remove_file(mode_path(&state.app_data_dir)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

pub fn has_sealed_password(app_data_dir: &Path) -> bool {
    sealed_path(app_data_dir).is_file()
}

pub fn forget_sealed_password(app_data_dir: &Path) -> Result<(), AppError> {
    match std::fs::remove_file(sealed_path(app_data_dir)) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error.into()),
    }
}

#[cfg(target_os = "android")]
mod platform {
    use super::*;
    use base64::{Engine as _, engine::general_purpose::STANDARD};
    use tauri::AppHandle;

    /// A sealed 32-byte password with a GCM nonce and tag, base64-encoded, is
    /// well under this. The bound is here so a corrupt or hostile file cannot
    /// be read into memory unbounded before it is rejected.
    const MAX_SEALED_BYTES: u64 = 4 * 1024;

    fn read_sealed(app_data_dir: &Path) -> Result<String, AppError> {
        let path = sealed_path(app_data_dir);
        if std::fs::metadata(&path)?.len() > MAX_SEALED_BYTES {
            return Err(AppError::InvalidInput(
                crate::i18n::tr("native_damaged_device_profile").into(),
            ));
        }
        Ok(String::from_utf8(std::fs::read(&path)?)
            .map_err(|_| {
                AppError::InvalidInput(crate::i18n::tr("native_damaged_device_profile").into())
            })?
            .trim()
            .to_owned())
    }

    /// Seals `password` with the keystore and writes the result beside the
    /// vault. The plaintext never reaches the file: what lands on disk is what
    /// the keystore handed back, and only the keystore can undo it.
    pub fn seal(app: &AppHandle, app_data_dir: &Path, password: &[u8]) -> Result<(), AppError> {
        let encoded = Zeroizing::new(STANDARD.encode(password));
        let sealed = crate::push_plugin::seal_device_secret(app, encoded.as_str())
            .map_err(AppError::InvalidInput)?;
        std::fs::create_dir_all(app_data_dir)?;
        crate::vault::write_private_file(&sealed_path(app_data_dir), sealed.as_bytes())?;
        Ok(())
    }

    pub fn open(app: &AppHandle, app_data_dir: &Path) -> Result<Zeroizing<Vec<u8>>, AppError> {
        let sealed = read_sealed(app_data_dir)?;
        let encoded = Zeroizing::new(
            crate::push_plugin::open_device_secret(app, sealed.as_str())
                .map_err(AppError::InvalidInput)?,
        );
        let password = STANDARD.decode(encoded.as_bytes()).map_err(|_| {
            AppError::InvalidInput(crate::i18n::tr("native_damaged_device_profile").into())
        })?;
        Ok(Zeroizing::new(password))
    }

    /// Drops both the sealed file and the keystore key it was sealed with.
    /// Leaving the key behind would keep a usable secret on the device after
    /// the fan asked for it to be gone.
    pub fn forget(app: &AppHandle, app_data_dir: &Path) -> Result<(), AppError> {
        forget_sealed_password(app_data_dir)?;
        crate::push_plugin::clear_device_secret(app).map_err(AppError::InvalidInput)
    }
}

#[cfg(not(target_os = "android"))]
mod platform {
    use super::*;
    use tauri::AppHandle;

    pub fn seal(_app: &AppHandle, _app_data_dir: &Path, _password: &[u8]) -> Result<(), AppError> {
        Err(AppError::NotConfigured)
    }

    pub fn open(_app: &AppHandle, _app_data_dir: &Path) -> Result<Zeroizing<Vec<u8>>, AppError> {
        Err(AppError::NotConfigured)
    }

    pub fn forget(_app: &AppHandle, app_data_dir: &Path) -> Result<(), AppError> {
        forget_sealed_password(app_data_dir)
    }
}

pub use platform::{forget, open, seal};
