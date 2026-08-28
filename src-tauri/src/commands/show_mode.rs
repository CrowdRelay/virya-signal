//! Offline "show mode": a signed CrowdRelay snapshot is downloaded once per
//! event, admissions are scanned and queued locally (even without
//! connectivity), and queued scans are synced back opportunistically.

use std::sync::Arc;

use base64::{Engine as _, engine::general_purpose::URL_SAFE_NO_PAD};
use futures_util::{StreamExt, stream};
use sha2::{Digest, Sha256};
use tauri::State;
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use zeroize::Zeroizing;

use crate::{
    AppError, AppState,
    models::{
        ShowModeQueuedScan, ShowModeScanResult, ShowModeScanState, ShowModeSession,
        ShowModeSnapshot, ShowModeStatus, ShowModeStore, ShowModeSyncResult,
    },
    session::{operator_profile, operator_vault_password, run_blocking},
    util::OptionValueOrExt,
    vault,
};

const SHOW_MODE_SYNC_CONCURRENCY: usize = 4;

fn unix_now_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |value| value.as_secs())
}

fn parse_snapshot_timestamp(value: &str) -> Result<u64, AppError> {
    let timestamp = OffsetDateTime::parse(value, &Rfc3339)
        .map_err(|_| {
            AppError::InvalidInput(crate::i18n::tr("native_snapshot_time_invalid").into())
        })?
        .unix_timestamp();
    u64::try_from(timestamp)
        .map_err(|_| AppError::InvalidInput(crate::i18n::tr("native_snapshot_time_invalid").into()))
}

fn hash_field(hasher: &mut Sha256, value: &str) {
    hasher.update(value.len().to_be_bytes());
    hasher.update(value.as_bytes());
}

fn show_mode_checksum(snapshot: &ShowModeSnapshot) -> String {
    let mut hasher = Sha256::new();
    hash_field(&mut hasher, "crowdrelay/show-mode/v1");
    hash_field(&mut hasher, &snapshot.schema_version.to_string());
    hash_field(&mut hasher, &snapshot.snapshot_id);
    hash_field(&mut hasher, &snapshot.event.slug);
    hash_field(&mut hasher, &snapshot.event.title);
    hash_field(&mut hasher, snapshot.event.venue.as_deref().value_or(""));
    hash_field(&mut hasher, &snapshot.event.starts_at);
    hash_field(&mut hasher, &snapshot.generated_at);
    hash_field(&mut hasher, &snapshot.expires_at);
    // Prepared snapshots are normalized by public_reference once, so every
    // checksum and scan can stream the same stable order without allocating.
    for pass in &snapshot.passes {
        hash_field(&mut hasher, &pass.public_reference);
        hash_field(&mut hasher, pass.holder_name.as_deref().value_or(""));
        hash_field(&mut hasher, &pass.holder_email_masked);
        hash_field(&mut hasher, pass.ticket_type_name.as_deref().value_or(""));
        hash_field(&mut hasher, if pass.offline_eligible { "1" } else { "0" });
        hash_field(&mut hasher, pass.qr_sha256.as_deref().value_or(""));
    }
    hex::encode(hasher.finalize())
}

fn snapshot_is_active(snapshot: &ShowModeSnapshot) -> bool {
    parse_snapshot_timestamp(&snapshot.expires_at)
        .is_ok_and(|expires_at| expires_at >= unix_now_secs())
}

