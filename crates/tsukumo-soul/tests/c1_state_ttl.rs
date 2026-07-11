//! Controlled-clock TTL selection and projection tests.

use tempfile::tempdir;
use tsukumo_kernel::{
    EventId, KernelEvent, KernelEventPayload, PersistedText, QuestId, SensitiveText, SessionId,
    SpiritId, StateId, StateLifecycleAction, Timestamp, KERNEL_EVENT_SCHEMA_VERSION,
};
use tsukumo_soul::{
    EvidenceStrength, ExtractionProvenance, OperatingSystem, SoulStore, StateDraft, StateKey,
    StateKind, StateScope, StateTransition, StateWriteRequest,
};

fn event(id: &str, timestamp: i64, payload: KernelEventPayload) -> KernelEvent {
    KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new(id),
        occurred_at: Timestamp::from_unix_millis(timestamp),
        quest_id: QuestId::new("quest-ttl"),
        session_id: SessionId::new("session-ttl"),
        spirit_id: SpiritId::new("yuka"),
        execution_id: None,
        runtime: None,
        causation_id: None,
        correlation_id: None,
        payload,
    }
}

#[test]
fn expired_state_leaves_active_queries_brief_fts_and_compatibility_exports() {
    let directory = tempdir().expect("create TTL directory");
    let source = event(
        "event-ttl-source",
        100,
        KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed("remember a temporary observation"),
        },
    );
    let state_id = StateId::new("state-ttl");
    let scope = StateScope::workspace_os("tsukumo", OperatingSystem::Windows);
    let key = StateKey::new("workspace.tsukumo.temporary");
    let draft = StateDraft {
        proposed_key: key.clone(),
        kind: StateKind::Fact,
        scope: scope.clone(),
        content: SensitiveText::new("Temporary observation"),
        claimed_strength: EvidenceStrength::Inferred,
        evidence_refs: vec![source.event_id.clone()],
        provenance: ExtractionProvenance::Recorded {
            fixture: "ttl".into(),
            schema_version: 1,
        },
        expires_at: Some(Timestamp::from_unix_millis(200)),
    };
    let lifecycle = event(
        "event-ttl-state",
        100,
        KernelEventPayload::StateLifecycle {
            state_id: state_id.clone(),
            action: StateLifecycleAction::Created,
            prior_state_id: None,
            reason: None,
        },
    );
    let mut store = SoulStore::open(directory.path()).expect("open TTL store");
    store
        .apply_state(
            StateWriteRequest::new(
                StateTransition::Create {
                    state_id: state_id.clone(),
                    draft,
                    created_at: Timestamp::from_unix_millis(100),
                },
                lifecycle,
            )
            .with_source_event(source),
        )
        .expect("create temporary state");

    assert!(store
        .active_state_at(&key, &scope, Timestamp::from_unix_millis(99))
        .expect("query before creation")
        .is_none());
    assert!(store
        .active_state_at(&key, &scope, Timestamp::from_unix_millis(199))
        .expect("query before expiry")
        .is_some());
    assert!(store
        .active_state_at(&key, &scope, Timestamp::from_unix_millis(200))
        .expect("query at expiry")
        .is_none());
    drop(store);

    let mut reopened = SoulStore::open(directory.path()).expect("reopen TTL store");
    assert!(reopened
        .active_state(&key, &scope)
        .expect("query expired current state")
        .is_none());
    assert!(reopened
        .list_active_states()
        .expect("list current states")
        .is_empty());

    let exports = reopened.rebuild_exports().expect("rebuild TTL exports");
    let state_markdown =
        std::fs::read_to_string(exports.state_markdown).expect("read state export");
    let memory_markdown =
        std::fs::read_to_string(exports.memory_markdown).expect("read memory export");
    assert!(state_markdown.contains("[expired]"));
    assert!(!memory_markdown.contains("Temporary observation"));
    assert!(reopened
        .search_states("Temporary", 5)
        .expect("search expired FTS")
        .is_empty());
    assert!(reopened
        .state(&state_id)
        .expect("query historical TTL state")
        .is_some());
}
