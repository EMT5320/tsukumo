mod common;

use common::{prepared_fixture, successful_outputs, FakeRunner, FixedClock, TestLedger};
use std::path::PathBuf;
use std::time::Duration;
use tsukumo_adapters::{ClaudeRuntimeProfile, RuntimeLaunchConfig};
use tsukumo_host::{
    CancellationToken, CleanupStatus, ExecutionContext, ExecutionFailure, ExecutionPolicy,
    ExecutionRequest, HostServices, Presentation, RuntimeOrchestrator, RuntimeOutput,
    RuntimeSelection,
};
use tsukumo_kernel::{KernelEventPayload, OutcomeStatus, SessionId, SpiritId};
use tsukumo_theater::{DirectorContext, StageWorld};

#[test]
fn malformed_truncated_timeout_and_nonzero_paths_have_distinct_reports() {
    let cases = vec![
        (
            vec![RuntimeOutput::StdoutLine(
                r#"{"type":"tool_result","content":"missing id"}"#.into(),
            )],
            OutcomeStatus::MalformedOutput,
            ExecutionFailure::MalformedOutput,
        ),
        (
            vec![RuntimeOutput::Exited(tsukumo_host::ProcessExit {
                code: Some(0),
                success: true,
            })],
            OutcomeStatus::MalformedOutput,
            ExecutionFailure::TruncatedStream,
        ),
        (
            vec![RuntimeOutput::Exited(tsukumo_host::ProcessExit {
                code: Some(9),
                success: false,
            })],
            OutcomeStatus::NonZeroExit,
            ExecutionFailure::NonZeroExit,
        ),
    ];

    for (outputs, expected_status, expected_failure) in cases {
        let (_directory, store, prepared) = prepared_fixture();
        let runner = FakeRunner::new(outputs);
        let mut ledger = TestLedger::new(store, &runner);
        let clock = FixedClock::new(4_000);
        let mut world = StageWorld::new();
        let director = DirectorContext::default();
        let profile = ClaudeRuntimeProfile::deny_unapproved();
        let launch = RuntimeLaunchConfig::new(PathBuf::from("fake-claude"), PathBuf::from("."));
        let mut host = RuntimeOrchestrator::new(
            HostServices::new(&mut ledger, &runner, &clock),
            Presentation::new(&mut world, &director),
            ExecutionPolicy::default(),
        );

        let report = host
            .execute(ExecutionRequest::new(
                &prepared,
                RuntimeSelection::new(&profile, &launch),
                ExecutionContext::new("quest-host", "session-host", "yuka"),
            ))
            .expect("controlled runtime failure");
        assert_eq!(report.status, expected_status);
        assert_eq!(report.failure, Some(expected_failure));
        assert!(matches!(
            ledger
                .execution_events(&prepared.receipt.execution_id)
                .last()
                .map(|event| &event.payload),
            Some(KernelEventPayload::Outcome { status, .. }) if *status == expected_status
        ));
    }

    // Given: an idle process and a one-millisecond execution budget.
    let (_directory, store, prepared) = prepared_fixture();
    let runner = FakeRunner::new([]);
    let mut ledger = TestLedger::new(store, &runner);
    let clock = FixedClock::new(5_000);
    let mut world = StageWorld::new();
    let director = DirectorContext::default();
    let profile = ClaudeRuntimeProfile::deny_unapproved();
    let launch = RuntimeLaunchConfig::new(PathBuf::from("fake-claude"), PathBuf::from("."));
    let policy = ExecutionPolicy::new(Duration::from_millis(1), Duration::from_millis(1))
        .expect("valid timeout policy");
    let mut host = RuntimeOrchestrator::new(
        HostServices::new(&mut ledger, &runner, &clock),
        Presentation::new(&mut world, &director),
        policy,
    );

    // When: no output or exit arrives before the budget.
    let report = host
        .execute(ExecutionRequest::new(
            &prepared,
            RuntimeSelection::new(&profile, &launch),
            ExecutionContext::new(
                tsukumo_kernel::QuestId::new("quest-host"),
                SessionId::new("session-host"),
                SpiritId::new("yuka"),
            ),
        ))
        .expect("timeout is a controlled report");

    // Then: timeout is preserved independently and cleanup is attempted once.
    assert_eq!(report.status, OutcomeStatus::TimedOut);
    assert_eq!(report.failure, Some(ExecutionFailure::TimedOut));
    assert_eq!(runner.cancel_count(), 1);
}

#[test]
fn explicit_cancellation_is_distinct_and_reaps_once() {
    // Given: a valid execution whose cooperative token is already cancelled.
    let (_directory, store, prepared) = prepared_fixture();
    let runner = FakeRunner::new(successful_outputs());
    let mut ledger = TestLedger::new(store, &runner);
    let clock = FixedClock::new(7_000);
    let mut world = StageWorld::new();
    let director = DirectorContext::default();
    let profile = ClaudeRuntimeProfile::deny_unapproved();
    let launch = RuntimeLaunchConfig::new(PathBuf::from("fake-claude"), PathBuf::from("."));
    let cancellation = CancellationToken::default();
    cancellation.cancel();
    let mut host = RuntimeOrchestrator::new(
        HostServices::new(&mut ledger, &runner, &clock),
        Presentation::new(&mut world, &director),
        ExecutionPolicy::default(),
    );

    // When: the running loop observes the caller-owned cancellation flag.
    let report = host
        .execute(
            ExecutionRequest::new(
                &prepared,
                RuntimeSelection::new(&profile, &launch),
                ExecutionContext::new("quest-host", "session-host", "yuka"),
            )
            .with_cancellation(cancellation),
        )
        .expect("cancellation is a controlled report");

    // Then: cancellation keeps its own outcome and exactly one cleanup request.
    assert_eq!(report.status, OutcomeStatus::Cancelled);
    assert_eq!(report.failure, Some(ExecutionFailure::Cancelled));
    assert_eq!(runner.cancel_count(), 1);
}

#[test]
fn launch_failure_is_persisted_without_cleanup_claims() {
    // Given: a receipt-valid execution whose process allocator rejects spawn.
    let (_directory, store, prepared) = prepared_fixture();
    let runner = FakeRunner::failing();
    let mut ledger = TestLedger::new(store, &runner);
    let clock = FixedClock::new(8_000);
    let mut world = StageWorld::new();
    let director = DirectorContext::default();
    let profile = ClaudeRuntimeProfile::deny_unapproved();
    let launch = RuntimeLaunchConfig::new(PathBuf::from("missing-claude"), PathBuf::from("."));
    let mut host = RuntimeOrchestrator::new(
        HostServices::new(&mut ledger, &runner, &clock),
        Presentation::new(&mut world, &director),
        ExecutionPolicy::default(),
    );

    // When: spawn fails after the start-requested event is durable.
    let report = host
        .execute(ExecutionRequest::new(
            &prepared,
            RuntimeSelection::new(&profile, &launch),
            ExecutionContext::new("quest-host", "session-host", "yuka"),
        ))
        .expect("launch failure is a controlled report");

    // Then: launch failure is terminal and no nonexistent child cleanup is claimed.
    assert_eq!(report.status, OutcomeStatus::LaunchFailed);
    assert_eq!(report.failure, Some(ExecutionFailure::LaunchFailed));
    assert_eq!(report.cleanup, CleanupStatus::NotStarted);
    assert_eq!(runner.spawn_count(), 1);
    assert_eq!(runner.cancel_count(), 0);
}
