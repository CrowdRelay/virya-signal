use std::{
    collections::HashMap,
    path::Path,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use reqwest::header::{ETAG, HeaderMap, LAST_MODIFIED};
use serde::{Deserialize, Serialize};

use crate::{
    AppError,
    models::{CitySignal, PublicEvent},
    util::OptionValueOrExt,
};

pub(super) const MAX_PUBLIC_EVENTS: usize = 100;
pub(super) const MAX_PUBLIC_CITIES: usize = 250;
pub(super) const EVENTS_CACHE_TTL: Duration = Duration::from_secs(30);
pub(super) const CITIES_CACHE_TTL: Duration = Duration::from_secs(5 * 60);
pub(super) const EVENTS_STALE_TTL: Duration = Duration::from_secs(6 * 60 * 60);
pub(super) const CITIES_STALE_TTL: Duration = Duration::from_secs(24 * 60 * 60);
pub(super) const MAX_CACHE_ORIGINS: usize = 8;
const PUBLIC_CACHE_VERSION: u8 = 1;
pub(super) const MAX_DISK_CACHE_BYTES: u64 = 2 * 1024 * 1024;

pub(super) struct CacheEntry<T> {
    pub value: T,
    pub fetched_at: Instant,
    pub stored_at_unix_secs: u64,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

#[derive(Default)]
pub(super) struct PublicCache {
    pub events: HashMap<String, CacheEntry<Vec<PublicEvent>>>,
    pub cities: HashMap<String, CacheEntry<Vec<CitySignal>>>,
}

#[derive(Default)]
pub(super) struct CacheValidators {
    pub etag: Option<String>,
    pub last_modified: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct DiskCacheEntry<T> {
    value: T,
    stored_at_unix_secs: u64,
    etag: Option<String>,
    last_modified: Option<String>,
}

#[derive(Default, Deserialize, Serialize)]
struct DiskPublicCache {
    version: u8,
    events: HashMap<String, DiskCacheEntry<Vec<PublicEvent>>>,
    cities: HashMap<String, DiskCacheEntry<Vec<CitySignal>>>,
}

pub(super) fn cache_key(api_base_url: &str) -> Result<String, AppError> {
    let mut url = super::http::endpoint(api_base_url, "")?;
    url.set_query(None);
    url.set_fragment(None);
    Ok(url.to_string())
}

pub(super) fn unix_now() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(value) => value.as_secs(),
        Err(error) => {
            eprintln!("[virya:clock] system clock predates Unix epoch: {error}");
            0
        }
    }
}

pub(super) fn response_validators(headers: &HeaderMap) -> (Option<String>, Option<String>) {
    let read = |name| {
        headers
            .get(name)
            .and_then(|value| value.to_str().ok())
            .filter(|value| value.len() <= 1024)
            .map(ToOwned::to_owned)
    };
    (read(ETAG), read(LAST_MODIFIED))
}

pub(super) fn prune_cache<T>(cache: &mut HashMap<String, CacheEntry<T>>, ttl: Duration) {
    cache.retain(|_, entry| entry.fetched_at.elapsed() < ttl);
    if cache.len() >= MAX_CACHE_ORIGINS {
        let oldest = cache
            .iter()
            .max_by_key(|(_, entry)| entry.fetched_at.elapsed())
            .map(|(key, _)| key.clone());
        if let Some(oldest) = oldest {
            cache.remove(&oldest);
        }
    }
}

pub(super) fn sanitize_public_events(mut values: Vec<PublicEvent>) -> Vec<PublicEvent> {
    values.retain(|value| {
        !value.slug.trim().is_empty() && !value.title.trim().is_empty() && value.slug.len() <= 128
    });
    values.sort_unstable_by(|left, right| {
        left.starts_at
            .cmp(&right.starts_at)
            .then_with(|| left.slug.cmp(&right.slug))
    });
    values.dedup_by(|left, right| left.slug == right.slug);
    values.truncate(MAX_PUBLIC_EVENTS);
    values
}

pub(super) fn sanitize_public_cities(mut values: Vec<CitySignal>) -> Vec<CitySignal> {
    values.retain(|value| {
        !value.slug.trim().is_empty() && !value.name.trim().is_empty() && value.slug.len() <= 128
    });
    values.sort_unstable_by(|left, right| {
        right
            .fan_count
            .cmp(&left.fan_count)
            .then_with(|| left.name.cmp(&right.name))
            .then_with(|| left.slug.cmp(&right.slug))
    });
    values.dedup_by(|left, right| left.slug == right.slug);
    values.truncate(MAX_PUBLIC_CITIES);
    values
}

pub(super) fn load_public_cache(path: &Path) -> Result<PublicCache, AppError> {
    let metadata = std::fs::metadata(path)?;
    if metadata.len() > MAX_DISK_CACHE_BYTES {
        return Err(AppError::InvalidInput(
            "Lokalny cache danych publicznych jest zbyt duży".into(),
        ));
    }
    let disk: DiskPublicCache = serde_json::from_slice(&std::fs::read(path)?)?;
    if disk.version != PUBLIC_CACHE_VERSION {
        return Ok(PublicCache::default());
    }
    Ok(PublicCache {
        events: restore_entries(disk.events, EVENTS_STALE_TTL),
        cities: restore_entries(disk.cities, CITIES_STALE_TTL),
    })
}

pub(super) fn write_public_cache(path: &Path, payload: &[u8]) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension("tmp");
    std::fs::write(&temporary, payload)?;
    std::fs::rename(temporary, path)
}

