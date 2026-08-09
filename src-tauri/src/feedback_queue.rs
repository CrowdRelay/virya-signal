//! Bounded anonymous-feedback outbox for transient offline periods.
//! It contains no fan/session identity; only the category, user-written message,
//! one random submission id and queue timestamp are persisted.

use serde::{Deserialize, Serialize};
use std::{
    fs,
    fs::File,
    path::{Path, PathBuf},
};

use crate::AppError;

const FILE_NAME: &str = "anonymous-feedback-outbox-v1.json";
const MAX_ENTRIES: usize = 8;
const MAX_BYTES: u64 = 64 * 1024;
const MAX_AGE_SECONDS: i64 = 7 * 24 * 60 * 60;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct QueuedFeedback {
    pub submission_id: String,
    pub category: String,
    pub message: String,
    pub queued_at_unix: i64,
}

fn path(dir: &Path) -> PathBuf {
    dir.join(FILE_NAME)
}

pub(crate) fn load(dir: &Path) -> Result<Vec<QueuedFeedback>, AppError> {
    let path = path(dir);
    let metadata = match fs::metadata(&path) {
        Ok(value) => value,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error.into()),
    };
    if metadata.len() > MAX_BYTES {
        let _ = fs::remove_file(&path);
        return Ok(Vec::new());
    }
    let bytes = fs::read(path)?;
    let mut values: Vec<QueuedFeedback> = serde_json::from_slice(&bytes).unwrap_or_default();
    prune(&mut values);
    Ok(values)
}

pub(crate) fn enqueue(dir: &Path, value: QueuedFeedback) -> Result<(), AppError> {
    let mut values = load(dir)?;
    values.push(value);
    prune(&mut values);
    save(dir, &values)
}

pub(crate) fn save(dir: &Path, values: &[QueuedFeedback]) -> Result<(), AppError> {
    fs::create_dir_all(dir)?;
    let final_path = path(dir);
    if values.is_empty() {
        match fs::remove_file(final_path) {
            Ok(()) => return Ok(()),
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error.into()),
        }
    }
    let bytes = serde_json::to_vec(values)?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(AppError::BackgroundTask);
    }
    let temp_path = dir.join(format!("{FILE_NAME}.tmp"));
    let backup_path = dir.join(format!("{FILE_NAME}.bak"));
    fs::write(&temp_path, bytes)?;
    File::open(&temp_path)?.sync_all()?;

    // Windows rename does not replace an existing destination. Preserve the
    // previous durable queue until the new payload has been promoted; if the
    // final rename fails we restore the backup instead of silently losing it.
    match fs::remove_file(&backup_path) {
        Ok(()) => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
        Err(error) => return Err(error.into()),
    }
    let had_previous = final_path.exists();
    if had_previous {
        fs::rename(&final_path, &backup_path)?;
    }
    if let Err(error) = fs::rename(&temp_path, &final_path) {
        if had_previous {
            let _ = fs::rename(&backup_path, &final_path);
        }
        return Err(error.into());
    }
    if had_previous {
        match fs::remove_file(&backup_path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn prune(values: &mut Vec<QueuedFeedback>) {
    let now = time::OffsetDateTime::now_utc().unix_timestamp();
    values.retain(|value| now.saturating_sub(value.queued_at_unix) <= MAX_AGE_SECONDS);
    if values.len() > MAX_ENTRIES {
        values.drain(0..values.len().saturating_sub(MAX_ENTRIES));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn queue_is_small_and_short_lived() {
        const {
            assert!(MAX_ENTRIES <= 8);
            assert!(MAX_BYTES <= 64 * 1024);
            assert!(MAX_AGE_SECONDS <= 7 * 24 * 60 * 60);
        }
    }
}
