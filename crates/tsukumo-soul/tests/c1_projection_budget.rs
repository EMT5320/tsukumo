//! Budget boundary tests for pinned projection constraints.

mod common;

use common::{
    checkpoint_event, event, persist_gnu_constraint, projection_event, projection_request,
};
use tempfile::tempdir;
use tsukumo_kernel::{
    CheckpointId, KernelEventPayload, PersistedText, ProjectionId, QuestId, Timestamp,
};
use tsukumo_soul::{
    CheckpointTrigger, CheckpointWriteRequest, HandoffCheckpoint, OperatingSystem, ProjectionError,
    ProjectionWriteRequest, SoulError, SoulStore, StateRef, StateScope,
};

#[test]
fn pinned_constraint_cannot_be_silently_dropped_for_budget() {
    // Given: the canonical projection whose full prompt uses 380 characters.
    let directory = tempdir().expect("create budget test directory");
    let mut store = SoulStore::open(directory.path()).expect("open budget store");
    let state = persist_gnu_constraint(
        &mut store,
        "state-budget",
        "budget",
        StateScope::workspace_os("tsukumo", OperatingSystem::Windows),
        100,
    );
    let source = event(
        "event-budget-source",
        190,
        KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed("Continue the Tsukumo MVP"),
        },
    );
    store.append_event(&source).expect("append budget source");
    let checkpoint = HandoffCheckpoint::new(
        CheckpointId::new("checkpoint-budget"),
        QuestId::new("quest-projection"),
        1,
        None,
        PersistedText::from_reviewed("Continue Tsukumo MVP"),
        Timestamp::from_unix_millis(200),
        CheckpointTrigger::RuntimeSwitch,
    )
    .with_constraint_refs(vec![StateRef::new(state.state_id.clone(), state.version)])
    .with_source_event_refs(vec![source.event_id]);
    store
        .save_checkpoint(CheckpointWriteRequest::new(
            checkpoint.clone(),
            checkpoint_event(&checkpoint),
        ))
        .expect("save budget checkpoint");

    // When: the declared budget is one character below the required prompt.
    let mut request = projection_request(
        "projection-budget",
        "execution-budget",
        &checkpoint.id,
        "Run the full test suite",
    );
    request.budget_chars = 379;
    let runtime = request.runtime.clone();
    let event = projection_event(
        "event-projection-budget",
        500,
        &request.projection_id,
        &checkpoint.id,
        &request.execution_id,
        &runtime,
    );
    let error = store
        .prepare_projection(ProjectionWriteRequest::new(request, event))
        .expect_err("pinned state cannot be omitted for budget");

    // Then: the typed error names the pinned state and commits no receipt.
    assert!(matches!(
        error,
        SoulError::Projection(ProjectionError::PinnedStateExceedsBudget {
            ref state_id,
            limit: 379,
        }) if state_id == &state.state_id
    ));
    assert!(store
        .projection_receipt(&ProjectionId::new("projection-budget"))
        .expect("query rejected projection")
        .is_none());
}
