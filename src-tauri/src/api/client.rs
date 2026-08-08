use std::{
    path::PathBuf,
    sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    },
    time::{Duration, Instant},
};

use reqwest::{
    Client, Method, Response, StatusCode,
    header::{ACCEPT, COOKIE, IF_MODIFIED_SINCE, IF_NONE_MATCH, ORIGIN},
};
use serde::{Serialize, de::DeserializeOwned};
use tokio::sync::{Mutex, RwLock};
use uuid::Uuid;

use crate::{
    AppError,
    models::{CitySignal, FanProfile, MerchCatalog, OperatorProfile, PublicEvent},
};

use super::{
    cache::{
        self, CITIES_CACHE_TTL, CITIES_STALE_TTL, CacheEntry, CacheValidators, EVENTS_CACHE_TTL,
        EVENTS_STALE_TTL, PublicCache,
    },
    http::{decode, endpoint},
    retry::retry_idempotent,
};

pub(super) const FAN_COOKIE: &str = "crowdrelay_fan";
pub(super) const PASS_COOKIE: &str = "crowdrelay_pass_session";
pub(super) const WALLET_REQUEST_TIMEOUT: Duration = Duration::from_secs(12);
const STAFF_GATE_URL: &str = "https://virya.music/api/staff/qr/login";
const STAFF_GATE_ORIGIN: &str = "https://virya.music";

#[derive(Serialize)]
struct StaffGateRequest<'a> {
    password: &'a str,
}

fn http_builder() -> reqwest::ClientBuilder {
    #[allow(unused_mut)]
    let mut builder = Client::builder()
        .timeout(Duration::from_secs(15))
        .connect_timeout(Duration::from_secs(5))
        .pool_idle_timeout(Duration::from_secs(60))
        .pool_max_idle_per_host(4)
        .tcp_keepalive(Duration::from_secs(30))
        .user_agent(concat!("virya-signal/", env!("CARGO_PKG_VERSION")))
        .https_only(!cfg!(debug_assertions));
    #[cfg(target_os = "android")]
    {
        builder = builder.use_preconfigured_tls(android_tls_config());
    }
    builder
}

#[derive(Clone)]
pub struct CrowdRelayClient {
    pub(super) http: Client,
    pub(super) site_http: Client,
    pub(super) public_cache: Arc<RwLock<PublicCache>>,
    events_fetch: Arc<Mutex<()>>,
    cities_fetch: Arc<Mutex<()>>,
    pub(super) cache_file: Arc<PathBuf>,
    cache_write: Arc<Mutex<()>>,
    cache_persisting: Arc<AtomicBool>,
    cache_dirty: Arc<AtomicBool>,
}

impl CrowdRelayClient {
    pub fn new(cache_file: PathBuf) -> Result<Self, AppError> {
        let _ = rustls::crypto::ring::default_provider().install_default();
        let http = http_builder().build()?;
        let site_http = http_builder()
            .redirect(reqwest::redirect::Policy::none())
            .pool_max_idle_per_host(1)
            .build()?;
        let public_cache = match cache::load_public_cache(&cache_file) {
            Ok(cache) => cache,
            Err(error) => {
                eprintln!("[virya:cache] public cache ignored after read failure: {error}");
                PublicCache::default()
            }
        };
        Ok(Self {
            http,
            site_http,
            public_cache: Arc::new(RwLock::new(public_cache)),
            events_fetch: Arc::new(Mutex::new(())),
            cities_fetch: Arc::new(Mutex::new(())),
            cache_file: Arc::new(cache_file),
            cache_write: Arc::new(Mutex::new(())),
            cache_persisting: Arc::new(AtomicBool::new(false)),
            cache_dirty: Arc::new(AtomicBool::new(false)),
        })
    }

