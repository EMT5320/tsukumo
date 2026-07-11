mod common;

use common::{prepared_fixture, successful_outputs, FakeRunner, FixedClock, TestLedger};
use std::path::PathBuf;
use std::time::Duration;
use tsukumo_adapters::{ClaudeRuntimeProfile, RuntimeLaunchConfig};
use tsukumo_host::{
    CleanupStatus, ExecutionContext, ExecutionPolicy, ExecutionRequest, HostError, HostServices,
    Presentation, ProcessError, ProcessTreeCapability, RuntimeOrchestrator, RuntimeOutput,
    RuntimeSelection,
};
use tsukumo_kernel::{ExecutionId, KernelEventPayload, OutcomeStatus, RuntimePhase};
use tsukumo_theater::{DirectorContext, StageWorld};

#[test]
fn execution_is_receipt_first_incremental_and_chronicle_before_theater() {
    // Given: a committed projection, a fake Claude stream, and an empty theater.
    let (_directory, store, prepared) = prepared_fixture();
    let expected_prompt = prepared.rendered_prompt().expose().to_owned();
    let mut outputs = successful_outputs();
    outputs.insert(
        1,
        RuntimeOutput::StdoutLine(r#"{"type":"future_vendor_event"}"#.into()),
    );
    let runner = FakeRunner::new(outputs);
    let mut ledger = TestLedger::new(store, &runner);
    let clock = FixedClock::new(1_000);
    let mut world = StageWorld::new();
    let director = DirectorContext::default();
    let profile = ClaudeRuntimeProfile::deny_unapproved();
    let launch = RuntimeLaunchConfig::new(PathBuf::from("fake-claude"), PathBuf::from("."));
    let policy = ExecutionPolicy::new(Duration::from_secs(1), Duration::from_millis(1))
        .expect("valid execution policy");

    // When: the composition root owns the execution to completion.
    let report = {
        let mut host = RuntimeOrchestrator::new(
            HostServices::new(&mut ledger, &runner, &clock),
            Presentation::new(&mut world, &director),
            policy,
        );
        host.execute(ExecutionRequest::new(
            &prepared,
            RuntimeSelection::new(&profile, &launch),
            ExecutionContext::new("quest-host", "session-host", "yuka"),
        ))
        .expect("execute reviewed projection")
    };

    // Then: prompt delivery, durable ordering, attribution, and theater fan-out agree.
    assert_eq!(runner.spawn_count(), 1);
    assert_eq!(
        runner.captured_prompt().as_deref(),
        Some(expected_prompt.as_str())
    );
    assert_eq!(report.status, OutcomeStatus::Succeeded);
    assert_eq!(report.process_tree, ProcessTreeCapability::DirectChildOnly);
    assert_eq!(report.known_ignored_lines, 1);
    assert_eq!(report.unknown_skipped_lines, 1);
    assert!(matches!(report.cleanup, CleanupStatus::Natural(_)));
    assert!(ledger
        .append_before_exit
        .iter()
        .any(|name| name == "tool_start"));
    let events = ledger.execution_events(&prepared.receipt.execution_id);
    let payloads = events
        .iter()
        .map(|event| &event.payload)
        .filter(|payload| !matches!(payload, KernelEventPayload::ProjectionCreated { .. }))
        .collect::<Vec<_>>();
    assert!(matches!(
        payloads.first(),
        Some(KernelEventPayload::RuntimeLifecycle {
            phase: RuntimePhase::Starting
        })
    ));
    assert!(payloads.iter().any(|payload| matches!(
        payload,
        KernelEventPayload::ToolStart {
            projection_id: Some(id),
            ..
        } if id == &prepared.receipt.id
    )));
    assert!(matches!(
        payloads.last(),
        Some(KernelEventPayload::Outcome {
            status: OutcomeStatus::Succeeded,
            projection_id: Some(id),
            ..
        }) if id == &prepared.receipt.id
    ));
    assert!(world
        .log
        .iter()
        .any(|line| line.contains("tool_start Read")));
    assert!(world
        .log
        .back()
        .is_some_and(|line| line.contains("outcome")));
}

#[test]
fn missing_receipt_blocks_spawn() {
    // Given: a prepared value whose receipt is absent from the selected ledger.
    let (_source_directory, _source_store, prepared) = prepared_fixture();
    let target_directory = tempfile::tempdir().expect("create empty target ledger");
    let target_store =
        tsukumo_soul::SoulStore::open(target_directory.path()).expect("open target ledger");
    let runner = FakeRunner::new(successful_outputs());
    let mut ledger = TestLedger::new(target_store, &runner);
    let clock = FixedClock::new(2_000);
    let mut world = StageWorld::new();
    let director = DirectorContext::default();
    let profile = ClaudeRuntimeProfile::deny_unapproved();
    let launch = RuntimeLaunchConfig::new(PathBuf::from("fake-claude"), PathBuf::from("."));

    // When: execution preflight compares the prepared receipt with durable authority.
    let mut host = RuntimeOrchestrator::new(
        HostServices::new(&mut ledger, &runner, &clock),
        Presentation::new(&mut world, &director),
        ExecutionPolicy::default(),
    );
    let error = host
        .execute(ExecutionRequest::new(
            &prepared,
            RuntimeSelection::new(&profile, &launch),
            ExecutionContext::new("quest-host", "session-host", "yuka"),
        ))
        .expect_err("missing receipt must fail closed");

    // Then: no child process or presentation event exists.
    assert!(matches!(error, HostError::MissingReceipt { .. }));
    assert_eq!(runner.spawn_count(), 0);
    assert!(world.log.is_empty());
}

#[test]
fn chronicle_failure_cancels_before_theater_can_show_the_event() {
    // Given: Chronicle fails exactly when the first tool event is appended.
    let (_directory, store, prepared) = prepared_fixture();
    let runner = FakeRunner::new(successful_outputs());
    let mut ledger = TestLedger::new(store, &runner).fail_on_append(3);
    let clock = FixedClock::new(3_000);
    let mut world = StageWorld::new();
    let director = DirectorContext::default();
    let profile = ClaudeRuntimeProfile::deny_unapproved();
    let launch = RuntimeLaunchConfig::new(PathBuf::from("fake-claude"), PathBuf::from("."));

    // When: the third append fails while the fake process is still running.
    let error = {
        let mut host = RuntimeOrchestrator::new(
            HostServices::new(&mut ledger, &runner, &clock),
            Presentation::new(&mut world, &director),
            ExecutionPolicy::default(),
        );
        host.execute(ExecutionRequest::new(
            &prepared,
            RuntimeSelection::new(&profile, &launch),
            ExecutionContext::new("quest-host", "session-host", "yuka"),
        ))
        .expect_err("Chronicle failure must stop execution")
    };

    // Then: cleanup runs once and Theater contains no uncommitted tool event.
    assert!(matches!(
        error,
        HostError::ChronicleDuringExecution {
            cleanup: CleanupStatus::Cancelled(_),
            ..
        }
    ));
    assert_eq!(runner.cancel_count(), 1);
    assert!(!world.log.iter().any(|line| line.contains("tool_start")));
    let events = ledger.execution_events(&ExecutionId::new("execution-host"));
    assert_eq!(events.len(), 3);
}

#[test]
fn chronicle_failure_retains_cleanup_error_evidence() {
    // Given: Chronicle and child cleanup both fail on the first tool event.
    let (_directory, store, prepared) = prepared_fixture();
    let runner = FakeRunner::new(successful_outputs()).with_cleanup_failure();
    let mut ledger = TestLedger::new(store, &runner).fail_on_append(3);
    let clock = FixedClock::new(3_500);
    let mut world = StageWorld::new();
    let director = DirectorContext::default();
    let profile = ClaudeRuntimeProfile::deny_unapproved();
    let launch = RuntimeLaunchConfig::new(PathBuf::from("fake-claude"), PathBuf::from("."));
    let mut host = RuntimeOrchestrator::new(
        HostServices::new(&mut ledger, &runner, &clock),
        Presentation::new(&mut world, &director),
        ExecutionPolicy::default(),
    );

    // When: durable commit failure triggers the mandatory cleanup path.
    let error = host
        .execute(ExecutionRequest::new(
            &prepared,
            RuntimeSelection::new(&profile, &launch),
            ExecutionContext::new("quest-host", "session-host", "yuka"),
        ))
        .expect_err("combined Chronicle and cleanup failure must surface");

    // Then: the primary Chronicle error keeps typed cleanup evidence for operators.
    assert!(matches!(
        error,
        HostError::ChronicleDuringExecution {
            cleanup: CleanupStatus::Failed,
            cleanup_error: Some(ProcessError::Kill(_)),
            ..
        }
    ));
    assert_eq!(runner.cancel_count(), 1);
}
