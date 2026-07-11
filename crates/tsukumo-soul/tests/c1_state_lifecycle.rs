//! C1 conflict, supersede, and revoke lifecycle tests.

use tempfile::tempdir;
use tsukumo_kernel::{
    EventId, KernelEvent, KernelEventPayload, PersistedText, QuestId, SensitiveText, SessionId,
    SpiritId, StateId, StateLifecycleAction, Timestamp, KERNEL_EVENT_SCHEMA_VERSION,
};
use tsukumo_soul::{
    ChronicleQuery, EvidenceStrength, ExtractionProvenance, OperatingSystem, SoulStore, StateDraft,
    StateKey, StateKind, StateScope, StateStatus, StateTransition, StateWriteOutcome,
    StateWriteRequest,
};

fn event(id: &str, payload: KernelEventPayload) -> KernelEvent {
    event_at(id, 1_750_000_300_000, payload)
}

fn event_at(id: &str, timestamp: i64, payload: KernelEventPayload) -> KernelEvent {
    KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new(id),
        occurred_at: Timestamp::from_unix_millis(timestamp),
        quest_id: QuestId::new("quest-lifecycle"),
        session_id: SessionId::new("session-lifecycle"),
        spirit_id: SpiritId::new("yuka"),
        execution_id: None,
        runtime: None,
        causation_id: None,
        correlation_id: None,
        payload,
    }
}

fn draft(evidence: EventId, text: &str) -> StateDraft {
    StateDraft {
        proposed_key: StateKey::new("workspace.tsukumo.rust.toolchain.windows"),
        kind: StateKind::Preference,
        scope: StateScope::workspace_os("tsukumo", OperatingSystem::Windows),
        content: SensitiveText::new(text),
        claimed_strength: EvidenceStrength::Inferred,
        evidence_refs: vec![evidence],
        provenance: ExtractionProvenance::Recorded {
            fixture: "lifecycle".into(),
            schema_version: 1,
        },
        expires_at: None,
    }
}

fn source(id: &str, text: &str) -> KernelEvent {
    event(
        id,
        KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed(text),
        },
    )
}

#[test]
fn supersede_then_revoke_preserves_versions_and_stops_active_selection() {
    // Given: an active version-one constraint.
    let directory = tempdir().expect("create transition test directory");
    let mut store = SoulStore::open(directory.path()).expect("open state store");
    let source_one = source("event-user-1", "Use GNU on Windows");
    let state_one = StateId::new("state-gnu-1");
    store
        .apply_state(
            StateWriteRequest::new(
                StateTransition::Create {
                    state_id: state_one.clone(),
                    draft: draft(source_one.event_id.clone(), "Use GNU on Windows"),
                    created_at: Timestamp::from_unix_millis(1_750_000_300_001),
                },
                event_at(
                    "event-state-1",
                    1_750_000_300_001,
                    KernelEventPayload::StateLifecycle {
                        state_id: state_one.clone(),
                        action: StateLifecycleAction::Created,
                        prior_state_id: None,
                        reason: None,
                    },
                ),
            )
            .with_source_event(source_one),
        )
        .expect("create first version");

    // When: a correction supersedes version one and a later user event revokes version two.
    let source_two = source(
        "event-user-2",
        "Always use GNU with the explicit target on Windows",
    );
    let state_two = StateId::new("state-gnu-2");
    let superseded = store
        .apply_state(
            StateWriteRequest::new(
                StateTransition::Supersede {
                    state_id: state_two.clone(),
                    prior: state_one.clone(),
                    draft: draft(
                        source_two.event_id.clone(),
                        "Always use GNU with the explicit target on Windows",
                    ),
                    created_at: Timestamp::from_unix_millis(1_750_000_300_002),
                },
                event_at(
                    "event-state-2",
                    1_750_000_300_002,
                    KernelEventPayload::StateLifecycle {
                        state_id: state_two.clone(),
                        action: StateLifecycleAction::Superseded,
                        prior_state_id: Some(state_one.clone()),
                        reason: None,
                    },
                ),
            )
            .with_source_event(source_two),
        )
        .expect("supersede state");
    store.rebuild_exports().expect("seed FTS before revocation");
    let evidence_before_revoke = store
        .state(&state_two)
        .expect("query state before revoke")
        .expect("state before revoke")
        .evidence_refs;
    let revoke_source = source("event-user-3", "Forget the GNU constraint");
    let revoked = store
        .apply_state(
            StateWriteRequest::new(
                StateTransition::Revoke {
                    prior: state_two.clone(),
                    evidence: revoke_source.event_id.clone(),
                    revoked_at: Timestamp::from_unix_millis(1_750_000_300_003),
                },
                event_at(
                    "event-state-3",
                    1_750_000_300_003,
                    KernelEventPayload::StateLifecycle {
                        state_id: state_two.clone(),
                        action: StateLifecycleAction::Revoked,
                        prior_state_id: None,
                        reason: Some(PersistedText::from_reviewed("user revoked constraint")),
                    },
                ),
            )
            .with_source_event(revoke_source),
        )
        .expect("revoke state");

    // Then: both historical versions remain queryable and no active value remains.
    assert!(matches!(superseded, StateWriteOutcome::Superseded(_)));
    assert!(matches!(revoked, StateWriteOutcome::Revoked(_)));
    assert_eq!(
        store
            .state(&state_one)
            .expect("query first")
            .expect("first state")
            .status,
        StateStatus::Superseded
    );
    assert_eq!(
        store
            .state(&state_two)
            .expect("query second")
            .expect("second state")
            .status,
        StateStatus::Revoked
    );
    assert_eq!(
        store
            .state(&state_two)
            .expect("query revoked evidence")
            .expect("revoked state")
            .evidence_refs,
        evidence_before_revoke
    );
    assert_eq!(
        store
            .active_state_at(
                &StateKey::new("workspace.tsukumo.rust.toolchain.windows"),
                &StateScope::workspace_os("tsukumo", OperatingSystem::Windows),
                Timestamp::from_unix_millis(1_750_000_300_001),
            )
            .expect("query version one historical instant")
            .expect("version one was active")
            .state_id,
        state_one
    );
    assert_eq!(
        store
            .active_state_at(
                &StateKey::new("workspace.tsukumo.rust.toolchain.windows"),
                &StateScope::workspace_os("tsukumo", OperatingSystem::Windows),
                Timestamp::from_unix_millis(1_750_000_300_002),
            )
            .expect("query version two historical instant")
            .expect("version two was active")
            .state_id,
        state_two
    );
    assert!(store
        .active_state(
            &StateKey::new("workspace.tsukumo.rust.toolchain.windows"),
            &StateScope::workspace_os("tsukumo", OperatingSystem::Windows),
        )
        .expect("query active state")
        .is_none());
    assert!(store
        .search_states("GNU", 5)
        .expect("search stale FTS after revoke")
        .is_empty());
    let replayed = store
        .replay_events(ChronicleQuery::default())
        .expect("replay transitions");
    assert_eq!(replayed.len(), 6);
    let revoke_lifecycle = replayed
        .iter()
        .find(|stored| stored.event.event_id.as_str() == "event-state-3")
        .expect("revoke lifecycle event");
    assert_eq!(
        revoke_lifecycle.event.causation_id,
        Some(EventId::new("event-user-3"))
    );
}
