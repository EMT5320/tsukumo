//! Temporal and repeated-evidence StateWriter regressions.

use tempfile::tempdir;
use tsukumo_kernel::{
    EventId, KernelEvent, KernelEventPayload, PersistedText, QuestId, SensitiveText, SessionId,
    SpiritId, StateId, StateLifecycleAction, Timestamp, KERNEL_EVENT_SCHEMA_VERSION,
};
use tsukumo_soul::{
    ChronicleQuery, EvidenceStrength, ExtractionProvenance, OperatingSystem, SoulStore, StateDraft,
    StateKey, StateKind, StateScope, StateTransition, StateWriteRequest,
};

fn event(id: &str, payload: KernelEventPayload) -> KernelEvent {
    KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new(id),
        occurred_at: Timestamp::from_unix_millis(1_750_000_200_000),
        quest_id: QuestId::new("quest-state"),
        session_id: SessionId::new("session-state"),
        spirit_id: SpiritId::new("yuka"),
        execution_id: None,
        runtime: None,
        causation_id: None,
        correlation_id: None,
        payload,
    }
}

fn lifecycle_event(id: &str, state_id: &StateId) -> KernelEvent {
    let mut lifecycle = event(
        id,
        KernelEventPayload::StateLifecycle {
            state_id: state_id.clone(),
            action: StateLifecycleAction::Created,
            prior_state_id: None,
            reason: None,
        },
    );
    lifecycle.occurred_at = Timestamp::from_unix_millis(1_750_000_200_001);
    lifecycle
}
#[test]
fn future_or_unlinked_evidence_is_rejected() {
    // Given: a state transition whose only evidence occurs after the transition.
    let directory = tempdir().expect("create temporal evidence directory");
    let mut source = event(
        "event-future-evidence",
        KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed("Remember this preference"),
        },
    );
    source.occurred_at = Timestamp::from_unix_millis(1_750_000_200_010);
    let state_id = StateId::new("state-future-evidence");
    let draft = StateDraft {
        proposed_key: StateKey::new("workspace.tsukumo.preference.temporal"),
        kind: StateKind::Preference,
        scope: StateScope::workspace_os("tsukumo", OperatingSystem::Windows),
        content: SensitiveText::new("Temporal preference"),
        claimed_strength: EvidenceStrength::Inferred,
        evidence_refs: vec![source.event_id.clone()],
        provenance: ExtractionProvenance::Recorded {
            fixture: "future-evidence".into(),
            schema_version: 1,
        },
        expires_at: None,
    };
    let request = StateWriteRequest::new(
        StateTransition::Create {
            state_id: state_id.clone(),
            draft,
            created_at: Timestamp::from_unix_millis(1_750_000_200_001),
        },
        lifecycle_event("event-future-state", &state_id),
    )
    .with_source_event(source);
    let mut store = SoulStore::open(directory.path()).expect("open temporal evidence store");

    // When/Then: causality and time validation reject the complete transaction.
    assert!(store.apply_state(request).is_err());
    assert!(store
        .replay_events(ChronicleQuery::default())
        .expect("replay rejected temporal state")
        .is_empty());
}

#[test]
fn repeated_strength_requires_distinct_chronicle_events() {
    // Given: one event duplicated in a draft that claims repeated evidence.
    let directory = tempdir().expect("create repeated evidence directory");
    let source = event(
        "event-repeated-once",
        KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed("One observation"),
        },
    );
    let state_id = StateId::new("state-repeated-once");
    let draft = StateDraft {
        proposed_key: StateKey::new("workspace.tsukumo.preference.repeated"),
        kind: StateKind::Preference,
        scope: StateScope::workspace_os("tsukumo", OperatingSystem::Windows),
        content: SensitiveText::new("Repeated preference"),
        claimed_strength: EvidenceStrength::Repeated,
        evidence_refs: vec![source.event_id.clone(), source.event_id.clone()],
        provenance: ExtractionProvenance::Recorded {
            fixture: "repeated-once".into(),
            schema_version: 1,
        },
        expires_at: None,
    };
    let mut lifecycle = lifecycle_event("event-repeated-state", &state_id);
    lifecycle.causation_id = Some(source.event_id.clone());
    let request = StateWriteRequest::new(
        StateTransition::Create {
            state_id,
            draft,
            created_at: Timestamp::from_unix_millis(1_750_000_200_001),
        },
        lifecycle,
    )
    .with_source_event(source);
    let mut store = SoulStore::open(directory.path()).expect("open repeated evidence store");

    // When/Then: duplicate references cannot manufacture repeated strength.
    assert!(store.apply_state(request).is_err());
}
