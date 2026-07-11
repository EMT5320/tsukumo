//! Contract tests for removed-state comparison and post-revoke receipt history.

mod common;

use common::{
    checkpoint_event, event, persist_gnu_constraint, projection_event, projection_request,
};
use tempfile::{tempdir, TempDir};
use tsukumo_kernel::{
    CheckpointId, KernelEventPayload, PersistedText, QuestId, StateId, StateLifecycleAction,
    Timestamp,
};
use tsukumo_soul::{
    compare_projection_receipts, CheckpointTrigger, CheckpointWriteRequest, HandoffCheckpoint,
    OperatingSystem, ProjectionOmissionReason, ProjectionSection, ProjectionWriteRequest,
    SoulStore, StateRecord, StateRef, StateScope, StateTransition, StateWriteOutcome,
    StateWriteRequest,
};

fn revoke_state(store: &mut SoulStore, state_id: &StateId, timestamp: i64) -> StateRecord {
    let source = event(
        &format!("event-revoke-source-{}", state_id.as_str()),
        timestamp,
        KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed("Forget the prior toolchain constraint"),
        },
    );
    let lifecycle = event(
        &format!("event-revoke-state-{}", state_id.as_str()),
        timestamp + 1,
        KernelEventPayload::StateLifecycle {
            state_id: state_id.clone(),
            action: StateLifecycleAction::Revoked,
            prior_state_id: None,
            reason: Some(PersistedText::from_reviewed("Owner revoked the state")),
        },
    );
    match store
        .apply_state(
            StateWriteRequest::new(
                StateTransition::Revoke {
                    prior: state_id.clone(),
                    evidence: source.event_id.clone(),
                    revoked_at: Timestamp::from_unix_millis(timestamp + 1),
                },
                lifecycle,
            )
            .with_source_event(source),
        )
        .expect("revoke deterministic state")
    {
        StateWriteOutcome::Revoked(record) | StateWriteOutcome::Unchanged(record) => record,
        StateWriteOutcome::Created(_) | StateWriteOutcome::Superseded(_) => {
            panic!("revoke fixture must deactivate the existing state")
        }
    }
}
fn setup_store() -> (TempDir, SoulStore, HandoffCheckpoint, StateRecord) {
    let directory = tempdir().expect("create comparison test directory");
    let mut store = SoulStore::open(directory.path()).expect("open comparison store");
    let state = persist_gnu_constraint(
        &mut store,
        "state-comparison",
        "comparison",
        StateScope::workspace_os("tsukumo", OperatingSystem::Windows),
        100,
    );
    let source = event(
        "event-comparison-source",
        190,
        KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed("Compare projected state deterministically"),
        },
    );
    store
        .append_event(&source)
        .expect("append comparison source");
    let checkpoint = HandoffCheckpoint::new(
        CheckpointId::new("checkpoint-comparison"),
        QuestId::new("quest-projection"),
        1,
        None,
        PersistedText::from_reviewed("Compare Tsukumo handoff behavior"),
        Timestamp::from_unix_millis(200),
        CheckpointTrigger::Milestone,
    )
    .with_constraint_refs(vec![StateRef::new(state.state_id.clone(), state.version)])
    .with_source_event_refs(vec![source.event_id]);
    store
        .save_checkpoint(CheckpointWriteRequest::new(
            checkpoint.clone(),
            checkpoint_event(&checkpoint),
        ))
        .expect("save comparison checkpoint");
    (directory, store, checkpoint, state)
}

#[test]
fn comparison_changes_only_the_target_state_section_and_dependent_digest() {
    // Given: one checkpoint-pinned state and two controlled projection requests.
    let (_directory, mut store, checkpoint, state) = setup_store();
    let with_request = projection_request(
        "projection-with",
        "execution-with",
        &checkpoint.id,
        "Run the full test suite",
    );
    let with_event = projection_event(
        "event-projection-with",
        500,
        &with_request.projection_id,
        &checkpoint.id,
        &with_request.execution_id,
        &with_request.runtime,
    );
    let with_state = store
        .prepare_projection(ProjectionWriteRequest::new(with_request, with_event))
        .expect("prepare with-state projection");
    let without_request = projection_request(
        "projection-without",
        "execution-without",
        &checkpoint.id,
        "Run the full test suite",
    )
    .excluding_states(vec![state.state_id.clone()]);
    let without_event = projection_event(
        "event-projection-without",
        500,
        &without_request.projection_id,
        &checkpoint.id,
        &without_request.execution_id,
        &without_request.runtime,
    );
    let without_state = store
        .prepare_projection(ProjectionWriteRequest::new(without_request, without_event))
        .expect("prepare without-state projection");

    // When: receipt metadata is compared under the V0 invariant contract.
    let comparison =
        compare_projection_receipts(&with_state.receipt, &without_state.receipt, &state.state_id)
            .expect("comparison invariants hold");

    // Then: only constraints and the dependent overall digest differ.
    assert_eq!(comparison.target_state_id, state.state_id);
    assert_eq!(
        comparison.changed_sections,
        vec![ProjectionSection::Constraints]
    );
    assert_ne!(comparison.with_digest, comparison.without_digest);
    assert!(without_state.receipt.omissions.iter().any(|omission| {
        omission.state_id == comparison.target_state_id
            && omission.reason == ProjectionOmissionReason::ExcludedByComparison
    }));
    let json = serde_json::to_string(&comparison).expect("serialize comparison metadata");
    assert!(!json.contains("GNU Rust toolchain"));
    assert!(!json.contains("rendered_prompt"));
}

#[test]
fn revoke_excludes_future_projection_and_preserves_historical_receipt() {
    // Given: a receipt that selected one active StateRef.
    let (_directory, mut store, checkpoint, state) = setup_store();
    let initial_request = projection_request(
        "projection-before-revoke",
        "execution-before-revoke",
        &checkpoint.id,
        "Run tests before revoke",
    );
    let initial_event = projection_event(
        "event-before-revoke",
        500,
        &initial_request.projection_id,
        &checkpoint.id,
        &initial_request.execution_id,
        &initial_request.runtime,
    );
    let initial = store
        .prepare_projection(ProjectionWriteRequest::new(initial_request, initial_event))
        .expect("prepare historical receipt");
    let historical = initial.receipt.clone();

    // When: the owner revokes the state and a later projection is prepared.
    revoke_state(&mut store, &state.state_id, 600);
    let mut later_request = projection_request(
        "projection-after-revoke",
        "execution-after-revoke",
        &checkpoint.id,
        "Run tests after revoke",
    );
    later_request.created_at = Timestamp::from_unix_millis(700);
    let later_event = projection_event(
        "event-after-revoke",
        700,
        &later_request.projection_id,
        &checkpoint.id,
        &later_request.execution_id,
        &later_request.runtime,
    );
    let later = store
        .prepare_projection(ProjectionWriteRequest::new(later_request, later_event))
        .expect("prepare post-revoke receipt");

    // Then: future selection omits the state and the old receipt is immutable.
    assert!(later.receipt.selected_state_refs.is_empty());
    assert!(later.receipt.omissions.iter().any(|omission| {
        omission.state_id == state.state_id && omission.reason == ProjectionOmissionReason::Inactive
    }));
    assert_eq!(
        store
            .projection_receipt(&historical.id)
            .expect("reload historical receipt"),
        Some(historical)
    );
}