    pub async fn public_events(&self, api_base_url: &str) -> Result<Vec<PublicEvent>, AppError> {
        let ck = cache::cache_key(api_base_url)?;
        if let Some(events) = self.cached_events(&ck, EVENTS_CACHE_TTL).await {
            return Ok(events);
        }
        let _fetch = self.events_fetch.lock().await;
        if let Some(events) = self.cached_events(&ck, EVENTS_CACHE_TTL).await {
            return Ok(events);
        }
        let stale = self.cached_events(&ck, EVENTS_STALE_TTL).await;
        let validators = self.cache_validators(&ck, true).await;
        let response = match self
            .public_response_base(api_base_url, "public/events?limit=50", validators)
            .await
        {
            Ok(response) => response,
            Err(error) => return stale.ok_or(error),
        };
        if response.status() == reqwest::StatusCode::NOT_MODIFIED {
            let events = stale.ok_or_else(|| AppError::Remote {
                status: reqwest::StatusCode::NOT_MODIFIED.as_u16(),
                detail: crate::i18n::tr("native_missing_events_cache").into(),
            })?;
            self.touch_cache(&ck, true).await;
            self.persist_public_cache_in_background();
            return Ok(events);
        }
        let (etag, last_modified) = cache::response_validators(response.headers());
        let response: crate::models::EventListResponse = decode(response).await?;
        let events = cache::sanitize_public_events(response.events);
        let mut c = self.public_cache.write().await;
        cache::prune_cache(&mut c.events, EVENTS_STALE_TTL);
        c.events.insert(
            ck,
            CacheEntry {
                value: events.clone(),
                fetched_at: Instant::now(),
                stored_at_unix_secs: cache::unix_now(),
                etag,
                last_modified,
            },
        );
        drop(c);
        self.persist_public_cache_in_background();
        Ok(events)
    }

    pub async fn public_merch_catalog(&self, api_base_url: &str) -> Result<MerchCatalog, AppError> {
        let response = self
            .public_response_base(
                api_base_url,
                "public/merch/catalog",
                CacheValidators::default(),
            )
            .await?;
        decode(response).await
    }

    pub async fn verify_staff_access(&self, password: &str) -> Result<(), AppError> {
        let response = self
            .site_http
            .post(STAFF_GATE_URL)
            .header(ACCEPT, "application/json")
            .header(ORIGIN, STAFF_GATE_ORIGIN)
            .json(&StaffGateRequest { password })
            .timeout(Duration::from_secs(12))
            .send()
            .await?;

        staff_gate_status(response.status())
    }

    pub async fn public_cities(&self, api_base_url: &str) -> Result<Vec<CitySignal>, AppError> {
        let ck = cache::cache_key(api_base_url)?;
        if let Some(cities) = self.cached_cities(&ck, CITIES_CACHE_TTL).await {
            return Ok(cities);
        }
        let _fetch = self.cities_fetch.lock().await;
        if let Some(cities) = self.cached_cities(&ck, CITIES_CACHE_TTL).await {
            return Ok(cities);
        }
        let stale = self.cached_cities(&ck, CITIES_STALE_TTL).await;
        let validators = self.cache_validators(&ck, false).await;
        let response = match self
            .public_response_base(api_base_url, "public/cities?limit=100", validators)
            .await
        {
            Ok(response) => response,
            Err(error) => return stale.ok_or(error),
        };
        if response.status() == reqwest::StatusCode::NOT_MODIFIED {
            let cities = stale.ok_or_else(|| AppError::Remote {
                status: reqwest::StatusCode::NOT_MODIFIED.as_u16(),
                detail: crate::i18n::tr("native_missing_cities_cache").into(),
            })?;
            self.touch_cache(&ck, false).await;
            self.persist_public_cache_in_background();
            return Ok(cities);
        }
        let (etag, last_modified) = cache::response_validators(response.headers());
        let response: crate::models::CityListResponse = decode(response).await?;
        let cities = cache::sanitize_public_cities(response.items);
        let mut c = self.public_cache.write().await;
        cache::prune_cache(&mut c.cities, CITIES_STALE_TTL);
        c.cities.insert(
            ck,
            CacheEntry {
                value: cities.clone(),
                fetched_at: Instant::now(),
                stored_at_unix_secs: cache::unix_now(),
                etag,
                last_modified,
            },
        );
        drop(c);
        self.persist_public_cache_in_background();
        Ok(cities)
    }