pub(super) fn serialize_cache(cache: &PublicCache) -> Option<Vec<u8>> {
    let disk_cache = DiskPublicCache {
        version: PUBLIC_CACHE_VERSION,
        events: cache
            .events
            .iter()
            .map(|(key, entry)| (key.clone(), disk_entry(entry)))
            .collect(),
        cities: cache
            .cities
            .iter()
            .map(|(key, entry)| (key.clone(), disk_entry(entry)))
            .collect(),
    };
    let payload = serde_json::to_vec(&disk_cache).ok()?;
    if payload.len() > MAX_DISK_CACHE_BYTES as usize {
        return None;
    }
    Some(payload)
}

fn disk_entry<T: Clone>(entry: &CacheEntry<T>) -> DiskCacheEntry<T> {
    DiskCacheEntry {
        value: entry.value.clone(),
        stored_at_unix_secs: entry.stored_at_unix_secs,
        etag: entry.etag.clone(),
        last_modified: entry.last_modified.clone(),
    }
}

fn restore_entries<T>(
    entries: HashMap<String, DiskCacheEntry<T>>,
    max_age: Duration,
) -> HashMap<String, CacheEntry<T>> {
    let now_unix = unix_now();
    let now = Instant::now();
    let mut entries = entries
        .into_iter()
        .filter_map(|(key, entry)| {
            let age = Duration::from_secs(now_unix.saturating_sub(entry.stored_at_unix_secs));
            (age <= max_age).then_some((key, entry, age))
        })
        .collect::<Vec<_>>();
    entries.sort_by_key(|(_, entry, _)| std::cmp::Reverse(entry.stored_at_unix_secs));
    entries
        .into_iter()
        .take(MAX_CACHE_ORIGINS)
        .map(|(key, entry, age)| {
            (
                key,
                CacheEntry {
                    value: entry.value,
                    fetched_at: now.checked_sub(age).value_or(now),
                    stored_at_unix_secs: entry.stored_at_unix_secs,
                    etag: entry.etag,
                    last_modified: entry.last_modified,
                },
            )
        })
        .collect()
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn cache_key_normalizes_trailing_slash() {
        assert_eq!(
            cache_key("https://signal-api.virya.music/v1").unwrap(),
            "https://signal-api.virya.music/v1/"
        );
    }

    #[test]
    fn cache_pruning_bounds_origins() {
        let mut cache = HashMap::new();
        for index in 0..MAX_CACHE_ORIGINS {
            cache.insert(
                format!("origin-{index}"),
                CacheEntry {
                    value: index,
                    fetched_at: Instant::now() - Duration::from_millis(index as u64),
                    stored_at_unix_secs: unix_now().saturating_sub(index as u64),
                    etag: None,
                    last_modified: None,
                },
            );
        }
        prune_cache(&mut cache, Duration::from_secs(60));
        assert_eq!(cache.len(), MAX_CACHE_ORIGINS - 1);
        assert!(!cache.contains_key(&format!("origin-{}", MAX_CACHE_ORIGINS - 1)));
    }

    #[test]
    fn cache_pruning_removes_expired_entries_before_capacity_eviction() {
        let mut cache = HashMap::from([
            (
                "fresh".to_owned(),
                CacheEntry {
                    value: 1,
                    fetched_at: Instant::now(),
                    stored_at_unix_secs: unix_now(),
                    etag: None,
                    last_modified: None,
                },
            ),
            (
                "expired".to_owned(),
                CacheEntry {
                    value: 2,
                    fetched_at: Instant::now() - Duration::from_secs(120),
                    stored_at_unix_secs: unix_now().saturating_sub(120),
                    etag: None,
                    last_modified: None,
                },
            ),
        ]);
        prune_cache(&mut cache, Duration::from_secs(60));
        assert_eq!(cache.len(), 1);
        assert!(cache.contains_key("fresh"));
    }

    #[test]
    fn disk_cache_restore_drops_expired_entries_and_preserves_validators() {
        let now = unix_now();
        let restored = restore_entries(
            HashMap::from([
                (
                    "fresh".to_owned(),
                    DiskCacheEntry {
                        value: vec![1_u8],
                        stored_at_unix_secs: now.saturating_sub(5),
                        etag: Some("\"events-v2\"".to_owned()),
                        last_modified: None,
                    },
                ),
                (
                    "expired".to_owned(),
                    DiskCacheEntry {
                        value: vec![2_u8],
                        stored_at_unix_secs: now.saturating_sub(120),
                        etag: None,
                        last_modified: None,
                    },
                ),
            ]),
            Duration::from_secs(60),
        );

        assert_eq!(restored.len(), 1);
        assert_eq!(restored["fresh"].value, vec![1]);
        assert_eq!(restored["fresh"].etag.as_deref(), Some("\"events-v2\""));
    }

    #[test]
    fn response_cache_validators_are_bounded() {
        use reqwest::header::HeaderMap;

        let mut headers = HeaderMap::new();
        headers.insert(ETAG, "\"events-v2\"".parse().unwrap());
        headers.insert(
            LAST_MODIFIED,
            "Sat, 01 Aug 2026 07:00:00 GMT".parse().unwrap(),
        );

        let (etag, last_modified) = response_validators(&headers);
        assert_eq!(etag.as_deref(), Some("\"events-v2\""));
        assert_eq!(
            last_modified.as_deref(),
            Some("Sat, 01 Aug 2026 07:00:00 GMT")
        );

        headers.insert(ETAG, "x".repeat(1025).parse().unwrap());
        assert!(response_validators(&headers).0.is_none());
    }

    #[test]
    fn sanitize_events_deduplicates_sorts_and_truncates() {
        let event = |slug: &str, starts_at: &str| PublicEvent {
            slug: slug.to_owned(),
            title: format!("Event {slug}"),
            starts_at: starts_at.to_owned(),
            description: None,
            city: None,
            venue: None,
            ticket_url: None,
            image_url: None,
            image_thumbnail_url: None,
        };
        let events = vec![
            event("b-event", "2026-09-01T20:00:00Z"),
            event("a-event", "2026-08-01T20:00:00Z"),
            event("a-event", "2026-08-01T20:00:00Z"),
            event("", "2026-10-01T20:00:00Z"),
        ];
        let result = sanitize_public_events(events);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].slug, "a-event");
        assert_eq!(result[1].slug, "b-event");
    }

    #[test]
    fn sanitize_cities_deduplicates_and_sorts_by_fan_count() {
        let city = |slug: &str, fan_count: u64| CitySignal {
            slug: slug.to_owned(),
            name: format!("City {slug}"),
            country_code: "PL".to_owned(),
            fan_count,
        };
        let cities = vec![
            city("small", 10),
            city("big", 500),
            city("big", 500),
            city("", 999),
        ];
        let result = sanitize_public_cities(cities);
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].slug, "big");
        assert_eq!(result[1].slug, "small");
    }
}
