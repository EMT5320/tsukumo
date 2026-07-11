//! Shared deterministic fixtures for C1 handoff/projection integration tests.

use tsukumo_kernel::{
    CheckpointId, CorrelationId, EventId, ExecutionId, KernelEvent, KernelEventPayload,
    PersistedText, ProjectionId, QuestId, RuntimeBinding, RuntimeKind, RuntimeMode, SensitiveText,
    SessionId, SpiritId, StateId, StateLifecycleAction, Timestamp, KERNEL_EVENT_SCHEMA_VERSION,
};
use tsukumo_soul::{
    ExtractionContext, HandoffCheckpoint, OperatingSystem, ProjectionRequest, ProjectionTarget,
    RuleStateExtractor, SoulStore, StateExtractor, StateRecord, StateScope, StateTransition,
    StateWriteOutcome, StateWriteRequest,
};

pub fn event(id: &str, timestamp: i64, payload: KernelEventPayload) -> KernelEvent {
    KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new(id),
        occurred_at: Timestamp::from_unix_millis(timestamp),
        quest_id: QuestId::new("quest-projection"),
        session_id: SessionId::new("session-projection"),
        spirit_id: SpiritId::new("yuka"),
        execution_id: None,
        runtime: None,
        causation_id: None,
        correlation_id: None,
        payload,
    }
}

pub fn checkpoint_event(checkpoint: &HandoffCheckpoint) -> KernelEvent {
    event(
        &format!("event-{}", checkpoint.id.as_str()),
        checkpoint.created_at.as_unix_millis(),
        KernelEventPayload::CheckpointCreated {
            checkpoint_id: checkpoint.id.clone(),
            version: checkpoint.version,
        },
    )
}

pub fn projection_event(
    id: &str,
    timestamp: i64,
    projection_id: &ProjectionId,
    checkpoint_id: &CheckpointId,
    execution_id: &ExecutionId,
    runtime: &RuntimeBinding,
) -> KernelEvent {
    KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new(id),
        occurred_at: Timestamp::from_unix_millis(timestamp),
        quest_id: QuestId::new("quest-projection"),
        session_id: SessionId::new("session-projection"),
        spirit_id: SpiritId::new("yuka"),
        execution_id: Some(execution_id.clone()),
        runtime: Some(runtime.clone()),
        causation_id: None,
        correlation_id: Some(CorrelationId::new(format!("correlation-{id}"))),
        payload: KernelEventPayload::ProjectionCreated {
            projection_id: projection_id.clone(),
            checkpoint_id: checkpoint_id.clone(),
        },
    }
}

pub fn projection_request(
    projection_id: &str,
    execution_id: &str,
    checkpoint_id: &CheckpointId,
    goal: &str,
) -> ProjectionRequest {
    ProjectionRequest::new(
        ProjectionTarget::new(
            ProjectionId::new(projection_id),
            ExecutionId::new(execution_id),
            RuntimeBinding::new(RuntimeKind::CodexCli, RuntimeMode::Fixture),
            checkpoint_id.clone(),
        ),
        StateScope::workspace_os("tsukumo", OperatingSystem::Windows),
        SensitiveText::new(goal),
        Timestamp::from_unix_millis(500),
        2_000,
    )
}

pub fn persist_gnu_constraint(
    store: &mut SoulStore,
    state_id: &str,
    event_suffix: &str,
    scope: StateScope,
    timestamp: i64,
) -> StateRecord {
    let source = event(
        &format!("event-source-{event_suffix}"),
        timestamp,
        KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed(
                "Tsukumo always uses the GNU Rust toolchain on Windows",
            ),
        },
    );
    let draft = RuleStateExtractor
        .extract(&ExtractionContext {
            event: &source,
            scope,
        })
        .expect("extract deterministic GNU state")
        .into_iter()
        .next()
        .expect("GNU rule yields one state draft");
    let state_id = StateId::new(state_id);
    let lifecycle = event(
        &format!("event-state-{event_suffix}"),
        timestamp + 1,
        KernelEventPayload::StateLifecycle {
            state_id: state_id.clone(),
            action: StateLifecycleAction::Created,
            prior_state_id: None,
            reason: None,
        },
    );
    match store
        .apply_state(
            StateWriteRequest::new(
                StateTransition::Create {
                    state_id,
                    draft,
                    created_at: Timestamp::from_unix_millis(timestamp + 1),
                },
                lifecycle,
            )
            .with_source_event(source),
        )
        .expect("persist deterministic GNU state")
    {
        StateWriteOutcome::Created(record) | StateWriteOutcome::Unchanged(record) => record,
        StateWriteOutcome::Superseded(_) | StateWriteOutcome::Revoked(_) => {
            panic!("new GNU fixture must create an active state")
        }
    }
}
