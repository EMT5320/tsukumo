//! Versioned, vendor-neutral Chronicle event contract.
//!
//! Adapters create KernelEventPayload values. The host assigns the durable
//! envelope fields before the event can enter Chronicle, replay, or theater.

use crate::identity::{
    CheckpointId, CorrelationId, EventId, ExecutionId, ProjectionId, QuestId, RuntimeBinding,
    SessionId, SpiritId, StateId,
};
use crate::value::{PersistedJson, PersistedText, Timestamp};
use serde::{Deserialize, Serialize};

/// Current wire version for KernelEvent.
pub const KERNEL_EVENT_SCHEMA_VERSION: u16 = 1;

/// Namespaced identifier retained from a vendor runtime event.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct VendorEventRef {
    pub namespace: String,
    pub id: String,
}

impl VendorEventRef {
    /// Creates a vendor reference without reusing it as a global event ID.
    pub fn new(namespace: impl Into<String>, id: impl Into<String>) -> Self {
        Self {
            namespace: namespace.into(),
            id: id.into(),
        }
    }
}

/// Normalized tool outcome suitable for Chronicle and theater.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolResult {
    pub summary: PersistedText,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub data: Option<PersistedJson>,
}

impl ToolResult {
    /// Creates a text-only result after boundary review.
    pub fn reviewed_text(summary: impl Into<String>) -> Self {
        Self {
            summary: PersistedText::from_reviewed(summary),
            data: None,
        }
    }
}

/// Runtime lifecycle phase recorded by the host.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum RuntimePhase {
    Starting,
    Started,
    Stopping,
    Completed,
    Failed,
    Cancelled,
}

/// User decision for one permission request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PermissionDecision {
    AllowOnce,
    AllowSession,
    Deny,
}

/// Canonical state lifecycle action exposed as Chronicle evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StateLifecycleAction {
    Created,
    Superseded,
    Revoked,
}

/// Terminal execution or quest outcome.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum OutcomeStatus {
    Succeeded,
    Failed,
    Cancelled,
    PermissionDenied,
    SafetyUnsupported,
    Degraded,
    TimedOut,
    MalformedOutput,
    NonZeroExit,
    LaunchFailed,
}

/// Vendor-neutral facts emitted by adapters, host, and state persistence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum KernelEventPayload {
    UserInput {
        content: PersistedText,
    },
    LegacyImported {
        source_id: String,
        kind: String,
        content: PersistedText,
    },
    RuntimeLifecycle {
        phase: RuntimePhase,
    },
    RuntimeSwitched {
        previous: Option<RuntimeBinding>,
        current: RuntimeBinding,
    },
    ToolStart {
        vendor_call: VendorEventRef,
        tool: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        args: Option<PersistedJson>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        projection_id: Option<ProjectionId>,
    },
    ToolEnd {
        vendor_call: VendorEventRef,
        result: ToolResult,
        is_error: bool,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        projection_id: Option<ProjectionId>,
    },
    PermissionRequested {
        vendor_request: VendorEventRef,
        tool: String,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        arguments: Option<PersistedJson>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        cwd: Option<PersistedText>,
        #[serde(default, skip_serializing_if = "Vec::is_empty")]
        risk_reasons: Vec<PersistedText>,
        reason: PersistedText,
    },
    PermissionDecided {
        vendor_request: VendorEventRef,
        decision: PermissionDecision,
    },
    StateLifecycle {
        state_id: StateId,
        action: StateLifecycleAction,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        prior_state_id: Option<StateId>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        reason: Option<PersistedText>,
    },
    CheckpointCreated {
        checkpoint_id: CheckpointId,
        version: u64,
    },
    ProjectionCreated {
        projection_id: ProjectionId,
        checkpoint_id: CheckpointId,
    },
    Outcome {
        status: OutcomeStatus,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        summary: Option<PersistedText>,
        #[serde(default, skip_serializing_if = "Option::is_none")]
        projection_id: Option<ProjectionId>,
    },
    Error {
        message: PersistedText,
        recoverable: bool,
    },
}

/// Durable event envelope shared by live input, Chronicle, replay, and tests.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct KernelEvent {
    pub schema_version: u16,
    pub event_id: EventId,
    pub occurred_at: Timestamp,
    pub quest_id: QuestId,
    pub session_id: SessionId,
    pub spirit_id: SpiritId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_id: Option<ExecutionId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub runtime: Option<RuntimeBinding>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub causation_id: Option<EventId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub correlation_id: Option<CorrelationId>,
    pub payload: KernelEventPayload,
}
