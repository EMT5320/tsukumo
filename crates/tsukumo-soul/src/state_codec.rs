//! Stable SQLite text codecs for canonical state enums.

use crate::state_model::{EvidenceStrength, StateKind, StateStatus};
use crate::storage::SoulError;

pub(crate) fn kind_value(kind: StateKind) -> &'static str {
    match kind {
        StateKind::Preference => "preference",
        StateKind::Fact => "fact",
        StateKind::Constraint => "constraint",
        StateKind::Procedure => "procedure",
        StateKind::Milestone => "milestone",
    }
}

pub(crate) fn strength_value(strength: EvidenceStrength) -> &'static str {
    match strength {
        EvidenceStrength::Imported => "imported",
        EvidenceStrength::Inferred => "inferred",
        EvidenceStrength::Repeated => "repeated",
        EvidenceStrength::Explicit => "explicit",
    }
}

pub(crate) fn status_value(status: StateStatus) -> &'static str {
    match status {
        StateStatus::Active => "active",
        StateStatus::Superseded => "superseded",
        StateStatus::Revoked => "revoked",
    }
}

pub(crate) fn parse_kind(value: &str) -> Result<StateKind, SoulError> {
    match value {
        "preference" => Ok(StateKind::Preference),
        "fact" => Ok(StateKind::Fact),
        "constraint" => Ok(StateKind::Constraint),
        "procedure" => Ok(StateKind::Procedure),
        "milestone" => Ok(StateKind::Milestone),
        _ => Err(invalid("state_records.kind", value)),
    }
}

pub(crate) fn parse_strength(value: &str) -> Result<EvidenceStrength, SoulError> {
    match value {
        "imported" => Ok(EvidenceStrength::Imported),
        "inferred" => Ok(EvidenceStrength::Inferred),
        "repeated" => Ok(EvidenceStrength::Repeated),
        "explicit" => Ok(EvidenceStrength::Explicit),
        _ => Err(invalid("state_records.strength", value)),
    }
}

pub(crate) fn parse_status(value: &str) -> Result<StateStatus, SoulError> {
    match value {
        "active" => Ok(StateStatus::Active),
        "superseded" => Ok(StateStatus::Superseded),
        "revoked" => Ok(StateStatus::Revoked),
        _ => Err(invalid("state_records.status", value)),
    }
}

fn invalid(field: &'static str, value: &str) -> SoulError {
    SoulError::InvalidStoredValue {
        field,
        value: value.to_owned(),
    }
}