fn parse_t1_reference(token: &str) -> Result<String, AppError> {
    let mut parts = token.trim().split('.');
    if parts.next() != Some("t1") {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_offline_t1_only").into(),
        ));
    }
    let payload = parts.next().ok_or_else(|| {
        AppError::InvalidInput(crate::i18n::tr("native_ticket_qr_invalid").into())
    })?;
    let signature = parts.next().ok_or_else(|| {
        AppError::InvalidInput(crate::i18n::tr("native_ticket_qr_invalid").into())
    })?;
    if parts.next().is_some() || signature.len() != 64 {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_ticket_qr_invalid").into(),
        ));
    }
    let bytes = URL_SAFE_NO_PAD
        .decode(payload)
        .map_err(|_| AppError::InvalidInput(crate::i18n::tr("native_ticket_qr_invalid").into()))?;
    if bytes.len() > 512 {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_ticket_qr_invalid").into(),
        ));
    }
    #[derive(serde::Deserialize)]
    struct TicketReferenceClaims {
        #[serde(rename = "r")]
        public_reference: String,
    }
    let claims: TicketReferenceClaims = serde_json::from_slice(&bytes)?;
    let reference = claims.public_reference;
    if reference.is_empty()
        || reference.len() > 64
        || !reference
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || byte == b'-')
    {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_ticket_qr_invalid").into(),
        ));
    }
    Ok(reference)
}

async fn ensure_show_store_loaded(state: &State<'_, AppState>) -> Result<(), AppError> {
    if state.show_mode_store.read().await.is_some() {
        return Ok(());
    }
    let password = operator_vault_password(state).await?;
    let app_data_dir = state.app_data_dir.clone();
    let mut store =
        run_blocking(move || vault::load_show_mode_with_password(&app_data_dir, password.as_ref()))
            .await?;
    normalize_show_store(&mut store);
    let mut cached = state.show_mode_store.write().await;
    if cached.is_none() {
        *cached = Some(store);
    }
    Ok(())
}

async fn persist_show_store(state: &State<'_, AppState>) -> Result<(), AppError> {
    let payload = {
        let cached = state.show_mode_store.read().await;
        let store = cached.as_ref().ok_or(AppError::Locked)?;
        Zeroizing::new(serde_json::to_vec(store)?)
    };
    let password = operator_vault_password(state).await?;
    let app_data_dir = state.app_data_dir.clone();
    let result = run_blocking(move || {
        vault::save_show_mode_bytes_with_password(
            &app_data_dir,
            password.as_ref(),
            payload.as_ref(),
        )
    })
    .await;
    if result.is_err() {
        // A failed durable write must not leave a newer memory-only queue that
        // would disappear after a process restart. Reload the last disk state.
        *state.show_mode_store.write().await = None;
    }
    result
}

fn normalize_show_store(store: &mut ShowModeStore) {
    let now = unix_now_secs();
    store.sessions.retain(|_, session| {
        let active = parse_snapshot_timestamp(&session.snapshot.expires_at)
            .is_ok_and(|expires_at| expires_at >= now);
        let has_unsynced_scans = session
            .scans
            .iter()
            .any(|scan| scan.state != ShowModeScanState::Synced);
        active || has_unsynced_scans
    });
    for session in store.sessions.values_mut() {
        session
            .snapshot
            .passes
            .sort_unstable_by(|left, right| left.public_reference.cmp(&right.public_reference));
        session
            .scans
            .sort_unstable_by(|left, right| left.public_reference.cmp(&right.public_reference));
    }
}

fn show_mode_status_for(event_slug: &str, store: &ShowModeStore) -> ShowModeStatus {
    let Some(session) = store.sessions.get(event_slug) else {
        return ShowModeStatus {
            event_slug: event_slug.to_owned(),
            ..ShowModeStatus::default()
        };
    };
    let (pending, synced, conflicts) = session.scans.iter().fold(
        (0_usize, 0_usize, 0_usize),
        |(pending, synced, conflicts), scan| match scan.state {
            ShowModeScanState::Pending => (pending + 1, synced, conflicts),
            ShowModeScanState::Synced => (pending, synced + 1, conflicts),
            ShowModeScanState::Conflict => (pending, synced, conflicts + 1),
        },
    );
    ShowModeStatus {
        prepared: snapshot_is_active(&session.snapshot),
        event_slug: event_slug.to_owned(),
        event_title: Some(session.snapshot.event.title.clone()),
        expires_at: Some(session.snapshot.expires_at.clone()),
        eligible_passes: session
            .snapshot
            .passes
            .iter()
            .filter(|pass| pass.offline_eligible && pass.qr_sha256.is_some())
            .count(),
        pending,
        synced,
        conflicts,
        checklist_loaded: session.checklist.is_some(),
        checklist_pending: session.checklist.as_ref().map_or(0, |value| {
            value
                .items
                .iter()
                .filter(|item| item.status == "pending" || item.status == "blocked")
                .count()
        }),
        pickup_order_count: session
            .commerce
            .as_ref()
            .map_or(0, |value| value.pickup_order_count),
        pickup_unit_count: session
            .commerce
            .as_ref()
            .map_or(0, |value| value.pickup_unit_count),
        pickup_items: session
            .commerce
            .as_ref()
            .map_or_else(Vec::new, |value| value.pickup_items.clone()),
        close_ready: pending == 0
            && conflicts == 0
            && session.checklist.as_ref().is_some_and(|value| {
                value.items.iter().any(|item| {
                    item.item_key == "post_show_reconciliation"
                        && matches!(item.status.as_str(), "done" | "skipped")
                })
            }),
        closed: session.closed_at_unix_secs.is_some(),
    }
}

