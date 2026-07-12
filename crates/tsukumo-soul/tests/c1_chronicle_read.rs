//! Bounded Chronicle authority-read contract tests.

use tempfile::tempdir;
use tsukumo_kernel::{
    CheckpointId, CorrelationId, EventId, ExecutionId, KernelEvent, KernelEventPayload,
    PersistedText, ProjectionId, QuestId, RuntimeBinding, RuntimeKind, RuntimeMode, RuntimePhase,
    SessionId, SpiritId, Timestamp, VendorEventRef, KERNEL_EVENT_SCHEMA_VERSION,
};
use tsukumo_soul::{SoulError, SoulStore};

fn attributed_event(
    id: &str,
    timestamp: i64,
    execution_id: &str,
    payload: KernelEventPayload,
) -> KernelEvent {
    KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new(id),
        occurred_at: Timestamp::from_unix_millis(timestamp),
        quest_id: QuestId::new("quest-authority"),
        session_id: SessionId::new("session-authority"),
        spirit_id: SpiritId::new("yuka"),
        execution_id: Some(ExecutionId::new(execution_id)),
        runtime: Some(RuntimeBinding::new(
            RuntimeKind::CodexCli,
            RuntimeMode::Fixture,
        )),
        causation_id: None,
        correlation_id: Some(CorrelationId::new(format!("correlation-{id}"))),
        payload,
    }
}

fn checkpoint_event(id: &str, timestamp: i64, version: u64) -> KernelEvent {
    KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new(id),
        occurred_at: Timestamp::from_unix_millis(timestamp),
        quest_id: QuestId::new("quest-authority"),
        session_id: SessionId::new("session-authority"),
        spirit_id: SpiritId::new("yuka"),
        execution_id: None,
        runtime: None,
        causation_id: None,
        correlation_id: None,
        payload: KernelEventPayload::CheckpointCreated {
            checkpoint_id: CheckpointId::new(format!("checkpoint-{version}")),
            version,
        },
    }
}

#[test]
fn permission_authority_when_event_budget_is_exceeded_returns_typed_receipt() {
    // Given: one durable normalized permission request.
    let directory = tempdir().expect("create permission authority directory");
    let mut store = SoulStore::open(directory.path()).expect("open Chronicle");
    store
        .append_event(&attributed_event(
            "event-permission-authority",
            100,
            "execution-permission",
            KernelEventPayload::PermissionRequested {
                vendor_request: VendorEventRef::new("fixture", "request-1"),
                tool: "shell".into(),
                arguments: None,
                cwd: None,
                risk_reasons: Vec::new(),
                reason: PersistedText::from_reviewed("approval required"),
            },
        ))
        .expect("append permission request");

    // When: the authority reader receives an event budget of zero.
    let error = store
        .replay_permission_events(0, usize::MAX)
        .expect_err("permission authority budget must fail closed");

    // Then: the typed receipt reports the exact event budget overrun.
    assert!(matches!(
        error,
        SoulError::ChronicleReadBudgetExceeded {
            event_count: 1,
            maximum_events: 0,
            ..
        }
    ));
}

#[test]
fn latest_projection_when_execution_is_selected_uses_its_newest_event() {
    // Given: projections from two executions interleaved in Chronicle order.
    let directory = tempdir().expect("create projection authority directory");
    let mut store = SoulStore::open(directory.path()).expect("open Chronicle");
    for (id, timestamp, execution, projection) in [
        ("event-projection-a", 100, "execution-a", "projection-a"),
        ("event-projection-b", 101, "execution-b", "projection-b"),
    ] {
        store
            .append_event(&attributed_event(
                id,
                timestamp,
                execution,
                KernelEventPayload::ProjectionCreated {
                    projection_id: ProjectionId::new(projection),
                    checkpoint_id: CheckpointId::new("checkpoint-authority"),
                },
            ))
            .expect("append projection event");
    }

    // When: the authority reader selects execution-a directly.
    let selected = store
        .latest_projection_event(Some(&ExecutionId::new("execution-a")))
        .expect("read selected projection")
        .expect("selected projection exists");

    // Then: a newer event from another execution cannot replace it.
    assert_eq!(selected.event.event_id.as_str(), "event-projection-a");
}

#[test]
fn latest_checkpoint_when_read_uses_chronicle_sequence() {
    // Given: two durable checkpoints in Chronicle order.
    let directory = tempdir().expect("create checkpoint authority directory");
    let mut store = SoulStore::open(directory.path()).expect("open Chronicle");
    store
        .append_event(&checkpoint_event("event-checkpoint-1", 200, 1))
        .expect("append first checkpoint");
    store
        .append_event(&checkpoint_event("event-checkpoint-2", 100, 2))
        .expect("append second checkpoint");

    // When: the newest checkpoint authority is read.
    let selected = store
        .latest_checkpoint_event()
        .expect("read latest checkpoint")
        .expect("latest checkpoint exists");

    // Then: append sequence wins over the event timestamp.
    assert_eq!(selected.event.event_id.as_str(), "event-checkpoint-2");
}

#[test]
fn latest_runtime_status_when_read_returns_coherent_coordinates() {
    // Given: two attributed runtime lifecycle events in Chronicle order.
    let directory = tempdir().expect("create runtime authority directory");
    let mut store = SoulStore::open(directory.path()).expect("open Chronicle");
    store
        .append_event(&attributed_event(
            "event-runtime-a",
            200,
            "execution-a",
            KernelEventPayload::RuntimeLifecycle {
                phase: RuntimePhase::Started,
            },
        ))
        .expect("append first runtime event");
    store
        .append_event(&attributed_event(
            "event-runtime-b",
            100,
            "execution-b",
            KernelEventPayload::RuntimeLifecycle {
                phase: RuntimePhase::Completed,
            },
        ))
        .expect("append second runtime event");

    // When: the latest coherent runtime event is read.
    let selected = store
        .latest_runtime_status_event()
        .expect("read latest runtime status")
        .expect("latest runtime status exists");

    // Then: execution and runtime stay sourced from the same durable envelope.
    assert_eq!(
        selected
            .event
            .execution_id
            .as_ref()
            .map(ExecutionId::as_str),
        Some("execution-b")
    );
    assert_eq!(
        selected.event.runtime,
        Some(RuntimeBinding::new(
            RuntimeKind::CodexCli,
            RuntimeMode::Fixture,
        ))
    );
}
