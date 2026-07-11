//! Trust-boundary tests for explicit state and revoke evidence.

use tempfile::tempdir;
use tsukumo_kernel::{
    CorrelationId, EventId, ExecutionId, KernelEvent, KernelEventPayload, OutcomeStatus,
    PersistedText, QuestId, RuntimeBinding, RuntimeKind, RuntimeMode, SensitiveText, SessionId,
    SpiritId, StateId, StateLifecycleAction, Timestamp, VendorEventRef,
    KERNEL_EVENT_SCHEMA_VERSION,
};
use tsukumo_soul::{
    ChronicleQuery, EvidenceStrength, ExtractionProvenance, OperatingSystem, SoulError, SoulStore,
    StateDraft, StateKey, StateKind, StateScope, StateTransition, StateValidationError,
    StateWriteRequest,
};

const SOURCE_TIME: i64 = 1_750_001_100_000;
const STATE_TIME: i64 = SOURCE_TIME + 1;

fn event(id: &str, timestamp: i64, payload: KernelEventPayload) -> KernelEvent {
    KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new(id),
        occurred_at: Timestamp::from_unix_millis(timestamp),
        quest_id: QuestId::new("quest-trust"),
        session_id: SessionId::new("session-trust"),
        spirit_id: SpiritId::new("yuka"),
        execution_id: None,
        runtime: None,
        causation_id: None,
        correlation_id: None,
        payload,
    }
}

fn lifecycle(
    id: &str,
    timestamp: i64,
    state_id: &StateId,
    action: StateLifecycleAction,
    reason: Option<&str>,
) -> KernelEvent {
    event(
        id,
        timestamp,
        KernelEventPayload::StateLifecycle {
            state_id: state_id.clone(),
            action,
            prior_state_id: None,
            reason: reason.map(PersistedText::from_reviewed),
        },
    )
}

fn explicit_draft(evidence: EventId, provenance: ExtractionProvenance) -> StateDraft {
    StateDraft {
        proposed_key: StateKey::new("workspace.tsukumo.rust.toolchain.windows"),
        kind: StateKind::Constraint,
        scope: StateScope::workspace_os("tsukumo", OperatingSystem::Windows),
        content: SensitiveText::new("Use the GNU Rust toolchain on Windows"),
        claimed_strength: EvidenceStrength::Explicit,
        evidence_refs: vec![evidence],
        provenance,
        expires_at: None,
    }
}

#[test]
fn structured_or_recorded_extractors_cannot_self_report_explicit_constraints() {
    let directory = tempdir().expect("create explicit trust directory");
    let source = event(
        "event-user-trust",
        SOURCE_TIME,
        KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed("Always use GNU on Windows"),
        },
    );
    let state_id = StateId::new("state-untrusted-explicit");
    let request = StateWriteRequest::new(
        StateTransition::Create {
            state_id: state_id.clone(),
            draft: explicit_draft(
                source.event_id.clone(),
                ExtractionProvenance::StructuredModel {
                    provider: "fixture".into(),
                    model: "recorded".into(),
                    schema_version: 1,
                },
            ),
            created_at: Timestamp::from_unix_millis(STATE_TIME),
        },
        lifecycle(
            "event-state-trust",
            STATE_TIME,
            &state_id,
            StateLifecycleAction::Created,
            None,
        ),
    )
    .with_source_event(source);
    let mut store = SoulStore::open(directory.path()).expect("open trust store");

    let error = store
        .apply_state(request)
        .expect_err("untrusted explicit claim must fail");
    assert!(matches!(
        error,
        SoulError::StateValidation(StateValidationError::UntrustedExplicit)
    ));
    assert!(store
        .replay_events(ChronicleQuery::default())
        .expect("replay rejected explicit claim")
        .is_empty());
}