#[tauri::command]
pub(crate) async fn show_mode_prepare(
    state: State<'_, AppState>,
    event_slug: String,
) -> Result<ShowModeStatus, AppError> {
    let _mutation = state.show_mode_mutation.lock().await;
    let profile = operator_profile(&state).await?;
    let event_slug = event_slug.trim();
    if event_slug.is_empty() || event_slug.len() > 128 {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_event_invalid").into(),
        ));
    }
    let mut snapshot = state.api.show_mode_snapshot(&profile, event_slug).await?;
    if snapshot.schema_version != 1 || snapshot.event.slug != event_slug {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_snapshot_event_mismatch").into(),
        ));
    }
    let generated_at = parse_snapshot_timestamp(&snapshot.generated_at)?;
    let expires_at = parse_snapshot_timestamp(&snapshot.expires_at)?;
    let now = unix_now_secs();
    if generated_at > now.saturating_add(300)
        || expires_at <= now
        || expires_at.saturating_sub(generated_at) > 72 * 60 * 60
    {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_snapshot_expired").into(),
        ));
    }
    if snapshot.passes.len() > 10_000 {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_snapshot_too_large").into(),
        ));
    }
    snapshot
        .passes
        .sort_unstable_by(|left, right| left.public_reference.cmp(&right.public_reference));
    let checksum = show_mode_checksum(&snapshot);
    if snapshot.checksum_sha256.len() != 64
        || !snapshot.checksum_sha256.eq_ignore_ascii_case(&checksum)
    {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_snapshot_integrity_failed").into(),
        ));
    }
    let checklist = state
        .api
        .operator_show_checklist(&profile, event_slug)
        .await
        .ok();
    let commerce = match checklist.as_ref() {
        Some(value) => state
            .api
            .operator_event_merch_summary(&profile, &value.event_id)
            .await
            .ok(),
        None => None,
    };
    // Surface a clear error if the checklist could not be loaded. Without it
    // the operator would scan against a stale or incomplete snapshot.
    if checklist.is_none() {
        return Err(AppError::Conflict(
            crate::i18n::tr("native_show_checklist_unavailable").into(),
        ));
    }
    ensure_show_store_loaded(&state).await?;
    let status = {
        let mut cached = state.show_mode_store.write().await;
        let store = cached.as_mut().ok_or(AppError::Locked)?;
        let mut previous_scans = store
            .sessions
            .remove(event_slug)
            .map_or_else(Vec::new, |session| session.scans);
        previous_scans
            .sort_unstable_by(|left, right| left.public_reference.cmp(&right.public_reference));
        store.sessions.insert(
            event_slug.to_owned(),
            ShowModeSession {
                snapshot,
                scans: previous_scans,
                checklist,
                commerce,
                closed_at_unix_secs: None,
            },
        );
        show_mode_status_for(event_slug, store)
    };
    persist_show_store(&state).await?;
    Ok(status)
}

