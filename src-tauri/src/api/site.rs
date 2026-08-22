use std::time::Duration;

use reqwest::header::{ACCEPT, ORIGIN};
use serde::{Deserialize, Serialize};
use url::Url;

use crate::AppError;

use super::{client::CrowdRelayClient, http::decode};

const MERCH_INVENTORY_URL: &str = "https://virya.music/api/merch/inventory";
const SIGNAL_FEEDBACK_URL: &str = "https://virya.music/api/signal-feedback";
const VIRYA_SITE_ORIGIN: &str = "https://virya.music";
const NETLIFY_IMAGE_PATH: &str = "/.netlify/images";
const MERCH_PREVIEW_IMAGE_WIDTH: u32 = 600;
const SITE_READ_TIMEOUT: Duration = Duration::from_secs(8);
const FEEDBACK_TIMEOUT: Duration = Duration::from_secs(15);
const MAX_BUNDLES: usize = 8;
const MAX_VARIANTS: usize = 8;
const MAX_INCLUDES: usize = 8;
const MAX_MESSAGE_CHARS: usize = 2_000;

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct SignalMerchBundleCatalog {
    #[serde(default)]
    pub bundles: Vec<SignalMerchBundle>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct SignalMerchBundle {
    pub slug: String,
    pub name: String,
    pub description: Option<String>,
    #[serde(default)]
    pub includes: Vec<String>,
    pub image_url: Option<String>,
    pub secondary_image_url: Option<String>,
    pub product_url: String,
    pub currency: String,
    pub price_gross_minor: i64,
    pub original_price_gross_minor: i64,
    pub available: bool,
    pub availability: String,
    #[serde(default)]
    pub variants: Vec<SignalMerchBundleVariant>,
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub(crate) struct SignalMerchBundleVariant {
    pub label: String,
    pub available: bool,
    pub availability: String,
}

#[derive(Serialize)]
struct SignalFeedbackRequest<'a> {
    submission_id: String,
    category: &'a str,
    message: &'a str,
    website: &'static str,
}

#[derive(Deserialize)]
struct SignalFeedbackResponse {
    ok: bool,
}

fn bounded(value: &str, label: &str, max: usize) -> Result<String, AppError> {
    let value = value.trim();
    if value.is_empty() || value.chars().count() > max || value.chars().any(char::is_control) {
        return Err(AppError::InvalidInput(crate::i18n::replace(
            "native_invalid_label",
            &[("label", label.to_owned())],
        )));
    }
    Ok(value.to_owned())
}

fn bounded_multiline(value: &str, label: &str, max: usize) -> Result<String, AppError> {
    let value = value.trim();
    if value.chars().count() < 8
        || value.chars().count() > max
        || value.chars().any(|character| {
            character == '\0' || (character.is_control() && character != '\n' && character != '\t')
        })
    {
        return Err(AppError::InvalidInput(crate::i18n::replace(
            "native_invalid_label",
            &[("label", label.to_owned())],
        )));
    }
    Ok(value.to_owned())
}

fn valid_slug(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value.bytes().all(|byte| {
            byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'-' | b'_')
        })
}

fn valid_availability(value: &str) -> bool {
    matches!(value, "available" | "low_stock" | "sold_out")
}

fn validated_site_url(value: &str, label: &str) -> Result<String, AppError> {
    let parsed = Url::parse(value.trim())?;
    if parsed.scheme() != "https"
        || !matches!(parsed.host_str(), Some("virya.music" | "www.virya.music"))
        || parsed.username() != ""
        || parsed.password().is_some()
        || parsed.fragment().is_some()
    {
        return Err(AppError::InvalidInput(crate::i18n::replace(
            "native_invalid_label",
            &[("label", label.to_owned())],
        )));
    }
    Ok(parsed.to_string())
}

