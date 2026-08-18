use reqwest::{Method, header::ACCEPT};
use serde::Serialize;
use url::Url;

use crate::{
    AppError,
    models::{
        BeaconEngagementResult, BeaconExchangeApi, BeaconHomeData, BeaconMutationResult,
        BeaconPreferences, BeaconPressRequestsData, BeaconPressRoomData, BeaconProfile,
        BeaconReleasesData, SignalNewsFeed,
    },
};

use super::{
    client::CrowdRelayClient,
    fan::{FanPushConfigApi, FanPushMutationApi},
    http::{decode, endpoint},
};

// A session is attributed to the client that exchanged the capability. Only the
// three canonical kinds are accepted upstream, so a desktop or dev build reports
// itself as `web` rather than claiming to be a phone.
pub(crate) const CLIENT_KIND: &str = if cfg!(target_os = "android") {
    "android"
} else if cfg!(target_os = "ios") {
    "ios"
} else {
    "web"
};

const NEWS_URL: &str = "https://virya.music/news/feed.json";
const MAX_NEWS_ITEMS: usize = 20;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct BeaconPreferencesInput<'a> {
    pub radius_km: i32,
    pub locale: &'a str,
    pub topics: &'a [String],
    pub nearby_gigs_enabled: bool,
}

impl CrowdRelayClient {
    pub async fn beacon_exchange(
        &self,
        api_base_url: &str,
        invite_token: &str,
        radius_km: i32,
        locale: &str,
        topics: &[String],
    ) -> Result<BeaconExchangeApi, AppError> {
        self.require_capability(api_base_url, "beacon_native_signal_v1")
            .await?;
        let body = serde_json::json!({
            "inviteToken": invite_token,
            "radiusKm": radius_km,
            "locale": locale,
            "topics": topics,
            "clientKind": CLIENT_KIND,
        });
        let response = self
            .http
            .post(endpoint(api_base_url, "beacon/invitations/exchange")?)
            .header(ACCEPT, "application/json")
            .json(&body)
            .send()
            .await?;
        decode(response).await
    }

    pub async fn beacon_me(&self, profile: &BeaconProfile) -> Result<BeaconHomeData, AppError> {
        self.beacon_json::<BeaconHomeData, ()>(profile, Method::GET, "beacon/me", None)
            .await
    }

    pub async fn beacon_preferences(
        &self,
        profile: &BeaconProfile,
        input: &BeaconPreferencesInput<'_>,
    ) -> Result<BeaconPreferences, AppError> {
        self.beacon_json(profile, Method::POST, "beacon/me/preferences", Some(input))
            .await
    }

    pub async fn beacon_press_room(
        &self,
        profile: &BeaconProfile,
        event_id: Option<&str>,
    ) -> Result<BeaconPressRoomData, AppError> {
        let path = match event_id {
            Some(id) => format!(
                "beacon/me/press-room?event_id={}",
                url::form_urlencoded::byte_serialize(id.as_bytes()).collect::<String>()
            ),
            None => "beacon/me/press-room".to_owned(),
        };
        self.beacon_json::<BeaconPressRoomData, ()>(profile, Method::GET, &path, None)
            .await
    }

    pub async fn beacon_press_requests(
        &self,
        profile: &BeaconProfile,
    ) -> Result<BeaconPressRequestsData, AppError> {
        self.beacon_json::<BeaconPressRequestsData, ()>(
            profile,
            Method::GET,
            "beacon/me/press-requests",
            None,
        )
        .await
    }

    pub async fn beacon_create_press_request(
        &self,
        profile: &BeaconProfile,
        event_id: Option<&str>,
        request_kind: &str,
        details: Option<&str>,
    ) -> Result<BeaconMutationResult, AppError> {
        let body = serde_json::json!({"eventId": event_id, "requestKind": request_kind, "details": details});
        self.beacon_json(
            profile,
            Method::POST,
            "beacon/me/press-requests",
            Some(&body),
        )
        .await
    }

    pub async fn beacon_engagement(
        &self,
        profile: &BeaconProfile,
        event_id: &str,
        action: &str,
        help_kind: Option<&str>,
        help_details: Option<&str>,
    ) -> Result<BeaconEngagementResult, AppError> {
        let body = serde_json::json!({"action": action, "helpKind": help_kind, "helpDetails": help_details});
        self.beacon_json(
            profile,
            Method::POST,
            &format!("beacon/me/events/{event_id}/engagement"),
            Some(&body),
        )
        .await
    }

