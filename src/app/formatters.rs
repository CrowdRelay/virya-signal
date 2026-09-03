use wasm_bindgen::JsValue;

use crate::{
    i18n::{self, Language, tr},
    util::OptionValueOrExt,
};

/// Picks the `_one` / `_few` / `_many` variant of a catalog key for a count.
///
/// Polish has three plural forms and the catalog held only one, so the AREA
/// progress card showed the five-or-more form to every fan who had two
/// credits. English supplies the same string for `_few` and `_many`, which is
/// why one selector serves both catalogs.
pub(super) fn plural_key(
    count: i64,
    one: &'static str,
    few: &'static str,
    many: &'static str,
) -> &'static str {
    let n = count.unsigned_abs();
    let last_two = n % 100;
    let last = n % 10;
    if n == 1 {
        one
    } else if (2..=4).contains(&last) && !(12..=14).contains(&last_two) {
        few
    } else {
        many
    }
}

pub(super) fn optional(value: String) -> Option<String> {
    let value = value.trim().to_owned();
    (!value.is_empty()).then_some(value)
}

pub(super) fn local_to_rfc3339(value: &str) -> Option<String> {
    if value.trim().is_empty() {
        return None;
    }
    let date = js_sys::Date::new(&JsValue::from_str(value));
    let time = date.get_time();
    if time.is_nan() {
        None
    } else {
        date.to_iso_string().as_string()
    }
}

pub(super) fn money(minor: i64, currency: &str) -> String {
    let absolute = minor.unsigned_abs();
    let sign = if minor < 0 { "-" } else { "" };
    let separator = if i18n::current() == Language::En {
        '.'
    } else {
        ','
    };
    format!(
        "{sign}{}{separator}{:02} {}",
        absolute / 100,
        absolute % 100,
        currency.to_uppercase()
    )
}

pub(super) fn human_time(value: &str) -> String {
    let date = js_sys::Date::new(&JsValue::from_str(value));
    if date.get_time().is_nan() {
        return value.chars().take(32).collect();
    }
    format!(
        "{:02}.{:02}.{} • {:02}:{:02}",
        date.get_date(),
        date.get_month() + 1,
        date.get_full_year(),
        date.get_hours(),
        date.get_minutes()
    )
}

/// City first, then venue. The venue alone was the whole subtitle, and on a
/// tour every title already carries the venue name — so a list of shows read
/// "Virya · Hydrozagadka / Hydrozagadka" and dropped the one field that tells
/// a fan or a staffer which show this is. Either half alone still stands on
/// its own when the other is missing.
pub(super) fn event_location(event: &crate::models::PublicEvent) -> String {
    let city = event.city.as_ref().map(|city| city.name.as_str());
    match (city, event.venue.as_deref()) {
        (Some(city), Some(venue)) if !city.eq_ignore_ascii_case(venue) => {
            format!("{city} · {venue}")
        }
        (Some(value), _) | (None, Some(value)) => value.to_owned(),
        (None, None) => tr("details_coming_soon").to_owned(),
    }
}

pub(super) fn event_time_location(starts_at: &str, venue: Option<&str>) -> String {
    format!(
        "{} · {}",
        human_time(starts_at),
        venue.value_or(tr("venue_coming_soon"))
    )
}

pub(super) fn elapsed_time(elapsed_ms: i64) -> String {
    let total_seconds = elapsed_ms.max(0) / 1_000;
    format!("{}:{:02}", total_seconds / 60, total_seconds % 60)
}

pub(super) fn synesthesia_best_summary(
    synesthesia: &crate::models::FanHomeSynesthesia,
) -> Option<String> {
    let elapsed_ms = synesthesia.best_elapsed_ms?;
    let mut parts = vec![i18n::format(
        "synesthesia_best_time",
        &[elapsed_time(elapsed_ms)],
    )];
    if synesthesia.leaderboard_published
        && let Some(rank) = synesthesia.leaderboard_rank
    {
        parts.push(i18n::format("synesthesia_rank", &[rank.to_string()]));
    }
    if synesthesia.completed_runs > 1 {
        parts.push(i18n::format(
            "synesthesia_runs_count",
            &[synesthesia.completed_runs.to_string()],
        ));
    }
    Some(parts.join(" · "))
}

pub(super) fn day(value: &str) -> String {
    let date = js_sys::Date::new(&JsValue::from_str(value));
    if date.get_time().is_nan() {
        "--".to_owned()
    } else {
        format!("{:02}", date.get_date())
    }
}

pub(super) fn month(value: &str) -> String {
    let date = js_sys::Date::new(&JsValue::from_str(value));
    let month = if date.get_time().is_nan() {
        u32::MAX
    } else {
        date.get_month()
    };
    match month {
        0 => tr("jan"),
        1 => tr("feb"),
        2 => tr("mar"),
        3 => tr("apr"),
        4 => tr("may"),
        5 => tr("jun"),
        6 => tr("jul"),
        7 => tr("aug"),
        8 => tr("sep"),
        9 => tr("oct"),
        10 => tr("nov"),
        11 => tr("dec"),
        _ => tr("text"),
    }
    .to_owned()
}
