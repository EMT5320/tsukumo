//! Controlled Claude-to-Codex removed-state replay and evidence manifest.

mod common;

use common::{
    materialize_cross_runtime_repository, prepare_post_revoke_projection,
    prepared_cross_runtime_comparison, CrossRuntimePrepared, FakeRunner, FixedClock, TestLedger,
};
use std::path::Path;
use tsukumo_adapters::{
    codex_0_135_0_gnu_with_state_fixture, codex_0_135_0_gnu_without_state_fixture,
    CodexRuntimeProfile, RuntimeLaunchConfig,
};
use tsukumo_host::{
    load_presentation_pack, ExecutionContext, ExecutionPolicy, ExecutionReport, ExecutionRequest,
    HostProductController, HostServices, Presentation, PresentationPackSource, ProcessExit,
    ProductControl, ProductController, RuntimeOrchestrator, RuntimeOutput, RuntimeSelection,
};
use tsukumo_kernel::{
    KernelEvent, KernelEventPayload, OutcomeStatus, RuntimeKind, RuntimeMode, Timestamp,
};
use tsukumo_soul::{
    PreparedProjection, ProjectionOmissionReason, ProjectionSection, SoulStore, StateStatus,
};
use tsukumo_theater::{DirectorContext, StageWorld, UiAction};

struct CaseEvidence {
    report: ExecutionReport,
    events: Vec<KernelEvent>,
    captured_prompt: String,
}

#[test]
fn comparison_replays_real_codex_commands_under_one_removed_state() {
    // Given: one Claude-bound GNU constraint, one checkpoint, and a controlled pair.
    let prepared = prepared_cross_runtime_comparison();
    let (repository, repository_digest) = materialize_cross_runtime_repository();
    assert_source_and_projection_bindings(&prepared);
    assert_eq!(repository_digest.len(), 64);
    for relative in ["Cargo.toml", "src/lib.rs"] {
        assert!(repository.path().join(relative).is_file());
    }

    // When: the Host replays the reviewed Codex 0.135.0 captures through production ports.
    let with_state = execute_case(
        prepared.directory.path(),
        repository.path(),
        &prepared.with_state,
        codex_0_135_0_gnu_with_state_fixture(),
        1_000,
    );
    let without_state = execute_case(
        prepared.directory.path(),
        repository.path(),
        &prepared.without_state,
        codex_0_135_0_gnu_without_state_fixture(),
        2_000,
    );

    // Then: actual projected prompts pair with the intended reviewed replay condition.
    assert!(with_state
        .captured_prompt
        .contains("[state:state-cross-runtime-gnu@v1] Use the GNU Rust toolchain on Windows"));
    assert!(!without_state
        .captured_prompt
        .contains("state-cross-runtime-gnu"));
    assert_eq!(with_state.report.status, OutcomeStatus::Succeeded);
    assert_eq!(without_state.report.status, OutcomeStatus::Succeeded);

    // Then: only the target state and dependent receipt hashes differ before execution.
    assert_eq!(
        prepared.comparison.changed_sections,
        [ProjectionSection::Constraints]
    );
    assert_ne!(
        prepared.comparison.with_digest,
        prepared.comparison.without_digest
    );
    assert_eq!(
        prepared.with_state.receipt.checkpoint_id,
        prepared.without_state.receipt.checkpoint_id
    );
    assert_eq!(
        prepared.with_state.receipt.runtime,
        prepared.without_state.receipt.runtime
    );
    assert_eq!(
        prepared.with_state.receipt.created_at,
        prepared.without_state.receipt.created_at
    );

    // Then: normalized command intent changes while both tool attempts remain declined.
    let with_commands = normalized_commands(&with_state.events);
    let without_commands = normalized_commands(&without_state.events);
    assert_eq!(with_commands.len(), 2);
    assert_eq!(without_commands.len(), 1);
    assert!(with_commands
        .iter()
        .all(|command| command.contains("stable-x86_64-pc-windows-gnu")));
    assert!(without_commands
        .iter()
        .all(|command| !command.contains("stable-x86_64-pc-windows-gnu")));
    assert!(tool_error_flags(&with_state.events)
        .iter()
        .all(|value| *value));
    assert!(tool_error_flags(&without_state.events)
        .iter()
        .all(|value| *value));
    assert_eq!(
        terminal_outcome(&with_state.events),
        OutcomeStatus::Succeeded
    );
    assert_eq!(
        terminal_outcome(&without_state.events),
        OutcomeStatus::Succeeded
    );

    // Then: the bounded manifest records facts and contains no rendered prompt snapshot.
    let manifest = serde_json::json!({
        "schema_version": 1,
        "source_runtime": "claude_cli/fixture",
        "target_runtime": "codex_cli/owned_process@0.135.0",
        "repository_fixture_digest": repository_digest,
        "source_event_id": prepared.source_event.event_id,
        "state_id": prepared.state.state_id,
        "checkpoint_id": prepared.checkpoint.id,
        "comparison": prepared.comparison,
        "with_state": {
            "projection_id": prepared.with_state.receipt.id,
            "selected_state_refs": prepared.with_state.receipt.selected_state_refs,
            "tool_commands": with_commands,
            "tool_errors": tool_error_flags(&with_state.events),
            "outcome": terminal_outcome(&with_state.events),
        },
        "without_state": {
            "projection_id": prepared.without_state.receipt.id,
            "selected_state_refs": prepared.without_state.receipt.selected_state_refs,
            "tool_commands": without_commands,
            "tool_errors": tool_error_flags(&without_state.events),
            "outcome": terminal_outcome(&without_state.events),
        },
        "claim_boundary": "controlled behavioral sensitivity only",
    });
    let serialized = serde_json::to_string_pretty(&manifest).expect("serialize evidence manifest");
    assert!(!serialized.contains("rendered_prompt"));
    assert!(!serialized.contains("Run the appropriate offline test command"));
    assert!(!serialized.contains(&repository.path().display().to_string()));
    assert!(!serialized.contains("auth.json"));
}