/// Rewrites a virya.music image URL to a card-sized variant served by the
/// Netlify Image CDN.
///
/// The store art is 1200x1200 and the fan storefront paints it into a card
/// roughly 190 CSS px wide. Decoding the original costs about 5.8 MB of bitmap
/// per image, and a screenful of them is what makes the merch tab heavy on
/// Android. Anything not served from the site is returned untouched.
pub(super) fn merch_preview_image_url(value: &str) -> String {
    let Ok(parsed) = Url::parse(value) else {
        return value.to_owned();
    };
    if parsed.scheme() != "https"
        || !matches!(parsed.host_str(), Some("virya.music" | "www.virya.music"))
        || parsed.path().starts_with(NETLIFY_IMAGE_PATH)
    {
        return value.to_owned();
    }
    let Ok(mut resized) = Url::parse(VIRYA_SITE_ORIGIN) else {
        return value.to_owned();
    };
    resized.set_path(NETLIFY_IMAGE_PATH);
    resized
        .query_pairs_mut()
        .append_pair("url", parsed.path())
        .append_pair("w", &MERCH_PREVIEW_IMAGE_WIDTH.to_string())
        .append_pair("fm", "webp")
        .append_pair("q", "72");
    resized.to_string()
}

fn validated_store_url(value: &str) -> Result<String, AppError> {
    let value = validated_site_url(value, crate::i18n::tr("native_store_url_label"))?;
    let parsed = Url::parse(&value)?;
    if !matches!(parsed.path(), "/pl/merch" | "/pl/merch/") {
        return Err(AppError::InvalidInput(
            crate::i18n::tr("native_invalid_store_url").into(),
        ));
    }
    Ok(value)
}

impl SignalMerchBundleCatalog {
    fn validate(mut self) -> Result<Self, AppError> {
        if self.bundles.len() > MAX_BUNDLES {
            return Err(AppError::InvalidInput(
                crate::i18n::tr("native_bundle_catalog_too_large").into(),
            ));
        }
        let mut seen = std::collections::HashSet::new();
        for bundle in &mut self.bundles {
            bundle.slug = bundle.slug.trim().to_owned();
            if !valid_slug(&bundle.slug) || !seen.insert(bundle.slug.clone()) {
                return Err(AppError::InvalidInput(
                    crate::i18n::tr("native_invalid_merch_bundle").into(),
                ));
            }
            bundle.name = bounded(
                &bundle.name,
                crate::i18n::tr("native_bundle_name_label"),
                120,
            )?;
            bundle.description = bundle
                .description
                .take()
                .map(|value| {
                    bounded_multiline(
                        &value,
                        crate::i18n::tr("native_bundle_description_label"),
                        600,
                    )
                })
                .transpose()?;
            if bundle.includes.len() > MAX_INCLUDES || bundle.variants.len() > MAX_VARIANTS {
                return Err(AppError::InvalidInput(
                    crate::i18n::tr("native_bundle_too_many_items").into(),
                ));
            }
            for include in &mut bundle.includes {
                *include = bounded(include, crate::i18n::tr("native_bundle_item_label"), 120)?;
            }
            bundle.image_url = bundle
                .image_url
                .take()
                .map(|url| validated_site_url(&url, crate::i18n::tr("native_image_url_label")))
                .transpose()?
                .map(|url| merch_preview_image_url(&url));
            bundle.secondary_image_url = bundle
                .secondary_image_url
                .take()
                .map(|url| validated_site_url(&url, crate::i18n::tr("native_image_url_label")))
                .transpose()?
                .map(|url| merch_preview_image_url(&url));
            bundle.product_url = validated_store_url(&bundle.product_url)?;
            bundle.currency = bundle.currency.trim().to_ascii_uppercase();
            if bundle.currency != "PLN"
                || !(0..=1_000_000).contains(&bundle.price_gross_minor)
                || !(0..=1_000_000).contains(&bundle.original_price_gross_minor)
                || bundle.price_gross_minor > bundle.original_price_gross_minor
                || !valid_availability(&bundle.availability)
                || bundle.available == (bundle.availability == "sold_out")
            {
                return Err(AppError::InvalidInput(
                    crate::i18n::tr("native_invalid_bundle_offer").into(),
                ));
            }
            let mut seen_variants = std::collections::HashSet::new();
            for variant in &mut bundle.variants {
                variant.label = bounded(
                    &variant.label,
                    crate::i18n::tr("native_bundle_variant_label"),
                    24,
                )?;
                if !seen_variants.insert(variant.label.clone())
                    || !valid_availability(&variant.availability)
                    || variant.available == (variant.availability == "sold_out")
                {
                    return Err(AppError::InvalidInput(
                        crate::i18n::tr("native_invalid_bundle_variant").into(),
                    ));
                }
            }
        }
        Ok(self)
    }
}

