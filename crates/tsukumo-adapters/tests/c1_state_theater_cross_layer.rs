//! Cross-crate GNU evidence chain from projection commit into theater replay.

use tempfile::tempdir;
use tsukumo_kernel::{
    CheckpointId, CorrelationId, EventId, ExecutionId, KernelEvent, KernelEventPayload,
    PersistedText, ProjectionId, QuestId, RuntimeBinding, RuntimeKind, RuntimeMode, SensitiveText,
    SessionId, SpiritId, StateId, StateLifecycleAction, Timestamp, KERNEL_EVENT_SCHEMA_VERSION,
};
use tsukumo_soul::{
    CheckpointTrigger, CheckpointWriteRequest, ChronicleQuery, ExtractionContext,
    HandoffCheckpoint, OperatingSystem, ProjectionRequest, ProjectionTarget,
    ProjectionWriteRequest, RuleStateExtractor, SoulStore, StateExtractor, StateRef, StateScope,
    StateTransition, StateWriteRequest,
};
use tsukumo_theater::{drive_kernel_events, DirectorContext, StageWorld};

/// Secret sentinel must stay in the launch value and out of durable/read-model surfaces.
const PROMPT_SECRET: &str = "token=projection-cross-layer-secret";

#[test]
fn reopened_projection_and_chronicle_replay_share_one_evidence_chain() {
    // Given: Chronicle-backed GNU state selected by a committed checkpoint.
    let directory = tempdir().expect("create cross-layer directory");
    let source = KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new("event-cross-user"),
        occurred_at: Timestamp::from_unix_millis(1_750_001_400_000),
        quest_id: QuestId::new("quest-cross"),
        session_id: SessionId::new("session-cross"),
        spirit_id: SpiritId::new("yuka"),
        execution_id: None,
        runtime: None,
        causation_id: None,
        correlation_id: None,
        payload: KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed("Tsukumo always uses GNU on Windows"),
        },
    };
    let scope = StateScope::workspace_os("tsukumo", OperatingSystem::Windows);
    let draft = RuleStateExtractor
        .extract(&ExtractionContext {
            event: &source,
            scope: scope.clone(),
        })
        .expect("extract GNU state")
        .into_iter()
        .next()
        .expect("GNU draft");
    let state_id = StateId::new("state-cross-gnu");
    let lifecycle = KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new("event-cross-state"),
        occurred_at: Timestamp::from_unix_millis(1_750_001_400_001),
        quest_id: source.quest_id.clone(),
        session_id: source.session_id.clone(),
        spirit_id: source.spirit_id.clone(),
        execution_id: None,
        runtime: None,
        causation_id: Some(source.event_id.clone()),
        correlation_id: None,
        payload: KernelEventPayload::StateLifecycle {
            state_id: state_id.clone(),
            action: StateLifecycleAction::Created,
            prior_state_id: None,
            reason: None,
        },
    };
    let mut store = SoulStore::open(directory.path()).expect("open cross-layer store");
    store
        .apply_state(
            StateWriteRequest::new(
                StateTransition::Create {
                    state_id: state_id.clone(),
                    draft,
                    created_at: lifecycle.occurred_at,
                },
                lifecycle,
            )
            .with_source_event(source.clone()),
        )
        .expect("write GNU evidence chain");
    let checkpoint = HandoffCheckpoint::new(
        CheckpointId::new("checkpoint-cross"),
        source.quest_id.clone(),
        1,
        None,
        PersistedText::from_reviewed("Continue the Tsukumo MVP"),
        Timestamp::from_unix_millis(1_750_001_400_002),
        CheckpointTrigger::RuntimeSwitch,
    )
    .with_constraint_refs(vec![StateRef::new(state_id.clone(), 1)])
    .with_source_event_refs(vec![source.event_id.clone()]);
    let checkpoint_event = KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new("event-cross-checkpoint"),
        occurred_at: checkpoint.created_at,
        quest_id: source.quest_id.clone(),
        session_id: source.session_id.clone(),
        spirit_id: source.spirit_id.clone(),
        execution_id: None,
        runtime: None,
        causation_id: Some(source.event_id.clone()),
        correlation_id: None,
        payload: KernelEventPayload::CheckpointCreated {
            checkpoint_id: checkpoint.id.clone(),
            version: checkpoint.version,
        },
    };
    store
        .save_checkpoint(CheckpointWriteRequest::new(
            checkpoint.clone(),
            checkpoint_event,
        ))
        .expect("commit cross-layer checkpoint");

    // When: a Codex projection commits before its sensitive launch value is exposed.
    let projection_id = ProjectionId::new("projection-cross");
    let execution_id = ExecutionId::new("execution-cross");
    let runtime = RuntimeBinding::new(RuntimeKind::CodexCli, RuntimeMode::Fixture);
    let projection_time = Timestamp::from_unix_millis(1_750_001_400_003);
    let request = ProjectionRequest::new(
        ProjectionTarget::new(
            projection_id.clone(),
            execution_id.clone(),
            runtime.clone(),
            checkpoint.id.clone(),
        ),
        scope,
        SensitiveText::new(format!("Run validation with {PROMPT_SECRET}")),
        projection_time,
        2_000,
    );
    let projection_event = KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new("event-cross-projection"),
        occurred_at: projection_time,
        quest_id: source.quest_id,
        session_id: source.session_id,
        spirit_id: source.spirit_id,
        execution_id: Some(execution_id),
        runtime: Some(runtime),
        causation_id: Some(checkpoint_event_id()),
        correlation_id: Some(CorrelationId::new("correlation-cross-projection")),
        payload: KernelEventPayload::ProjectionCreated {
            projection_id: projection_id.clone(),
            checkpoint_id: checkpoint.id,
        },
    };
    let prepared = store
        .prepare_projection(ProjectionWriteRequest::new(request, projection_event))
        .expect("commit cross-layer projection");
    assert!(prepared.rendered_prompt().expose().contains(PROMPT_SECRET));
    assert!(prepared
        .rendered_prompt()
        .expose()
        .contains("[state:state-cross-gnu@v1] Use the GNU Rust toolchain on Windows"));
    assert_eq!(prepared.receipt.redactions.len(), 1);
    let expected_receipt = prepared.receipt.clone();
    drop(store);

    // Then: reopen proves the receipt and Theater replays only safe event metadata.
    let reopened = SoulStore::open(directory.path()).expect("reopen evidence chain");
    assert_eq!(
        reopened
            .projection_receipt(&projection_id)
            .expect("query durable projection receipt"),
        Some(expected_receipt)
    );
    let replay = reopened
        .replay_events(ChronicleQuery::default())
        .expect("replay Chronicle")
        .into_iter()
        .map(|persisted| persisted.event)
        .collect::<Vec<_>>();
    let mut world = StageWorld::new();
    drive_kernel_events(&mut world, &replay, &DirectorContext::default());

    assert_eq!(replay.len(), 4);
    assert!(world.log.iter().any(|line| line
        .text
        .contains("state_lifecycle state-cross-gnu Created")));
    assert!(world.log.iter().any(|line| {
        line.text
            .contains("projection_created projection-cross from checkpoint-cross")
    }));
    assert!(!world
        .log
        .iter()
        .any(|line| line.text.contains(PROMPT_SECRET)));
}

/// Keeps the projection event causation edge explicit without retaining prompt data.
fn checkpoint_event_id() -> EventId {
    EventId::new("event-cross-checkpoint")
}
