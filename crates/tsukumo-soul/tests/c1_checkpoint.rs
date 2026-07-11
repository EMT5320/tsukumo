//! Contract tests for versioned handoff checkpoints and open-loop continuity.

use tempfile::tempdir;
use tsukumo_kernel::{
    ArtifactId, CheckpointId, EventId, KernelEvent, KernelEventPayload, PersistedText, QuestId,
    SessionId, SpiritId, StateId, Timestamp, KERNEL_EVENT_SCHEMA_VERSION,
};
use tsukumo_soul::{
    ArtifactReference, CheckpointTrigger, CheckpointWriteRequest, Decision, HandoffCheckpoint,
    HandoffError, NextAction, OpenLoop, OpenLoopId, OpenLoopTransition, ProgressItem,
    ProgressStatus, SoulError, SoulStore, StateRef,
};

fn event(id: &str, timestamp: i64, payload: KernelEventPayload) -> KernelEvent {
    KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new(id),
        occurred_at: Timestamp::from_unix_millis(timestamp),
        quest_id: QuestId::new("quest-checkpoint"),
        session_id: SessionId::new("session-checkpoint"),
        spirit_id: SpiritId::new("yuka"),
        execution_id: None,
        runtime: None,
        causation_id: None,
        correlation_id: None,
        payload,
    }
}

fn checkpoint_event(id: &str, timestamp: i64, checkpoint: &HandoffCheckpoint) -> KernelEvent {
    event(
        id,
        timestamp,
        KernelEventPayload::CheckpointCreated {
            checkpoint_id: checkpoint.id.clone(),
            version: checkpoint.version,
        },
    )
}

fn first_checkpoint(source: &KernelEvent, created_at: i64) -> HandoffCheckpoint {
    HandoffCheckpoint::new(
        CheckpointId::new("checkpoint-1"),
        QuestId::new("quest-checkpoint"),
        1,
        None,
        PersistedText::from_reviewed("Ship the first Tsukumo handoff"),
        Timestamp::from_unix_millis(created_at),
        CheckpointTrigger::RuntimeSwitch,
    )
    .with_progress(vec![ProgressItem::new(
        PersistedText::from_reviewed("Chronicle and StateWriter are complete"),
        ProgressStatus::Completed,
    )])
    .with_decisions(vec![Decision::new(PersistedText::from_reviewed(
        "SQLite remains the durable authority",
    ))])
    .with_artifacts(vec![ArtifactReference::new(
        ArtifactId::new("artifact-design"),
        PersistedText::from_reviewed("DESIGN.md"),
    )])
    .with_open_loops(vec![
        OpenLoop::new(
            OpenLoopId::new("loop-projection"),
            PersistedText::from_reviewed("Implement projection receipts"),
        ),
        OpenLoop::new(
            OpenLoopId::new("loop-host"),
            PersistedText::from_reviewed("Connect the runtime host"),
        ),
    ])
    .with_next_actions(vec![NextAction::new(PersistedText::from_reviewed(
        "Implement deterministic projection",
    ))])
    .with_source_event_refs(vec![source.event_id.clone()])
}

#[test]
fn checkpoint_reopens_after_complete_open_loop_transitions() {
    // Given: a persisted source event and a first checkpoint with two open loops.
    let directory = tempdir().expect("create checkpoint test directory");
    let mut store = SoulStore::open(directory.path()).expect("open checkpoint store");
    let source = event(
        "event-source-1",
        100,
        KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed("Continue the Tsukumo MVP"),
        },
    );
    store.append_event(&source).expect("append source event");
    let first = first_checkpoint(&source, source.occurred_at.as_unix_millis() + 1);
    store
        .save_checkpoint(CheckpointWriteRequest::new(
            first.clone(),
            checkpoint_event("event-checkpoint-1", 101, &first),
        ))
        .expect("save first checkpoint");

    // Given: content-identical retry under another EventId cannot claim a new append.
    let retry_event = checkpoint_event("event-checkpoint-retry", 101, &first);
    let retry_event_id = retry_event.event_id.clone();
    let retry_error = store
        .save_checkpoint(CheckpointWriteRequest::new(first.clone(), retry_event))
        .expect_err("checkpoint retry must keep its original Chronicle EventId");
    assert!(matches!(
        retry_error,
        SoulError::Handoff(HandoffError::ConflictingCheckpoint(ref id)) if id == &first.id
    ));
    assert!(store
        .event(&retry_event_id)
        .expect("query rejected checkpoint retry event")
        .is_none());

    // When: version two inherits one loop and replaces the other explicitly.
    let replacement = OpenLoopId::new("loop-codex");
    let second = HandoffCheckpoint::new(
        CheckpointId::new("checkpoint-2"),
        QuestId::new("quest-checkpoint"),
        2,
        Some(first.id.clone()),
        PersistedText::from_reviewed("Complete cross-runtime continuity"),
        Timestamp::from_unix_millis(102),
        CheckpointTrigger::Milestone,
    )
    .with_open_loops(vec![
        OpenLoop::new(
            OpenLoopId::new("loop-projection"),
            PersistedText::from_reviewed("Implement projection receipts"),
        ),
        OpenLoop::new(
            replacement.clone(),
            PersistedText::from_reviewed("Add the Codex runtime"),
        ),
    ])
    .with_open_loop_transitions(vec![
        OpenLoopTransition::inherited(OpenLoopId::new("loop-projection")),
        OpenLoopTransition::replaced_by(OpenLoopId::new("loop-host"), replacement),
    ])
    .with_source_event_refs(vec![source.event_id.clone()]);
    store
        .save_checkpoint(CheckpointWriteRequest::new(
            second.clone(),
            checkpoint_event("event-checkpoint-2", 102, &second),
        ))
        .expect("save second checkpoint");
    drop(store);

    // Then: reopening preserves the immutable version and every transition.
    let reopened = SoulStore::open(directory.path()).expect("reopen checkpoint store");
    let loaded = reopened
        .checkpoint(&second.id)
        .expect("load checkpoint")
        .expect("checkpoint exists");
    assert_eq!(loaded, second);
    assert_eq!(loaded.open_loop_transitions.len(), 2);
    assert_eq!(loaded.source_event_refs, vec![source.event_id]);
}