#[tauri::command]
pub(crate) async fn show_mode_status(
    state: State<'_, AppState>,
    event_slug: String,
) -> Result<ShowModeStatus, AppError> {
    let _mutation = state.show_mode_mutation.lock().await;
    ensure_show_store_loaded(&state).await?;
    let cached = state.show_mode_store.read().await;
    let store = cached.as_ref().ok_or(AppError::Locked)?;
    Ok(show_mode_status_for(event_slug.trim(), store))
}

#[tauri::command]
pub(crate) async fn show_mode_scan(
    state: State<'_, AppState>,
    event_slug: String,
    code: String,
) -> Result<ShowModeScanResult, AppError> {
    let _mutation = state.show_mode_mutation.lock().await;
    let event_slug = event_slug.trim();
    let token = code.trim();
    if token.len() > 4_096 {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_qr_too_long").into(),
        ));
    }
    let reference = parse_t1_reference(token)?;
    let token_hash = hex::encode(Sha256::digest(token.as_bytes()));
    ensure_show_store_loaded(&state).await?;
    let result = {
        let mut cached = state.show_mode_store.write().await;
        let store = cached.as_mut().ok_or(AppError::Locked)?;
        let session = store.sessions.get_mut(event_slug).ok_or_else(|| {
            AppError::Conflict(crate::i18n::tr("native_prepare_offline_event_first").into())
        })?;
        if session.closed_at_unix_secs.is_some() {
            return Err(AppError::Conflict("show_already_closed".to_owned()));
        }
        if !snapshot_is_active(&session.snapshot) {
            return Err(AppError::Conflict(
                crate::i18n::tr("native_snapshot_refresh_required").into(),
            ));
        }
        let pass_index = session
            .snapshot
            .passes
            .binary_search_by(|pass| pass.public_reference.as_str().cmp(reference.as_str()))
            .map_err(|_| {
                AppError::Conflict(crate::i18n::tr("native_ticket_not_in_snapshot").into())
            })?;
        let pass = &session.snapshot.passes[pass_index];
        if !pass.offline_eligible || pass.qr_sha256.as_deref() != Some(token_hash.as_str()) {
            return Err(AppError::Conflict(
                crate::i18n::tr("native_ticket_not_in_snapshot").into(),
            ));
        }
        match session
            .scans
            .binary_search_by(|scan| scan.public_reference.as_str().cmp(reference.as_str()))
        {
            Ok(index) => {
                let existing = &session.scans[index];
                return Ok(ShowModeScanResult {
                    accepted: existing.state != ShowModeScanState::Conflict,
                    duplicate: true,
                    public_reference: existing.public_reference.clone(),
                    holder_name: existing.holder_name.clone(),
                    holder_email_masked: existing.holder_email_masked.clone(),
                    state: existing.state.clone(),
                });
            }
            Err(insert_at) => {
                if session.scans.len() >= 10_000 {
                    return Err(AppError::Conflict(
                        crate::i18n::tr("native_scan_queue_full").into(),
                    ));
                }
                let queued = ShowModeQueuedScan {
                    scan_id: uuid::Uuid::new_v4().to_string(),
                    public_reference: reference.clone(),
                    holder_name: pass.holder_name.clone(),
                    holder_email_masked: pass.holder_email_masked.clone(),
                    scanned_at_unix_secs: unix_now_secs(),
                    state: ShowModeScanState::Pending,
                    result_status: None,
                };
                let result = ShowModeScanResult {
                    accepted: true,
                    duplicate: false,
                    public_reference: reference,
                    holder_name: queued.holder_name.clone(),
                    holder_email_masked: queued.holder_email_masked.clone(),
                    state: queued.state.clone(),
                };
                session.scans.insert(insert_at, queued);
                result
            }
        }
    };
    persist_show_store(&state).await?;
    Ok(result)
}

