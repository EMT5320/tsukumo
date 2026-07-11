//! Typed validation failures for state selection, rendering, and receipts.

use thiserror::Error;
use tsukumo_kernel::{CheckpointId, ProjectionId, StateId};

/// Deterministic projection preparation failure.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum ProjectionError {
    #[error("projection checkpoint {0} does not exist")]
    MissingCheckpoint(CheckpointId),
    #[error("projection state {0} does not exist")]
    MissingState(StateId),
    #[error("projection state {state_id} expected version {expected}, found {found}")]
    StateVersionMismatch {
        state_id: StateId,
        expected: u64,
        found: u64,
    },
    #[error("projection creation event does not match the request")]
    EventMismatch,
    #[error("projection field {0} is empty or invalid")]
    InvalidField(&'static str),
    #[error("projection field {0} contains sensitive material")]
    SensitiveField(&'static str),
    #[error("projection base content uses {used} characters but budget is {limit}")]
    BudgetTooSmall { used: usize, limit: usize },
    #[error("pinned state {state_id} cannot fit projection budget {limit}")]
    PinnedStateExceedsBudget { state_id: StateId, limit: usize },
    #[error("projection {0} already exists with different content")]
    ConflictingReceipt(ProjectionId),
    #[error("stored receipt edges do not match receipt {0}")]
    StoredEdgeMismatch(ProjectionId),
    #[error("with/without comparison changed a controlled non-target input")]
    ComparisonInvariant,
}