#[test]
fn checkpoint_rejects_a_silently_dropped_open_loop() {
    // Given: a prior checkpoint containing one unresolved loop.
    let directory = tempdir().expect("create checkpoint test directory");
    let mut store = SoulStore::open(directory.path()).expect("open checkpoint store");
    let source = event(
        "event-source-drop",
        200,
        KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed("Do not lose the open loop"),
        },
    );
    store.append_event(&source).expect("append source event");
    let first = first_checkpoint(&source, source.occurred_at.as_unix_millis() + 1);
    store
        .save_checkpoint(CheckpointWriteRequest::new(
            first.clone(),
            checkpoint_event("event-checkpoint-drop-1", 201, &first),
        ))
        .expect("save first checkpoint");

    // When: the next version omits every prior loop and supplies no transition.
    let invalid = HandoffCheckpoint::new(
        CheckpointId::new("checkpoint-drop-2"),
        QuestId::new("quest-checkpoint"),
        2,
        Some(first.id.clone()),
        PersistedText::from_reviewed("Pretend the work is complete"),
        Timestamp::from_unix_millis(202),
        CheckpointTrigger::Milestone,
    )
    .with_source_event_refs(vec![source.event_id]);
    let error = store
        .save_checkpoint(CheckpointWriteRequest::new(
            invalid.clone(),
            checkpoint_event("event-checkpoint-drop-2", 202, &invalid),
        ))
        .expect_err("silent loop disappearance must fail");

    // Then: the missing transition is typed and no checkpoint row is committed.
    assert!(matches!(
        error,
        SoulError::Handoff(HandoffError::UnresolvedPriorLoop(ref id))
            if id.as_str() == "loop-projection" || id.as_str() == "loop-host"
    ));
    assert!(store
        .checkpoint(&invalid.id)
        .expect("query invalid checkpoint")
        .is_none());
}
#[test]
fn checkpoint_rejects_an_unresolved_constraint_reference_atomically() {
    // Given: a valid source and a checkpoint that names an absent StateId.
    let directory = tempdir().expect("create missing-state test directory");
    let mut store = SoulStore::open(directory.path()).expect("open missing-state store");
    let source = event(
        "event-source-missing-state",
        300,
        KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed("Preserve only evidenced constraints"),
        },
    );
    store
        .append_event(&source)
        .expect("append missing-state source");
    let checkpoint = HandoffCheckpoint::new(
        CheckpointId::new("checkpoint-missing-state"),
        QuestId::new("quest-checkpoint"),
        1,
        None,
        PersistedText::from_reviewed("Reject unresolved checkpoint state"),
        Timestamp::from_unix_millis(301),
        CheckpointTrigger::Milestone,
    )
    .with_constraint_refs(vec![StateRef::new(StateId::new("state-absent"), 1)])
    .with_source_event_refs(vec![source.event_id]);
    let creation = checkpoint_event("event-checkpoint-missing-state", 301, &checkpoint);
    let creation_id = creation.event_id.clone();

    // When: the checkpoint transaction validates its StateRef edges.
    let error = store
        .save_checkpoint(CheckpointWriteRequest::new(checkpoint.clone(), creation))
        .expect_err("unresolved checkpoint constraint must fail");

    // Then: the typed error leaves both checkpoint and creation event absent.
    assert!(matches!(
        error,
        SoulError::Handoff(HandoffError::MissingState(ref id)) if id.as_str() == "state-absent"
    ));
    assert!(store
        .checkpoint(&checkpoint.id)
        .expect("query rejected checkpoint")
        .is_none());
    assert!(store
        .event(&creation_id)
        .expect("query rejected creation event")
        .is_none());
}
