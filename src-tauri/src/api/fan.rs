use reqwest::{Method, header::ACCEPT};
use serde::Deserialize;
use std::{
    collections::hash_map::DefaultHasher,
    hash::{Hash, Hasher},
    time::Instant,
};
use uuid::Uuid;

use crate::{
    AppError,
    models::{
        AdmissionPass, FanAuthResult, FanConfirmationInput, FanEventInterest, FanHomeData,
        FanLocationState, FanProfile, FanPushPreferences, FanPushPreferencesUpdate, FanSignupInput,
        PublicEventsResult, ReferralProgress,
    },
};

use super::{
    cache::{self, CacheEntry, restored_fetched_at},
    client::{FAN_COOKIE, FAN_HOME_CACHE_TTL, FAN_HOME_STALE_TTL},
    http::{
        MAX_TOKEN_BYTES, bounded_required, decode, endpoint, normalized_optional, response_cookie,
        segment,
    },
};

#[derive(Deserialize)]
struct FanSignupApiResponse {
    #[serde(default)]
    email_kind: Option<String>,
    #[serde(default)]
    email_queued: Option<bool>,
    #[serde(default)]
    retry_after_seconds: Option<u32>,
}

#[derive(Deserialize)]
struct FanConfirmationApiResponse {
    fan_session_token: Option<String>,
    email: String,
    #[serde(default)]
    display_name: Option<String>,
}

#[derive(Deserialize)]
struct FanAccountDeletionApiResponse {
    deleted: bool,
}

fn fan_home_key(profile: &FanProfile) -> String {
    let mut hasher = DefaultHasher::new();
    profile.api_base_url.hash(&mut hasher);
    profile.fan_session_token.hash(&mut hasher);
    format!("fan-home-{:016x}", hasher.finish())
}

impl super::CrowdRelayClient {
    pub async fn fan_home(&self, profile: &FanProfile) -> Result<FanHomeData, AppError> {
        self.require_capability(&profile.api_base_url, "signal_fan_context_v1")
            .await?;
        let key = fan_home_key(profile);
        if let Some(home) = self.cached_fan_home(&key, FAN_HOME_CACHE_TTL).await {
            return Ok(home);
        }
        let _fetch = self.fan_home_fetch.lock().await;
        if let Some(home) = self.cached_fan_home(&key, FAN_HOME_CACHE_TTL).await {
            return Ok(home);
        }
        let stale = self.cached_fan_home(&key, FAN_HOME_STALE_TTL).await;
        match self
            .fan_json::<FanHomeData, ()>(profile, Method::GET, "me/home", None)
            .await
        {
            Ok(mut home) => {
                home.stale = false;
                let mut cache_map = self.fan_home_cache.write().await;
                cache::prune_cache(&mut cache_map, FAN_HOME_STALE_TTL);
                cache_map.insert(
                    key,
                    CacheEntry {
                        value: home.clone(),
                        fetched_at: Instant::now(),
                        stored_at_unix_secs: cache::unix_now(),
                        etag: None,
                        last_modified: None,
                    },
                );
                Ok(home)
            }
            Err(error) if super::retry::is_transient_failure(&error) => match stale {
                Some(mut home) => {
                    home.stale = true;
                    Ok(home)
                }
                None => Err(error),
            },
            Err(error) => Err(error),
        }
    }

    pub async fn seed_fan_home_snapshot(
        &self,
        profile: &FanProfile,
        mut home: FanHomeData,
        stored_at_unix_secs: u64,
    ) -> Option<FanHomeData> {
        let now_unix = cache::unix_now();
        let future_skew = stored_at_unix_secs.saturating_sub(now_unix);
        if future_skew > 5 * 60 {
            return None;
        }
        let age = std::time::Duration::from_secs(now_unix.saturating_sub(stored_at_unix_secs));
        if age > FAN_HOME_STALE_TTL || !home.has_supported_schema() {
            return None;
        }
        home.stale = true;
        let effective_age = age.max(FAN_HOME_CACHE_TTL);
        let fetched_at = restored_fetched_at(effective_age);
        let key = fan_home_key(profile);
        let mut cache_map = self.fan_home_cache.write().await;
        cache::prune_cache(&mut cache_map, FAN_HOME_STALE_TTL);
        cache_map.insert(
            key,
            CacheEntry {
                value: home.clone(),
                fetched_at,
                stored_at_unix_secs,
                etag: None,
                last_modified: None,
            },
        );
        Some(home)
    }

    async fn cached_fan_home(
        &self,
        key: &str,
        max_age: std::time::Duration,
    ) -> Option<FanHomeData> {
        self.fan_home_cache
            .read()
            .await
            .get(key)
            .filter(|entry| entry.fetched_at.elapsed() < max_age)
            .map(|entry| entry.value.clone())
    }

