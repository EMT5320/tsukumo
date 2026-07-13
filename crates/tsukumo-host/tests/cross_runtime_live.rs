//! Explicitly gated dual-runtime live smoke through receipt-first Host paths.

mod common;

use common::prepared_dual_runtime_live_fixture;
use std::path::PathBuf;
use std::time::Duration;
use tsukumo_adapters::{
    ClaudeRuntimeProfile, CodexRuntimeProfile, RuntimeCommandSpec, RuntimeLaunchConfig,
    RuntimeProfile,
};
use tsukumo_host::{
    ExecutionContext, ExecutionPolicy, ExecutionRequest, HostServices, Presentation, ProcessLaunch,
    ProcessLimits, ProcessRunner, RuntimeOrchestrator, RuntimeOutput, RuntimeSelection,
    StandardProcessRunner, SystemClock,
};
use tsukumo_kernel::OutcomeStatus;
use tsukumo_soul::{PreparedProjection, SoulStore};
use tsukumo_theater::{DirectorContext, StageWorld};

const LIVE_GOAL: &str = "Reply with exactly TSUKUMO_CROSS_RUNTIME_LIVE_OK and do not use tools.";
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
    "Reply with exactly TSUKUMO_CROSS_RUNTIME_LIVE_OK and do not use tools.\n",
);

#[test]
fn dual_runtime_live_payloads_are_allowlisted_and_share_one_checkpoint() {
    // Given: two receipt-committed projections from one Spirit and checkpoint.
    let (_directory, _store, claude, codex) = prepared_dual_runtime_live_fixture(LIVE_GOAL);

    // Then: both outbound prompts equal the reviewed credential-free literal.
    assert_reviewed_projection(&claude);
    assert_reviewed_projection(&codex);
    assert_eq!(claude.receipt.checkpoint_id, codex.receipt.checkpoint_id);
    assert_eq!(
        claude.receipt.rendered_digest,
        codex.receipt.rendered_digest
    );
}

#[test]
#[ignore = "requires TSUKUMO_RUN_LIVE_SMOKE=1, both local CLI auth states, and model budget"]
fn dual_runtime_owned_process_live_smoke_is_explicit_and_fail_closed() {
    // Given: one explicit gate and both locally authenticated executables.
    assert_eq!(
        std::env::var("TSUKUMO_RUN_LIVE_SMOKE").as_deref(),
        Ok("1"),
        "set TSUKUMO_RUN_LIVE_SMOKE=1 to authorize both live model calls"
    );
    let claude_executable = executable_from_env("TSUKUMO_CLAUDE_BIN", "claude.cmd", "claude");
    let codex_executable = executable_from_env("TSUKUMO_CODEX_BIN", "codex.cmd", "codex");
    let claude_directory = tempfile::tempdir().expect("create isolated Claude directory");
    let codex_directory = tempfile::tempdir().expect("create isolated Codex directory");
    let claude_launch =
        RuntimeLaunchConfig::new(claude_executable, claude_directory.path().to_path_buf());
    let codex_launch =
        RuntimeLaunchConfig::new(codex_executable, codex_directory.path().to_path_buf());
    let claude_profile = ClaudeRuntimeProfile::isolated_smoke();
    let codex_profile = CodexRuntimeProfile::isolated_smoke();
    let runner = StandardProcessRunner;

    // Given: both prompt-free version probes succeed before model budget is used.
    let claude_version = probe_version(
        &runner,
        claude_profile
            .version_command(&claude_launch)
            .expect("construct Claude version probe"),
        "Claude",
    );
    let codex_version = probe_version(
        &runner,
        codex_profile
            .version_command(&codex_launch)
            .expect("construct Codex version probe"),
        "Codex",
    );
    eprintln!("dual-runtime live versions: {claude_version}; {codex_version}");

    // When: one checkpoint is delivered to Claude and Codex through the same Host port.
    let (_directory, mut store, claude, codex) = prepared_dual_runtime_live_fixture(LIVE_GOAL);
    assert_reviewed_projection(&claude);
    assert_reviewed_projection(&codex);
    let mut world = StageWorld::new();
    let director = DirectorContext::default();
    let (claude_report, codex_report) = {
        let mut harness = LiveHarness {
            store: &mut store,
            runner: &runner,
            world: &mut world,
            director: &director,
        };
        let claude_report = harness.execute(&claude, &claude_profile, &claude_launch);
        let codex_report = harness.execute(&codex, &codex_profile, &codex_launch);
        (claude_report, codex_report)
    };

    // Then: both owned processes close with durable successful outcomes.
    assert_eq!(claude_report.status, OutcomeStatus::Succeeded);
    assert_eq!(codex_report.status, OutcomeStatus::Succeeded);
    assert!(
        world
            .log
            .iter()
            .filter(|line| line.text.contains("outcome"))
            .count()
            >= 2
    );
}

fn probe_version(
    runner: &StandardProcessRunner,
    command: RuntimeCommandSpec,
    runtime_name: &str,
) -> String {
    let mut process = runner
        .spawn(ProcessLaunch::new(command, None, ProcessLimits::default()))
        .unwrap_or_else(|error| panic!("launch {runtime_name} version probe: {error}"));
    let mut lines = Vec::new();
    let exit = loop {
        match process
            .next(Duration::from_secs(5))
            .unwrap_or_else(|error| panic!("read {runtime_name} version output: {error}"))
        {
            RuntimeOutput::StdoutLine(line) | RuntimeOutput::StderrLine(line) => {
                if !line.trim().is_empty() {
                    lines.push(line);
                }
            }
            RuntimeOutput::Idle => {}
            RuntimeOutput::Exited(exit) => break exit,
        }
    };
    assert!(
        exit.success,
        "{runtime_name} version probe failed: {lines:?}"
    );
    assert!(!lines.is_empty(), "{runtime_name} version output is empty");
    lines.join(" | ")
}

struct LiveHarness<'a> {
    store: &'a mut SoulStore,
    runner: &'a StandardProcessRunner,
    world: &'a mut StageWorld,
    director: &'a DirectorContext,
}

impl LiveHarness<'_> {
    fn execute(
        &mut self,
        prepared: &PreparedProjection,
        profile: &dyn RuntimeProfile,
        launch: &RuntimeLaunchConfig,
    ) -> tsukumo_host::ExecutionReport {
        let clock = SystemClock;
        let policy = ExecutionPolicy::new(Duration::from_secs(120), Duration::from_millis(20))
            .expect("construct live execution policy");
        let mut host = RuntimeOrchestrator::new(
            HostServices::new(self.store, self.runner, &clock),
            Presentation::new(self.world, self.director),
            policy,
        );
        host.execute(ExecutionRequest::new(
            prepared,
            RuntimeSelection::new(profile, launch),
            ExecutionContext::new("quest-host", "session-host", "yuka"),
        ))
        .expect("execute explicitly enabled live runtime")
    }
}

fn assert_reviewed_projection(prepared: &PreparedProjection) {
    assert_eq!(
        prepared.rendered_prompt().expose(),
        REVIEWED_LIVE_PROJECTION
    );
}

fn executable_from_env(variable: &str, windows_default: &str, unix_default: &str) -> PathBuf {
    std::env::var_os(variable)
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            if cfg!(windows) {
                PathBuf::from(windows_default)
            } else {
                PathBuf::from(unix_default)
            }
        })
}
