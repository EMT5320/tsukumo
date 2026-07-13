//! Shared strict field decoders and bounded vendor labels.

use crate::stream_json::DecodeError;
use serde_json::Value;
use tsukumo_kernel::{contains_sensitive_material, redact_sensitive_text};

const MAX_VENDOR_LABEL_CHARS: usize = 128;

pub(crate) fn required_label(
    value: &Value,
    field: &'static str,
    event_type: &'static str,
) -> Result<String, DecodeError> {
    let value = required_string(value, field, event_type)?;
    reviewed_label(value, event_type, field)
}

pub(crate) fn reviewed_label(
    value: &str,
    event_type: &'static str,
    field: &'static str,
) -> Result<String, DecodeError> {
    if value.trim().is_empty() || contains_sensitive_material(value) {
        return Err(DecodeError::invalid(event_type, field));
    }
    let value = redact_sensitive_text(value);
    if value.trim().is_empty() {
        return Err(DecodeError::invalid(event_type, field));
    }
    Ok(truncate(&value, MAX_VENDOR_LABEL_CHARS))
}

pub(crate) fn required_string<'a>(
    value: &'a Value,
    field: &'static str,
    event_type: &'static str,
) -> Result<&'a str, DecodeError> {
    value
        .get(field)
        .ok_or_else(|| DecodeError::missing(event_type, field))?
        .as_str()
        .ok_or_else(|| DecodeError::invalid(event_type, field))
}

pub(crate) fn optional_string<'a>(
    value: &'a Value,
    field: &'static str,
    event_type: &'static str,
) -> Result<Option<&'a str>, DecodeError> {
    value
        .get(field)
        .map(|field_value| {
            field_value
                .as_str()
                .ok_or_else(|| DecodeError::invalid(event_type, field))
        })
        .transpose()
}

pub(crate) fn required_i64(
    value: &Value,
    field: &'static str,
    event_type: &'static str,
) -> Result<i64, DecodeError> {
    value
        .get(field)
        .ok_or_else(|| DecodeError::missing(event_type, field))?
        .as_i64()
        .ok_or_else(|| DecodeError::invalid(event_type, field))
}

pub(crate) fn required_bool(
    value: &Value,
    field: &'static str,
    event_type: &'static str,
) -> Result<bool, DecodeError> {
    value
        .get(field)
        .ok_or_else(|| DecodeError::missing(event_type, field))?
        .as_bool()
        .ok_or_else(|| DecodeError::invalid(event_type, field))
}

pub(crate) fn optional_bool(
    value: &Value,
    field: &'static str,
    event_type: &'static str,
) -> Result<Option<bool>, DecodeError> {
    value
        .get(field)
        .map(|field_value| {
            field_value
                .as_bool()
                .ok_or_else(|| DecodeError::invalid(event_type, field))
        })
        .transpose()
}

pub(crate) fn optional_string_array<'a>(
    value: &'a Value,
    field: &'static str,
    event_type: &'static str,
) -> Result<Vec<&'a str>, DecodeError> {
    let Some(field_value) = value.get(field) else {
        return Ok(Vec::new());
    };
    let values = field_value
        .as_array()
        .ok_or_else(|| DecodeError::invalid(event_type, field))?;
    values
        .iter()
        .map(|item| {
            item.as_str()
                .ok_or_else(|| DecodeError::invalid(event_type, field))
        })
        .collect()
}

pub(crate) fn truncate(text: &str, max_chars: usize) -> String {
    if text.chars().count() <= max_chars {
        text.to_owned()
    } else {
        let prefix = text
            .chars()
            .take(max_chars.saturating_sub(1))
            .collect::<String>();
        format!("{prefix}\u{2026}")
    }
}