    pub async fn fan_delete_account(&self, profile: &FanProfile) -> Result<(), AppError> {
        self.require_capability(&profile.api_base_url, "fan_account_deletion_v1")
            .await?;
        let response = self
            .fan_json::<FanAccountDeletionApiResponse, ()>(
                profile,
                Method::DELETE,
                "me/account",
                None,
            )
            .await?;
        if !response.deleted {
            return Err(AppError::Conflict(
                "fan_account_deletion_not_confirmed".to_owned(),
            ));
        }
        self.invalidate_fan_home(profile).await;
        Ok(())
    }

    pub(super) async fn invalidate_fan_home(&self, profile: &FanProfile) {
        let key = fan_home_key(profile);
        self.fan_home_cache.write().await.remove(&key);
    }

    pub async fn fan_events(&self, profile: &FanProfile) -> Result<PublicEventsResult, AppError> {
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
        let claim_token = bounded_required(
            claim_token,
            crate::i18n::tr("native_admission_token_label"),
            MAX_TOKEN_BYTES,
        )?;
        let response = self
            .http
            .post(endpoint(&profile.api_base_url, "passes/claim")?)
            .header(ACCEPT, "application/json")
            .header("Idempotency-Key", Uuid::new_v4().to_string())
            .json(&serde_json::json!({"token": claim_token}))
            .send()
            .await?;
        let session_token = response_cookie(response.headers(), super::client::PASS_COOKIE)
            .ok_or_else(|| AppError::Remote {
                status: response.status().as_u16(),
                detail: crate::i18n::tr("native_admission_session_missing").into(),
            })?;
        let body = decode(response).await?;
        Ok((body, session_token))
    }

    pub async fn admission_qr(&self, profile: &FanProfile) -> Result<serde_json::Value, AppError> {
        let token = profile.pass_session_token.as_deref().ok_or_else(|| {
            AppError::InvalidInput(crate::i18n::tr("native_claim_pass_first").into())
        })?;
        self.pass_json::<serde_json::Value, ()>(
            &profile.api_base_url,
            token,
            Method::GET,
            "me/pass/qr",
            None,
        )
        .await
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
        let body: FanSignupApiResponse = decode(response).await?;
        Ok((
            FanAuthResult {
                session_created: token.is_some(),
                email_kind: body.email_kind,
                email_queued: body.email_queued,
                retry_after_seconds: body.retry_after_seconds,
            },
            token,
        ))
    }

    pub async fn fan_request_access(
        &self,
        api_base_url: &str,
        email: &str,
        locale: &str,
    ) -> Result<serde_json::Value, AppError> {
        let body = serde_json::json!({
            "email": email,
            "locale": locale,
        });
        let response = self
            .http
            .post(endpoint(api_base_url, "fans/access")?)
            .header(ACCEPT, "application/json")
            .header("Idempotency-Key", Uuid::new_v4().to_string())
            .json(&body)
            .send()
            .await?;
        decode(response).await
    }

    pub async fn fan_confirm(
        &self,
        input: &FanConfirmationInput,
    ) -> Result<(FanAuthResult, String, String, Option<String>), AppError> {
        let response = self
            .http
            .post(endpoint(&input.api_base_url, "fans/confirm")?)
            .header(ACCEPT, "application/json")
            .header("Idempotency-Key", Uuid::new_v4().to_string())
            .json(&serde_json::json!({"token": input.token.trim()}))
            .send()
            .await?;
        let cookie_token = response_cookie(response.headers(), FAN_COOKIE);
        let body: FanConfirmationApiResponse = match decode(response).await {
            Ok(body) => body,
            Err(AppError::Conflict(_)) => {
                return Err(AppError::Conflict(
                    crate::i18n::tr("native_code_already_used").into(),
                ));
            }
            Err(AppError::NotFound) => {
                return Err(AppError::InvalidInput(
                    crate::i18n::tr("native_code_invalid_or_expired").into(),
                ));
            }
            Err(error) => return Err(error),
        };
        let session_token = body
            .fan_session_token
            .or(cookie_token)
            .filter(|value| !value.is_empty() && value.len() <= MAX_TOKEN_BYTES)
            .ok_or_else(|| AppError::Remote {
                status: 200,
                detail: crate::i18n::tr("native_fan_session_missing").into(),
            })?;
        let canonical_email = bounded_required(&body.email, "fan email", 320)?.to_owned();
        let canonical_name = normalized_optional(&body.display_name);
        Ok((
            FanAuthResult {
                session_created: true,
                email_kind: None,
                email_queued: None,
                retry_after_seconds: None,
            },
            session_token,
            canonical_email,
            canonical_name,
        ))
    }
}

