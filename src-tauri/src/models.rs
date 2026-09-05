use serde::{Deserialize, Deserializer, Serialize, de::Error as _};
use serde_json::{Map, Value};
use zeroize::Zeroize;

fn deserialize_string_or_bytes<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Value::deserialize(deserializer)?;
    normalize_compat_string(value).map_err(D::Error::custom)
}

fn deserialize_optional_string_or_bytes<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    value
        .map(normalize_compat_string)
        .transpose()
        .map_err(D::Error::custom)
}

fn deserialize_string_or_default<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    match value {
        Some(value) => normalize_compat_string(value).map_err(D::Error::custom),
        None => Ok(String::new()),
    }
}

fn deserialize_area_wallet_id<'de, D>(deserializer: D) -> Result<String, D::Error>
where
    D: Deserializer<'de>,
{
    let value = Option::<Value>::deserialize(deserializer)?;
    match value {
        Some(value) => {
            let normalized = normalize_compat_string(value).map_err(D::Error::custom)?;
            if normalized.trim().is_empty() {
                Ok(new_area_wallet_id())
            } else {
                Ok(normalized)
            }
        }
        None => Ok(new_area_wallet_id()),
    }
}

fn normalize_compat_string(value: Value) -> Result<String, String> {
    match value {
        Value::String(value) => Ok(value),
        Value::Array(values) => normalize_compat_array(values),
        Value::Object(values) => normalize_compat_object(values),
        Value::Number(value) => Ok(value.to_string()),
        Value::Bool(value) => Ok(value.to_string()),
        Value::Null => Err("expected text, received null".to_owned()),
    }
}

fn normalize_compat_array(values: Vec<Value>) -> Result<String, String> {
    if values.is_empty() {
        return Ok(String::new());
    }

    if values.iter().all(Value::is_number) {
        return decode_numeric_sequence(&values);
    }

    if values.iter().all(Value::is_string) {
        let mut output = String::new();
        for value in values {
            match value {
                Value::String(value) => output.push_str(&value),
                _ => return Err("text sequence contained a non-text element".to_owned()),
            }
        }
        return Ok(output);
    }

    if values.len() == 1 {
        let value = values.into_iter().next().ok_or_else(|| {
            "expected one compatibility value, but the sequence was empty".to_owned()
        })?;
        return normalize_compat_string(value);
    }

    let mut output = String::new();
    for value in values {
        output.push_str(&normalize_compat_string(value)?);
    }
    Ok(output)
}

fn normalize_compat_object(mut values: Map<String, Value>) -> Result<String, String> {
    const WRAPPER_KEYS: [&str; 12] = [
        "value",
        "data",
        "bytes",
        "buffer",
        "buf",
        "content",
        "string",
        "chars",
        "code_units",
        "String",
        "Bytes",
        "$value",
    ];

    for key in WRAPPER_KEYS {
        if let Some(value) = values.remove(key) {
            return normalize_compat_string(value);
        }
    }

    if values.keys().all(|key| key.parse::<usize>().is_ok()) {
        let mut indexed_values = Vec::with_capacity(values.len());
        for (key, value) in values {
            let index = key
                .parse::<usize>()
                .map_err(|_| format!("unsupported text object key `{key}`"))?;
            indexed_values.push((index, value));
        }
        indexed_values.sort_unstable_by_key(|(index, _)| *index);

        for (expected, (actual, _)) in indexed_values.iter().enumerate() {
            if *actual != expected {
                return Err(format!(
                    "text object byte indexes are not contiguous: expected {expected}, received {actual}"
                ));
            }
        }

        return normalize_compat_array(
            indexed_values.into_iter().map(|(_, value)| value).collect(),
        );
    }

    if values.len() == 1 {
        let value = values
            .into_iter()
            .next()
            .map(|(_, value)| value)
            .ok_or_else(|| "expected one compatibility object value".to_owned())?;
        return normalize_compat_string(value);
    }

    Err(format!(
        "unsupported text object with keys: {}",
        values.keys().cloned().collect::<Vec<_>>().join(", ")
    ))
}

fn decode_numeric_sequence(values: &[Value]) -> Result<String, String> {
    let code_units = values
        .iter()
        .map(value_as_u16)
        .collect::<Result<Vec<_>, _>>()?;

    if looks_like_utf16_le_bytes(&code_units) {
        return decode_utf16_bytes(&code_units, true);
    }
    if looks_like_utf16_be_bytes(&code_units) {
        return decode_utf16_bytes(&code_units, false);
    }

    if code_units.iter().all(|value| *value <= u8::MAX as u16) {
        let bytes = code_units
            .iter()
            .map(|value| *value as u8)
            .collect::<Vec<_>>();
        if let Ok(text) = String::from_utf8(bytes) {
            return Ok(text);
        }
    }

    String::from_utf16(&code_units)
        .map_err(|error| format!("invalid UTF-8/UTF-16 text sequence: {error}"))
}

fn value_as_u16(value: &Value) -> Result<u16, String> {
    if let Some(number) = value.as_u64() {
        return u16::try_from(number)
            .map_err(|_| format!("text code unit {number} exceeds the UTF-16 range"));
    }

    if let Some(number) = value.as_i64()
        && (-128..=-1).contains(&number)
    {
        return Ok((number as i8 as u8) as u16);
    }

    Err(format!(
        "expected a byte or UTF-16 code unit, received {value}"
    ))
}

fn looks_like_utf16_le_bytes(values: &[u16]) -> bool {
    values.len() >= 2
        && values.len().is_multiple_of(2)
        && values
            .iter()
            .enumerate()
            .all(|(index, value)| index % 2 == 0 || *value == 0)
}

fn looks_like_utf16_be_bytes(values: &[u16]) -> bool {
    values.len() >= 2
        && values.len().is_multiple_of(2)
        && values
            .iter()
            .enumerate()
            .all(|(index, value)| index % 2 != 0 || *value == 0)
}

fn decode_utf16_bytes(values: &[u16], little_endian: bool) -> Result<String, String> {
    let mut code_units = Vec::with_capacity(values.len() / 2);
    for pair in values.chunks_exact(2) {
        let first = pair[0] as u8;
        let second = pair[1] as u8;
        code_units.push(if little_endian {
            u16::from_le_bytes([first, second])
        } else {
            u16::from_be_bytes([first, second])
        });
    }
    String::from_utf16(&code_units)
        .map_err(|error| format!("invalid UTF-16 byte sequence: {error}"))
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum OperatorRole {
    Owner,
    Staff,
}

mod staff_pairing;
pub use staff_pairing::{StaffPairingExchange, StaffPairingPayload};

include!("models/session_fan.rs");
include!("models/commerce_events.rs");
include!("models/area.rs");
include!("models/signal.rs");
include!("models/showmode_inputs.rs");
include!("models/tests.rs");
include!("models/beacon.rs");