#[test]
fn trace_and_host_revoke_preserve_the_historical_codex_receipt() {
    // Given: one replayed Codex execution whose receipt selected the GNU state.
    let prepared = prepared_cross_runtime_comparison();
    let (repository, _repository_digest) = materialize_cross_runtime_repository();
    let executed = execute_case(
        prepared.directory.path(),
        repository.path(),
        &prepared.with_state,
        codex_0_135_0_gnu_with_state_fixture(),
        3_000,
    );

    // Then: every durable edge can be reopened from source through terminal outcome.
    let reopened = SoulStore::open(prepared.directory.path()).expect("reopen trace store");
    assert_eq!(
        reopened
            .event(&prepared.source_event.event_id)
            .expect("load source event")
            .expect("source event exists")
            .event,
        prepared.source_event
    );
    assert_eq!(
        reopened
            .state(&prepared.state.state_id)
            .expect("load source state")
            .expect("source state exists"),
        prepared.state
    );
    assert_eq!(
        reopened
            .checkpoint(&prepared.checkpoint.id)
            .expect("load checkpoint")
            .expect("checkpoint exists"),
        prepared.checkpoint
    );
    let historical = reopened
        .projection_receipt(&prepared.with_state.receipt.id)
        .expect("load historical receipt")
        .expect("historical receipt exists");
    assert_eq!(historical, prepared.with_state.receipt);
    drop(reopened);

    assert!(historical.selected_state_refs.iter().any(|reference| {
        reference.state_id == prepared.state.state_id && reference.version == prepared.state.version
    }));
    let mut tool_start_seen = false;
    let mut tool_end_seen = false;
    let mut outcome_seen = false;
    for event in &executed.events {
        if matches!(
            event.payload,
            KernelEventPayload::ToolStart {
                projection_id: Some(ref id),
                ..
            } if id == &historical.id
        ) {
            tool_start_seen = true;
        }
        if matches!(
            event.payload,
            KernelEventPayload::ToolEnd {
                projection_id: Some(ref id),
                ..
            } if id == &historical.id
        ) {
            tool_end_seen = true;
        }
        if matches!(
            event.payload,
            KernelEventPayload::Outcome {
                projection_id: Some(ref id),
                ..
            } if id == &historical.id
        ) {
            outcome_seen = true;
        }
        if matches!(
            event.payload,
            KernelEventPayload::ToolStart { .. }
                | KernelEventPayload::ToolEnd { .. }
                | KernelEventPayload::Outcome { .. }
        ) {
            assert_eq!(event.execution_id.as_ref(), Some(&historical.execution_id));
            assert_eq!(event.runtime.as_ref(), Some(&historical.runtime));
            assert_eq!(event.spirit_id, prepared.source_event.spirit_id);
        }
    }
    assert!(tool_start_seen && tool_end_seen && outcome_seen);

    // When: the typed product action revokes the state through the Host controller.
    let pack = load_presentation_pack(&PresentationPackSource::EmbeddedDefault)
        .expect("load embedded presentation pack");
    let mut controller = HostProductController::open(prepared.directory.path(), &pack)
        .expect("open product controller on trace store");
    let before = controller.refresh().expect("refresh active state view");
    assert!(before
        .view
        .states
        .iter()
        .any(|state| state.id.as_str() == prepared.state.state_id.as_str()));
    assert_eq!(
        controller
            .apply(UiAction::RevokeState(prepared.state.state_id.clone()))
            .expect("revoke selected state through Host"),
        ProductControl::Continue
    );
    drop(controller);

    // Then: the next projection excludes it while the prior receipt stays immutable.
    let mut reopened = SoulStore::open(prepared.directory.path()).expect("reopen revoked store");
    let revoked = reopened
        .state(&prepared.state.state_id)
        .expect("load revoked state")
        .expect("revoked state remains historical");
    assert_eq!(revoked.status, StateStatus::Revoked);
    let next_timestamp = Timestamp::from_unix_millis(
        revoked
            .deactivated_at
            .expect("revoked state has deactivation time")
            .as_unix_millis()
            .saturating_add(1),
    );
    let next = prepare_post_revoke_projection(&mut reopened, &prepared, next_timestamp);
    assert!(next
        .receipt
        .selected_state_refs
        .iter()
        .all(|reference| { reference.state_id != prepared.state.state_id }));
    assert!(next.receipt.omissions.iter().any(|omission| {
        omission.state_id == prepared.state.state_id
            && omission.reason == ProjectionOmissionReason::Inactive
    }));
    assert_eq!(
        reopened
            .projection_receipt(&historical.id)
            .expect("reload prior receipt")
            .expect("prior receipt remains"),
        historical
    );
}

