use std::{collections::HashSet, time::Duration};

use reqwest::header::{ACCEPT, ORIGIN};
use serde::{Deserialize, Serialize};
use time::{OffsetDateTime, format_description::well_known::Rfc3339};
use url::Url;
use uuid::Uuid;

use crate::{AppError, models::FanProfile, validation::clean_optional};

use super::{
    CrowdRelayClient,
    http::{MAX_TOKEN_BYTES, bounded_required, decode, endpoint, segment},
};

const TICKET_CHECKOUT_URL: &str = "https://virya.music/api/ticket-checkout";
const VIRYA_SITE_ORIGIN: &str = "https://virya.music";
const CHECKOUT_TIMEOUT: Duration = Duration::from_secs(20);
const TICKET_SALE_TIMEOUT: Duration = Duration::from_secs(8);
const MAX_CHECKOUT_LINES: usize = 10;
const MAX_CHECKOUT_QUANTITY: u32 = 100;
const MAX_TICKET_TYPES: usize = 32;

fn invalid_remote(detail: &str) -> AppError {
    AppError::Remote {
        status: 502,
        detail: detail.to_owned(),
    }
}

fn valid_slug(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_'))
}

fn valid_text(value: &str, max: usize) -> bool {
    !value.trim().is_empty() && value.chars().count() <= max && !value.chars().any(char::is_control)
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct TicketTypeOffer {
    pub id: String,
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    pub price_gross_minor: i64,
    pub capacity: Option<i32>,
    pub sold: i32,
    pub reserved: i32,
    pub available: i32,
    pub sort_order: i32,
    pub active: bool,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct TicketSaleOffer {
    pub event_id: String,
    pub event_slug: String,
    pub event_title: String,
    pub event_status: String,
    pub venue: Option<String>,
    pub timezone: String,
    pub starts_at: String,
    pub currency: String,
    pub vat_rate_basis_points: i32,
    pub capacity: i32,
    pub sold: i32,
    pub reserved: i32,
    pub available: i32,
    pub max_per_order: i32,
    pub sales_open_at: String,
    pub sales_close_at: String,
    pub active: bool,
    pub sales_state: String,
    pub ticket_types: Vec<TicketTypeOffer>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TicketCheckoutInput {
    pub event_slug: String,
    pub buyer_name: Option<String>,
    pub items: Vec<TicketCheckoutItemInput>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TicketCheckoutItemInput {
    pub ticket_type_slug: String,
    pub quantity: u32,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct TicketCheckoutStart {
    pub url: String,
    pub order_id: String,
    pub order_reference: String,
    pub expires_at: String,
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct SiteTicketCheckoutRequest<'a> {
    event_slug: &'a str,
    buyer_email: &'a str,
    buyer_name: Option<&'a str>,
    lang: &'static str,
    checkout_request_id: String,
    invoice_requested: bool,
    items: &'a [TicketCheckoutItemInput],
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SiteTicketCheckoutResponse {
    pub(crate) url: String,
    pub(crate) order_id: String,
    pub(crate) order_reference: String,
    pub(crate) checkout_token: String,
    pub(crate) expires_at: String,
}

impl TicketSaleOffer {
    fn validate(self, expected_event_slug: &str) -> Result<Self, AppError> {
        if self.event_slug != expected_event_slug
            || !valid_slug(&self.event_slug)
            || Uuid::parse_str(self.event_id.trim()).is_err()
            || !valid_text(&self.event_title, 300)
            || !valid_text(&self.event_status, 64)
            || !valid_text(&self.timezone, 100)
            || self.venue.as_ref().is_some_and(|value| {
                value.chars().count() > 240 || value.chars().any(char::is_control)
            })
            || !matches!(
                self.sales_state.as_str(),
                "open" | "upcoming" | "closed" | "sold_out" | "inactive" | "event_unavailable"
            )
            || !(0..=10_000).contains(&self.vat_rate_basis_points)
            || self.currency.len() != 3
            || !self.currency.bytes().all(|byte| byte.is_ascii_alphabetic())
            || self.capacity < 0
            || self.sold < 0
            || self.reserved < 0
            || self.available < 0
            || !(0..=MAX_CHECKOUT_QUANTITY as i32).contains(&self.max_per_order)
            || self.ticket_types.len() > MAX_TICKET_TYPES
            || OffsetDateTime::parse(self.starts_at.trim(), &Rfc3339).is_err()
            || OffsetDateTime::parse(self.sales_open_at.trim(), &Rfc3339).is_err()
            || OffsetDateTime::parse(self.sales_close_at.trim(), &Rfc3339).is_err()
        {
            return Err(invalid_remote(
                "Serwer zwrócił nieprawidłową ofertę biletową",
            ));
        }

        let mut slugs = HashSet::with_capacity(self.ticket_types.len());
        for ticket_type in &self.ticket_types {
            if Uuid::parse_str(ticket_type.id.trim()).is_err()
                || !valid_slug(&ticket_type.slug)
                || !slugs.insert(ticket_type.slug.as_str())
                || !valid_text(&ticket_type.name, 200)
                || ticket_type.description.as_ref().is_some_and(|value| {
                    value.chars().count() > 1_000 || value.chars().any(char::is_control)
                })
                || ticket_type.price_gross_minor < 0
                || ticket_type.capacity.is_some_and(|capacity| capacity < 0)
                || ticket_type.sold < 0
                || ticket_type.reserved < 0
                || ticket_type.available < 0
            {
                return Err(invalid_remote("Serwer zwrócił nieprawidłową pulę biletów"));
            }
        }
        Ok(self)
    }
}

impl TicketCheckoutInput {
    pub(crate) fn normalize(&mut self) -> Result<(), AppError> {
        self.event_slug = segment(&self.event_slug)?;
        if self.event_slug.len() > 128 {
            return Err(AppError::InvalidInput(
                "Nieprawidłowy identyfikator koncertu".into(),
            ));
        }
        self.buyer_name = clean_optional(self.buyer_name.take());
        if self
            .buyer_name
            .as_ref()
            .is_some_and(|name| name.chars().count() > 160 || name.chars().any(char::is_control))
        {
            return Err(AppError::InvalidInput(
                "Imię i nazwisko jest zbyt długie".into(),
            ));
        }
        if self.items.is_empty() || self.items.len() > MAX_CHECKOUT_LINES {
            return Err(AppError::InvalidInput("Wybierz bilety".into()));
        }

        let mut seen = HashSet::with_capacity(self.items.len());
        let mut total = 0_u32;
        for item in &mut self.items {
            item.ticket_type_slug = segment(&item.ticket_type_slug)?;
            if item.ticket_type_slug.len() > 128
                || item.quantity == 0
                || item.quantity > MAX_CHECKOUT_QUANTITY
                || !seen.insert(item.ticket_type_slug.clone())
            {
                return Err(AppError::InvalidInput("Nieprawidłowy wybór biletów".into()));
            }
            total = total.saturating_add(item.quantity);
            if total > MAX_CHECKOUT_QUANTITY {
                return Err(AppError::InvalidInput("Wybrano zbyt wiele biletów".into()));
            }
        }
        Ok(())
    }
}

impl TicketCheckoutStart {
    pub(crate) fn from_site(value: &SiteTicketCheckoutResponse) -> Result<Self, AppError> {
        let url = Url::parse(value.url.trim())?;
        if url.scheme() != "https"
            || url.host_str() != Some("checkout.stripe.com")
            || url.username() != ""
            || url.password().is_some()
        {
            return Err(invalid_remote(
                "Serwer zwrócił nieprawidłowy adres płatności",
            ));
        }
        let order_id = Uuid::parse_str(value.order_id.trim())
            .map(|id| id.to_string())
            .map_err(|_| invalid_remote("Serwer zwrócił nieprawidłowe zamówienie"))?;
        let checkout_token = bounded_required(
            value.checkout_token.as_str(),
            "token zamówienia",
            MAX_TOKEN_BYTES,
        )?;
        if checkout_token.len() != 64
            || !checkout_token.bytes().all(|byte| byte.is_ascii_hexdigit())
            || !valid_text(&value.order_reference, 200)
            || value.expires_at.len() > 80
            || OffsetDateTime::parse(value.expires_at.trim(), &Rfc3339).is_err()
        {
            return Err(invalid_remote("Serwer zwrócił niepełne dane zamówienia"));
        }
        Ok(Self {
            url: url.to_string(),
            order_id,
            order_reference: value.order_reference.trim().to_owned(),
            expires_at: value.expires_at.trim().to_owned(),
        })
    }
}

impl CrowdRelayClient {
    pub async fn public_ticket_sale(
        &self,
        api_base_url: &str,
        event_slug: &str,
    ) -> Result<TicketSaleOffer, AppError> {
        let event_slug = segment(event_slug)?;
        let response = self
            .http
            .get(endpoint(
                api_base_url,
                &format!("public/events/{event_slug}/tickets"),
            )?)
            .header(ACCEPT, "application/json")
            .timeout(TICKET_SALE_TIMEOUT)
            .send()
            .await?;
        let offer: TicketSaleOffer = decode(response).await?;
        offer.validate(&event_slug)
    }

    pub async fn start_ticket_checkout(
        &self,
        profile: &FanProfile,
        input: &TicketCheckoutInput,
    ) -> Result<SiteTicketCheckoutResponse, AppError> {
        let response = self
            .site_http
            .post(Url::parse(TICKET_CHECKOUT_URL)?)
            .header(ACCEPT, "application/json")
            .header(ORIGIN, VIRYA_SITE_ORIGIN)
            .json(&SiteTicketCheckoutRequest {
                event_slug: input.event_slug.as_str(),
                buyer_email: profile.email.trim(),
                buyer_name: input.buyer_name.as_deref(),
                lang: "pl",
                checkout_request_id: Uuid::new_v4().to_string(),
                invoice_requested: false,
                items: &input.items,
            })
            .timeout(CHECKOUT_TIMEOUT)
            .send()
            .await?;
        decode(response).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn checkout_input_rejects_duplicate_lines() {
        let mut input = TicketCheckoutInput {
            event_slug: "show-1".into(),
            buyer_name: None,
            items: vec![
                TicketCheckoutItemInput {
                    ticket_type_slug: "regular".into(),
                    quantity: 1,
                },
                TicketCheckoutItemInput {
                    ticket_type_slug: "regular".into(),
                    quantity: 1,
                },
            ],
        };
        assert!(input.normalize().is_err());
    }

    fn valid_ticket_sale() -> TicketSaleOffer {
        TicketSaleOffer {
            event_id: Uuid::new_v4().to_string(),
            event_slug: "show-1".into(),
            event_title: "Virya live".into(),
            event_status: "published".into(),
            venue: Some("Art Space".into()),
            timezone: "Europe/Warsaw".into(),
            starts_at: "2026-09-05T17:30:00Z".into(),
            currency: "PLN".into(),
            vat_rate_basis_points: 8,
            capacity: 100,
            sold: 10,
            reserved: 2,
            available: 88,
            max_per_order: 8,
            sales_open_at: "2026-08-01T00:00:00Z".into(),
            sales_close_at: "2026-09-05T16:30:00Z".into(),
            active: true,
            sales_state: "open".into(),
            ticket_types: vec![TicketTypeOffer {
                id: Uuid::new_v4().to_string(),
                slug: "regular".into(),
                name: "Bilet regular".into(),
                description: None,
                price_gross_minor: 3_000,
                capacity: Some(100),
                sold: 10,
                reserved: 2,
                available: 88,
                sort_order: 0,
                active: true,
            }],
        }
    }

    #[test]
    fn ticket_sale_rejects_a_different_event_slug() {
        assert!(valid_ticket_sale().validate("another-show").is_err());
    }

    #[test]
    fn ticket_sale_accepts_a_bounded_consistent_offer() {
        assert!(valid_ticket_sale().validate("show-1").is_ok());
    }

    fn valid_checkout_response() -> SiteTicketCheckoutResponse {
        SiteTicketCheckoutResponse {
            url: "https://checkout.stripe.com/c/pay/cs_test_123".into(),
            order_id: Uuid::new_v4().to_string(),
            order_reference: "VRY-1".into(),
            checkout_token: "a".repeat(64),
            expires_at: "2026-08-05T20:00:00Z".into(),
        }
    }

    #[test]
    fn checkout_response_accepts_bounded_stripe_destination() {
        assert!(TicketCheckoutStart::from_site(&valid_checkout_response()).is_ok());
    }

    #[test]
    fn checkout_response_rejects_non_stripe_destination() {
        let mut response = valid_checkout_response();
        response.url = "https://example.com/pay".into();
        assert!(TicketCheckoutStart::from_site(&response).is_err());
    }

    #[test]
    fn checkout_response_rejects_malformed_secret() {
        let mut response = valid_checkout_response();
        response.checkout_token = "not-a-checkout-token".into();
        assert!(TicketCheckoutStart::from_site(&response).is_err());
    }
}
