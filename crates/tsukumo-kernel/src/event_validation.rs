//! Validation shared by JSONL fixture loading and Chronicle persistence.

mod payload;

use crate::event::{KernelEvent, VendorEventRef, KERNEL_EVENT_SCHEMA_VERSION};
use crate::redaction::{contains_sensitive_material, contains_unredacted_sensitive_json};
use crate::value::PersistedText;
use crate::EventId;
use thiserror::Error;

const MAX_LABEL_CHARS: usize = 256;
const MAX_PERSISTED_JSON_BYTES: usize = 65_536;
const MAX_PERSISTED_TEXT_CHARS: usize = 65_536;
const MAX_PERSISTED_TEXT_ITEMS: usize = 64;

/// Durable event contract violation detected before replay or persistence.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum EventContractError {
    #[error("unsupported event schema {found}; supported schema is {supported}")]
    UnsupportedSchema { found: u16, supported: u16 },
    #[error("event {event_id} is missing required {field}")]
    MissingAttribution {
        event_id: EventId,
        field: &'static str,
    },
    #[error("event {event_id} has invalid {field}")]
    InvalidField {
        event_id: EventId,
        field: &'static str,
    },
    #[error("event {event_id} contains unredacted sensitive content")]
    SensitiveContent { event_id: EventId },
}

/// Validates the version, attribution, bounded labels, and redaction contract.
pub fn validate_kernel_event(event: &KernelEvent) -> Result<(), EventContractError> {
    if event.schema_version != KERNEL_EVENT_SCHEMA_VERSION {
        return Err(EventContractError::UnsupportedSchema {
            found: event.schema_version,
            supported: KERNEL_EVENT_SCHEMA_VERSION,
        });
    }
    for (field, value) in [
        ("event_id", event.event_id.as_str()),
        ("quest_id", event.quest_id.as_str()),
        ("session_id", event.session_id.as_str()),
        ("spirit_id", event.spirit_id.as_str()),
    ] {
        validate_label(event, field, value)?;
    }
    for (field, value) in [
        (
            "execution_id",
            event.execution_id.as_ref().map(|id| id.as_str()),
        ),
        (
            "causation_id",
            event.causation_id.as_ref().map(|id| id.as_str()),
        ),
        (
            "correlation_id",
            event.correlation_id.as_ref().map(|id| id.as_str()),
        ),
    ] {
        if let Some(value) = value {
            validate_label(event, field, value)?;
        }
    }

    payload::validate_payload(event)
}

pub(super) fn require_execution_runtime(event: &KernelEvent) -> Result<(), EventContractError> {
    require(event, "execution_id", event.execution_id.is_some())?;
    require(event, "runtime", event.runtime.is_some())
}

pub(super) fn require_execution_runtime_correlation(
    event: &KernelEvent,
) -> Result<(), EventContractError> {
    require_execution_runtime(event)?;
    require(event, "correlation_id", event.correlation_id.is_some())
}

pub(super) fn require(
    event: &KernelEvent,
    field: &'static str,
    condition: bool,
) -> Result<(), EventContractError> {
    if condition {
        Ok(())
    } else {
        Err(EventContractError::MissingAttribution {
            event_id: event.event_id.clone(),
            field,
        })
    }
}

pub(super) fn validate_vendor_ref(
    event: &KernelEvent,
    reference: &VendorEventRef,
) -> Result<(), EventContractError> {
    validate_label(event, "vendor.namespace", &reference.namespace)?;
    validate_label(event, "vendor.id", &reference.id)
}

pub(super) fn validate_label(
    event: &KernelEvent,
    field: &'static str,
    value: &str,
) -> Result<(), EventContractError> {
    if value.trim().is_empty()
        || value.chars().count() > MAX_LABEL_CHARS
        || value.chars().any(char::is_control)
        || contains_sensitive_material(value)
    {
        return Err(EventContractError::InvalidField {
            event_id: event.event_id.clone(),
            field,
        });
    }
    Ok(())
}

pub(super) fn validate_text(
    event: &KernelEvent,
    text: &PersistedText,
) -> Result<(), EventContractError> {
    if text.as_str().chars().count() > MAX_PERSISTED_TEXT_CHARS {
        return Err(EventContractError::InvalidField {
            event_id: event.event_id.clone(),
            field: "persisted_text",
        });
    }
    if contains_sensitive_material(text.as_str()) {
        return Err(EventContractError::SensitiveContent {
            event_id: event.event_id.clone(),
        });
    }
    Ok(())
}

pub(super) fn validate_json(
    event: &KernelEvent,
    value: Option<&serde_json::Value>,
) -> Result<(), EventContractError> {
    let Some(value) = value else {
        return Ok(());
    };
    if contains_unredacted_sensitive_json(value) {
        return Err(EventContractError::SensitiveContent {
            event_id: event.event_id.clone(),
        });
    }
    let byte_count = serde_json::to_vec(value)
        .map_err(|_| EventContractError::InvalidField {
            event_id: event.event_id.clone(),
            field: "persisted_json",
        })?
        .len();
    if byte_count > MAX_PERSISTED_JSON_BYTES {
        return Err(EventContractError::InvalidField {
            event_id: event.event_id.clone(),
            field: "persisted_json",
        });
    }
    Ok(())
}