#[derive(Clone, Debug, Deserialize)]
pub struct FanPushConfigApi {
    pub enabled: bool,
    pub android_fcm: bool,
}

#[derive(Clone, Debug, Deserialize)]
pub struct FanPushMutationApi {
    pub registered: bool,
}

#[derive(Clone, Debug, Deserialize)]
struct FanPushPreferencesApi {
    shows: bool,
    releases: bool,
    community: bool,
    merch: bool,
    quiet_hours_enabled: bool,
    quiet_start: String,
    quiet_end: String,
    quiet_timezone: String,
}

impl From<FanPushPreferencesApi> for FanPushPreferences {
    fn from(value: FanPushPreferencesApi) -> Self {
        Self {
            shows: value.shows,
            releases: value.releases,
            community: value.community,
            merch: value.merch,
            quiet_hours_enabled: value.quiet_hours_enabled,
            quiet_start: value.quiet_start,
            quiet_end: value.quiet_end,
            quiet_timezone: value.quiet_timezone,
        }
    }
}

impl super::CrowdRelayClient {
    pub async fn fan_push_config(
        &self,
        profile: &FanProfile,
    ) -> Result<FanPushConfigApi, AppError> {
        self.require_capability(&profile.api_base_url, "fan_push_delivery_v1")
            .await?;
        let response = self
            .http
            .get(endpoint(&profile.api_base_url, "public/push/config")?)
            .header(ACCEPT, "application/json")
            .send()
            .await?;
        decode(response).await
    }

    pub async fn fan_register_android_push(
        &self,
        profile: &FanProfile,
        installation_id: &str,
        fcm_token: &str,
    ) -> Result<FanPushMutationApi, AppError> {
        self.require_capability(&profile.api_base_url, "fan_push_delivery_v1")
            .await?;
        let body = serde_json::json!({
            "installation_id": installation_id,
            "transport": "android_fcm",
            "endpoint": fcm_token,
            "p256dh": null,
            "auth": null,
        });
        self.fan_json(profile, Method::POST, "me/push/endpoints", Some(&body))
            .await
    }

    pub async fn fan_push_preferences(
        &self,
        profile: &FanProfile,
    ) -> Result<FanPushPreferences, AppError> {
        let wire = self
            .fan_json::<FanPushPreferencesApi, ()>(
                profile,
                Method::GET,
                "me/push/preferences",
                None,
            )
            .await?;
        Ok(wire.into())
    }

    pub async fn fan_update_push_preferences(
        &self,
        profile: &FanProfile,
        value: &FanPushPreferencesUpdate,
    ) -> Result<FanPushPreferences, AppError> {
        let body = serde_json::json!({
            "shows": value.shows,
            "releases": value.releases,
            "community": value.community,
            "merch": value.merch,
            "quiet_hours_enabled": value.quiet_hours_enabled,
            "quiet_start": value.quiet_start,
            "quiet_end": value.quiet_end,
        });
        let wire: FanPushPreferencesApi = self
            .fan_json(profile, Method::POST, "me/push/preferences", Some(&body))
            .await?;
        Ok(wire.into())
    }

    /// Sets the fan's city and nearby-show preference against a proved session.
    ///
    /// Signup is the only other writer and refuses to touch an address that is
    /// already active, so before this existed a fan who bought a ticket or
    /// moved city could never establish a location -- and nearby delivery is
    /// keyed on one. The reply reports whether targeting can actually work, so
    /// the app can stop implying shows are on their way when the chosen city
    /// has no coordinates yet.
    pub async fn fan_set_location(
        &self,
        profile: &FanProfile,
        city_slug: &str,
        nearby_gigs_enabled: bool,
        radius_km: u16,
    ) -> Result<FanLocationState, AppError> {
        let body = serde_json::json!({
            "city_slug": city_slug,
            "nearby_gigs_enabled": nearby_gigs_enabled,
            "radius_km": radius_km,
        });
        self.fan_json(profile, Method::POST, "me/location", Some(&body))
            .await
    }

    pub async fn fan_disable_android_push(
        &self,
        profile: &FanProfile,
        installation_id: &str,
    ) -> Result<FanPushMutationApi, AppError> {
        self.require_capability(&profile.api_base_url, "fan_push_delivery_v1")
            .await?;
        let body = serde_json::json!({
            "installation_id": installation_id,
            "transport": "android_fcm",
        });
        self.fan_json(
            profile,
            Method::POST,
            "me/push/endpoints/disable",
            Some(&body),
        )
        .await
    }
}
