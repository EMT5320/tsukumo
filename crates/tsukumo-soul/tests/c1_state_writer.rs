//! C1 deterministic StateWriter transaction and validation tests.

use tempfile::tempdir;
use tsukumo_kernel::{
    CorrelationId, EventId, ExecutionId, KernelEvent, KernelEventPayload, PersistedText, QuestId,
    RuntimeBinding, RuntimeKind, RuntimeMode, SensitiveText, SessionId, SpiritId, StateId,
    StateLifecycleAction, Timestamp, VendorEventRef, KERNEL_EVENT_SCHEMA_VERSION,
};
use tsukumo_soul::{
    ChronicleQuery, EvidenceStrength, ExtractionContext, ExtractionProvenance, OperatingSystem,
    RuleStateExtractor, SoulError, SoulStore, StateDraft, StateExtractor, StateKey, StateKind,
    StateScope, StateTransition, StateWriteOutcome, StateWriteRequest,
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

fn gnu_draft(evidence_id: EventId, strength: EvidenceStrength) -> StateDraft {
    StateDraft {
        proposed_key: StateKey::new("workspace.tsukumo.rust.toolchain.windows"),
        kind: StateKind::Constraint,
        scope: StateScope::workspace_os("tsukumo", OperatingSystem::Windows),
        content: SensitiveText::new("Use the GNU Rust toolchain on Windows"),
        claimed_strength: strength,
        evidence_refs: vec![evidence_id],
        provenance: ExtractionProvenance::Rule {
            name: "explicit_gnu_constraint".into(),
            version: 1,
        },
        expires_at: None,
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
fn source_event_state_and_lifecycle_commit_atomically_and_reopen() {
    // Given: an explicit user event and a state draft referencing it.
    let directory = tempdir().expect("create state test directory");
    let source = event(
        "event-user-1",
        KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed("Tsukumo uses the GNU Rust toolchain on Windows"),
        },
    );
    let extracted = RuleStateExtractor
        .extract(&ExtractionContext {
            event: &source,
            scope: StateScope::workspace_os("tsukumo", OperatingSystem::Windows),
        })
        .expect("extract explicit GNU constraint");
    let draft = extracted
        .into_iter()
        .next()
        .expect("one explicit GNU draft");
    let state_id = StateId::new("state-gnu-1");
    let request = StateWriteRequest::new(
        StateTransition::Create {
            state_id: state_id.clone(),
            draft,
            created_at: Timestamp::from_unix_millis(1_750_000_200_001),
        },
        lifecycle_event("event-state-1", &state_id),
    )
    .with_source_event(source);

    // When: StateWriter applies the request and the database is reopened.
    {
        let mut store = SoulStore::open(directory.path()).expect("open state store");
        let outcome = store.apply_state(request).expect("apply explicit state");
        assert!(matches!(outcome, StateWriteOutcome::Created(_)));
    }
    let store = SoulStore::open(directory.path()).expect("reopen state store");
    let record = store
        .state(&state_id)
        .expect("query state")
        .expect("persisted state");
    assert!(store
        .list_active_states_limited(0)
        .expect("query zero state limit")
        .is_empty());
    assert_eq!(
        store
            .list_active_states_limited(1)
            .expect("query one state limit")
            .len(),
        1
    );

    // Then: state content, evidence, version, and both Chronicle events survive.
    assert_eq!(
        record.content.as_str(),
        "Use the GNU Rust toolchain on Windows"
    );
    assert_eq!(record.version, 1);
    assert_eq!(record.evidence_refs, vec![EventId::new("event-user-1")]);
    assert_eq!(
        store
            .replay_events(ChronicleQuery::default())
            .expect("replay state transaction")
            .len(),
        2
    );
}

#[test]
fn inferred_hard_constraint_rejects_and_rolls_back_source_event() {
    // Given: an inferred draft attempting to create a hard constraint.
    let directory = tempdir().expect("create inferred test directory");
    let source = event(
        "event-user-1",
        KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed("Maybe GNU was used once"),
        },
    );
    let state_id = StateId::new("state-gnu-1");
    let request = StateWriteRequest::new(
        StateTransition::Create {
            state_id: state_id.clone(),
            draft: gnu_draft(source.event_id.clone(), EvidenceStrength::Inferred),
            created_at: Timestamp::from_unix_millis(1_750_000_200_001),
        },
        lifecycle_event("event-state-1", &state_id),
    )
    .with_source_event(source);
    let mut store = SoulStore::open(directory.path()).expect("open state store");

    // When: deterministic validation evaluates the request.
    let error = store
        .apply_state(request)
        .expect_err("inferred constraint must fail");

    // Then: neither the source event nor a partial state is committed.
    assert!(matches!(error, SoulError::StateValidation(_)));
    assert!(store
        .state(&state_id)
        .expect("query rejected state")
        .is_none());
    assert!(store
        .replay_events(ChronicleQuery::default())
        .expect("replay after rejection")
        .is_empty());
}

#[test]
fn permission_event_cannot_become_relationship_state() {
    // Given: a permission request used as the only proposed evidence.
    let directory = tempdir().expect("create permission test directory");
    let source = event(
        "event-permission-1",
        KernelEventPayload::PermissionRequested {
            vendor_request: VendorEventRef::new("fixture", "perm-1"),
            tool: "shell".into(),
            arguments: None,
            cwd: None,
            risk_reasons: Vec::new(),
            reason: PersistedText::from_reviewed("shell: cargo test"),
        },
    );
    let mut source = source;
    source.execution_id = Some(ExecutionId::new("execution-permission-1"));
    source.runtime = Some(RuntimeBinding::new(
        RuntimeKind::ClaudeCli,
        RuntimeMode::Fixture,
    ));
    source.correlation_id = Some(CorrelationId::new("perm-1"));
    let state_id = StateId::new("state-permission-1");
    let mut draft = gnu_draft(source.event_id.clone(), EvidenceStrength::Explicit);
    draft.proposed_key = StateKey::new("permissions.auto_approve.shell");
    draft.content = SensitiveText::new("Always approve shell commands");
    let request = StateWriteRequest::new(
        StateTransition::Create {
            state_id: state_id.clone(),
            draft,
            created_at: Timestamp::from_unix_millis(1_750_000_200_001),
        },
        lifecycle_event("event-state-1", &state_id),
    )
    .with_source_event(source);
    let mut store = SoulStore::open(directory.path()).expect("open state store");

    // When: StateWriter evaluates the permission-derived draft.
    let error = store
        .apply_state(request)
        .expect_err("permission evidence must fail");

    // Then: the safety decision remains outside canonical relationship state.
    assert!(matches!(error, SoulError::StateValidation(_)));
    assert!(store
        .replay_events(ChronicleQuery::default())
        .expect("replay after permission rejection")
        .is_empty());
}
