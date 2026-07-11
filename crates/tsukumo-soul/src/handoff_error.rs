//! Typed validation errors for checkpoint and handoff persistence.

use crate::handoff_loop::OpenLoopId;
use thiserror::Error;
use tsukumo_kernel::{CheckpointId, EventId, StateId};

/// Deterministic checkpoint validation failure.
#[derive(Debug, Error, Clone, PartialEq, Eq)]
pub enum HandoffError {
    #[error("checkpoint field {0} is empty or invalid")]
    InvalidField(&'static str),
    #[error("checkpoint field {0} contains sensitive material")]
    SensitiveField(&'static str),
    #[error("checkpoint creation event does not match the checkpoint")]
    EventMismatch,
    #[error("checkpoint source event {0} does not exist")]
    MissingSourceEvent(EventId),
    #[error("checkpoint state {0} does not exist")]
    MissingState(StateId),
    #[error("checkpoint state {state_id} expected version {expected}, found {found}")]
    StateVersionMismatch {
        state_id: StateId,
        expected: u64,
        found: u64,
    },
    #[error("checkpoint state {0} was not active when captured")]
    InactiveState(StateId),
    #[error("previous checkpoint {0} does not exist")]
    MissingPrevious(CheckpointId),
    #[error("checkpoint version or previous link is invalid")]
    InvalidVersion,
    #[error("checkpoint quest does not match its previous version")]
    QuestMismatch,
    #[error("open loop {0} appears more than once")]
    DuplicateOpenLoop(OpenLoopId),
    #[error("prior open loop {0} has more than one transition")]
    DuplicateTransition(OpenLoopId),
    #[error("transition references unknown prior open loop {0}")]
    UnknownPriorLoop(OpenLoopId),
    #[error("prior open loop {0} has no explicit transition")]
    UnresolvedPriorLoop(OpenLoopId),
    #[error("open-loop transition for {0} does not match the next checkpoint")]
    InvalidOpenLoopTransition(OpenLoopId),
    #[error("checkpoint {0} already exists with different content")]
    ConflictingCheckpoint(CheckpointId),
}
