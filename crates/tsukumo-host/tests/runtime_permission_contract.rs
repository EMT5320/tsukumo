mod common;

use common::{prepared_fixture, FakeRunner, FixedClock, TestLedger};
use std::path::PathBuf;
use tsukumo_adapters::{parse_stream_json_line, ClaudeRuntimeProfile, RuntimeLaunchConfig};
use tsukumo_host::{
    ExecutionContext, ExecutionFailure, ExecutionPolicy, ExecutionRequest, HostError, HostServices,
    PermissionController, PermissionRequest, PermissionScope, Presentation, RuntimeOrchestrator,
    RuntimeOutput, RuntimeSelection,
};
use tsukumo_kernel::{
    KernelEventPayload, OutcomeStatus, PermissionDecision, RuntimeBinding, RuntimeKind,
    RuntimeMode, SessionId,
};
use tsukumo_theater::{DirectorContext, StageWorld};
const PERMISSION_LINE: &str = r#"{"type":"sdk_control_request","request":{"subtype":"permission","request_id":"permission_host","tool_name":"Bash","tool_input":{"command":"cargo test"}}}"#;

#[test]
fn permission_request_fails_closed_when_vendor_bridge_is_unwired() {
    // Given: Claude emits a vendor permission request during an owned execution.
    let (_directory, store, prepared) = prepared_fixture();
    let runner = FakeRunner::new([RuntimeOutput::StdoutLine(PERMISSION_LINE.into())]);
    let mut ledger = TestLedger::new(store, &runner);
    let clock = FixedClock::new(6_000);
    let mut world = StageWorld::new();
    let director = DirectorContext::default();
    let profile = ClaudeRuntimeProfile::deny_unapproved();
    let launch = RuntimeLaunchConfig::new(PathBuf::from("fake-claude"), PathBuf::from("."));
    let mut host = RuntimeOrchestrator::new(
        HostServices::new(&mut ledger, &runner, &clock),
        Presentation::new(&mut world, &director),
        ExecutionPolicy::default(),
    );

    // When: the Host reaches the explicitly unwired permission handoff.
    let report = host
        .execute(ExecutionRequest::new(
            &prepared,
            RuntimeSelection::new(&profile, &launch),
            ExecutionContext::new("quest-host", "session-host", "yuka"),
        ))
        .expect("unsupported safety is a controlled report");

    // Then: request evidence is durable, no decision is fabricated, and the child is reaped.
    assert_eq!(report.status, OutcomeStatus::SafetyUnsupported);
    assert_eq!(report.failure, Some(ExecutionFailure::SafetyUnsupported));
    assert_eq!(runner.cancel_count(), 1);

    // When: a human denial is resolved after the unsupported runtime handoff.
    let payload = parse_stream_json_line(PERMISSION_LINE)
        .expect("decode permission evidence")
        .pop()
        .expect("permission payload");
    let scope = PermissionScope::new(
        prepared.receipt.execution_id.clone(),
        SessionId::new("session-host"),
        RuntimeBinding::new(RuntimeKind::ClaudeCli, RuntimeMode::OwnedProcess),
    );
    let request = PermissionRequest::from_payload(scope, &payload)
        .expect("create pending permission evidence");
    let vendor_request = request.vendor_request.clone();
    let mut controller = PermissionController::default();
    controller
        .register(request)
        .expect("register permission evidence");
    let resolution = controller
        .decide(&vendor_request, PermissionDecision::Deny)
        .expect("record human denial");
    host.record_permission_resolution(
        &prepared.receipt,
        ExecutionContext::new("quest-host", "session-host", "yuka"),
        resolution,
    )
    .expect("persist permission decision");
    let events = ledger.execution_events(&prepared.receipt.execution_id);
    assert!(events.iter().any(|event| matches!(
        event.payload,
        KernelEventPayload::PermissionRequested { .. }
    )));
    assert!(events
        .iter()
        .any(|event| matches!(event.payload, KernelEventPayload::PermissionDecided { .. })));
}

#[test]
fn decision_recording_requires_matching_scope_and_durable_request_evidence() {
    // Given: a valid receipt and a controller resolution with no Chronicle request event.
    let (_directory, store, prepared) = prepared_fixture();
    let runner = FakeRunner::new([]);
    let mut ledger = TestLedger::new(store, &runner);
    let clock = FixedClock::new(6_500);
    let mut world = StageWorld::new();
    let director = DirectorContext::default();
    let payload = parse_stream_json_line(PERMISSION_LINE)
        .expect("decode missing permission evidence")
        .pop()
        .expect("permission payload");
    let scope = PermissionScope::new(
        prepared.receipt.execution_id.clone(),
        SessionId::new("session-host"),
        prepared.receipt.runtime.clone(),
    );
    let request = PermissionRequest::from_payload(scope, &payload)
        .expect("create missing permission evidence");
    let vendor_request = request.vendor_request.clone();
    let mut controller = PermissionController::default();
    controller
        .register(request)
        .expect("register missing evidence");
    let resolution = controller
        .decide(&vendor_request, PermissionDecision::Deny)
        .expect("resolve missing evidence");
    let mut host = RuntimeOrchestrator::new(
        HostServices::new(&mut ledger, &runner, &clock),
        Presentation::new(&mut world, &director),
        ExecutionPolicy::default(),
    );

    // When/Then: mismatched session scope fails before the ledger lookup.
    let scope_error = host
        .record_permission_resolution(
            &prepared.receipt,
            ExecutionContext::new("quest-host", "other-session", "yuka"),
            resolution.clone(),
        )
        .expect_err("mismatched permission scope must fail");
    assert!(matches!(scope_error, HostError::PermissionScopeMismatch));

    // When/Then: matching scope still cannot fabricate a decision without request evidence.
    let evidence_error = host
        .record_permission_resolution(
            &prepared.receipt,
            ExecutionContext::new("quest-host", "session-host", "yuka"),
            resolution,
        )
        .expect_err("missing request evidence must fail");
    assert!(matches!(
        evidence_error,
        HostError::MissingPermissionRequest
    ));
    assert_eq!(runner.spawn_count(), 0);
}
