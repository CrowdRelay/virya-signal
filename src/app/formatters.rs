use wasm_bindgen::JsValue;

use crate::util::{OptionValueOrElseExt, OptionValueOrExt};

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
    format!(
        "{sign}{},{:02} {}",
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
pub(super) fn event_location(event: &crate::models::PublicEvent) -> String {
    event
        .venue
        .clone()
        .or_else(|| event.city.as_ref().map(|city| city.name.clone()))
        .value_or_else(|| "Szczegóły wkrótce".to_owned())
}
pub(super) fn event_time_location(starts_at: &str, venue: Option<&str>) -> String {
    format!(
        "{} · {}",
        human_time(starts_at),
        venue.value_or("miejsce wkrótce")
    )
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
        0 => "STY",
        1 => "LUT",
        2 => "MAR",
        3 => "KWI",
        4 => "MAJ",
        5 => "CZE",
        6 => "LIP",
        7 => "SIE",
        8 => "WRZ",
        9 => "PAŹ",
        10 => "LIS",
        11 => "GRU",
        _ => "---",
    }
    .to_owned()
}
