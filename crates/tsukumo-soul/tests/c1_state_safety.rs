//! C1 StateWriter evidence, scope, and secret-policy tests.

use tempfile::tempdir;
use tsukumo_kernel::{
    EventId, KernelEvent, KernelEventPayload, PersistedText, QuestId, SensitiveText, SessionId,
    SpiritId, StateId, StateLifecycleAction, Timestamp, KERNEL_EVENT_SCHEMA_VERSION,
};
use tsukumo_soul::{
    ChronicleQuery, EvidenceStrength, ExtractionProvenance, OperatingSystem, SoulError, SoulStore,
    StateDraft, StateKey, StateKind, StateScope, StateTransition, StateValidationError,
    StateWriteRequest,
};

fn event(id: &str, payload: KernelEventPayload) -> KernelEvent {
    event_at(id, 1_750_000_700_000, payload)
}

fn event_at(id: &str, timestamp: i64, payload: KernelEventPayload) -> KernelEvent {
    KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new(id),
        occurred_at: Timestamp::from_unix_millis(timestamp),
        quest_id: QuestId::new("quest-safety"),
        session_id: SessionId::new("session-safety"),
        spirit_id: SpiritId::new("yuka"),
        execution_id: None,
        runtime: None,
        causation_id: None,
        correlation_id: None,
        payload,
    }
}

fn request(source: Option<KernelEvent>, draft: StateDraft) -> StateWriteRequest {
    let causation_id = draft.evidence_refs.first().cloned();
    let state_id = StateId::new("state-safety");
    let request = StateWriteRequest::new(
        StateTransition::Create {
            state_id: state_id.clone(),
            draft,
            created_at: Timestamp::from_unix_millis(1_750_000_700_001),
        },
        {
            let mut lifecycle = event_at(
                "event-state-safety",
                1_750_000_700_001,
                KernelEventPayload::StateLifecycle {
                    state_id,
                    action: StateLifecycleAction::Created,
                    prior_state_id: None,
                    reason: None,
                },
            );
            lifecycle.causation_id = causation_id;
            lifecycle
        },
    );
    match source {
        Some(source) => request.with_source_event(source),
        None => request,
    }
}

fn draft(evidence: EventId, content: &str, scope: StateScope) -> StateDraft {
    StateDraft {
        proposed_key: StateKey::new("workspace.tsukumo.safety"),
        kind: StateKind::Fact,
        scope,
        content: SensitiveText::new(content),
        claimed_strength: EvidenceStrength::Inferred,
        evidence_refs: vec![evidence],
        provenance: ExtractionProvenance::Recorded {
            fixture: "state-safety".into(),
            schema_version: 1,
        },
        expires_at: None,
    }
}

#[test]
fn missing_evidence_rejects_without_lifecycle_event() {
    // Given: a draft referencing an event absent from Chronicle.
    let directory = tempdir().expect("create missing evidence directory");
    let evidence = EventId::new("event-missing");
    let mut store = SoulStore::open(directory.path()).expect("open state store");

    // When: StateWriter evaluates the incomplete request.
    let error = store
        .apply_state(request(
            None,
            draft(
                evidence.clone(),
                "ordinary fact",
                StateScope::workspace_os("tsukumo", OperatingSystem::Windows),
            ),
        ))
        .expect_err("missing evidence must fail");

    // Then: the failure is typed and no lifecycle event leaks into Chronicle.
    assert!(matches!(
        error,
        SoulError::StateValidation(StateValidationError::MissingEvidence(id))
            if id == evidence
    ));
    assert!(store
        .replay_events(ChronicleQuery::default())
        .expect("replay rejected request")
        .is_empty());
}

