//! Tsukumo kernel: durable identities and normalized event contracts.
//!
//! Adapters produce KernelEventPayload values. A host assigns the durable
//! KernelEvent envelope before Chronicle or theater consumes the event.

pub mod event;
mod event_validation;
pub mod identity;
mod redaction;
pub mod session;
pub mod value;

pub use event::{
    KernelEvent, KernelEventPayload, OutcomeStatus, PermissionDecision, RuntimePhase,
    StateLifecycleAction, ToolResult, VendorEventRef, KERNEL_EVENT_SCHEMA_VERSION,
};
pub use event_validation::{validate_kernel_event, EventContractError};
pub use identity::{
    ArtifactId, CheckpointId, CorrelationId, EventId, ExecutionId, OwnerId, ProjectionId, QuestId,
    RuntimeBinding, RuntimeKind, RuntimeMode, SessionId, SpiritId, StateId, WorkspaceId,
};
pub use redaction::{
    contains_sensitive_material, contains_unredacted_sensitive_json, is_terminal_unsafe_character,
    redact_sensitive_text, sanitize_untrusted_json,
};
pub use session::{
    parse_jsonl_line, read_jsonl_events, read_jsonl_reader, EventDecodeError, SessionError,
};
pub use value::{PersistedJson, PersistedText, SensitiveText, Timestamp};
