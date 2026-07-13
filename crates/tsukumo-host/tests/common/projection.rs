//! Receipt-committed projection fixture for Host integration tests.

use tempfile::TempDir;
use tsukumo_kernel::{
    CheckpointId, CorrelationId, EventId, ExecutionId, KernelEvent, KernelEventPayload,
    PersistedText, ProjectionId, QuestId, RuntimeBinding, RuntimeKind, RuntimeMode, SessionId,
    SpiritId, Timestamp, KERNEL_EVENT_SCHEMA_VERSION,
};
use tsukumo_soul::{
    CheckpointTrigger, CheckpointWriteRequest, HandoffCheckpoint, OperatingSystem,
    PreparedProjection, ProjectionRequest, ProjectionTarget, ProjectionWriteRequest, SoulStore,
    StateScope,
};

pub fn prepared_fixture() -> (TempDir, SoulStore, PreparedProjection) {
    prepared_fixture_with_goal("Run the reviewed Host fixture")
}

pub fn prepared_fixture_with_goal(goal: &str) -> (TempDir, SoulStore, PreparedProjection) {
    let directory = tempfile::tempdir().expect("create host fixture directory");
    let mut store = SoulStore::open(directory.path()).expect("open host fixture store");
    let source = base_event(
        "event-host-source",
        100,
        KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed("Continue the Tsukumo Host MVP"),
        },
    );
    store.append_event(&source).expect("append host source");
    let checkpoint = HandoffCheckpoint::new(
        CheckpointId::new("checkpoint-host"),
        QuestId::new("quest-host"),
        1,
        None,
        PersistedText::from_reviewed("Host process contract is ready"),
        Timestamp::from_unix_millis(101),
        CheckpointTrigger::Milestone,
    )
    .with_source_event_refs(vec![source.event_id]);
    store
        .save_checkpoint(CheckpointWriteRequest::new(
            checkpoint.clone(),
            base_event(
                "event-host-checkpoint",
                101,
                KernelEventPayload::CheckpointCreated {
                    checkpoint_id: checkpoint.id.clone(),
                    version: checkpoint.version,
                },
            ),
        ))
        .expect("save host checkpoint");
    let runtime = RuntimeBinding::new(RuntimeKind::ClaudeCli, RuntimeMode::OwnedProcess);
    let request = ProjectionRequest::new(
        ProjectionTarget::new(
            ProjectionId::new("projection-host"),
            ExecutionId::new("execution-host"),
            runtime.clone(),
            checkpoint.id.clone(),
        ),
        StateScope::workspace_os("tsukumo", OperatingSystem::Windows),
        tsukumo_kernel::SensitiveText::new(goal),
        Timestamp::from_unix_millis(102),
        2_000,
    );
    let projection_event = KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new("event-host-projection"),
        occurred_at: Timestamp::from_unix_millis(102),
        quest_id: QuestId::new("quest-host"),
        session_id: SessionId::new("session-host"),
        spirit_id: SpiritId::new("yuka"),
        execution_id: Some(request.execution_id.clone()),
        runtime: Some(runtime),
        causation_id: None,
        correlation_id: Some(CorrelationId::new("correlation-host-projection")),
        payload: KernelEventPayload::ProjectionCreated {
            projection_id: request.projection_id.clone(),
            checkpoint_id: checkpoint.id,
        },
    };
    let prepared = store
        .prepare_projection(ProjectionWriteRequest::new(request, projection_event))
        .expect("prepare host projection");
    (directory, store, prepared)
}

pub fn prepared_dual_runtime_live_fixture(
    goal: &str,
) -> (TempDir, SoulStore, PreparedProjection, PreparedProjection) {
    let (directory, mut store, claude) = prepared_fixture_with_goal(goal);
    let runtime = RuntimeBinding::new(RuntimeKind::CodexCli, RuntimeMode::OwnedProcess);
    let request = ProjectionRequest::new(
        ProjectionTarget::new(
            ProjectionId::new("projection-codex-live"),
            ExecutionId::new("execution-codex-live"),
            runtime.clone(),
            claude.receipt.checkpoint_id.clone(),
        ),
        StateScope::workspace_os("tsukumo", OperatingSystem::Windows),
        tsukumo_kernel::SensitiveText::new(goal),
        Timestamp::from_unix_millis(103),
        2_000,
    );
    let event = KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new("event-codex-live-projection"),
        occurred_at: request.created_at,
        quest_id: QuestId::new("quest-host"),
        session_id: SessionId::new("session-host"),
        spirit_id: SpiritId::new("yuka"),
        execution_id: Some(request.execution_id.clone()),
        runtime: Some(runtime),
        causation_id: None,
        correlation_id: Some(CorrelationId::new("correlation-codex-live-projection")),
        payload: KernelEventPayload::ProjectionCreated {
            projection_id: request.projection_id.clone(),
            checkpoint_id: request.checkpoint_id.clone(),
        },
    };
    let codex = store
        .prepare_projection(ProjectionWriteRequest::new(request, event))
        .expect("prepare Codex live projection");
    (directory, store, claude, codex)
}

fn base_event(id: &str, timestamp: i64, payload: KernelEventPayload) -> KernelEvent {
    KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new(id),
        occurred_at: Timestamp::from_unix_millis(timestamp),
        quest_id: QuestId::new("quest-host"),
        session_id: SessionId::new("session-host"),
        spirit_id: SpiritId::new("yuka"),
        execution_id: None,
        runtime: None,
        causation_id: None,
        correlation_id: None,
        payload,
    }
}