#[tauri::command]
pub(crate) async fn show_mode_sync(
    state: State<'_, AppState>,
    event_slug: String,
) -> Result<ShowModeSyncResult, AppError> {
    let _mutation = state.show_mode_mutation.lock().await;
    let profile = operator_profile(&state).await?;
    let event_slug = event_slug.trim().to_owned();
    ensure_show_store_loaded(&state).await?;
    let pending = {
        let cached = state.show_mode_store.read().await;
        let store = cached.as_ref().ok_or(AppError::Locked)?;
        let session = store.sessions.get(&event_slug).ok_or_else(|| {
            AppError::Conflict(crate::i18n::tr("native_no_prepared_event").into())
        })?;
        session
            .scans
            .iter()
            .enumerate()
            .filter(|(_, scan)| scan.state == ShowModeScanState::Pending)
            .map(|(index, scan)| (index, scan.public_reference.clone()))
            .collect::<Vec<_>>()
    };
    let api = state.api.clone();
    let outcomes = stream::iter(pending.iter().cloned())
        .map(|(index, reference)| {
            let api = api.clone();
            let profile = Arc::clone(&profile);
            let event_slug = event_slug.clone();
            async move {
                let outcome = api
                    .redeem_admission(profile.as_ref(), &event_slug, &reference)
                    .await;
                (index, outcome)
            }
        })
        .buffer_unordered(SHOW_MODE_SYNC_CONCURRENCY)
        .collect::<Vec<_>>()
        .await;

    let mut result = ShowModeSyncResult {
        attempted: pending.len(),
        ..ShowModeSyncResult::default()
    };
    let mut unexpected = None;
    {
        let mut cached = state.show_mode_store.write().await;
        let store = cached.as_mut().ok_or(AppError::Locked)?;
        let session = store.sessions.get_mut(&event_slug).ok_or_else(|| {
            AppError::Conflict(crate::i18n::tr("native_no_prepared_event").into())
        })?;
        for (index, outcome) in outcomes {
            let Some(scan) = session.scans.get_mut(index) else {
                unexpected.get_or_insert(AppError::BackgroundTask);
                continue;
            };
            match outcome {
                Ok(value) => {
                    let status = value
                        .get("status")
                        .and_then(serde_json::Value::as_str)
                        .value_or("redeemed")
                        .to_owned();
                    scan.state = ShowModeScanState::Synced;
                    scan.result_status = Some(status);
                    result.synced += 1;
                }
                Err(
                    AppError::Conflict(_)
                    | AppError::NotFound
                    | AppError::Remote {
                        status: 404 | 409 | 422,
                        ..
                    },
                ) => {
                    scan.state = ShowModeScanState::Conflict;
                    scan.result_status = Some("conflict".into());
                    result.conflicts += 1;
                }
                Err(
                    AppError::Network(_)
                    | AppError::Unauthorized
                    | AppError::Remote {
                        status: 429 | 500..=599,
                        ..
                    },
                ) => {}
                Err(error) => {
                    unexpected.get_or_insert(error);
                }
            }
        }
        result.pending = session
            .scans
            .iter()
            .filter(|scan| scan.state == ShowModeScanState::Pending)
            .count();
    }
    persist_show_store(&state).await?;
    if let Some(error) = unexpected {
        return Err(error);
    }
    Ok(result)
}