#[test]
fn trusted_rule_still_requires_matching_user_input_evidence() {
    let directory = tempdir().expect("create evidence trust directory");
    let source = event(
        "event-outcome-trust",
        SOURCE_TIME,
        KernelEventPayload::Outcome {
            status: OutcomeStatus::Succeeded,
            summary: Some(PersistedText::from_reviewed("GNU build completed")),
            projection_id: None,
        },
    );
    let state_id = StateId::new("state-wrong-evidence");
    let request = StateWriteRequest::new(
        StateTransition::Create {
            state_id: state_id.clone(),
            draft: explicit_draft(
                source.event_id.clone(),
                ExtractionProvenance::Rule {
                    name: "explicit_gnu_constraint".into(),
                    version: 1,
                },
            ),
            created_at: Timestamp::from_unix_millis(STATE_TIME),
        },
        lifecycle(
            "event-state-wrong-evidence",
            STATE_TIME,
            &state_id,
            StateLifecycleAction::Created,
            None,
        ),
    )
    .with_source_event(source);
    let mut store = SoulStore::open(directory.path()).expect("open evidence store");

    let error = store
        .apply_state(request)
        .expect_err("non-user evidence must not become explicit");
    assert!(matches!(
        error,
        SoulError::StateValidation(StateValidationError::ExplicitEvidenceRequired)
    ));
}

#[test]
fn permission_evidence_cannot_revoke_relationship_state() {
    let directory = tempdir().expect("create revoke trust directory");
    let mut store = SoulStore::open(directory.path()).expect("open revoke store");
    let source = event(
        "event-user-create",
        SOURCE_TIME,
        KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed("Always use GNU on Windows"),
        },
    );
    let state_id = StateId::new("state-revoke-trust");
    store
        .apply_state(
            StateWriteRequest::new(
                StateTransition::Create {
                    state_id: state_id.clone(),
                    draft: explicit_draft(
                        source.event_id.clone(),
                        ExtractionProvenance::Rule {
                            name: "explicit_gnu_constraint".into(),
                            version: 1,
                        },
                    ),
                    created_at: Timestamp::from_unix_millis(STATE_TIME),
                },
                lifecycle(
                    "event-state-create",
                    STATE_TIME,
                    &state_id,
                    StateLifecycleAction::Created,
                    None,
                ),
            )
            .with_source_event(source),
        )
        .expect("create trusted state");

    let mut permission = event(
        "event-permission-revoke",
        STATE_TIME + 1,
        KernelEventPayload::PermissionRequested {
            vendor_request: VendorEventRef::new("fixture", "perm-revoke"),
            tool: "shell".into(),
            arguments: None,
            cwd: None,
            risk_reasons: Vec::new(),
            reason: PersistedText::from_reviewed("shell approval"),
        },
    );
    permission.execution_id = Some(ExecutionId::new("execution-revoke"));
    permission.runtime = Some(RuntimeBinding::new(
        RuntimeKind::ClaudeCli,
        RuntimeMode::Fixture,
    ));
    permission.correlation_id = Some(CorrelationId::new("perm-revoke"));
    let revoke_time = STATE_TIME + 2;
    let request = StateWriteRequest::new(
        StateTransition::Revoke {
            prior: state_id.clone(),
            evidence: permission.event_id.clone(),
            revoked_at: Timestamp::from_unix_millis(revoke_time),
        },
        lifecycle(
            "event-state-revoke",
            revoke_time,
            &state_id,
            StateLifecycleAction::Revoked,
            Some("permission cannot revoke state"),
        ),
    )
    .with_source_event(permission);

    let error = store
        .apply_state(request)
        .expect_err("permission evidence must not revoke state");
    assert!(matches!(
        error,
        SoulError::StateValidation(StateValidationError::PermissionEvidence(_))
    ));
    assert!(store
        .active_state_at(
            &StateKey::new("workspace.tsukumo.rust.toolchain.windows"),
            &StateScope::workspace_os("tsukumo", OperatingSystem::Windows),
            Timestamp::from_unix_millis(revoke_time),
        )
        .expect("query state after rejected revoke")
        .is_some());
}