    pub(super) async fn auth_json<T, B>(
        &self,
        profile: &OperatorProfile,
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<T, AppError>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        let url = endpoint(&profile.api_base_url, path)?;
        let is_read = method == Method::GET;
        let idempotency_key = if is_read {
            None
        } else {
            Some(Uuid::new_v4().to_string())
        };
        let attempt = || async {
            let mut request = self
                .http
                .request(method.clone(), url.clone())
                .header(ACCEPT, "application/json")
                .bearer_auth(profile.bearer_token.trim());
            if let Some(ref key) = idempotency_key {
                request = request.header("Idempotency-Key", key.as_str());
            }
            if let Some(body) = body {
                request = request.json(body);
            }
            decode(request.send().await?).await
        };
        if is_read {
            retry_idempotent(attempt).await
        } else {
            attempt().await
        }
    }

    pub(super) async fn fan_json<T, B>(
        &self,
        profile: &FanProfile,
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<T, AppError>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        let url = endpoint(&profile.api_base_url, path)?;
        let is_read = method == Method::GET;
        let idempotency_key = if is_read {
            None
        } else {
            Some(Uuid::new_v4().to_string())
        };
        let cookie = format!("{FAN_COOKIE}={}", profile.fan_session_token);
        let attempt = || async {
            let mut request = self
                .http
                .request(method.clone(), url.clone())
                .header(ACCEPT, "application/json")
                .header(COOKIE, cookie.as_str());
            if let Some(ref key) = idempotency_key {
                request = request.header("Idempotency-Key", key.as_str());
            }
            if let Some(body) = body {
                request = request.json(body);
            }
            decode(request.send().await?).await
        };
        if is_read {
            retry_idempotent(attempt).await
        } else {
            attempt().await
        }
    }

    pub(super) async fn pass_json<T, B>(
        &self,
        api_base_url: &str,
        pass_session_token: &str,
        method: Method,
        path: &str,
        body: Option<&B>,
    ) -> Result<T, AppError>
    where
        T: DeserializeOwned,
        B: Serialize + ?Sized,
    {
        let url = endpoint(api_base_url, path)?;
        let is_read = method == Method::GET;
        let idempotency_key = if is_read {
            None
        } else {
            Some(Uuid::new_v4().to_string())
        };
        let cookie = format!("{PASS_COOKIE}={pass_session_token}");
        let attempt = || async {
            let mut request = self
                .http
                .request(method.clone(), url.clone())
                .header(ACCEPT, "application/json")
                .header(COOKIE, cookie.as_str());
            if let Some(ref key) = idempotency_key {
                request = request.header("Idempotency-Key", key.as_str());
            }
            if let Some(body) = body {
                request = request.json(body);
            }
            decode(request.send().await?).await
        };
        if is_read {
            retry_idempotent(attempt).await
        } else {
            attempt().await
        }
    }

    async fn cached_events(&self, cache_key: &str, max_age: Duration) -> Option<Vec<PublicEvent>> {
        let c = self.public_cache.read().await;
        c.events
            .get(cache_key)
            .filter(|entry| entry.fetched_at.elapsed() < max_age)
            .map(|entry| entry.value.clone())
    }

    async fn cached_cities(&self, cache_key: &str, max_age: Duration) -> Option<Vec<CitySignal>> {
        let c = self.public_cache.read().await;
        c.cities
            .get(cache_key)
            .filter(|entry| entry.fetched_at.elapsed() < max_age)
            .map(|entry| entry.value.clone())
    }

    async fn cache_validators(&self, cache_key: &str, events: bool) -> CacheValidators {
        let c = self.public_cache.read().await;
        let entry = if events {
            c.events
                .get(cache_key)
                .map(|entry| (entry.etag.as_ref(), entry.last_modified.as_ref()))
        } else {
            c.cities
                .get(cache_key)
                .map(|entry| (entry.etag.as_ref(), entry.last_modified.as_ref()))
        };
        match entry {
            Some((etag, last_modified)) => CacheValidators {
                etag: etag.cloned(),
                last_modified: last_modified.cloned(),
            },
            None => CacheValidators::default(),
        }
    }

