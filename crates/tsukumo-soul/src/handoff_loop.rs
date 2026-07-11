//! Stable open-loop identity and transition vocabulary for checkpoint versions.

use serde::{Deserialize, Serialize};
use std::fmt;
use tsukumo_kernel::PersistedText;

/// Stable identifier for one open task loop across checkpoint versions.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct OpenLoopId(String);

impl OpenLoopId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for OpenLoopId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}

/// One unresolved task loop.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenLoop {
    pub id: OpenLoopId,
    pub summary: PersistedText,
}

impl OpenLoop {
    pub const fn new(id: OpenLoopId, summary: PersistedText) -> Self {
        Self { id, summary }
    }
}

/// Resolution of one loop from the previous checkpoint version.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum OpenLoopOutcome {
    Inherited,
    Completed,
    Abandoned,
    ReplacedBy { replacement: OpenLoopId },
}

/// Explicit transition for one prior open loop.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OpenLoopTransition {
    pub prior: OpenLoopId,
    pub outcome: OpenLoopOutcome,
}

impl OpenLoopTransition {
    pub const fn inherited(prior: OpenLoopId) -> Self {
        Self {
            prior,
            outcome: OpenLoopOutcome::Inherited,
        }
    }

    pub const fn completed(prior: OpenLoopId) -> Self {
        Self {
            prior,
            outcome: OpenLoopOutcome::Completed,
        }
    }

    pub const fn abandoned(prior: OpenLoopId) -> Self {
        Self {
            prior,
            outcome: OpenLoopOutcome::Abandoned,
        }
    }

    pub const fn replaced_by(prior: OpenLoopId, replacement: OpenLoopId) -> Self {
        Self {
            prior,
            outcome: OpenLoopOutcome::ReplacedBy { replacement },
        }
    }
}