#[tauri::command]
pub(crate) async fn show_mode_close(
    state: State<'_, AppState>,
    event_slug: String,
) -> Result<ShowModeStatus, AppError> {
    let _mutation = state.show_mode_mutation.lock().await;
    let event_slug = event_slug.trim().to_owned();
    ensure_show_store_loaded(&state).await?;

    // Closing is locally idempotent. Once the durable encrypted session is
    // closed, a repeated tap must not require network access just to return
    // the already-established terminal state.
    {
        let cached = state.show_mode_store.read().await;
        let store = cached.as_ref().ok_or(AppError::Locked)?;
        let session = store.sessions.get(&event_slug).ok_or_else(|| {
            AppError::Conflict(crate::i18n::tr("native_no_prepared_event").into())
        })?;
        if session.closed_at_unix_secs.is_some() {
            return Ok(show_mode_status_for(&event_slug, store));
        }
    }

    let profile = operator_profile(&state).await?;
    let checklist = state
        .api
        .operator_show_checklist(&profile, &event_slug)
        .await?;
    let commerce = state
        .api
        .operator_event_merch_summary(&profile, &checklist.event_id)
        .await
        .ok();
    {
        let mut cached = state.show_mode_store.write().await;
        let store = cached.as_mut().ok_or(AppError::Locked)?;
        let session = store.sessions.get_mut(&event_slug).ok_or_else(|| {
            AppError::Conflict(crate::i18n::tr("native_no_prepared_event").into())
        })?;
        session.checklist = Some(checklist);
        session.commerce = commerce;
        let has_pending = session
            .scans
            .iter()
            .any(|scan| scan.state == ShowModeScanState::Pending);
        let has_conflict = session
            .scans
            .iter()
            .any(|scan| scan.state == ShowModeScanState::Conflict);
        let reconciled = session.checklist.as_ref().is_some_and(|value| {
            value.items.iter().any(|item| {
                item.item_key == "post_show_reconciliation"
                    && matches!(item.status.as_str(), "done" | "skipped")
            })
        });
        if has_pending || has_conflict || !reconciled {
            return Err(AppError::Conflict("show_not_ready_to_close".to_owned()));
        }
        session.closed_at_unix_secs = Some(unix_now_secs());
    }
    persist_show_store(&state).await?;
    let cached = state.show_mode_store.read().await;
    let store = cached.as_ref().ok_or(AppError::Locked)?;
    Ok(show_mode_status_for(&event_slug, store))
}

