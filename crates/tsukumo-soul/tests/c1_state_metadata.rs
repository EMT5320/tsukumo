//! State metadata trust and persistence boundary regressions.

use tempfile::tempdir;
use tsukumo_kernel::{
    EventId, KernelEvent, KernelEventPayload, PersistedText, QuestId, SensitiveText, SessionId,
    SpiritId, StateId, StateLifecycleAction, Timestamp, KERNEL_EVENT_SCHEMA_VERSION,
};
use tsukumo_soul::{
    ChronicleQuery, EvidenceStrength, ExtractionProvenance, OperatingSystem, SoulStore, StateDraft,
    StateKey, StateKind, StateScope, StateTransition, StateWriteRequest,
};

fn source() -> KernelEvent {
    KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new("event-metadata-source"),
        occurred_at: Timestamp::from_unix_millis(100),
        quest_id: QuestId::new("quest-metadata"),
        session_id: SessionId::new("session-metadata"),
        spirit_id: SpiritId::new("yuka"),
        execution_id: None,
        runtime: None,
        causation_id: None,
        correlation_id: None,
        payload: KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed("Remember a safe preference"),
        },
    }
}

fn request(key: &str, scope: StateScope) -> StateWriteRequest {
    let source = source();

    let state_id = StateId::new("state-metadata");
    let lifecycle = KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new("event-metadata-state"),
        occurred_at: Timestamp::from_unix_millis(101),
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
    StateWriteRequest::new(
        StateTransition::Create {
            state_id,
            draft: StateDraft {
                proposed_key: StateKey::new(key),
                kind: StateKind::Preference,
                scope,
                content: SensitiveText::new("Safe preference"),
                claimed_strength: EvidenceStrength::Inferred,
                evidence_refs: vec![source.event_id.clone()],
                provenance: ExtractionProvenance::Recorded {
                    fixture: "metadata".into(),
                    schema_version: 1,
                },
                expires_at: None,
            },
            created_at: Timestamp::from_unix_millis(101),
        },
        lifecycle,
    )
    .with_source_event(source)
}

#[test]
fn secret_bearing_state_metadata_is_rejected_atomically() {
    // Given: a safe body with credential material hidden in state metadata.
    let directory = tempdir().expect("create metadata secret directory");
    let mut store = SoulStore::open(directory.path()).expect("open metadata secret store");

    // When/Then: the writer rejects the whole transaction before persistence.
    let mut scope = StateScope::workspace_os("tsukumo", OperatingSystem::Windows);
    scope
        .applicability
        .task_tags
        .push("api_token=SENTINEL-Aa1234567890_METADATA_SECRET".into());

    assert!(store
        .apply_state(request("workspace.tsukumo.preference.safe", scope))
        .is_err());
    assert!(store
        .replay_events(ChronicleQuery::default())
        .expect("replay rejected metadata")
        .is_empty());
}

#[test]
fn workspace_state_key_cannot_name_another_workspace() {
    // Given: a trusted tsukumo scope paired with a key for another workspace.
    let directory = tempdir().expect("create metadata scope directory");
    let mut scope = StateScope::workspace_os("tsukumo", OperatingSystem::Windows);
    scope.applicability.task_tags.clear();
    let mut store = SoulStore::open(directory.path()).expect("open metadata scope store");

    // When/Then: key and scope coordinates must describe the same owner.
    assert!(store
        .apply_state(request("workspace.other.preference", scope))
        .is_err());
}
