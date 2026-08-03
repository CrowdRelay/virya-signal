//! Native crash evidence: a single bounded file written from the panic hook
//! (see `run()` in `lib.rs`) and read back once on next launch so the WebView
//! can show and clear the report. See `docs/ARCHITECTURE.md`.

use std::{path::PathBuf, sync::OnceLock};

use tauri::State;

use crate::{AppError, AppState};

pub(crate) static NATIVE_CRASH_REPORT_PATH: OnceLock<PathBuf> = OnceLock::new();
pub(crate) const NATIVE_CRASH_REPORT_FILE: &str = "last-native-crash-v2.txt";
const MAX_NATIVE_CRASH_REPORT_CHARS: usize = 16_384;

pub(crate) fn write_native_crash_report(report: &str) {
    let Some(path) = NATIVE_CRASH_REPORT_PATH.get() else {
        return;
    };
    let bounded: String = report.chars().take(MAX_NATIVE_CRASH_REPORT_CHARS).collect();
    let temporary = path.with_extension("tmp");
    let result = (|| -> std::io::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let mut file = std::fs::OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .open(&temporary)?;
        std::io::Write::write_all(&mut file, bounded.as_bytes())?;
        file.sync_all()?;
        std::fs::rename(&temporary, path)?;
        Ok(())
    })();
    if let Err(error) = result {
        let _ = std::fs::remove_file(&temporary);
        eprintln!("[virya:native-panic] could not persist crash report: {error}");
    }
}

#[tauri::command]
pub(crate) fn native_crash_report(state: State<'_, AppState>) -> Result<Option<String>, AppError> {
    let path = state.app_data_dir.join(NATIVE_CRASH_REPORT_FILE);
    match std::fs::read_to_string(path) {
        Ok(report) => Ok(Some(
            report.chars().take(MAX_NATIVE_CRASH_REPORT_CHARS).collect(),
        )),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(AppError::Io(error)),
    }
}

#[tauri::command]
pub(crate) fn acknowledge_native_crash(state: State<'_, AppState>) -> Result<(), AppError> {
    let path = state.app_data_dir.join(NATIVE_CRASH_REPORT_FILE);
    match std::fs::remove_file(path) {
        Ok(()) => Ok(()),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(AppError::Io(error)),
    }
}