#[test]
fn secret_material_rejects_and_rolls_back_source_event() {
    // Given: a real user event and a draft containing credential-like text.
    let directory = tempdir().expect("create secret policy directory");
    let source = event(
        "event-user-safety",
        KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed("remember a setting"),
        },
    );
    let mut store = SoulStore::open(directory.path()).expect("open state store");

    // When: StateWriter evaluates the secret-bearing proposal.
    let error = store
        .apply_state(request(
            Some(source.clone()),
            draft(
                source.event_id,
                "password=hunter2",
                StateScope::workspace_os("tsukumo", OperatingSystem::Windows),
            ),
        ))
        .expect_err("secret material must fail");

    // Then: secret policy is typed and the source event is rolled back.
    assert!(matches!(
        error,
        SoulError::StateValidation(StateValidationError::SecretMaterial)
    ));
    assert!(store
        .replay_events(ChronicleQuery::default())
        .expect("replay secret rejection")
        .is_empty());
}

#[test]
fn unresolved_scope_is_reserved_for_explicit_legacy_import() {
    // Given: a normal rule draft with unresolved scope.
    let directory = tempdir().expect("create scope policy directory");
    let source = event(
        "event-user-scope",
        KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed("remember a fact"),
        },
    );
    let mut store = SoulStore::open(directory.path()).expect("open state store");

    // When: StateWriter evaluates the unresolved non-legacy draft.
    let error = store
        .apply_state(request(
            Some(source.clone()),
            draft(source.event_id, "ordinary fact", StateScope::unresolved()),
        ))
        .expect_err("unresolved normal scope must fail");

    // Then: only the explicit legacy importer may use unresolved scope.
    assert!(matches!(
        error,
        SoulError::StateValidation(StateValidationError::UnresolvedScope)
    ));
}

#[test]
fn ordinary_words_containing_key_prefix_text_are_not_secrets() {
    // Given: a legitimate sentence containing the characters "sk-".
    let directory = tempdir().expect("create secret false-positive directory");
    let source = event(
        "event-user-legitimate",
        KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed("remember approval behavior"),
        },
    );
    let mut store = SoulStore::open(directory.path()).expect("open state store");

    // When: deterministic secret policy evaluates ordinary prose.
    let outcome = store.apply_state(request(
        Some(source.clone()),
        draft(
            source.event_id,
            "Ask-for approval before shell commands",
            StateScope::workspace_os("tsukumo", OperatingSystem::Windows),
        ),
    ));

    // Then: short prefix-like prose remains valid state content.
    assert!(
        outcome.is_ok(),
        "unexpected secret false positive: {outcome:?}"
    );
}

#[test]
fn common_secret_shapes_are_rejected_before_state_or_export_persistence() {
    let samples = [
        r#"api_key: "SENTINEL-Aa1234567890_SECRET""#,
        r#"{"password":"SENTINEL-Aa1234567890_SECRET"}"#,
        "github_pat_1234567890AaBbCcDdEe",
        "AKIA1234567890ABCDEF",
        "xoxb-1234567890-AaBbCcDdEeFf",
        "Aa1234567890_BbCcDdEeFfGgHhIiJjKkLl",
    ];
    for (index, content) in samples.into_iter().enumerate() {
        let directory = tempdir().expect("create secret matrix directory");
        let source = event(
            &format!("event-secret-matrix-{index}"),
            KernelEventPayload::UserInput {
                content: PersistedText::from_reviewed("remember a safe label"),
            },
        );
        let mut store = SoulStore::open(directory.path()).expect("open secret matrix store");

        let error = store
            .apply_state(request(
                Some(source.clone()),
                draft(
                    source.event_id,
                    content,
                    StateScope::workspace_os("tsukumo", OperatingSystem::Windows),
                ),
            ))
            .expect_err("secret-shaped state must fail");
        assert!(
            matches!(
                error,
                SoulError::StateValidation(StateValidationError::SecretMaterial)
            ),
            "unexpected error for sample {index}: {error:?}"
        );
        assert!(store
            .replay_events(ChronicleQuery::default())
            .expect("replay rejected secret")
            .is_empty());
    }
}