    async fn touch_cache(&self, cache_key: &str, events: bool) {
        let now = Instant::now();
        let unix = cache::unix_now();
        let mut c = self.public_cache.write().await;
        if events {
            if let Some(entry) = c.events.get_mut(cache_key) {
                entry.fetched_at = now;
                entry.stored_at_unix_secs = unix;
            }
        } else if let Some(entry) = c.cities.get_mut(cache_key) {
            entry.fetched_at = now;
            entry.stored_at_unix_secs = unix;
        }
    }

    pub(super) fn persist_public_cache_in_background(&self) {
        self.cache_dirty.store(true, Ordering::Release);
        if self.cache_persisting.swap(true, Ordering::AcqRel) {
            return;
        }
        let client = self.clone();
        tokio::spawn(async move {
            loop {
                client.cache_dirty.store(false, Ordering::Release);
                client.persist_public_cache().await;
                if client.cache_dirty.load(Ordering::Acquire) {
                    continue;
                }
                client.cache_persisting.store(false, Ordering::Release);
                if !client.cache_dirty.load(Ordering::Acquire) {
                    break;
                }
                if client.cache_persisting.swap(true, Ordering::AcqRel) {
                    break;
                }
            }
        });
    }

    async fn persist_public_cache(&self) {
        let _write = self.cache_write.lock().await;
        let payload = {
            let c = self.public_cache.read().await;
            cache::serialize_cache(&c)
        };
        let Some(payload) = payload else { return };
        let cache_file = self.cache_file.as_ref().clone();
        let _ =
            tokio::task::spawn_blocking(move || cache::write_public_cache(&cache_file, &payload))
                .await;
    }

    async fn public_response_base(
        &self,
        api_base_url: &str,
        path: &str,
        validators: CacheValidators,
    ) -> Result<Response, AppError> {
        let mut request = self
            .http
            .get(endpoint(api_base_url, path)?)
            .header(ACCEPT, "application/json");
        if let Some(etag) = validators.etag {
            request = request.header(IF_NONE_MATCH, etag);
        }
        if let Some(last_modified) = validators.last_modified {
            request = request.header(IF_MODIFIED_SINCE, last_modified);
        }
        Ok(request.send().await?)
    }
}

fn staff_gate_status(status: StatusCode) -> Result<(), AppError> {
    match status {
        StatusCode::OK => Ok(()),
        StatusCode::UNAUTHORIZED => Err(AppError::InvalidInput(
            crate::i18n::tr("native_invalid_staff_password").to_owned(),
        )),
        StatusCode::TOO_MANY_REQUESTS => Err(AppError::InvalidInput(
            crate::i18n::tr("native_staff_rate_limited").to_owned(),
        )),
        StatusCode::SERVICE_UNAVAILABLE => Err(AppError::Remote {
            status: status.as_u16(),
            detail: crate::i18n::tr("native_staff_verification_unavailable").to_owned(),
        }),
        _ => Err(AppError::Remote {
            status: status.as_u16(),
            detail: crate::i18n::tr("native_staff_verification_failed").to_owned(),
        }),
    }
}

#[cfg(test)]
mod staff_gate_tests {
    use super::*;

    #[test]
    fn accepts_only_success_status() {
        assert!(staff_gate_status(StatusCode::OK).is_ok());
        assert!(staff_gate_status(StatusCode::UNAUTHORIZED).is_err());
        assert!(staff_gate_status(StatusCode::TOO_MANY_REQUESTS).is_err());
        assert!(staff_gate_status(StatusCode::SERVICE_UNAVAILABLE).is_err());
    }
}

#[cfg(target_os = "android")]
fn android_tls_config() -> rustls::ClientConfig {
    let root_store = rustls::RootCertStore {
        roots: webpki_roots::TLS_SERVER_ROOTS.to_vec(),
    };
    rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth()
}