#[tauri::command]
pub(crate) async fn show_mode_clear(
    state: State<'_, AppState>,
    event_slug: String,
) -> Result<ShowModeStatus, AppError> {
    let _mutation = state.show_mode_mutation.lock().await;
    let event_slug = event_slug.trim().to_owned();
    ensure_show_store_loaded(&state).await?;
    {
        let mut cached = state.show_mode_store.write().await;
        let store = cached.as_mut().ok_or(AppError::Locked)?;
        if let Some(session) = store.sessions.get(&event_slug) {
            let has_unsynced_scans = session
                .scans
                .iter()
                .any(|scan| scan.state != ShowModeScanState::Synced);
            if has_unsynced_scans && session.closed_at_unix_secs.is_none() {
                return Err(AppError::Conflict("show_has_unsynced_scans".to_owned()));
            }
        }
        store.sessions.remove(&event_slug);
    }
    persist_show_store(&state).await?;
    Ok(ShowModeStatus {
        event_slug,
        ..ShowModeStatus::default()
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models;

    fn test_value<T, E>(result: Result<T, E>) -> T
    where
        E: std::fmt::Debug,
    {
        match result {
            Ok(value) => value,
            Err(error) => panic!("test setup failed: {error:?}"),
        }
    }

    fn sample_show_snapshot() -> ShowModeSnapshot {
        ShowModeSnapshot {
            schema_version: 1,
            snapshot_id: "snapshot-1".into(),
            event: models::ShowModeEvent {
                slug: "virya-live".into(),
                title: "Virya Live".into(),
                venue: Some("Club".into()),
                starts_at: "2026-08-02T18:00:00Z".into(),
            },
            generated_at: "2026-08-02T12:00:00Z".into(),
            expires_at: "2099-08-02T23:00:00Z".into(),
            checksum_sha256: String::new(),
            passes: vec![models::ShowModePass {
                public_reference: "VRY-TICKET-1".into(),
                holder_name: Some("Fan".into()),
                holder_email_masked: "f***@example.com".into(),
                ticket_type_name: Some("Regular".into()),
                offline_eligible: true,
                qr_sha256: Some("ab".repeat(32)),
            }],
        }
    }

    #[test]
    fn durable_t1_parser_extracts_only_bounded_public_reference() {
        let claims = serde_json::json!({"r": "VRY-TICKET-1"});
        let payload = URL_SAFE_NO_PAD.encode(test_value(serde_json::to_vec(&claims)));
        let token = format!("t1.{payload}.{}", "a".repeat(64));
        assert_eq!(test_value(parse_t1_reference(&token)), "VRY-TICKET-1");
        assert!(parse_t1_reference("v1.not-durable").is_err());
        assert!(parse_t1_reference(&format!("t1.{payload}.short")).is_err());
    }

    #[test]
    fn show_snapshot_checksum_is_deterministic_and_content_sensitive() {
        let snapshot = sample_show_snapshot();
        let checksum = show_mode_checksum(&snapshot);
        assert_eq!(checksum, show_mode_checksum(&snapshot));
        let mut changed = snapshot;
        changed.passes[0].holder_name = Some("Other".into());
        assert_ne!(checksum, show_mode_checksum(&changed));
    }

    #[test]
    fn show_store_normalization_sorts_for_binary_search_and_counts_states() {
        let mut snapshot = sample_show_snapshot();
        let mut other = snapshot.passes[0].clone();
        other.public_reference = "VRY-TICKET-0".into();
        let duplicate = snapshot.passes[0].clone();
        snapshot.passes.insert(0, duplicate);
        snapshot.passes[0].public_reference = "VRY-TICKET-2".into();
        snapshot.passes.push(other);
        let mut store = ShowModeStore::default();
        store.sessions.insert(
            "virya-live".into(),
            ShowModeSession {
                snapshot,
                scans: vec![
                    ShowModeQueuedScan {
                        scan_id: "2".into(),
                        public_reference: "VRY-TICKET-2".into(),
                        holder_name: None,
                        holder_email_masked: "—".into(),
                        scanned_at_unix_secs: 2,
                        state: ShowModeScanState::Conflict,
                        result_status: None,
                    },
                    ShowModeQueuedScan {
                        scan_id: "1".into(),
                        public_reference: "VRY-TICKET-1".into(),
                        holder_name: None,
                        holder_email_masked: "—".into(),
                        scanned_at_unix_secs: 1,
                        state: ShowModeScanState::Pending,
                        result_status: None,
                    },
                ],
                checklist: None,
                commerce: None,
                closed_at_unix_secs: None,
            },
        );
        normalize_show_store(&mut store);
        let session = &store.sessions["virya-live"];
        assert!(
            session
                .snapshot
                .passes
                .windows(2)
                .all(|w| w[0].public_reference <= w[1].public_reference)
        );
        assert!(
            session
                .scans
                .windows(2)
                .all(|w| w[0].public_reference <= w[1].public_reference)
        );
        let status = show_mode_status_for("virya-live", &store);
        assert_eq!((status.pending, status.synced, status.conflicts), (1, 0, 1));
    }

    #[test]
    fn show_store_normalization_prunes_only_expired_fully_synced_sessions() {
        let mut expired_clean = sample_show_snapshot();
        expired_clean.expires_at = "2020-01-01T00:00:00Z".into();
        let mut expired_pending = expired_clean.clone();
        expired_pending.snapshot_id = "snapshot-pending".into();
        let mut store = ShowModeStore::default();
        store.sessions.insert(
            "expired-clean".into(),
            ShowModeSession {
                snapshot: expired_clean,
                scans: vec![ShowModeQueuedScan {
                    scan_id: "clean".into(),
                    public_reference: "VRY-TICKET-1".into(),
                    holder_name: None,
                    holder_email_masked: "—".into(),
                    scanned_at_unix_secs: 1,
                    state: ShowModeScanState::Synced,
                    result_status: Some("redeemed".into()),
                }],
                checklist: None,
                commerce: None,
                closed_at_unix_secs: None,
            },
        );
        store.sessions.insert(
            "expired-pending".into(),
            ShowModeSession {
                snapshot: expired_pending,
                scans: vec![ShowModeQueuedScan {
                    scan_id: "pending".into(),
                    public_reference: "VRY-TICKET-1".into(),
                    holder_name: None,
                    holder_email_masked: "—".into(),
                    scanned_at_unix_secs: 1,
                    state: ShowModeScanState::Pending,
                    result_status: None,
                }],
                checklist: None,
                commerce: None,
                closed_at_unix_secs: None,
            },
        );
        normalize_show_store(&mut store);
        assert!(!store.sessions.contains_key("expired-clean"));
        assert!(store.sessions.contains_key("expired-pending"));
    }
}
