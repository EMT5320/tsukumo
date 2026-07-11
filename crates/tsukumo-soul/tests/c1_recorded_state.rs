//! Recorded structured extraction through deterministic StateWriter.

use tempfile::tempdir;
use tsukumo_kernel::{
    EventId, KernelEvent, KernelEventPayload, PersistedText, QuestId, SessionId, SpiritId, StateId,
    StateLifecycleAction, Timestamp, KERNEL_EVENT_SCHEMA_VERSION,
};
use tsukumo_soul::{
    ExtractionContext, OperatingSystem, RecordedStateExtractor, SoulStore, StateExtractor,
    StateScope, StateTransition, StateWriteOutcome, StateWriteRequest,
};

#[test]
fn recorded_dto_parses_writes_and_reopens_without_network() {
    let directory = tempdir().expect("create recorded extraction directory");
    let source = KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new("event-recorded-source"),
        occurred_at: Timestamp::from_unix_millis(1_750_001_300_000),
        quest_id: QuestId::new("quest-recorded"),
        session_id: SessionId::new("session-recorded"),
        spirit_id: SpiritId::new("yuka"),
        execution_id: None,
        runtime: None,
        causation_id: None,
        correlation_id: None,
        payload: KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed("I prefer compact progress reports"),
        },
    };
    let scope = StateScope::workspace_os("tsukumo", OperatingSystem::Windows);
    let body = serde_json::json!({
        "schema_version": 1,
        "drafts": [{
            "proposed_key": "workspace.tsukumo.progress.style",
            "kind": "preference",
            "content": "Prefer compact progress reports",
            "expires_at": null
        }]
    })
    .to_string();
    let extractor =
        RecordedStateExtractor::from_json("recorded-preference", &body).expect("parse DTO");
    let draft = extractor
        .extract(&ExtractionContext {
            event: &source,
            scope: scope.clone(),
        })
        .expect("extract recorded draft")
        .into_iter()
        .next()
        .expect("one recorded draft");
    let state_id = StateId::new("state-recorded-preference");
    let lifecycle = KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new("event-recorded-state"),
        occurred_at: Timestamp::from_unix_millis(1_750_001_300_001),
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

    let mut store = SoulStore::open(directory.path()).expect("open recorded store");
    let outcome = store
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
        .expect("write recorded state");
    assert!(matches!(outcome, StateWriteOutcome::Created(_)));
    drop(store);

    let reopened = SoulStore::open(directory.path()).expect("reopen recorded store");
    let state = reopened
        .state(&state_id)
        .expect("query recorded state")
        .expect("recorded state exists");
    assert_eq!(state.content.as_str(), "Prefer compact progress reports");
    assert_eq!(state.scope, scope);
}

#[test]
fn recorded_dto_cannot_supply_trust_fields_or_exceed_the_input_budget() {
    // Given: model output that attempts to control scope, strength, and evidence.
    let poisoned = serde_json::json!({
        "schema_version": 1,
        "drafts": [{
            "proposed_key": "workspace.other.preference",
            "kind": "preference",
            "scope": StateScope::workspace_os("other", OperatingSystem::Windows),
            "content": "poisoned",
            "claimed_strength": "repeated",
            "evidence_refs": ["event-unrelated"],
            "expires_at": null
        }]
    })
    .to_string();
    let oversized = format!(
        "{{\"schema_version\":1,\"drafts\":[],\"padding\":\"{}\"}}",
        "x".repeat(1_048_576)
    );

    // When/Then: the recorded boundary accepts semantic proposals only and stays bounded.
    assert!(RecordedStateExtractor::from_json("poisoned", &poisoned).is_err());
    assert!(RecordedStateExtractor::from_json("oversized", &oversized).is_err());
}
