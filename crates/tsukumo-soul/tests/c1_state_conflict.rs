//! C1 idempotent create and conflict rollback tests.

use tempfile::tempdir;
use tsukumo_kernel::{
    EventId, KernelEvent, KernelEventPayload, PersistedText, QuestId, SensitiveText, SessionId,
    SpiritId, StateId, StateLifecycleAction, Timestamp, KERNEL_EVENT_SCHEMA_VERSION,
};
use tsukumo_soul::{
    EvidenceStrength, ExtractionProvenance, OperatingSystem, SoulStore, StateDraft, StateKey,
    StateKind, StateScope, StateTransition, StateValidationError, StateWriteOutcome,
    StateWriteRequest,
};

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
    source_at(id, 1_750_000_300_000, text)
}

fn source_at(id: &str, timestamp: i64, text: &str) -> KernelEvent {
    event_at(
        id,
        timestamp,
        KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed(text),
        },
    )
}

#[test]
fn identical_retry_is_unchanged_while_conflict_rolls_back() {
    // Given: one active GNU constraint.
    let directory = tempdir().expect("create lifecycle test directory");
    let source_one = source("event-user-1", "Use GNU on Windows");
    let state_one = StateId::new("state-gnu-1");
    let request = StateWriteRequest::new(
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
    .with_source_event(source_one);
    let mut store = SoulStore::open(directory.path()).expect("open state store");
    assert!(matches!(
        store.apply_state(request.clone()).expect("create state"),
        StateWriteOutcome::Created(_)
    ));

    // When: the exact request is retried, then a conflicting create is attempted.
    let retry = store.apply_state(request).expect("retry state write");
    let source_two = source("event-user-2", "Always use GNU nightly on Windows");
    let conflicting_state = StateId::new("state-gnu-2");
    let conflict = store
        .apply_state(
            StateWriteRequest::new(
                StateTransition::Create {
                    state_id: conflicting_state.clone(),
                    draft: draft(
                        source_two.event_id.clone(),
                        "Always use GNU nightly on Windows",
                    ),
                    created_at: Timestamp::from_unix_millis(1_750_000_300_002),
                },
                event_at(
                    "event-state-2",
                    1_750_000_300_002,
                    KernelEventPayload::StateLifecycle {
                        state_id: conflicting_state,
                        action: StateLifecycleAction::Created,
                        prior_state_id: None,
                        reason: None,
                    },
                ),
            )
            .with_source_event(source_two),
        )
        .expect_err("conflicting state must fail");

    // Then: retry is explicit, conflict is typed, and its source event rolls back.
    assert!(matches!(retry, StateWriteOutcome::Unchanged(_)));
    assert!(matches!(
        conflict,
        tsukumo_soul::SoulError::StateValidation(StateValidationError::Conflict(_))
    ));
    assert!(store
        .event(&EventId::new("event-user-2"))
        .expect("query rolled-back source")
        .is_none());
}

#[test]
fn create_rejects_a_transition_older_than_an_existing_version() {
    // Given: one state version already committed at a later logical instant.
    let directory = tempdir().expect("create backdated state directory");
    let mut store = SoulStore::open(directory.path()).expect("open state store");
    let later_source = source_at("event-later-user", 1_750_000_300_100, "Use GNU on Windows");
    let later_state = StateId::new("state-later");
    store
        .apply_state(
            StateWriteRequest::new(
                StateTransition::Create {
                    state_id: later_state.clone(),
                    draft: draft(later_source.event_id.clone(), "Use GNU on Windows"),
                    created_at: Timestamp::from_unix_millis(1_750_000_300_200),
                },
                event_at(
                    "event-later-state",
                    1_750_000_300_200,
                    KernelEventPayload::StateLifecycle {
                        state_id: later_state,
                        action: StateLifecycleAction::Created,
                        prior_state_id: None,
                        reason: None,
                    },
                ),
            )
            .with_source_event(later_source),
        )
        .expect("create later state");

    // When: a different state attempts to backfill the same key and scope.
    let backdated_source = source_at(
        "event-backdated-user",
        1_750_000_300_140,
        "Use GNU stable on Windows",
    );
    let backdated_state = StateId::new("state-backdated");
    let result = store.apply_state(
        StateWriteRequest::new(
            StateTransition::Create {
                state_id: backdated_state.clone(),
                draft: draft(
                    backdated_source.event_id.clone(),
                    "Use GNU stable on Windows",
                ),
                created_at: Timestamp::from_unix_millis(1_750_000_300_150),
            },
            event_at(
                "event-backdated-state",
                1_750_000_300_150,
                KernelEventPayload::StateLifecycle {
                    state_id: backdated_state.clone(),
                    action: StateLifecycleAction::Created,
                    prior_state_id: None,
                    reason: None,
                },
            ),
        )
        .with_source_event(backdated_source),
    );

    // Then: the writer preserves non-overlapping version history atomically.
    assert!(result.is_err());
    assert!(store
        .state(&backdated_state)
        .expect("query rejected state")
        .is_none());
    assert!(store
        .event(&EventId::new("event-backdated-user"))
        .expect("query rejected source")
        .is_none());
}
