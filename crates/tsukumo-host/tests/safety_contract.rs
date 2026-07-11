use tsukumo_host::{
    BridgeError, PermissionBridge, PermissionController, PermissionRegistration, PermissionRequest,
    PermissionResolutionSource, PermissionScope, SafetyError, UnwiredPermissionBridge,
};
use tsukumo_kernel::{
    CorrelationId, EventId, ExecutionId, KernelEvent, KernelEventPayload, PermissionDecision,
    PersistedJson, PersistedText, QuestId, RuntimeBinding, RuntimeKind, RuntimeMode, SessionId,
    SpiritId, Timestamp, VendorEventRef, KERNEL_EVENT_SCHEMA_VERSION,
};
use tsukumo_soul::{
    ExtractionContext, OperatingSystem, RuleStateExtractor, StateExtractor, StateScope,
};

fn request(id: &str, session: &str, tool: &str) -> PermissionRequest {
    let payload = KernelEventPayload::PermissionRequested {
        vendor_request: VendorEventRef::new("claude_cli", id),
        tool: tool.into(),
        arguments: Some(PersistedJson::from_reviewed(
            serde_json::json!({"path": "DESIGN.md"}),
        )),
        cwd: Some(PersistedText::from_reviewed("D:/WorkSpace/tsukumo")),
        risk_reasons: vec![PersistedText::from_reviewed("filesystem read")],
        reason: PersistedText::from_reviewed("Review project design"),
    };
    PermissionRequest::from_payload(
        PermissionScope::new(
            ExecutionId::new(format!("execution-{id}")),
            SessionId::new(session),
            RuntimeBinding::new(RuntimeKind::ClaudeCli, RuntimeMode::OwnedProcess),
        ),
        &payload,
    )
    .expect("build permission request")
}

#[test]
fn allow_once_does_not_cover_repeats_and_resolved_requests_are_stale() {
    // Given: one pending runtime permission request.
    let first = request("permission-1", "session-a", "Read");
    let first_ref = first.vendor_request.clone();
    let mut controller = PermissionController::default();
    assert_eq!(
        controller
            .register(first.clone())
            .expect("register request"),
        PermissionRegistration::Pending
    );

    // When/Then: duplicates are rejected until one human decision resolves the request.
    assert!(matches!(
        controller.register(first),
        Err(SafetyError::DuplicateRequest { .. })
    ));
    let resolution = controller
        .decide(&first_ref, PermissionDecision::AllowOnce)
        .expect("allow one request");
    assert_eq!(resolution.source, PermissionResolutionSource::HumanDecision);
    assert!(matches!(
        controller.decide(&first_ref, PermissionDecision::Deny),
        Err(SafetyError::StaleRequest { .. })
    ));

    // Then: the same tool under a new vendor request still requires a decision.
    assert_eq!(
        controller
            .register(request("permission-2", "session-a", "Read"))
            .expect("register repeated tool"),
        PermissionRegistration::Pending
    );
}

#[test]
fn allow_session_covers_only_the_same_session_runtime_and_tool() {
    // Given: a human grants one tool for the current session.
    let first = request("permission-1", "session-a", "Read");
    let first_ref = first.vendor_request.clone();
    let mut controller = PermissionController::default();
    controller.register(first).expect("register first request");
    controller
        .decide(&first_ref, PermissionDecision::AllowSession)
        .expect("grant current session");

    // When: the same tool asks again under the same runtime and session.
    let covered = controller
        .register(request("permission-2", "session-a", "Read"))
        .expect("apply session grant");

    // Then: the prior human grant is cited, while another session remains pending.
    assert!(matches!(
        covered,
        PermissionRegistration::Covered(ref resolution)
            if resolution.decision == PermissionDecision::AllowSession
                && resolution.source == PermissionResolutionSource::SessionGrant
    ));
    assert_eq!(
        controller
            .register(request("permission-3", "session-b", "Read"))
            .expect("register another session"),
        PermissionRegistration::Pending
    );
}

#[test]
fn unwired_bridge_is_explicit_and_permission_decisions_do_not_create_state() {
    // Given: one denied request resolved by the deterministic controller.
    let request = request("permission-denied", "session-a", "Bash");
    let vendor_request = request.vendor_request.clone();
    let scope = request.scope.clone();
    let mut controller = PermissionController::default();
    controller
        .register(request)
        .expect("register denied request");
    let resolution = controller
        .decide(&vendor_request, PermissionDecision::Deny)
        .expect("deny request");

    // When: C1 attempts to apply it through the deliberately unwired vendor seam.
    let mut bridge = UnwiredPermissionBridge;
    let bridge_error = bridge
        .apply(&resolution)
        .expect_err("unwired bridge must fail closed");

    // Then: unsupported wiring is typed and the decision is evidence-only for state extraction.
    assert_eq!(bridge_error, BridgeError::Unsupported);
    let event = KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new("event-permission-denied"),
        occurred_at: Timestamp::from_unix_millis(10),
        quest_id: QuestId::new("quest-host"),
        session_id: scope.session_id,
        spirit_id: SpiritId::new("yuka"),
        execution_id: Some(scope.execution_id),
        runtime: Some(scope.runtime),
        causation_id: None,
        correlation_id: Some(CorrelationId::new("correlation-permission-denied")),
        payload: resolution.into_payload(),
    };
    let drafts = RuleStateExtractor
        .extract(&ExtractionContext {
            event: &event,
            scope: StateScope::workspace_os("tsukumo", OperatingSystem::Windows),
        })
        .expect("extract permission decision");
    assert!(drafts.is_empty());
}
