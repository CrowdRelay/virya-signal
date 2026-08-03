trait OptionValueOrExt<T> {
    fn value_or(self, fallback: T) -> T;
}

impl<T> OptionValueOrExt<T> for Option<T> {
    #[allow(clippy::manual_unwrap_or)]
    fn value_or(self, fallback: T) -> T {
        match self {
            Some(value) => value,
            None => fallback,
        }
    }
}

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use reqwest::{
    header::{
        HeaderMap, ACCEPT, COOKIE, ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, LAST_MODIFIED,
        SET_COOKIE,
    },
    Client, Method, RequestBuilder, Response, StatusCode,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use tokio::sync::{Mutex, RwLock};
use url::Url;
use uuid::Uuid;

use crate::{
    models::{
        AdmissionPass, AreaWallet, CityListResponse, CitySignal, ConcertQrOverview,
        CreateQrCampaignInput, EventListResponse, FanAuthResult, FanConfirmationInput,
        FanEventInterest, FanProfile, FanSignupInput, IssuePassInput, OperatorOpsOverview,
        OperatorProfile, OperatorRole, OperatorSignalOverview, OpsDeliveryItem, OpsOutboxItem,
        OpsRetryResult, OpsSummary, PublicEvent, ReferralProgress, RequestedCityInput,
        RequestedCityResult, ShowModeSnapshot, TicketWalletApi, TicketingOverview,
    },
    AppError,
};

const FAN_COOKIE: &str = "crowdrelay_fan";
const AREA_COOKIE: &str = "virya-area-wallet";
const AREA_WALLET_URL: &str = "https://virya.music/api/area/wallet";
const PASS_COOKIE: &str = "crowdrelay_pass_session";
const MAX_RESPONSE_BYTES: u64 = 2 * 1024 * 1024;
const MAX_PUBLIC_EVENTS: usize = 100;
const MAX_PUBLIC_CITIES: usize = 250;
const MAX_TOKEN_BYTES: usize = 4096;
const WALLET_REQUEST_TIMEOUT: Duration = Duration::from_secs(8);
const EVENTS_CACHE_TTL: Duration = Duration::from_secs(30);
const CITIES_CACHE_TTL: Duration = Duration::from_secs(5 * 60);
const EVENTS_STALE_TTL: Duration = Duration::from_secs(6 * 60 * 60);
const CITIES_STALE_TTL: Duration = Duration::from_secs(24 * 60 * 60);
const MAX_CACHE_ORIGINS: usize = 8;
const PUBLIC_CACHE_VERSION: u8 = 1;
const MAX_DISK_CACHE_BYTES: u64 = 2 * 1024 * 1024;

struct CacheEntry<T> {
    value: T,
    fetched_at: Instant,
    stored_at_unix_secs: u64,
    etag: Option<String>,
    last_modified: Option<String>,
}

#[derive(Default)]
struct PublicCache {
    events: HashMap<String, CacheEntry<Vec<PublicEvent>>>,
    cities: HashMap<String, CacheEntry<Vec<CitySignal>>>,
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

#[derive(Default)]
struct CacheValidators {
    etag: Option<String>,
    last_modified: Option<String>,
}

#[derive(Clone)]
pub struct CrowdRelayClient {
    http: Client,
    public_cache: Arc<RwLock<PublicCache>>,
    events_fetch: Arc<Mutex<()>>,
    cities_fetch: Arc<Mutex<()>>,
    cache_file: Arc<PathBuf>,
    cache_write: Arc<Mutex<()>>,
}

impl CrowdRelayClient {
    pub fn new(cache_file: PathBuf) -> Result<Self, AppError> {
        // Ring is materially smaller and faster to compile for Android than the
        // default AWS-LC provider. Installing it once also makes the TLS choice
        // explicit instead of depending on transitive feature defaults.
        let _ = rustls::crypto::ring::default_provider().install_default();
        let http = Client::builder()
            .timeout(Duration::from_secs(15))
            .connect_timeout(Duration::from_secs(5))
            .pool_idle_timeout(Duration::from_secs(60))
            .pool_max_idle_per_host(4)
            .tcp_keepalive(Duration::from_secs(30))
            .user_agent(concat!("crowdrelay-mobile/", env!("CARGO_PKG_VERSION")))
            .https_only(!cfg!(debug_assertions))
            .build()?;
        let public_cache = match load_public_cache(&cache_file) {
            Ok(cache) => cache,
            Err(error) => {
                eprintln!("[virya:cache] public cache ignored after read failure: {error}");
                PublicCache::default()
            }
        };
        Ok(Self {
            http,
            public_cache: Arc::new(RwLock::new(public_cache)),
            events_fetch: Arc::new(Mutex::new(())),
            cities_fetch: Arc::new(Mutex::new(())),
            cache_file: Arc::new(cache_file),
            cache_write: Arc::new(Mutex::new(())),
        })
    }

    pub async fn validate(&self, profile: &OperatorProfile) -> Result<(), AppError> {
        let path = match profile.role {
            OperatorRole::Owner => "admin/event-qr/overview",
            OperatorRole::Staff => "staff/event-qr/overview",
        };
        let _: serde_json::Value = self
            .auth_json(profile, Method::GET, path, Option::<&()>::None)
            .await?;
        Ok(())
    }

    pub async fn operator_events(
        &self,
        profile: &OperatorProfile,
    ) -> Result<Vec<PublicEvent>, AppError> {
        self.public_events(&profile.api_base_url).await
    }

    pub async fn operator_qr(
        &self,
        profile: &OperatorProfile,
    ) -> Result<ConcertQrOverview, AppError> {
        let qr_path = match profile.role {
            OperatorRole::Owner => "admin/event-qr/overview",
            OperatorRole::Staff => "staff/event-qr/overview",
        };
        self.auth_json::<ConcertQrOverview, ()>(profile, Method::GET, qr_path, None)
            .await
    }

    pub async fn public_events(&self, api_base_url: &str) -> Result<Vec<PublicEvent>, AppError> {
        let cache_key = cache_key(api_base_url)?;
        if let Some(events) = self.cached_events(&cache_key, EVENTS_CACHE_TTL).await {
            return Ok(events);
        }
        let _fetch = self.events_fetch.lock().await;
        if let Some(events) = self.cached_events(&cache_key, EVENTS_CACHE_TTL).await {
            return Ok(events);
        }
        let stale = self.cached_events(&cache_key, EVENTS_STALE_TTL).await;
        let validators = self.cache_validators(&cache_key, true).await;
        let response = match self
            .public_response_base(api_base_url, "public/events?limit=50", validators)
            .await
        {
            Ok(response) => response,
            Err(error) => return stale.ok_or(error),
        };
        if response.status() == StatusCode::NOT_MODIFIED {
            let events = stale.ok_or_else(|| AppError::Remote {
                status: StatusCode::NOT_MODIFIED.as_u16(),
                detail: "Backend potwierdził nieistniejący cache koncertów".into(),
            })?;
            self.touch_cache(&cache_key, true).await;
            self.persist_public_cache_in_background();
            return Ok(events);
        }
        let (etag, last_modified) = response_validators(response.headers());
        let response: EventListResponse = decode(response).await?;
        let events = sanitize_public_events(response.events);
        let mut cache = self.public_cache.write().await;
        prune_cache(&mut cache.events, EVENTS_STALE_TTL);
        cache.events.insert(
            cache_key,
            CacheEntry {
                value: events.clone(),
                fetched_at: Instant::now(),
                stored_at_unix_secs: unix_now(),
                etag,
                last_modified,
            },
        );
        drop(cache);
        self.persist_public_cache_in_background();
        Ok(events)
    }

    pub async fn public_cities(&self, api_base_url: &str) -> Result<Vec<CitySignal>, AppError> {
        let cache_key = cache_key(api_base_url)?;
        if let Some(cities) = self.cached_cities(&cache_key, CITIES_CACHE_TTL).await {
            return Ok(cities);
        }
        let _fetch = self.cities_fetch.lock().await;
        if let Some(cities) = self.cached_cities(&cache_key, CITIES_CACHE_TTL).await {
            return Ok(cities);
        }
        let stale = self.cached_cities(&cache_key, CITIES_STALE_TTL).await;
        let validators = self.cache_validators(&cache_key, false).await;
        let response = match self
            .public_response_base(api_base_url, "public/cities?limit=100", validators)
            .await
        {
            Ok(response) => response,
            Err(error) => return stale.ok_or(error),
        };
        if response.status() == StatusCode::NOT_MODIFIED {
            let cities = stale.ok_or_else(|| AppError::Remote {
                status: StatusCode::NOT_MODIFIED.as_u16(),
                detail: "Backend potwierdził nieistniejący cache miast".into(),
            })?;
            self.touch_cache(&cache_key, false).await;
            self.persist_public_cache_in_background();
            return Ok(cities);
        }
        let (etag, last_modified) = response_validators(response.headers());
        let response: CityListResponse = decode(response).await?;
        let cities = sanitize_public_cities(response.items);
        let mut cache = self.public_cache.write().await;
        prune_cache(&mut cache.cities, CITIES_STALE_TTL);
        cache.cities.insert(
            cache_key,
            CacheEntry {
                value: cities.clone(),
                fetched_at: Instant::now(),
                stored_at_unix_secs: unix_now(),
                etag,
                last_modified,
            },
        );
        drop(cache);
        self.persist_public_cache_in_background();
        Ok(cities)
    }

    pub async fn ticketing_overview(
        &self,
        profile: &OperatorProfile,
        event_slug: &str,
    ) -> Result<TicketingOverview, AppError> {
        let prefix = match profile.role {
            OperatorRole::Owner => "admin",
            OperatorRole::Staff => "staff",
        };
        self.auth_json(
            profile,
            Method::GET,
            &format!("{prefix}/events/{}/ticketing", segment(event_slug)?),
            Option::<&()>::None,
        )
        .await
    }

    pub async fn redeem_admission(
        &self,
        profile: &OperatorProfile,
        event_slug: &str,
        raw_code: &str,
    ) -> Result<serde_json::Value, AppError> {
        let token = normalize_scanned_code(raw_code)?;
        let body = if token.starts_with("v1.") || token.starts_with("t1.") {
            serde_json::json!({"event_slug": event_slug, "qr_token": token, "public_reference": null})
        } else {
            serde_json::json!({"event_slug": event_slug, "qr_token": null, "public_reference": token})
        };
        self.auth_json(profile, Method::POST, "staff/admission/redeem", Some(&body))
            .await
    }

    pub async fn redeem_coupon(
        &self,
        profile: &OperatorProfile,
        code: &str,
        order_reference: &str,
    ) -> Result<serde_json::Value, AppError> {
        let code = bounded_required(code, "kod kuponu", 128)?;
        let order_reference = bounded_required(order_reference, "numer sprzedaży", 200)?;
        let body = serde_json::json!({"code": code.to_ascii_uppercase(), "order_reference": order_reference});
        self.auth_json(profile, Method::POST, "staff/coupons/redeem", Some(&body))
            .await
    }

    pub async fn issue_pass(
        &self,
        profile: &OperatorProfile,
        input: &IssuePassInput,
    ) -> Result<serde_json::Value, AppError> {
        require_owner(profile)?;
        self.auth_json(profile, Method::POST, "admin/admission/passes", Some(input))
            .await
    }

    pub async fn revoke_pass(
        &self,
        profile: &OperatorProfile,
        reference: &str,
    ) -> Result<serde_json::Value, AppError> {
        require_owner(profile)?;
        self.auth_json(
            profile,
            Method::POST,
            &format!("admin/admission/passes/{}/revoke", segment(reference)?),
            Option::<&()>::None,
        )
        .await
    }

    pub async fn create_qr_campaign(
        &self,
        profile: &OperatorProfile,
        input: &CreateQrCampaignInput,
    ) -> Result<serde_json::Value, AppError> {
        self.auth_json(
            profile,
            Method::POST,
            "staff/event-qr/campaigns",
            Some(input),
        )
        .await
    }

    pub async fn revoke_qr_campaign(
        &self,
        profile: &OperatorProfile,
        campaign_id: &str,
    ) -> Result<serde_json::Value, AppError> {
        self.auth_json(
            profile,
            Method::POST,
            &format!("staff/event-qr/campaigns/{}/revoke", segment(campaign_id)?),
            Option::<&()>::None,
        )
        .await
    }

    pub async fn operator_signal_overview(
        &self,
        profile: &OperatorProfile,
    ) -> Result<OperatorSignalOverview, AppError> {
        require_owner(profile)?;
        let mut overview = self
            .auth_json::<OperatorSignalOverview, ()>(
                profile,
                Method::GET,
                "admin/signal/overview",
                None,
            )
            .await?;

        let summary = &mut overview.summary;
        summary.total_fans = summary.total_fans.max(0);
        summary.active_fans = summary.active_fans.max(0);
        summary.pending_fans = summary.pending_fans.max(0);
        summary.unsubscribed_fans = summary.unsubscribed_fans.max(0);
        summary.suppressed_fans = summary.suppressed_fans.max(0);
        summary.marketing_opted_in = summary.marketing_opted_in.max(0);
        summary.nearby_enabled = summary.nearby_enabled.max(0);

        let activity = &mut overview.activity;
        activity.new_fans_7d = activity.new_fans_7d.max(0);
        activity.new_fans_30d = activity.new_fans_30d.max(0);
        activity.referral_attributions_total = activity.referral_attributions_total.max(0);
        activity.referral_attributions_30d = activity.referral_attributions_30d.max(0);
        activity.event_interests_total = activity.event_interests_total.max(0);
        activity.event_interests_30d = activity.event_interests_30d.max(0);
        activity.nearby_notifications_30d = activity.nearby_notifications_30d.max(0);
        activity.pending_city_requests = activity.pending_city_requests.max(0);

        overview.top_cities.retain(|city| {
            !city.name.trim().is_empty()
                && city.name.chars().count() <= 160
                && city.country_code.len() == 2
                && city
                    .country_code
                    .bytes()
                    .all(|byte| byte.is_ascii_uppercase())
                && city.active_fans >= 0
        });
        overview.top_cities.truncate(10);
        overview.unavailable_sources.retain(|source| {
            !source.trim().is_empty()
                && source.len() <= 64
                && source
                    .bytes()
                    .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-'))
        });
        overview.unavailable_sources.truncate(8);
        overview.generated_at = overview.generated_at.chars().take(64).collect();

        Ok(overview)
    }

    pub async fn operator_ops_overview(
        &self,
        profile: &OperatorProfile,
    ) -> Result<OperatorOpsOverview, AppError> {
        require_owner(profile)?;
        let summary_request =
            self.auth_json::<OpsSummary, ()>(profile, Method::GET, "admin/ops/summary", None);
        let deliveries_request = self.auth_json::<Vec<OpsDeliveryItem>, ()>(
            profile,
            Method::GET,
            "admin/ops/deliveries?status=dead&limit=25",
            None,
        );
        let outbox_request = self.auth_json::<Vec<OpsOutboxItem>, ()>(
            profile,
            Method::GET,
            "admin/ops/outbox?status=dead&limit=25",
            None,
        );
        let (summary_result, deliveries_result, outbox_result) =
            futures_util::future::join3(summary_request, deliveries_request, outbox_request).await;
        if summary_result.is_err() && deliveries_result.is_err() && outbox_result.is_err() {
            return match summary_result {
                Err(error) => Err(error),
                Ok(_) => Err(AppError::BackgroundTask),
            };
        }
        let mut unavailable_sources = Vec::new();
        let summary = match summary_result {
            Ok(value) => value,
            Err(_) => {
                unavailable_sources.push("summary".to_owned());
                OpsSummary::default()
            }
        };
        let dead_deliveries = match deliveries_result {
            Ok(value) => value,
            Err(_) => {
                unavailable_sources.push("deliveries".to_owned());
                Vec::new()
            }
        };
        let dead_outbox = match outbox_result {
            Ok(value) => value,
            Err(_) => {
                unavailable_sources.push("outbox".to_owned());
                Vec::new()
            }
        };
        Ok(OperatorOpsOverview {
            summary,
            dead_deliveries,
            dead_outbox,
            unavailable_sources,
        })
    }

    pub async fn operator_retry(
        &self,
        profile: &OperatorProfile,
        target_kind: &str,
        target_id: &str,
    ) -> Result<OpsRetryResult, AppError> {
        require_owner(profile)?;
        let target_kind = match target_kind {
            "outbox" | "deliveries" => target_kind,
            _ => return Err(AppError::InvalidInput("Nieprawidłowy typ kolejki".into())),
        };
        let target_id = uuid_segment(target_id)?;
        self.auth_json::<OpsRetryResult, ()>(
            profile,
            Method::POST,
            &format!("admin/ops/{target_kind}/{target_id}/retry"),
            None,
        )
        .await
    }

    pub async fn show_mode_snapshot(
        &self,
        profile: &OperatorProfile,
        event_slug: &str,
    ) -> Result<ShowModeSnapshot, AppError> {
        self.auth_json::<ShowModeSnapshot, ()>(
            profile,
            Method::GET,
            &format!("staff/ops/show-snapshot/{}", segment(event_slug)?),
            None,
        )
        .await
    }

    pub async fn fan_area_wallet(&self, profile: &FanProfile) -> Result<AreaWallet, AppError> {
        let wallet_id = Uuid::parse_str(profile.area_wallet_id.trim()).map_err(|_| {
            AppError::InvalidInput("Nieprawidłowy identyfikator portfela AREA".into())
        })?;
        let response = self
            .http
            .get(Url::parse(AREA_WALLET_URL)?)
            .header(ACCEPT, "application/json")
            .header(COOKIE, format!("{AREA_COOKIE}={wallet_id}"))
            .timeout(WALLET_REQUEST_TIMEOUT)
            .send()
            .await?;
        decode(response).await
    }

    pub async fn request_city(
        &self,
        api_base_url: &str,
        input: &RequestedCityInput,
    ) -> Result<RequestedCityResult, AppError> {
        let response = self
            .http
            .post(endpoint(api_base_url, "public/cities/requests")?)
            .header(ACCEPT, "application/json")
            .header("Idempotency-Key", Uuid::new_v4().to_string())
            .json(input)
            .send()
            .await?;
        decode(response).await
    }

    pub async fn fan_signup(
        &self,
        input: &FanSignupInput,
    ) -> Result<(FanAuthResult, Option<String>), AppError> {
        let body = serde_json::json!({
            "email": input.email.trim(),
            "display_name": normalized_optional(&input.display_name),
            "city_slug": input.city_slug.trim(),
            "locale": input.locale.trim(),
            "referral_code": normalized_optional(&input.referral_code),
            "campaign_id": null,
            "consent": {
                "marketing": true,
                "policy_version": input.policy_version.trim(),
            },
            "nearby_gigs": {
                "enabled": input.nearby_gigs_enabled,
                "radius_km": input.nearby_radius_km,
            }
        });
        let response = self
            .http
            .post(endpoint(&input.api_base_url, "fans")?)
            .header(ACCEPT, "application/json")
            .header("Idempotency-Key", Uuid::new_v4().to_string())
            .json(&body)
            .send()
            .await?;
        let token = response_cookie(response.headers(), FAN_COOKIE);
        let _: serde_json::Value = decode(response).await?;
        Ok((
            FanAuthResult {
                session_created: token.is_some(),
            },
            token,
        ))
    }

    pub async fn fan_confirm(
        &self,
        input: &FanConfirmationInput,
    ) -> Result<(FanAuthResult, String), AppError> {
        let response = self
            .http
            .post(endpoint(&input.api_base_url, "fans/confirm")?)
            .header(ACCEPT, "application/json")
            .header("Idempotency-Key", Uuid::new_v4().to_string())
            .json(&serde_json::json!({"token": input.token.trim()}))
            .send()
            .await?;
        let token =
            response_cookie(response.headers(), FAN_COOKIE).ok_or_else(|| AppError::Remote {
                status: response.status().as_u16(),
                detail: "Backend nie zwrócił sesji fana".into(),
            })?;
        let _: serde_json::Value = decode(response).await?;
        Ok((
            FanAuthResult {
                session_created: true,
            },
            token,
        ))
    }

    pub async fn fan_events(&self, profile: &FanProfile) -> Result<Vec<PublicEvent>, AppError> {
        self.public_events(&profile.api_base_url).await
    }

    pub async fn fan_referral(&self, profile: &FanProfile) -> Result<ReferralProgress, AppError> {
        self.fan_json::<ReferralProgress, ()>(profile, Method::GET, "me/referral", None)
            .await
    }

    pub async fn fan_interests(
        &self,
        profile: &FanProfile,
    ) -> Result<Vec<FanEventInterest>, AppError> {
        self.fan_json::<Vec<FanEventInterest>, ()>(profile, Method::GET, "me/events?limit=50", None)
            .await
    }

    pub async fn fan_admission_pass(
        &self,
        profile: &FanProfile,
    ) -> Result<Option<AdmissionPass>, AppError> {
        match profile.pass_session_token.as_deref() {
            Some(token) => self
                .pass_json::<AdmissionPass, ()>(
                    &profile.api_base_url,
                    token,
                    Method::GET,
                    "me/pass",
                    None,
                )
                .await
                .map(Some),
            None => Ok(None),
        }
    }

    pub async fn register_interest(
        &self,
        profile: &FanProfile,
        event_slug: &str,
    ) -> Result<serde_json::Value, AppError> {
        let body = serde_json::json!({"campaign_id": null, "source": "mobile_app"});
        self.fan_json(
            profile,
            Method::POST,
            &format!("events/{}/interest", segment(event_slug)?),
            Some(&body),
        )
        .await
    }

    pub async fn claim_pass(
        &self,
        profile: &FanProfile,
        claim_token: &str,
    ) -> Result<(AdmissionPass, String), AppError> {
        let claim_token = bounded_required(claim_token, "token wejściówki", MAX_TOKEN_BYTES)?;
        let response = self
            .http
            .post(endpoint(&profile.api_base_url, "passes/claim")?)
            .header(ACCEPT, "application/json")
            .header("Idempotency-Key", Uuid::new_v4().to_string())
            .json(&serde_json::json!({"token": claim_token}))
            .send()
            .await?;
        let session_token =
            response_cookie(response.headers(), PASS_COOKIE).ok_or_else(|| AppError::Remote {
                status: response.status().as_u16(),
                detail: "Backend nie zwrócił sesji wejściówki".into(),
            })?;
        let body = decode(response).await?;
        Ok((body, session_token))
    }

    pub async fn admission_qr(&self, profile: &FanProfile) -> Result<serde_json::Value, AppError> {
        let token = profile
            .pass_session_token
            .as_deref()
            .ok_or_else(|| AppError::InvalidInput("Najpierw odbierz wejściówkę".into()))?;
        self.pass_json::<serde_json::Value, ()>(
            &profile.api_base_url,
            token,
            Method::GET,
            "me/pass/qr",
            None,
        )
        .await
    }

    pub async fn ticket_wallet(
        &self,
        api_base_url: &str,
        order_id: &str,
        checkout_token: &str,
    ) -> Result<TicketWalletApi, AppError> {
        let order_id = uuid_segment(order_id)?;
        let checkout_token = bounded_required(checkout_token, "token zamówienia", MAX_TOKEN_BYTES)?;
        let response = self
            .http
            .get(endpoint(
                api_base_url,
                &format!("public/ticket-orders/{order_id}/wallet"),
            )?)
            .header(ACCEPT, "application/json")
            .bearer_auth(checkout_token)
            .timeout(WALLET_REQUEST_TIMEOUT)
            .send()
            .await?;
        decode(response).await
    }

    pub async fn request_ticket_delivery(
        &self,
        api_base_url: &str,
        order_id: &str,
        checkout_token: &str,
    ) -> Result<serde_json::Value, AppError> {
        let order_id = uuid_segment(order_id)?;
        let checkout_token = bounded_required(checkout_token, "token zamówienia", MAX_TOKEN_BYTES)?;
        let response = self
            .http
            .post(endpoint(
                api_base_url,
                &format!("public/ticket-orders/{order_id}/delivery-requests"),
            )?)
            .header(ACCEPT, "application/json")
            .bearer_auth(checkout_token)
            .header("Idempotency-Key", Uuid::new_v4().to_string())
            .send()
            .await?;
        decode(response).await
    }

    async fn cached_events(&self, cache_key: &str, max_age: Duration) -> Option<Vec<PublicEvent>> {
        let cache = self.public_cache.read().await;
        cache
            .events
            .get(cache_key)
            .filter(|entry| entry.fetched_at.elapsed() < max_age)
            .map(|entry| entry.value.clone())
    }

    async fn cached_cities(&self, cache_key: &str, max_age: Duration) -> Option<Vec<CitySignal>> {
        let cache = self.public_cache.read().await;
        cache
            .cities
            .get(cache_key)
            .filter(|entry| entry.fetched_at.elapsed() < max_age)
            .map(|entry| entry.value.clone())
    }

    async fn cache_validators(&self, cache_key: &str, events: bool) -> CacheValidators {
        let cache = self.public_cache.read().await;
        let entry = if events {
            cache
                .events
                .get(cache_key)
                .map(|entry| (entry.etag.as_ref(), entry.last_modified.as_ref()))
        } else {
            cache
                .cities
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
        let unix_now = unix_now();
        let mut cache = self.public_cache.write().await;
        if events {
            if let Some(entry) = cache.events.get_mut(cache_key) {
                entry.fetched_at = now;
                entry.stored_at_unix_secs = unix_now;
            }
        } else if let Some(entry) = cache.cities.get_mut(cache_key) {
            entry.fetched_at = now;
            entry.stored_at_unix_secs = unix_now;
        }
    }

    fn persist_public_cache_in_background(&self) {
        let client = self.clone();
        tokio::spawn(async move {
            client.persist_public_cache().await;
        });
    }

    async fn persist_public_cache(&self) {
        let _write = self.cache_write.lock().await;
        let disk_cache = {
            let cache = self.public_cache.read().await;
            DiskPublicCache {
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
            }
        };
        let Ok(payload) = serde_json::to_vec(&disk_cache) else {
            return;
        };
        if payload.len() > MAX_DISK_CACHE_BYTES as usize {
            return;
        }
        let cache_file = self.cache_file.as_ref().clone();
        let _ =
            tokio::task::spawn_blocking(move || write_public_cache(&cache_file, &payload)).await;
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

    async fn auth_json<T, B>(
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
        let mut request = self
            .request(method, endpoint(&profile.api_base_url, path)?)
            .bearer_auth(profile.bearer_token.trim());
        if let Some(body) = body {
            request = request.json(body);
        }
        decode(request.send().await?).await
    }

    async fn fan_json<T, B>(
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
        let mut request = self
            .request(method, endpoint(&profile.api_base_url, path)?)
            .header(
                COOKIE,
                format!("{FAN_COOKIE}={}", profile.fan_session_token),
            );
        if let Some(body) = body {
            request = request.json(body);
        }
        decode(request.send().await?).await
    }

    async fn pass_json<T, B>(
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
        let mut request = self
            .request(method, endpoint(api_base_url, path)?)
            .header(COOKIE, format!("{PASS_COOKIE}={pass_session_token}"));
        if let Some(body) = body {
            request = request.json(body);
        }
        decode(request.send().await?).await
    }

    fn request(&self, method: Method, url: Url) -> RequestBuilder {
        let needs_idempotency_key = !matches!(method, Method::GET | Method::HEAD | Method::OPTIONS);
        let request = self
            .http
            .request(method, url)
            .header(ACCEPT, "application/json");
        if needs_idempotency_key {
            request.header("Idempotency-Key", Uuid::new_v4().to_string())
        } else {
            request
        }
    }
}

fn sanitize_public_events(mut values: Vec<PublicEvent>) -> Vec<PublicEvent> {
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

fn sanitize_public_cities(mut values: Vec<CitySignal>) -> Vec<CitySignal> {
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

fn cache_key(api_base_url: &str) -> Result<String, AppError> {
    let mut url = endpoint(api_base_url, "")?;
    url.set_query(None);
    url.set_fragment(None);
    Ok(url.to_string())
}

fn unix_now() -> u64 {
    match SystemTime::now().duration_since(UNIX_EPOCH) {
        Ok(value) => value.as_secs(),
        Err(error) => {
            eprintln!("[virya:clock] system clock predates Unix epoch: {error}");
            0
        }
    }
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

fn load_public_cache(path: &Path) -> Result<PublicCache, AppError> {
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

fn write_public_cache(path: &Path, payload: &[u8]) -> Result<(), std::io::Error> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let temporary = path.with_extension("tmp");
    std::fs::write(&temporary, payload)?;
    std::fs::rename(temporary, path)
}

fn response_validators(headers: &HeaderMap) -> (Option<String>, Option<String>) {
    let read = |name| {
        headers
            .get(name)
            .and_then(|value| value.to_str().ok())
            .filter(|value| value.len() <= 1024)
            .map(ToOwned::to_owned)
    };
    (read(ETAG), read(LAST_MODIFIED))
}

fn prune_cache<T>(cache: &mut HashMap<String, CacheEntry<T>>, ttl: Duration) {
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

async fn decode<T: DeserializeOwned>(mut response: Response) -> Result<T, AppError> {
    let status = response.status();
    let content_length = response.content_length();
    if content_length.is_some_and(|length| length > MAX_RESPONSE_BYTES) {
        return Err(AppError::Remote {
            status: status.as_u16(),
            detail: "Odpowiedź CrowdRelay jest zbyt duża".into(),
        });
    }
    let initial_capacity = content_length.value_or(0).min(MAX_RESPONSE_BYTES) as usize;
    let mut bytes = Vec::with_capacity(initial_capacity);
    while let Some(chunk) = response.chunk().await? {
        if bytes.len().saturating_add(chunk.len()) > MAX_RESPONSE_BYTES as usize {
            return Err(AppError::Remote {
                status: status.as_u16(),
                detail: "Odpowiedź CrowdRelay jest zbyt duża".into(),
            });
        }
        bytes.extend_from_slice(&chunk);
    }
    if status.is_success() {
        return serde_json::from_slice(if bytes.is_empty() {
            b"null".as_slice()
        } else {
            &bytes
        })
        .map_err(AppError::from);
    }
    let body = match serde_json::from_slice(&bytes) {
        Ok(value) => value,
        Err(error) => {
            eprintln!("[virya:http] error response was not valid JSON: {error}");
            serde_json::Value::Null
        }
    };
    let detail = remote_detail(&body);
    match status {
        StatusCode::UNAUTHORIZED => Err(AppError::Unauthorized),
        StatusCode::FORBIDDEN => Err(AppError::Forbidden),
        StatusCode::CONFLICT => Err(AppError::Conflict(detail)),
        StatusCode::NOT_FOUND => Err(AppError::NotFound),
        StatusCode::UNPROCESSABLE_ENTITY | StatusCode::BAD_REQUEST => {
            Err(AppError::InvalidInput(detail))
        }
        _ => Err(AppError::Remote {
            status: status.as_u16(),
            detail,
        }),
    }
}

fn remote_detail(body: &serde_json::Value) -> String {
    for key in ["detail", "message", "error"] {
        if let Some(value) = body.get(key) {
            if let Some(message) = value.as_str() {
                return message.chars().take(500).collect();
            }
            if let Some(items) = value.as_array() {
                let messages = items
                    .iter()
                    .filter_map(|item| item.get("msg").and_then(serde_json::Value::as_str))
                    .take(3)
                    .collect::<Vec<_>>()
                    .join(", ");
                if !messages.is_empty() {
                    return messages.chars().take(500).collect();
                }
            }
        }
    }
    "CrowdRelay odrzucił operację".into()
}

fn endpoint(base: &str, path: &str) -> Result<Url, AppError> {
    let mut base = Url::parse(base.trim())?;
    let allowed_scheme =
        base.scheme() == "https" || (cfg!(debug_assertions) && base.scheme() == "http");
    if !allowed_scheme {
        return Err(AppError::InvalidInput(
            "Produkcyjny API URL musi używać HTTPS".into(),
        ));
    }
    if base.host_str().is_none()
        || base.username() != ""
        || base.password().is_some()
        || base.query().is_some()
        || base.fragment().is_some()
    {
        return Err(AppError::InvalidInput(
            "Nieprawidłowy bazowy URL API".into(),
        ));
    }
    if !base.path().ends_with('/') {
        base.set_path(&format!("{}/", base.path()));
    }
    base.join(path).map_err(AppError::from)
}

fn segment(value: &str) -> Result<String, AppError> {
    let value = value.trim();
    if value.is_empty()
        || value.len() > 200
        || !value
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '-' | '_'))
    {
        return Err(AppError::InvalidInput("Nieprawidłowy identyfikator".into()));
    }
    Ok(value.to_owned())
}

fn uuid_segment(value: &str) -> Result<String, AppError> {
    Uuid::parse_str(value.trim())
        .map(|value| value.to_string())
        .map_err(|_| AppError::InvalidInput("Nieprawidłowy identyfikator zamówienia".into()))
}

fn normalize_scanned_code(value: &str) -> Result<String, AppError> {
    let trimmed = value.trim();
    if trimmed.is_empty() || trimmed.len() > MAX_TOKEN_BYTES {
        return Err(AppError::InvalidInput("Nieprawidłowy kod QR".into()));
    }
    if let Ok(url) = Url::parse(trimmed) {
        if let Some((_, token)) = url.query_pairs().find(|(name, _)| name == "token") {
            return bounded_required(token.as_ref(), "token QR", MAX_TOKEN_BYTES)
                .map(ToOwned::to_owned);
        }
        if let Some(fragment) = url.fragment() {
            if let Some((_, token)) =
                url::form_urlencoded::parse(fragment.as_bytes()).find(|(name, _)| name == "token")
            {
                return bounded_required(token.as_ref(), "token QR", MAX_TOKEN_BYTES)
                    .map(ToOwned::to_owned);
            }
        }
    }
    Ok(trimmed.to_owned())
}

fn response_cookie(headers: &HeaderMap, expected_name: &str) -> Option<String> {
    headers
        .get_all(SET_COOKIE)
        .iter()
        .filter_map(|header| header.to_str().ok())
        .filter_map(|header| header.split(';').next())
        .filter_map(|pair| pair.trim().split_once('='))
        .find_map(|(name, value)| {
            (name == expected_name && !value.is_empty() && value.len() <= MAX_TOKEN_BYTES)
                .then(|| value.to_owned())
        })
}

fn bounded_required<'a>(value: &'a str, label: &str, max: usize) -> Result<&'a str, AppError> {
    let value = value.trim();
    if value.is_empty() || value.len() > max {
        Err(AppError::InvalidInput(format!("Nieprawidłowy {label}")))
    } else {
        Ok(value)
    }
}

fn normalized_optional(value: &Option<String>) -> Option<String> {
    value
        .as_ref()
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
}

fn require_owner(profile: &OperatorProfile) -> Result<(), AppError> {
    if profile.role == OperatorRole::Owner {
        Ok(())
    } else {
        Err(AppError::Forbidden)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_value<T, E>(result: Result<T, E>) -> T
    where
        E: std::fmt::Debug,
    {
        match result {
            Ok(value) => value,
            Err(error) => panic!("test setup failed: {error:?}"),
        }
    }

    #[test]
    fn extracts_fragment_token() {
        assert_eq!(
            test_value(normalize_scanned_code(
                "https://virya.music/win#token=v1.abc"
            )),
            "v1.abc"
        );
    }

    #[test]
    fn extracts_token_after_another_query_parameter() {
        assert_eq!(
            test_value(normalize_scanned_code(
                "https://virya.music/win?source=app&token=t1.abc"
            )),
            "t1.abc"
        );
    }

    #[test]
    fn leaves_manual_reference() {
        assert_eq!(test_value(normalize_scanned_code(" VRY-ABCD ")), "VRY-ABCD");
    }

    #[test]
    fn rejects_non_uuid_order_id() {
        assert!(uuid_segment("order-1").is_err());
    }

    #[test]
    fn rejects_credentialed_or_fragmented_api_base() {
        assert!(endpoint("https://user@example.com/v1", "events").is_err());
        assert!(endpoint("https://example.com/v1#fragment", "events").is_err());
    }

    #[test]
    fn cache_key_normalizes_trailing_slash() {
        assert_eq!(
            test_value(cache_key("https://signal-api.virya.music/v1")),
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
        let mut headers = HeaderMap::new();
        headers.insert(ETAG, test_value("\"events-v2\"".parse()));
        headers.insert(
            LAST_MODIFIED,
            test_value("Sat, 01 Aug 2026 07:00:00 GMT".parse()),
        );

        let (etag, last_modified) = response_validators(&headers);
        assert_eq!(etag.as_deref(), Some("\"events-v2\""));
        assert_eq!(
            last_modified.as_deref(),
            Some("Sat, 01 Aug 2026 07:00:00 GMT")
        );

        headers.insert(ETAG, test_value("x".repeat(1025).parse()));
        assert!(response_validators(&headers).0.is_none());
    }
}
