//! Durable projection receipt and in-memory preparation vocabulary.

use crate::handoff_model::StateRef;
use crate::state_model::StateScope;
use serde::{Deserialize, Serialize};
use tsukumo_kernel::{
    CheckpointId, ExecutionId, KernelEvent, ProjectionId, RuntimeBinding, SensitiveText, StateId,
    Timestamp,
};

/// Current production projection contract version.
pub const PROJECTION_VERSION: u16 = 1;
/// Current canonical renderer contract version.
pub const RENDERER_VERSION: u16 = 1;

/// Digest algorithm recorded with every durable hash.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DigestAlgorithm {
    Sha256,
}

/// Content digest with an explicit algorithm.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ContentDigest {
    pub algorithm: DigestAlgorithm,
    pub value: String,
}

/// Canonical renderer section identity.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionSection {
    Header,
    Goal,
    Progress,
    Decisions,
    Constraints,
    Artifacts,
    OpenLoops,
    NextActions,
    DelegationGoal,
}

/// Digest and exact size of one canonical section.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionSectionDigest {
    pub section: ProjectionSection,
    pub digest: ContentDigest,
    pub byte_count: usize,
    pub char_count: usize,
}

/// Honest unit used for V0 projection admission.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum BudgetUnit {
    Characters,
}

/// Projection capacity consumed by the exact rendered prompt.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionBudgetUsage {
    pub used: usize,
    pub limit: usize,
    pub unit: BudgetUnit,
}

/// Deterministic reason why one state did not enter the projection.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionOmissionReason {
    ExcludedByComparison,
    ScopeMismatch,
    Inactive,
    BudgetExceeded,
}

/// One considered state omitted from the final projection.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionOmission {
    pub state_id: StateId,
    pub reason: ProjectionOmissionReason,
}

/// Metadata-only redaction record without the removed value.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RedactionRecord {
    pub location: String,
    pub category: String,
    pub action: String,
}

/// Immutable receipt proving the inputs and bytes selected for one runtime.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionReceipt {
    pub id: ProjectionId,
    pub execution_id: ExecutionId,
    pub checkpoint_id: CheckpointId,
    pub runtime: RuntimeBinding,
    pub selected_state_refs: Vec<StateRef>,
    pub projection_version: u16,
    pub renderer_version: u16,
    pub rendered_digest: ContentDigest,
    pub rendered_byte_count: usize,
    pub rendered_char_count: usize,
    pub sections: Vec<ProjectionSectionDigest>,
    pub budget: ProjectionBudgetUsage,
    pub omissions: Vec<ProjectionOmission>,
    pub redactions: Vec<RedactionRecord>,
    pub created_at: Timestamp,
}

/// Stable runtime target coordinates for one projection.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectionTarget {
    pub projection_id: ProjectionId,
    pub execution_id: ExecutionId,
    pub runtime: RuntimeBinding,
    pub checkpoint_id: CheckpointId,
}

impl ProjectionTarget {
    pub const fn new(
        projection_id: ProjectionId,
        execution_id: ExecutionId,
        runtime: RuntimeBinding,
        checkpoint_id: CheckpointId,
    ) -> Self {
        Self {
            projection_id,
            execution_id,
            runtime,
            checkpoint_id,
        }
    }
}
/// Inputs for deterministic selection and rendering.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProjectionRequest {
    pub projection_id: ProjectionId,
    pub execution_id: ExecutionId,
    pub runtime: RuntimeBinding,
    pub checkpoint_id: CheckpointId,
    pub scope: StateScope,
    pub delegation_goal: SensitiveText,
    pub created_at: Timestamp,
    pub budget_chars: usize,
    pub excluded_state_ids: Vec<StateId>,
}

impl ProjectionRequest {
    pub fn new(
        target: ProjectionTarget,
        scope: StateScope,
        delegation_goal: SensitiveText,
        created_at: Timestamp,
        budget_chars: usize,
    ) -> Self {
        Self {
            projection_id: target.projection_id,
            execution_id: target.execution_id,
            runtime: target.runtime,
            checkpoint_id: target.checkpoint_id,
            scope,
            delegation_goal,
            created_at,
            budget_chars,
            excluded_state_ids: Vec::new(),
        }
    }

    pub fn excluding_states(mut self, state_ids: Vec<StateId>) -> Self {
        self.excluded_state_ids = state_ids;
        self
    }
}

/// Atomic projection inputs plus the matching Chronicle event.
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectionWriteRequest {
    pub request: ProjectionRequest,
    pub event: KernelEvent,
}

impl ProjectionWriteRequest {
    pub const fn new(request: ProjectionRequest, event: KernelEvent) -> Self {
        Self { request, event }
    }
}

/// Receipt-committed prompt value that alone may reach a runtime host.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreparedProjection {
    pub receipt: ProjectionReceipt,
    rendered_prompt: SensitiveText,
}

impl PreparedProjection {
    pub(crate) const fn new(receipt: ProjectionReceipt, rendered_prompt: SensitiveText) -> Self {
        Self {
            receipt,
            rendered_prompt,
        }
    }

    pub const fn rendered_prompt(&self) -> &SensitiveText {
        &self.rendered_prompt
    }
}