impl CrowdRelayClient {
    pub async fn public_merch_bundles(&self) -> Result<SignalMerchBundleCatalog, AppError> {
        let response = self
            .site_http
            .get(Url::parse(MERCH_INVENTORY_URL)?)
            .header(ACCEPT, "application/json")
            .header(ORIGIN, VIRYA_SITE_ORIGIN)
            .timeout(SITE_READ_TIMEOUT)
            .send()
            .await?;
        let catalog: SignalMerchBundleCatalog = decode(response).await?;
        catalog.validate()
    }

    pub async fn submit_anonymous_feedback(
        &self,
        submission_id: &str,
        category: &str,
        message: &str,
    ) -> Result<(), AppError> {
        let category = category.trim();
        if !matches!(category, "idea" | "bug" | "concert" | "merch" | "other") {
            return Err(AppError::InvalidInput(
                crate::i18n::tr("native_choose_feedback_category").into(),
            ));
        }
        let message = bounded_multiline(
            message,
            crate::i18n::tr("native_feedback_content_label"),
            MAX_MESSAGE_CHARS,
        )?;
        let response = self
            .site_http
            .post(Url::parse(SIGNAL_FEEDBACK_URL)?)
            .header(ACCEPT, "application/json")
            .header(ORIGIN, VIRYA_SITE_ORIGIN)
            .json(&SignalFeedbackRequest {
                submission_id: submission_id.to_owned(),
                category,
                message: &message,
                website: "",
            })
            .timeout(FEEDBACK_TIMEOUT)
            .send()
            .await?;
        let result: SignalFeedbackResponse = decode(response).await?;
        if result.ok {
            Ok(())
        } else {
            Err(AppError::Remote {
                status: 502,
                detail: crate::i18n::tr("native_feedback_failed").into(),
            })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn valid_catalog() -> SignalMerchBundleCatalog {
        SignalMerchBundleCatalog {
            bundles: vec![SignalMerchBundle {
                slug: "bundle-stage-pack".into(),
                name: "Pakiet Sceniczny".into(),
                description: Some("Koszulka i album w jednym zestawie.".into()),
                includes: vec!["Koszulka".into(), "Album".into()],
                image_url: Some("https://virya.music/images/merch/echoes.webp".into()),
                secondary_image_url: None,
                product_url: "https://virya.music/pl/merch/?product=bundle-stage-pack".into(),
                currency: "PLN".into(),
                price_gross_minor: 7_000,
                original_price_gross_minor: 10_000,
                available: true,
                availability: "available".into(),
                variants: vec![SignalMerchBundleVariant {
                    label: "L".into(),
                    available: true,
                    availability: "available".into(),
                }],
            }],
        }
    }

    #[test]
    fn accepts_bounded_same_origin_bundle_catalog() {
        assert!(valid_catalog().validate().is_ok());
    }

    #[test]
    fn rejects_external_bundle_destination() {
        let mut catalog = valid_catalog();
        catalog.bundles[0].product_url = "https://example.com/store".into();
        assert!(catalog.validate().is_err());
    }

    #[test]
    fn rewrites_site_images_to_card_sized_previews() {
        let image = valid_catalog()
            .validate()
            .ok()
            .and_then(|catalog| catalog.bundles.into_iter().next())
            .and_then(|bundle| bundle.image_url)
            .unwrap_or_default();
        assert!(image.starts_with("https://virya.music/.netlify/images?"));
        assert!(image.contains("w=600"));
        assert!(image.contains("url=%2Fimages%2Fmerch%2Fechoes.webp"));
    }

    #[test]
    fn leaves_already_resized_and_foreign_images_alone() {
        let resized = "https://virya.music/.netlify/images?url=%2Fa.webp&w=600";
        assert_eq!(merch_preview_image_url(resized), resized);
        let foreign = "https://cdn.example.com/a.webp";
        assert_eq!(merch_preview_image_url(foreign), foreign);
    }

    #[test]
    fn rejects_inconsistent_availability() {
        let mut catalog = valid_catalog();
        catalog.bundles[0].availability = "sold_out".into();
        assert!(catalog.validate().is_err());
    }
}
