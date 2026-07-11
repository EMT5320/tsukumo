//! Contract tests for projection expiry and deterministic candidate ranking.

mod common;

use common::{
    checkpoint_event, event, persist_gnu_constraint, projection_event, projection_request,
};
use tempfile::tempdir;
use tsukumo_kernel::{
    CheckpointId, KernelEventPayload, PersistedText, QuestId, StateId, StateLifecycleAction,
    Timestamp,
};
use tsukumo_soul::{
    CheckpointTrigger, CheckpointWriteRequest, ExtractionContext, HandoffCheckpoint,
    OperatingSystem, ProjectionOmissionReason, ProjectionWriteRequest, RuleStateExtractor,
    SoulStore, StateExtractor, StateRef, StateScope, StateTransition, StateWriteOutcome,
    StateWriteRequest,
};

#[test]
fn expired_checkpoint_constraint_is_reported_as_inactive() {
    // Given: a checkpoint pins a GNU constraint that expires before projection time.
    let directory = tempdir().expect("create expiry test directory");
    let mut store = SoulStore::open(directory.path()).expect("open expiry store");
    let scope = StateScope::workspace_os("tsukumo", OperatingSystem::Windows);
    let source = event(
        "event-source-expiring",
        100,
        KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed(
                "Tsukumo always uses the GNU Rust toolchain on Windows",
            ),
        },
    );
    let mut draft = RuleStateExtractor
        .extract(&ExtractionContext {
            event: &source,
            scope: scope.clone(),
        })
        .expect("extract expiring GNU state")
        .into_iter()
        .next()
        .expect("GNU rule yields an expiring draft");
    draft.expires_at = Some(Timestamp::from_unix_millis(250));
    let state_id = StateId::new("state-expiring");
    let lifecycle = event(
        "event-state-expiring",
        101,
        KernelEventPayload::StateLifecycle {
            state_id: state_id.clone(),
            action: StateLifecycleAction::Created,
            prior_state_id: None,
            reason: None,
        },
    );
    let record = match store
        .apply_state(
            StateWriteRequest::new(
                StateTransition::Create {
                    state_id,
                    draft,
                    created_at: Timestamp::from_unix_millis(101),
                },
                lifecycle,
            )
            .with_source_event(source),
        )
        .expect("persist expiring state")
    {
        StateWriteOutcome::Created(record) | StateWriteOutcome::Unchanged(record) => record,
        StateWriteOutcome::Superseded(_) | StateWriteOutcome::Revoked(_) => {
            panic!("new expiry fixture must create an active state")
        }
    };
    let checkpoint_source = event(
        "event-expiry-checkpoint-source",
        190,
        KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed("Continue before the constraint expires"),
        },
    );
    store
        .append_event(&checkpoint_source)
        .expect("append expiry checkpoint source");
    let checkpoint = HandoffCheckpoint::new(
        CheckpointId::new("checkpoint-expiry"),
        QuestId::new("quest-projection"),
        1,
        None,
        PersistedText::from_reviewed("Verify expiry selection"),
        Timestamp::from_unix_millis(200),
        CheckpointTrigger::Milestone,
    )
    .with_constraint_refs(vec![StateRef::new(record.state_id.clone(), record.version)])
    .with_source_event_refs(vec![checkpoint_source.event_id]);
    store
        .save_checkpoint(CheckpointWriteRequest::new(
            checkpoint.clone(),
            checkpoint_event(&checkpoint),
        ))
        .expect("save expiry checkpoint");

    // When: projection occurs after the pinned state expiry.
    let request = projection_request(
        "projection-expiry",
        "execution-expiry",
        &checkpoint.id,
        "Run expiry validation",
    );
    let projection = projection_event(
        "event-projection-expiry",
        500,
        &request.projection_id,
        &checkpoint.id,
        &request.execution_id,
        &request.runtime,
    );
    let prepared = store
        .prepare_projection(ProjectionWriteRequest::new(request, projection))
        .expect("prepare projection after expiry");

    // Then: the StateRef is excluded with an auditable deterministic reason.
    assert!(prepared.receipt.selected_state_refs.is_empty());
    assert_eq!(
        prepared.receipt.omissions,
        vec![tsukumo_soul::ProjectionOmission {
            state_id: record.state_id,
            reason: ProjectionOmissionReason::Inactive,
        }]
    );
}

#[test]
fn specificity_precedes_freshness_in_candidate_ranking() {
    // Given: an older exact scope and a newer broader scope both apply.
    let directory = tempdir().expect("create ranking test directory");
    let mut store = SoulStore::open(directory.path()).expect("open ranking store");
    let exact_scope = StateScope::workspace_os("tsukumo", OperatingSystem::Windows);
    let exact = persist_gnu_constraint(
        &mut store,
        "state-ranking-exact",
        "ranking-exact",
        exact_scope.clone(),
        100,
    );
    let mut broad_scope = exact_scope;
    broad_scope.applicability.task_tags.clear();
    broad_scope.applicability.language_tags.clear();
    let broad = persist_gnu_constraint(
        &mut store,
        "state-ranking-broad",
        "ranking-broad",
        broad_scope,
        120,
    );
    let checkpoint_source = event(
        "event-ranking-checkpoint-source",
        190,
        KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed("Rank applicable projection state"),
        },
    );
    store
        .append_event(&checkpoint_source)
        .expect("append ranking checkpoint source");
    let checkpoint = HandoffCheckpoint::new(
        CheckpointId::new("checkpoint-ranking"),
        QuestId::new("quest-projection"),
        1,
        None,
        PersistedText::from_reviewed("Verify deterministic state ranking"),
        Timestamp::from_unix_millis(200),
        CheckpointTrigger::Milestone,
    )
    .with_source_event_refs(vec![checkpoint_source.event_id]);
    store
        .save_checkpoint(CheckpointWriteRequest::new(
            checkpoint.clone(),
            checkpoint_event(&checkpoint),
        ))
        .expect("save ranking checkpoint");

    // When: both candidates fit the same projection budget.
    let request = projection_request(
        "projection-ranking",
        "execution-ranking",
        &checkpoint.id,
        "Run ranking validation",
    );
    let projection = projection_event(
        "event-projection-ranking",
        500,
        &request.projection_id,
        &checkpoint.id,
        &request.execution_id,
        &request.runtime,
    );
    let prepared = store
        .prepare_projection(ProjectionWriteRequest::new(request, projection))
        .expect("prepare ranked projection");

    // Then: specificity wins before the newer creation timestamp.
    assert_eq!(
        prepared.receipt.selected_state_refs,
        vec![
            StateRef::new(exact.state_id, exact.version),
            StateRef::new(broad.state_id, broad.version),
        ]
    );
}