fn assert_source_and_projection_bindings(prepared: &CrossRuntimePrepared) {
    assert_eq!(prepared.source_event.spirit_id.as_str(), "yuka");
    assert_eq!(
        prepared
            .source_event
            .runtime
            .as_ref()
            .map(|runtime| runtime.kind),
        Some(RuntimeKind::ClaudeCli)
    );
    assert_eq!(
        prepared
            .source_event
            .runtime
            .as_ref()
            .map(|runtime| runtime.mode),
        Some(RuntimeMode::Fixture)
    );
    assert_eq!(
        prepared.state.evidence_refs.as_slice(),
        std::slice::from_ref(&prepared.source_event.event_id)
    );
    assert!(prepared.checkpoint.constraint_refs.iter().any(|reference| {
        reference.state_id == prepared.state.state_id && reference.version == prepared.state.version
    }));
    for projection in [&prepared.with_state, &prepared.without_state] {
        assert_eq!(projection.receipt.checkpoint_id, prepared.checkpoint.id);
        assert_eq!(projection.receipt.runtime.kind, RuntimeKind::CodexCli);
        assert_eq!(projection.receipt.runtime.mode, RuntimeMode::OwnedProcess);
    }
}

fn execute_case(
    store_directory: &Path,
    repository: &Path,
    prepared: &PreparedProjection,
    fixture: &str,
    clock_start: i64,
) -> CaseEvidence {
    let runner = FakeRunner::new(codex_outputs(fixture));
    let store = SoulStore::open(store_directory).expect("reopen comparison store");
    let mut ledger = TestLedger::new(store, &runner);
    let clock = FixedClock::new(clock_start);
    let mut world = StageWorld::new();
    let director = DirectorContext::default();
    let profile = CodexRuntimeProfile::workspace_write();
    let launch = RuntimeLaunchConfig::new(repository.join("codex-fixture"), repository.to_owned());
    let report = {
        let mut host = RuntimeOrchestrator::new(
            HostServices::new(&mut ledger, &runner, &clock),
            Presentation::new(&mut world, &director),
            ExecutionPolicy::default(),
        );
        host.execute(ExecutionRequest::new(
            prepared,
            RuntimeSelection::new(&profile, &launch),
            ExecutionContext::new("quest-cross-runtime", "session-cross-runtime", "yuka"),
        ))
        .expect("replay reviewed Codex comparison fixture")
    };
    let events = ledger.execution_events(&prepared.receipt.execution_id);
    let captured_prompt = runner
        .captured_prompt()
        .expect("Host writes prompt to stdin");
    CaseEvidence {
        report,
        events,
        captured_prompt,
    }
}

fn codex_outputs(fixture: &str) -> Vec<RuntimeOutput> {
    let mut outputs = fixture
        .lines()
        .map(|line| RuntimeOutput::StdoutLine(line.to_owned()))
        .collect::<Vec<_>>();
    outputs.push(RuntimeOutput::Exited(ProcessExit {
        code: Some(0),
        success: true,
    }));
    outputs
}

fn normalized_commands(events: &[KernelEvent]) -> Vec<String> {
    events
        .iter()
        .filter_map(|event| match &event.payload {
            KernelEventPayload::ToolStart {
                args: Some(arguments),
                ..
            } => arguments
                .as_value()
                .get("command")
                .and_then(serde_json::Value::as_str)
                .map(str::to_owned),
            _ => None,
        })
        .collect()
}

fn tool_error_flags(events: &[KernelEvent]) -> Vec<bool> {
    events
        .iter()
        .filter_map(|event| match event.payload {
            KernelEventPayload::ToolEnd { is_error, .. } => Some(is_error),
            _ => None,
        })
        .collect()
}

fn terminal_outcome(events: &[KernelEvent]) -> OutcomeStatus {
    events
        .iter()
        .find_map(|event| match event.payload {
            KernelEventPayload::Outcome { status, .. } => Some(status),
            _ => None,
        })
        .expect("execution has one terminal outcome")
}
