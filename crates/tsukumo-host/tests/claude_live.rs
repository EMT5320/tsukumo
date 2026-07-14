mod common;

use common::prepared_fixture_with_goal;
use std::path::PathBuf;
use std::time::Duration;
use tsukumo_adapters::{ClaudeRuntimeProfile, RuntimeLaunchConfig};
use tsukumo_host::{
    ExecutionContext, ExecutionPolicy, ExecutionRequest, HostServices, Presentation, ProcessLaunch,
    ProcessLimits, ProcessRunner, RuntimeOrchestrator, RuntimeOutput, RuntimeSelection,
    StandardProcessRunner, SystemClock,
};
use tsukumo_kernel::OutcomeStatus;
use tsukumo_soul::PreparedProjection;
use tsukumo_theater::{DirectorContext, StageWorld};

const LIVE_GOAL: &str = "Complete one tool-free Host connectivity turn.";
const REVIEWED_LIVE_PROJECTION: &str = concat!(
    "# Tsukumo handoff v1\n",
    "Precedence: current user instructions and repository rules override this handoff.\n\n",
    "## Goal\nHost process contract is ready\n\n",
    "## Current progress\n- (none)\n\n",
    "## Decisions\n- (none)\n\n",
    "## Constraints\n- (none)\n\n",
    "## Artifacts\n- (none)\n\n",
    "## Open loops\n- (none)\n\n",
    "## Next actions\n- (none)\n\n",
    "## Delegation goal\n",
    "Complete one tool-free Host connectivity turn.\n",
);

#[test]
fn connectivity_smoke_projection_payload_is_allowlisted() {
    // Given: the synthetic receipt-first projection reserved for external smoke testing.
    let (_directory, _store, prepared) = prepared_fixture_with_goal(LIVE_GOAL);

    // When: the exact in-memory outbound payload is inspected before any CLI launch.
    let outbound = prepared.rendered_prompt().expose();

    // Then: only the reviewed synthetic handoff is eligible for the live gate.
    assert_eq!(outbound, REVIEWED_LIVE_PROJECTION);
}

#[test]
#[ignore = "requires TSUKUMO_RUN_LIVE_SMOKE=1, local Claude auth, and model budget"]
fn claude_owned_process_connectivity_smoke_is_explicit_and_fail_closed() {
    // Given: an explicit operator gate and a locally authenticated Claude executable.
    assert_eq!(
        std::env::var("TSUKUMO_RUN_LIVE_SMOKE").as_deref(),
        Ok("1"),
        "set TSUKUMO_RUN_LIVE_SMOKE=1 to authorize the live model call"
    );
    let executable = std::env::var_os("TSUKUMO_CLAUDE_BIN")
        .map(PathBuf::from)
        .unwrap_or_else(default_claude_executable);
    let runtime_directory = tempfile::tempdir().expect("create isolated live working directory");
    let launch = RuntimeLaunchConfig::new(executable, runtime_directory.path().to_path_buf());
    let profile = ClaudeRuntimeProfile::isolated_smoke();
    let runner = StandardProcessRunner;

    // Given: the prompt-free version probe succeeds before any model budget is used.
    let version_spec = profile
        .version_command(&launch)
        .expect("construct Claude version command");
    let mut version = runner
        .spawn(ProcessLaunch::new(
            version_spec,
            None,
            ProcessLimits::default(),
        ))
        .expect("launch Claude version probe");
    let mut version_line_seen = false;
    let version_exit = loop {
        match version
            .next(Duration::from_secs(5))
            .expect("read Claude version output")
        {
            RuntimeOutput::StdoutLine(line) | RuntimeOutput::StderrLine(line) => {
                version_line_seen |= !line.trim().is_empty();
            }
            RuntimeOutput::Idle => {}
            RuntimeOutput::Exited(exit) => break exit,
        }
    };
    assert!(version_exit.success);
    assert!(version_line_seen);

    // When: the receipt-first Host executes one tool-free prompt under dontAsk permissions.
    let (_directory, mut store, prepared) = prepared_fixture_with_goal(LIVE_GOAL);
    assert_reviewed_projection(&prepared);
    let clock = SystemClock;
    let mut world = StageWorld::new();
    let director = DirectorContext::default();
    let policy = ExecutionPolicy::new(Duration::from_secs(120), Duration::from_millis(20))
        .expect("valid live policy");
    let mut host = RuntimeOrchestrator::new(
        HostServices::new(&mut store, &runner, &clock),
        Presentation::new(&mut world, &director),
        policy,
    );
    let report = host
        .execute(ExecutionRequest::new(
            &prepared,
            RuntimeSelection::new(&profile, &launch),
            ExecutionContext::new("quest-host", "session-host", "yuka"),
        ))
        .expect("execute Claude live smoke");

    // Then: a real owned process reaches one durable successful runtime outcome.
    // Assistant text is intentionally outside this connectivity smoke contract.
    assert_eq!(report.status, OutcomeStatus::Succeeded);
    assert!(world
        .log
        .back()
        .is_some_and(|line| line.text.contains("outcome")));
}

fn assert_reviewed_projection(prepared: &PreparedProjection) {
    assert_eq!(
        prepared.rendered_prompt().expose(),
        REVIEWED_LIVE_PROJECTION
    );
}

fn default_claude_executable() -> PathBuf {
    if cfg!(windows) {
        PathBuf::from("claude.cmd")
    } else {
        PathBuf::from("claude")
    }
}