    pub async fn beacon_coverage(
        &self,
        profile: &BeaconProfile,
        event_id: &str,
        coverage_kind: &str,
        url: &str,
        title: Option<&str>,
    ) -> Result<BeaconMutationResult, AppError> {
        let body = serde_json::json!({"coverageKind": coverage_kind, "url": url, "title": title});
        self.beacon_json(
            profile,
            Method::POST,
            &format!("beacon/me/events/{event_id}/coverage"),
            Some(&body),
        )
        .await
    }

    pub async fn beacon_releases(
        &self,
        profile: &BeaconProfile,
    ) -> Result<BeaconReleasesData, AppError> {
        self.beacon_json::<BeaconReleasesData, ()>(profile, Method::GET, "beacon/me/releases", None)
            .await
    }

    pub async fn beacon_confirm_release(
        &self,
        profile: &BeaconProfile,
        campaign_id: &str,
        recipient_name: &str,
        recipient_phone: &str,
        parcel_locker_code: &str,
    ) -> Result<BeaconMutationResult, AppError> {
        let body = serde_json::json!({
            "recipientName": recipient_name,
            "recipientPhone": recipient_phone,
            "parcelLockerCode": parcel_locker_code,
        });
        self.beacon_json(
            profile,
            Method::POST,
            &format!("beacon/me/releases/{campaign_id}/delivery"),
            Some(&body),
        )
        .await
    }

    pub async fn beacon_decline_release(
        &self,
        profile: &BeaconProfile,
        campaign_id: &str,
    ) -> Result<BeaconMutationResult, AppError> {
        self.beacon_json(
            profile,
            Method::POST,
            &format!("beacon/me/releases/{campaign_id}/decline"),
            Some(&serde_json::json!({})),
        )
        .await
    }

    pub async fn beacon_logout(&self, profile: &BeaconProfile) -> Result<(), AppError> {
        let _: serde_json::Value = self
            .beacon_json(
                profile,
                Method::POST,
                "beacon/me/logout",
                Some(&serde_json::json!({})),
            )
            .await?;
        Ok(())
    }

    pub async fn beacon_leave(
        &self,
        profile: &BeaconProfile,
        do_not_contact: bool,
    ) -> Result<(), AppError> {
        let _: serde_json::Value = self
            .beacon_json(
                profile,
                Method::POST,
                "beacon/me/leave",
                Some(&serde_json::json!({"doNotContact": do_not_contact})),
            )
            .await?;
        Ok(())
    }

    pub async fn beacon_push_config(
        &self,
        profile: &BeaconProfile,
    ) -> Result<FanPushConfigApi, AppError> {
        let response = self
            .http
            .get(endpoint(&profile.api_base_url, "public/push/config")?)
            .header(ACCEPT, "application/json")
            .send()
            .await?;
        decode(response).await
    }

    pub async fn beacon_register_android_push(
        &self,
        profile: &BeaconProfile,
        installation_id: &str,
        fcm_token: &str,
    ) -> Result<FanPushMutationApi, AppError> {
        let body = serde_json::json!({"installation_id":installation_id,"transport":"android_fcm","endpoint":fcm_token,"p256dh":null,"auth":null});
        self.beacon_json(profile, Method::POST, "beacon/push/endpoints", Some(&body))
            .await
    }

    pub async fn beacon_disable_android_push(
        &self,
        profile: &BeaconProfile,
        installation_id: &str,
    ) -> Result<FanPushMutationApi, AppError> {
        let body = serde_json::json!({"installation_id":installation_id,"transport":"android_fcm"});
        self.beacon_json(
            profile,
            Method::POST,
            "beacon/push/endpoints/disable",
            Some(&body),
        )
        .await
    }

    pub async fn signal_news(&self) -> Result<SignalNewsFeed, AppError> {
        let response = self
            .site_http
            .get(Url::parse(NEWS_URL)?)
            .header(ACCEPT, "application/json")
            .send()
            .await?;
        let feed: SignalNewsFeed = decode(response).await?;
        if feed.items.len() > MAX_NEWS_ITEMS {
            return Err(AppError::InvalidInput("news_feed_too_large".to_owned()));
        }
        Ok(feed)
    }
}
