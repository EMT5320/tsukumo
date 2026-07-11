//! C1 rule and recorded StateExtractor contract tests.

use tsukumo_kernel::{
    EventId, KernelEvent, KernelEventPayload, PersistedText, QuestId, SessionId, SpiritId,
    Timestamp, VendorEventRef, WorkspaceId, KERNEL_EVENT_SCHEMA_VERSION,
};
use tsukumo_soul::{
    extract_non_blocking, EvidenceStrength, ExtractError, ExtractionAttempt, ExtractionContext,
    OperatingSystem, RecordedStateExtractor, RuleStateExtractor, StateExtractor, StateKind,
    StateScope, StateSubject,
};

fn event(payload: KernelEventPayload) -> KernelEvent {
    KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new("event-extract"),
        occurred_at: Timestamp::from_unix_millis(1_750_000_400_000),
        quest_id: QuestId::new("quest-extract"),
        session_id: SessionId::new("session-extract"),
        spirit_id: SpiritId::new("yuka"),
        execution_id: None,
        runtime: None,
        causation_id: None,
        correlation_id: None,
        payload,
    }
}

#[test]
fn explicit_gnu_user_input_produces_scoped_constraint_draft() {
    // Given: explicit user evidence and a resolved workspace/OS scope.
    let source = event(KernelEventPayload::UserInput {
        content: PersistedText::from_reviewed(
            "Tsukumo always uses the GNU Rust toolchain on Windows",
        ),
    });
    let context = ExtractionContext {
        event: &source,
        scope: StateScope::workspace_os("tsukumo", OperatingSystem::Windows),
    };

    // When: the deterministic rule extractor evaluates the event.
    let drafts = RuleStateExtractor
        .extract(&context)
        .expect("extract explicit constraint");

    // Then: it proposes one explicit constraint with Chronicle evidence.
    assert_eq!(drafts.len(), 1);
    assert_eq!(drafts[0].kind, StateKind::Constraint);
    assert_eq!(drafts[0].claimed_strength, EvidenceStrength::Explicit);
    assert_eq!(drafts[0].evidence_refs, vec![source.event_id.clone()]);
    assert_eq!(
        drafts[0].proposed_key.as_str(),
        "workspace.tsukumo.rust.toolchain.windows"
    );
    assert_eq!(
        drafts[0].scope.subject,
        StateSubject::Workspace {
            workspace_id: WorkspaceId::new("tsukumo")
        }
    );
    assert_eq!(
        drafts[0].scope.applicability.task_tags,
        vec!["rust_build", "rust_test"]
    );
    assert_eq!(drafts[0].scope.applicability.language_tags, vec!["rust"]);
}

#[test]
fn permission_events_and_irrelevant_text_produce_no_state_drafts() {
    // Given: one permission request and one unrelated user message.
    let permission = event(KernelEventPayload::PermissionRequested {
        vendor_request: VendorEventRef::new("fixture", "perm-1"),
        tool: "shell".into(),
        arguments: None,
        cwd: None,
        risk_reasons: Vec::new(),
        reason: PersistedText::from_reviewed("shell: cargo test"),
    });
    let unrelated = event(KernelEventPayload::UserInput {
        content: PersistedText::from_reviewed("Please summarize the README"),
    });
    let scope = StateScope::workspace_os("tsukumo", OperatingSystem::Windows);

    // When: deterministic extraction evaluates both events.
    let permission_drafts = RuleStateExtractor
        .extract(&ExtractionContext {
            event: &permission,
            scope: scope.clone(),
        })
        .expect("extract permission");
    let unrelated_drafts = RuleStateExtractor
        .extract(&ExtractionContext {
            event: &unrelated,
            scope,
        })
        .expect("extract unrelated text");

    // Then: neither event gains accidental durable relationship meaning.
    assert!(permission_drafts.is_empty());
    assert!(unrelated_drafts.is_empty());
}

#[test]
fn recorded_extractor_failures_become_recoverable_observable_skips() {
    let source = event(KernelEventPayload::UserInput {
        content: PersistedText::from_reviewed("Remember this preference"),
    });
    let context = ExtractionContext {
        event: &source,
        scope: StateScope::workspace_os("tsukumo", OperatingSystem::Windows),
    };

    let malformed = extract_non_blocking(
        &RecordedStateExtractor::malformed("fixture-malformed"),
        &context,
    );
    let timeout = extract_non_blocking(
        &RecordedStateExtractor::timeout("fixture-timeout"),
        &context,
    );
    let parsed = RecordedStateExtractor::from_json("invalid-json", "{")
        .expect_err("invalid recorded DTO must fail schema parsing");

    for attempt in [malformed, timeout] {
        let ExtractionAttempt::Skipped { error, event } = attempt else {
            panic!("failure must become a skipped attempt");
        };
        assert!(matches!(
            error,
            ExtractError::Malformed { .. } | ExtractError::Timeout { .. }
        ));
        assert!(matches!(
            event,
            KernelEventPayload::Error {
                recoverable: true,
                ..
            }
        ));
    }
    assert!(matches!(parsed, ExtractError::Malformed { .. }));
}
