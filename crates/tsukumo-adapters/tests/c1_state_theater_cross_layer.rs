//! Cross-crate GNU evidence chain from Chronicle reopen into theater replay.

use tempfile::tempdir;
use tsukumo_kernel::{
    EventId, KernelEvent, KernelEventPayload, PersistedText, QuestId, SessionId, SpiritId, StateId,
    StateLifecycleAction, Timestamp, KERNEL_EVENT_SCHEMA_VERSION,
};
use tsukumo_soul::{
    ChronicleQuery, ExtractionContext, OperatingSystem, RuleStateExtractor, SoulStore,
    StateExtractor, StateScope, StateTransition, StateWriteRequest,
};
use tsukumo_theater::{drive_kernel_events, DirectorContext, StageWorld};

#[test]
fn reopened_gnu_state_and_chronicle_replay_share_one_evidence_chain() {
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
            .with_source_event(source),
        )
        .expect("write GNU evidence chain");
    drop(store);

    let reopened = SoulStore::open(directory.path()).expect("reopen evidence chain");
    let state = reopened
        .state(&state_id)
        .expect("query GNU state")
        .expect("GNU state exists");
    let replay = reopened
        .replay_events(ChronicleQuery::default())
        .expect("replay Chronicle")
        .into_iter()
        .map(|persisted| persisted.event)
        .collect::<Vec<_>>();
    let mut world = StageWorld::new();
    drive_kernel_events(&mut world, &replay, &DirectorContext::default());

    assert_eq!(state.evidence_refs, vec![EventId::new("event-cross-user")]);
    assert_eq!(replay.len(), 2);
    assert!(world
        .log
        .iter()
        .any(|line| line.contains("state_lifecycle state-cross-gnu Created")));
}
