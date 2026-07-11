//! Canonical state vocabulary shared by extraction, writing, and exports.

use serde::{Deserialize, Serialize};
use std::fmt;
use thiserror::Error;
use tsukumo_kernel::{EventId, KernelEvent, PersistedText, SensitiveText, StateId, Timestamp};

/// Stable semantic key grouping versions of the same state.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StateKey(String);

impl StateKey {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for StateKey {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// Meaning carried by a canonical state record.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StateKind {
    Preference,
    Fact,
    Constraint,
    Procedure,
    Milestone,
}

pub use crate::state_scope::{OperatingSystem, StateApplicability, StateScope, StateSubject};

/// Strength of Chronicle evidence supporting a state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EvidenceStrength {
    Imported,
    Inferred,
    Repeated,
    Explicit,
}

/// Current lifecycle status for a persisted state version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StateStatus {
    Active,
    Superseded,
    Revoked,
}

/// Extractor identity used for diagnosis without replacing evidence refs.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ExtractionProvenance {
    Rule {
        name: String,
        version: u32,
    },
    StructuredModel {
        provider: String,
        model: String,
        schema_version: u16,
    },
    Recorded {
        fixture: String,
        schema_version: u16,
    },
    LegacyImport {
        table: String,
    },
}

/// In-memory proposal produced by an extractor without database authority.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StateDraft {
    pub proposed_key: StateKey,
    pub kind: StateKind,
    pub scope: StateScope,
    pub content: SensitiveText,
    pub claimed_strength: EvidenceStrength,
    pub evidence_refs: Vec<EventId>,
    pub provenance: ExtractionProvenance,
    pub expires_at: Option<Timestamp>,
}

/// One persisted, evidence-backed state version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StateRecord {
    pub state_id: StateId,
    pub state_key: StateKey,
    pub kind: StateKind,
    pub scope: StateScope,
    pub content: PersistedText,
    pub strength: EvidenceStrength,
    pub status: StateStatus,
    pub evidence_refs: Vec<EventId>,
    pub provenance: ExtractionProvenance,
    pub version: u64,
    pub created_at: Timestamp,
    pub expires_at: Option<Timestamp>,
    pub deactivated_at: Option<Timestamp>,
    pub supersedes_state_id: Option<StateId>,
}

impl StateRecord {
    /// Returns whether this version is selectable at the provided instant.
    pub fn is_active_at(&self, as_of: Timestamp) -> bool {
        self.created_at <= as_of
            && self
                .expires_at
                .map_or(true, |expires_at| expires_at > as_of)
            && self
                .deactivated_at
                .map_or(true, |deactivated_at| deactivated_at > as_of)
    }
}

/// Requested deterministic state lifecycle transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateTransition {
    Create {
        state_id: StateId,
        draft: StateDraft,
        created_at: Timestamp,
    },
    Supersede {
        state_id: StateId,
        prior: StateId,
        draft: StateDraft,
        created_at: Timestamp,
    },
    Revoke {
        prior: StateId,
        evidence: EventId,
        revoked_at: Timestamp,
    },
}

/// Atomic inputs for one StateWriter operation.
#[derive(Debug, Clone, PartialEq)]
pub struct StateWriteRequest {
    pub source_event: Option<KernelEvent>,
    pub lifecycle_event: KernelEvent,
    pub transition: StateTransition,
}

impl StateWriteRequest {
    pub fn new(transition: StateTransition, lifecycle_event: KernelEvent) -> Self {
        Self {
            source_event: None,
            lifecycle_event,
            transition,
        }
    }

    /// Attaches a newly observed source and links the lifecycle event to it.
    pub fn with_source_event(mut self, source_event: KernelEvent) -> Self {
        if self.lifecycle_event.causation_id.is_none() {
            self.lifecycle_event.causation_id = Some(source_event.event_id.clone());
        }
        self.source_event = Some(source_event);
        self
    }
}

/// Explicit result of applying a state transition.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum StateWriteOutcome {
    Created(StateRecord),
    Superseded(StateRecord),
    Revoked(StateRecord),
    Unchanged(StateRecord),
}

/// Deterministic state validation failure.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum StateValidationError {
    #[error("state key is empty")]
    EmptyKey,
    #[error("state content is empty")]
    EmptyContent,
    #[error("state draft has no Chronicle evidence")]
    EmptyEvidence,
    #[error("Chronicle evidence {0} does not exist")]
    MissingEvidence(EventId),
    #[error("permission event {0} cannot create relationship state")]
    PermissionEvidence(EventId),
    #[error("inferred evidence cannot create a hard constraint")]
    InferredConstraint,
    #[error("the extractor provenance cannot claim explicit evidence")]
    UntrustedExplicit,
    #[error("explicit state requires matching user-input evidence")]
    ExplicitEvidenceRequired,
    #[error("repeated strength requires at least two distinct Chronicle events")]
    RepeatedEvidenceRequired,
    #[error("Chronicle evidence {0} occurs after the state transition")]
    EvidenceAfterTransition(EventId),
    #[error("state lifecycle causation or source identity does not match its evidence")]
    EvidenceChainMismatch,
    #[error("state scope is invalid")]
    InvalidScope,
    #[error("state metadata is invalid or sensitive")]
    InvalidMetadata,
    #[error("state scope is unresolved")]
    UnresolvedScope,
    #[error("state expiry is not later than creation")]
    InvalidExpiry,
    #[error("state content matches the secret-material policy")]
    SecretMaterial,
    #[error("active state conflicts for key {0}")]
    Conflict(StateKey),
    #[error("state transition predates a newer version for key {0}")]
    BackdatedTransition(StateKey),
    #[error("state {0} was not found")]
    StateNotFound(StateId),
    #[error("state {0} is not active")]
    StateInactive(StateId),
    #[error("lifecycle event does not match the transition")]
    InvalidLifecycleEvent,
}
