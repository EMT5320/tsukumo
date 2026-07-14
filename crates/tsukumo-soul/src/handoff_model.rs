//! Versioned task-state vocabulary for runtime handoff checkpoints.

use crate::handoff_loop::{OpenLoop, OpenLoopTransition};
use serde::{Deserialize, Serialize};
use tsukumo_kernel::{
    ArtifactId, CheckpointId, EventId, KernelEvent, PersistedText, QuestId, StateId, Timestamp,
};

/// Stable reference to one immutable canonical state version.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct StateRef {
    pub state_id: StateId,
    pub version: u64,
}

impl StateRef {
    pub const fn new(state_id: StateId, version: u64) -> Self {
        Self { state_id, version }
    }
}

/// Low-frequency reason for compiling a checkpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CheckpointTrigger {
    RuntimeSwitch,
    ContextCompression,
    Pause,
    Milestone,
    Completion,
    UserRequest,
}

/// User-visible progress state recorded in a checkpoint.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProgressStatus {
    Planned,
    InProgress,
    Completed,
    Blocked,
}

/// One progress statement in the handoff.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProgressItem {
    pub summary: PersistedText,
    pub status: ProgressStatus,
}

impl ProgressItem {
    pub const fn new(summary: PersistedText, status: ProgressStatus) -> Self {
        Self { summary, status }
    }
}

/// One durable task decision.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Decision {
    pub summary: PersistedText,
}

impl Decision {
    pub const fn new(summary: PersistedText) -> Self {
        Self { summary }
    }
}

/// One artifact needed by the next runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactReference {
    pub artifact_id: ArtifactId,
    pub location: PersistedText,
}

impl ArtifactReference {
    pub const fn new(artifact_id: ArtifactId, location: PersistedText) -> Self {
        Self {
            artifact_id,
            location,
        }
    }
}

/// One recommended next action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NextAction {
    pub summary: PersistedText,
}

impl NextAction {
    pub const fn new(summary: PersistedText) -> Self {
        Self { summary }
    }
}

/// Immutable task-state handoff compiled for a runtime boundary.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct HandoffCheckpoint {
    pub id: CheckpointId,
    pub quest_id: QuestId,
    pub version: u64,
    pub previous_id: Option<CheckpointId>,
    pub goal: PersistedText,
    pub progress: Vec<ProgressItem>,
    pub decisions: Vec<Decision>,
    pub constraint_refs: Vec<StateRef>,
    pub artifacts: Vec<ArtifactReference>,
    pub open_loops: Vec<OpenLoop>,
    pub open_loop_transitions: Vec<OpenLoopTransition>,
    pub next_actions: Vec<NextAction>,
    pub source_event_refs: Vec<EventId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub registration_digest: Option<String>,
    pub created_at: Timestamp,
    pub trigger: CheckpointTrigger,
}

impl HandoffCheckpoint {
    pub fn new(
        id: CheckpointId,
        quest_id: QuestId,
        version: u64,
        previous_id: Option<CheckpointId>,
        goal: PersistedText,
        created_at: Timestamp,
        trigger: CheckpointTrigger,
    ) -> Self {
        Self {
            id,
            quest_id,
            version,
            previous_id,
            goal,
            progress: Vec::new(),
            decisions: Vec::new(),
            constraint_refs: Vec::new(),
            artifacts: Vec::new(),
            open_loops: Vec::new(),
            open_loop_transitions: Vec::new(),
            next_actions: Vec::new(),
            source_event_refs: Vec::new(),
            registration_digest: None,
            created_at,
            trigger,
        }
    }

    pub fn with_progress(mut self, progress: Vec<ProgressItem>) -> Self {
        self.progress = progress;
        self
    }

    pub fn with_decisions(mut self, decisions: Vec<Decision>) -> Self {
        self.decisions = decisions;
        self
    }

    pub fn with_constraint_refs(mut self, refs: Vec<StateRef>) -> Self {
        self.constraint_refs = refs;
        self
    }

    pub fn with_artifacts(mut self, artifacts: Vec<ArtifactReference>) -> Self {
        self.artifacts = artifacts;
        self
    }

    pub fn with_open_loops(mut self, open_loops: Vec<OpenLoop>) -> Self {
        self.open_loops = open_loops;
        self
    }

    pub fn with_open_loop_transitions(mut self, transitions: Vec<OpenLoopTransition>) -> Self {
        self.open_loop_transitions = transitions;
        self
    }

    pub fn with_next_actions(mut self, next_actions: Vec<NextAction>) -> Self {
        self.next_actions = next_actions;
        self
    }

    pub fn with_source_event_refs(mut self, refs: Vec<EventId>) -> Self {
        self.source_event_refs = refs;
        self
    }

    /// Attaches a non-rendered digest that freezes reviewed registration metadata.
    pub fn with_registration_digest(mut self, registration_digest: String) -> Self {
        self.registration_digest = Some(registration_digest);
        self
    }
}

/// Atomic checkpoint plus its Chronicle creation event.
#[derive(Debug, Clone, PartialEq)]
pub struct CheckpointWriteRequest {
    pub checkpoint: HandoffCheckpoint,
    pub event: KernelEvent,
}

impl CheckpointWriteRequest {
    pub const fn new(checkpoint: HandoffCheckpoint, event: KernelEvent) -> Self {
        Self { checkpoint, event }
    }
}
