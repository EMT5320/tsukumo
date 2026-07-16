//! Production episode seed/resume contracts without external model calls.

mod common;

use common::{FakeRunner, FixedClock};
use sha2::{Digest, Sha256};
use std::path::Path;
use tempfile::tempdir;
use tsukumo_host::{
    read_episode_spec, resume_episode_with_services, seed_episode_with_clock, EpisodeCheckpointV1,
    EpisodeCondition, EpisodeDelayV1, EpisodeError, EpisodeExecutionProfile, EpisodeProjectionV1,
    EpisodeRuntimeKind, EpisodeRuntimeV1, EpisodeSpecV1, ProcessExit, RuntimeOutput,
};
use tsukumo_kernel::{
    EventId, KernelEvent, KernelEventPayload, OutcomeStatus, PersistedText, QuestId, SessionId,
    SpiritId, Timestamp, KERNEL_EVENT_SCHEMA_VERSION,
};
use tsukumo_soul::{OperatingSystem, SoulStore, StateScope};

#[derive(Default)]
struct FixedProbe {
    calls: std::sync::atomic::AtomicUsize,
    version: &'static str,
}

impl FixedProbe {
    fn codex() -> Self {
        Self::with_version("0.135.0")
    }

    fn claude() -> Self {
        Self::with_version("2.1.205")
    }

    fn with_version(version: &'static str) -> Self {
        Self {
            calls: std::sync::atomic::AtomicUsize::new(0),
            version,
        }
    }

    fn calls(&self) -> usize {
        self.calls.load(std::sync::atomic::Ordering::SeqCst)
    }
}

impl tsukumo_host::RuntimeProbe for FixedProbe {
    fn probe(
        &self,
        profile: &dyn tsukumo_adapters::RuntimeProfile,
        _launch: &tsukumo_adapters::RuntimeLaunchConfig,
    ) -> Result<tsukumo_host::RuntimeIdentity, tsukumo_host::RuntimeProbeError> {
        self.calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        Ok(tsukumo_host::RuntimeIdentity {
            binding: profile.binding(),
            version: self.version.into(),
        })
    }
}

#[test]
fn c0_seed_stays_outside_tsukumo_storage_and_processes() {
    // Given: the repository-native manual baseline points at a path that does not exist.
    let directory = tempdir().expect("create C0 parent");
    let data_dir = directory.path().join("must-remain-absent");
    let spec = episode_spec(EpisodeCondition::C0);
    let clock = FixedClock::new(100);

    // When: the baseline is registered through the bounded command boundary.
    let summary = seed_episode_with_clock(&spec, &data_dir, &clock).expect("describe C0 seed");

    // Then: no Chronicle, checkpoint, receipt, or runtime path was created.
    assert!(summary.manual_baseline_required);
    assert_eq!(summary.checkpoint_id, None);
    assert!(!data_dir.exists());
}

#[test]
fn c0_resume_is_rejected_without_probe_receipt_or_spawn() {
    // Given: a C0 baseline that never owns Tsukumo storage.
    let directory = tempdir().expect("create C0 resume parent");
    let data_dir = directory.path().join("must-remain-absent");
    let spec = episode_spec(EpisodeCondition::C0);
    let probe = FixedProbe::codex();
    let runner = FakeRunner::new(codex_success_outputs());

    // When: resume is attempted through the Host episode boundary.
    let error = resume_episode_with_services(
        &spec,
        &data_dir,
        Path::new("codex"),
        directory.path(),
        false,
        true,
        &probe,
        &runner,
        &FixedClock::new(200),
    )
    .expect_err("C0 must remain a manual baseline");

    // Then: the baseline refuses launch before probe, receipt, or process spawn.
    assert!(matches!(error, EpisodeError::ManualBaseline));
    assert_eq!(probe.calls(), 0);
    assert_eq!(runner.spawn_count(), 0);
    assert!(!data_dir.exists());
}

#[test]
fn resume_without_live_run_confirmation_does_not_prepare_or_spawn() {
    // Given: a seeded C2 episode that is otherwise ready to resume.
    let directory = tempdir().expect("create live-confirm store");
    let spec = episode_spec(EpisodeCondition::C2);
    seed_episode_with_clock(&spec, directory.path(), &FixedClock::new(100)).expect("seed C2");
    let probe = FixedProbe::codex();
    let runner = FakeRunner::new(codex_success_outputs());

    // When: resume omits the explicit live-run confirmation.
    let error = resume_episode_with_services(
        &spec,
        directory.path(),
        Path::new("codex"),
        directory.path(),
        false,
        false,
        &probe,
        &runner,
        &FixedClock::new(200),
    )
    .expect_err("live confirmation is required");

    // Then: no probe, projection receipt, or target process is created.
    assert!(matches!(error, EpisodeError::LiveRunConfirmationRequired));
    assert_eq!(probe.calls(), 0);
    assert_eq!(runner.spawn_count(), 0);
    let store = SoulStore::open(directory.path()).expect("reopen seeded store");
    assert!(store
        .latest_projection_event(None)
        .expect("read projection events")
        .is_none());
}

#[test]
fn workspace_write_target_without_acknowledgement_does_not_prepare_or_spawn() {
    // Given: a reviewed Codex workspace-write target already seeded.
    let directory = tempdir().expect("create workspace-write store");
    let mut spec = episode_spec(EpisodeCondition::C2);
    spec.target_runtime.execution_profile = EpisodeExecutionProfile::CodexWorkspaceWrite;
    seed_episode_with_clock(&spec, directory.path(), &FixedClock::new(100))
        .expect("seed workspace-write episode");
    let probe = FixedProbe::codex();
    let runner = FakeRunner::new(codex_success_outputs());

    // When: resume omits the matching --workspace-write acknowledgement.
    let error = resume_episode_with_services(
        &spec,
        directory.path(),
        Path::new("codex"),
        directory.path(),
        false,
        true,
        &probe,
        &runner,
        &FixedClock::new(200),
    )
    .expect_err("workspace-write acknowledgement is required");

    // Then: the profile gate fails closed before probe, receipt, or spawn.
    assert!(matches!(
        error,
        EpisodeError::WorkspaceWriteAcknowledgementRequired
    ));
    assert_eq!(probe.calls(), 0);
    assert_eq!(runner.spawn_count(), 0);
    let store = SoulStore::open(directory.path()).expect("reopen seeded store");
    assert!(store
        .latest_projection_event(None)
        .expect("read projection events")
        .is_none());
}

#[test]
fn workspace_write_acknowledgement_mismatch_does_not_prepare_or_spawn() {
    // Given: a reviewed Codex read-only target already seeded.
    let directory = tempdir().expect("create workspace-write mismatch store");
    let spec = episode_spec(EpisodeCondition::C2);
    seed_episode_with_clock(&spec, directory.path(), &FixedClock::new(100)).expect("seed C2");
    let probe = FixedProbe::codex();
    let runner = FakeRunner::new(codex_success_outputs());

    // When: resume supplies --workspace-write against the read-only profile.
    let error = resume_episode_with_services(
        &spec,
        directory.path(),
        Path::new("codex"),
        directory.path(),
        true,
        true,
        &probe,
        &runner,
        &FixedClock::new(200),
    )
    .expect_err("workspace-write must match the reviewed profile");

    // Then: the mismatch fails closed before probe, receipt, or spawn.
    assert!(matches!(
        error,
        EpisodeError::WorkspaceWriteAcknowledgementMismatch
    ));
    assert_eq!(probe.calls(), 0);
    assert_eq!(runner.spawn_count(), 0);
    let store = SoulStore::open(directory.path()).expect("reopen seeded store");
    assert!(store
        .latest_projection_event(None)
        .expect("read projection events")
        .is_none());
}

#[test]
fn claude_target_resume_retains_receipt_and_spawns_reviewed_runtime() {
    // Given: a reviewed Codex→Claude migration seeded under C2 visibility.
    let directory = tempdir().expect("create Claude target store");
    let mut spec = episode_spec(EpisodeCondition::C2);
    spec.episode_id = "episode-claude-target".into();
    spec.quest_id = QuestId::new("quest-claude-target");
    spec.source_session_id = SessionId::new("source-claude-target");
    spec.target_session_id = SessionId::new("target-claude-target");
    spec.source_runtime = EpisodeRuntimeV1 {
        kind: EpisodeRuntimeKind::CodexCli,
        version: "0.135.0".into(),
        execution_profile: EpisodeExecutionProfile::CodexReadOnly,
    };
    spec.target_runtime = EpisodeRuntimeV1 {
        kind: EpisodeRuntimeKind::ClaudeCli,
        version: "2.1.205".into(),
        execution_profile: EpisodeExecutionProfile::ClaudeDenyUnapproved,
    };
    seed_episode_with_clock(&spec, directory.path(), &FixedClock::new(100))
        .expect("seed Claude target episode");
    let runner = FakeRunner::new(claude_success_outputs());
    let probe = FixedProbe::claude();

    // When: resume probes the reviewed Claude identity and executes the fixture stream.
    let summary = resume_episode_with_services(
        &spec,
        directory.path(),
        Path::new("claude"),
        directory.path(),
        false,
        true,
        &probe,
        &runner,
        &FixedClock::new(200),
    )
    .expect("resume Claude target");

    // Then: the Claude family, version, and receipt-first summary are retained.
    assert_eq!(summary.runtime, EpisodeRuntimeKind::ClaudeCli);
    assert_eq!(
        summary.execution_profile,
        EpisodeExecutionProfile::ClaudeDenyUnapproved
    );
    assert_eq!(summary.runtime_version, "2.1.205");
    assert_eq!(summary.status, OutcomeStatus::Succeeded);
    assert_eq!(runner.spawn_count(), 1);
    assert!(summary.projection_id.is_some());
    let store = SoulStore::open(directory.path()).expect("reopen Claude target store");
    assert!(store
        .projection_receipt(summary.projection_id.as_ref().expect("C2 projection ID"))
        .expect("load Claude projection receipt")
        .is_some());
    assert_eq!(probe.calls(), 1);
}

#[test]
fn c1_and_c2_use_the_same_projection_while_only_c2_exposes_evidence_controls() {
    // Given: equivalent reviewed C1 and C2 specs with a real deterministic StateWriter input.
    let c1_dir = tempdir().expect("create C1 store");
    let c2_dir = tempdir().expect("create C2 store");
    let c1 = episode_spec(EpisodeCondition::C1);
    let c2 = episode_spec(EpisodeCondition::C2);
    let c1_seed =
        seed_episode_with_clock(&c1, c1_dir.path(), &FixedClock::new(100)).expect("seed C1");
    let c2_seed =
        seed_episode_with_clock(&c2, c2_dir.path(), &FixedClock::new(100)).expect("seed C2");
    let c1_runner = FakeRunner::new(codex_success_outputs());
    let c2_runner = FakeRunner::new(codex_success_outputs());

    // When: both conditions resume through receipt-first Host execution.
    let c1_run = resume_episode_with_services(
        &c1,
        c1_dir.path(),
        Path::new("codex"),
        c1_dir.path(),
        false,
        true,
        &FixedProbe::codex(),
        &c1_runner,
        &FixedClock::new(200),
    )
    .expect("resume C1");
    let c2_run = resume_episode_with_services(
        &c2,
        c2_dir.path(),
        Path::new("codex"),
        c2_dir.path(),
        false,
        true,
        &FixedProbe::codex(),
        &c2_runner,
        &FixedClock::new(200),
    )
    .expect("resume C2");

    // Then: condition visibility does not alter the bytes sent to the runtime.
    assert_eq!(c1_runner.captured_prompt(), c2_runner.captured_prompt());
    assert_eq!(c1_run.status, OutcomeStatus::Succeeded);
    assert_eq!(c2_run.status, OutcomeStatus::Succeeded);
    assert_eq!(c1_run.runtime_version, "0.135.0");
    assert_eq!(c1_run.episode_started_at_unix_ms, 201);
    assert_eq!(c2_run.episode_started_at_unix_ms, 201);
    assert!(!c1_seed.evidence_controls_exposed);
    assert_eq!(c1_seed.checkpoint_id, None);
    assert_eq!(c1_seed.source_event_count, None);
    assert_eq!(c1_seed.state_count, None);
    assert!(!c1_run.evidence_controls_exposed);
    assert_eq!(c1_run.projection_id, None);
    assert_eq!(c1_run.selected_state_count, None);
    assert!(c2_seed.evidence_controls_exposed);
    assert_eq!(c2_seed.source_event_count, Some(2));
    assert_eq!(c2_seed.state_count, Some(1));
    assert!(c2_run.evidence_controls_exposed);
    assert_eq!(c2_run.selected_state_count, Some(1));
    assert!(c2_run.rendered_digest_sha256.is_some());

    // And: the C2 receipt survives reopen in the same product database.
    let reopened = SoulStore::open(c2_dir.path()).expect("reopen C2 store");
    let receipt = reopened
        .projection_receipt(
            c2_run
                .projection_id
                .as_ref()
                .expect("C2 exposes projection ID"),
        )
        .expect("load C2 receipt");
    assert!(receipt.is_some());
}

#[test]
fn seed_is_idempotent_and_a_second_resume_does_not_spawn() {
    // Given: one committed C2 seed and one completed execution.
    let directory = tempdir().expect("create idempotency store");
    let spec = episode_spec(EpisodeCondition::C2);
    let first_seed = seed_episode_with_clock(&spec, directory.path(), &FixedClock::new(100))
        .expect("first seed");
    let retry_seed = seed_episode_with_clock(&spec, directory.path(), &FixedClock::new(500))
        .expect("retry seed");
    assert_eq!(retry_seed, first_seed);
    let first_runner = FakeRunner::new(codex_success_outputs());
    resume_episode_with_services(
        &spec,
        directory.path(),
        Path::new("codex"),
        directory.path(),
        false,
        true,
        &FixedProbe::codex(),
        &first_runner,
        &FixedClock::new(200),
    )
    .expect("first resume");

    // When: the same spec attempts the same deterministic execution again.
    let second_runner = FakeRunner::new(codex_success_outputs());
    let error = resume_episode_with_services(
        &spec,
        directory.path(),
        Path::new("codex"),
        directory.path(),
        false,
        true,
        &FixedProbe::codex(),
        &second_runner,
        &FixedClock::new(300),
    )
    .expect_err("duplicate execution must fail closed");

    // Then: Host detects the durable start event before a second process spawn.
    assert!(matches!(error, EpisodeError::Host(_)));
    assert_eq!(second_runner.spawn_count(), 0);
}

#[test]
fn interrupted_seed_reuses_the_committed_source_timestamp() {
    // Given: a crash left the deterministic source-summary event but no checkpoint.
    let directory = tempdir().expect("create interrupted seed store");
    let spec = episode_spec(EpisodeCondition::C2);
    let source_event = KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: source_summary_event_id(&spec),
        occurred_at: Timestamp::from_unix_millis(123),
        quest_id: spec.quest_id.clone(),
        session_id: spec.source_session_id.clone(),
        spirit_id: spec.spirit_id.clone(),
        execution_id: None,
        runtime: None,
        causation_id: None,
        correlation_id: None,
        payload: KernelEventPayload::UserInput {
            content: spec.source_summary.clone(),
        },
    };
    let mut partial = SoulStore::open(directory.path()).expect("open partial seed store");
    partial
        .append_event(&source_event)
        .expect("append partial source event");
    drop(partial);

    // When: the identical reviewed spec retries under a later wall clock.
    let summary = seed_episode_with_clock(&spec, directory.path(), &FixedClock::new(999))
        .expect("resume interrupted seed");

    // Then: every dependent event and the checkpoint reuse the original durable time.
    assert_eq!(summary.seeded_at_unix_ms, Some(123));
    let reopened = SoulStore::open(directory.path()).expect("reopen completed seed");
    let checkpoint = reopened
        .checkpoint(summary.checkpoint_id.as_ref().expect("C2 checkpoint ID"))
        .expect("load checkpoint")
        .expect("checkpoint exists");
    assert_eq!(checkpoint.created_at, Timestamp::from_unix_millis(123));
}

#[test]
fn delayed_resume_before_the_window_does_not_prepare_or_spawn() {
    // Given: a reviewed delayed C1 spec committed at t=100ms.
    let directory = tempdir().expect("create delayed store");
    let mut spec = episode_spec(EpisodeCondition::C1);
    spec.delay = EpisodeDelayV1 {
        minimum_hours: 48,
        maximum_hours: 72,
    };
    seed_episode_with_clock(&spec, directory.path(), &FixedClock::new(100))
        .expect("seed delayed episode");
    let runner = FakeRunner::new(codex_success_outputs());

    // When: a target attempts to resume before 48 hours.
    let error = resume_episode_with_services(
        &spec,
        directory.path(),
        Path::new("codex"),
        directory.path(),
        false,
        true,
        &FixedProbe::codex(),
        &runner,
        &FixedClock::new(200),
    )
    .expect_err("early resume must fail");

    // Then: no projection or runtime process is created.
    assert!(matches!(error, EpisodeError::ResumeTooEarly { .. }));
    assert_eq!(runner.spawn_count(), 0);
}

#[test]
fn machine_summary_excludes_runtime_paths_and_rendered_prompt() {
    // Given: a C2 run whose executable and working directory carry private-looking labels.
    let directory = tempdir().expect("create summary store");
    let spec = episode_spec(EpisodeCondition::C2);
    seed_episode_with_clock(&spec, directory.path(), &FixedClock::new(100)).expect("seed C2");
    let runner = FakeRunner::new(codex_success_outputs());
    let working_dir = directory.path().join("private-workspace-location");
    std::fs::create_dir(&working_dir).expect("create private-looking working directory");

    // When: the redacted machine summary is serialized.
    let summary = resume_episode_with_services(
        &spec,
        directory.path(),
        Path::new("private-runtime-location"),
        &working_dir,
        false,
        true,
        &FixedProbe::codex(),
        &runner,
        &FixedClock::new(200),
    )
    .expect("resume C2");
    let json = serde_json::to_string(&summary).expect("serialize run summary");

    // Then: process paths and exact rendered bytes are absent.
    assert!(!json.contains("private-runtime-location"));
    assert!(!json.contains("private-workspace-location"));
    assert!(!json.contains("# Tsukumo handoff"));
    assert!(!json.contains(&spec.projection.delegation_goal));
}

#[test]
fn seeded_condition_is_frozen_before_probe_receipt_or_spawn() {
    let directory = tempdir().expect("create frozen registration store");
    let c1 = episode_spec(EpisodeCondition::C1);
    seed_episode_with_clock(&c1, directory.path(), &FixedClock::new(100)).expect("seed C1");
    let c2 = episode_spec(EpisodeCondition::C2);
    let probe = FixedProbe::codex();
    let runner = FakeRunner::new(codex_success_outputs());

    let error = resume_episode_with_services(
        &c2,
        directory.path(),
        Path::new("codex"),
        directory.path(),
        false,
        true,
        &probe,
        &runner,
        &FixedClock::new(200),
    )
    .expect_err("condition mutation must fail closed");

    assert!(matches!(error, EpisodeError::RegistrationMismatch));
    assert_eq!(probe.calls(), 0);
    assert_eq!(runner.spawn_count(), 0);
}

#[test]
fn seeded_execution_profile_is_frozen_before_probe_receipt_or_spawn() {
    let directory = tempdir().expect("create sandbox registration store");
    let registered = episode_spec(EpisodeCondition::C2);
    seed_episode_with_clock(&registered, directory.path(), &FixedClock::new(100))
        .expect("seed read-only registration");
    let mut mutated = registered.clone();
    mutated.target_runtime.execution_profile = EpisodeExecutionProfile::CodexWorkspaceWrite;
    let probe = FixedProbe::codex();
    let runner = FakeRunner::new(codex_success_outputs());

    let error = resume_episode_with_services(
        &mutated,
        directory.path(),
        Path::new("codex"),
        directory.path(),
        true,
        true,
        &probe,
        &runner,
        &FixedClock::new(200),
    )
    .expect_err("execution profile mutation must fail closed");

    assert!(matches!(error, EpisodeError::RegistrationMismatch));
    assert_eq!(probe.calls(), 0);
    assert_eq!(runner.spawn_count(), 0);
}

#[test]
fn observed_runtime_version_mismatch_precedes_receipt_and_target_spawn() {
    let directory = tempdir().expect("create runtime mismatch store");
    let spec = episode_spec(EpisodeCondition::C2);
    seed_episode_with_clock(&spec, directory.path(), &FixedClock::new(100)).expect("seed C2");
    let probe = FixedProbe::with_version("0.999.0");
    let runner = FakeRunner::new(codex_success_outputs());

    let error = resume_episode_with_services(
        &spec,
        directory.path(),
        Path::new("codex"),
        directory.path(),
        false,
        true,
        &probe,
        &runner,
        &FixedClock::new(200),
    )
    .expect_err("unreviewed runtime version must fail closed");

    assert!(matches!(error, EpisodeError::RuntimeVersionMismatch));
    assert_eq!(probe.calls(), 1);
    assert_eq!(runner.spawn_count(), 0);
}

#[test]
fn starting_event_that_crosses_the_window_does_not_spawn_runtime() {
    let directory = tempdir().expect("create start-window store");
    let mut spec = episode_spec(EpisodeCondition::C2);
    spec.delay = EpisodeDelayV1 {
        minimum_hours: 0,
        maximum_hours: 0,
    };
    seed_episode_with_clock(&spec, directory.path(), &FixedClock::new(100))
        .expect("seed exact-window episode");
    let probe = FixedProbe::codex();
    let runner = FakeRunner::new(codex_success_outputs());

    let error = resume_episode_with_services(
        &spec,
        directory.path(),
        Path::new("codex"),
        directory.path(),
        false,
        true,
        &probe,
        &runner,
        &FixedClock::new(100),
    )
    .expect_err("Starting outside the exact window must fail");

    assert!(matches!(error, EpisodeError::ResumeWindowClosed { .. }));
    assert_eq!(probe.calls(), 1);
    assert_eq!(runner.spawn_count(), 0);
}

#[test]
fn nested_unknown_scope_field_is_rejected_by_reviewed_json_boundary() {
    let directory = tempdir().expect("create strict JSON directory");
    let spec_path = directory.path().join("episode.json");
    let mut value =
        serde_json::to_value(episode_spec(EpisodeCondition::C1)).expect("serialize episode spec");
    value["projection"]["scope"]["applicability"]
        .as_object_mut()
        .expect("scope applicability object")
        .insert(
            "operating_systm".into(),
            serde_json::Value::String("windows".into()),
        );
    std::fs::write(
        &spec_path,
        serde_json::to_vec(&value).expect("serialize mutated spec"),
    )
    .expect("write reviewed JSON candidate");

    let error = match read_episode_spec(&spec_path) {
        Ok(_) => panic!("unknown nested field must fail"),
        Err(error) => error,
    };

    assert!(matches!(error, EpisodeError::SpecJson(_)));
}

#[test]
fn unsafe_reviewed_text_fails_before_data_directory_creation() {
    let directory = tempdir().expect("create invalid input parent");
    let data_dir = directory.path().join("must-remain-absent");
    let mut spec = episode_spec(EpisodeCondition::C2);
    spec.projection.delegation_goal = format!(
        "Continue audit{}",
        char::from_u32(0x202e).expect("valid bidi control scalar")
    );

    let error = seed_episode_with_clock(&spec, &data_dir, &FixedClock::new(100))
        .expect_err("bidi control must fail closed");

    assert!(matches!(
        error,
        EpisodeError::InvalidSpec("projection.delegation_goal")
    ));
    assert!(!data_dir.exists());
}

fn episode_spec(condition: EpisodeCondition) -> EpisodeSpecV1 {
    EpisodeSpecV1 {
        schema_version: 1,
        episode_id: "episode-visibility-pair".into(),
        condition,
        episode_type: "natural_delayed_resumption".into(),
        workload_block: "toolchain_claim_audit".into(),
        fault: "none".into(),
        reviewed_git_head: None,
        quest_id: QuestId::new("quest-visibility-pair"),
        source_session_id: SessionId::new("source-visibility-pair"),
        target_session_id: SessionId::new("target-visibility-pair"),
        spirit_id: SpiritId::new("yuka"),
        source_runtime: EpisodeRuntimeV1 {
            kind: EpisodeRuntimeKind::ClaudeCli,
            version: "2.1.205".into(),
            execution_profile: EpisodeExecutionProfile::ClaudeDenyUnapproved,
        },
        target_runtime: EpisodeRuntimeV1 {
            kind: EpisodeRuntimeKind::CodexCli,
            version: "0.135.0".into(),
            execution_profile: EpisodeExecutionProfile::CodexReadOnly,
        },
        source_summary: PersistedText::from_reviewed(
            "A real source-runtime toolchain audit left one release-claim open loop",
        ),
        explicit_state_input: Some(PersistedText::from_reviewed(
            "Tsukumo always uses the GNU Rust toolchain on Windows",
        )),
        checkpoint: EpisodeCheckpointV1 {
            goal: PersistedText::from_reviewed("Audit supported toolchain release claims"),
            progress: Vec::new(),
            decisions: Vec::new(),
            artifacts: Vec::new(),
            open_loops: Vec::new(),
            next_actions: Vec::new(),
            constraint_state_ids: Vec::new(),
        },
        projection: EpisodeProjectionV1 {
            scope: StateScope::workspace_os("tsukumo", OperatingSystem::Windows),
            budget_chars: 8_000,
            delegation_goal: "Continue the reviewed toolchain claim audit".into(),
        },
        delay: EpisodeDelayV1 {
            minimum_hours: 0,
            maximum_hours: 1,
        },
        quality_gate: vec!["Use only retained, reproducible toolchain evidence".into()],
    }
}

fn codex_success_outputs() -> Vec<RuntimeOutput> {
    let mut outputs = tsukumo_adapters::codex_0_135_0_success_fixture()
        .lines()
        .map(|line| RuntimeOutput::StdoutLine(line.to_owned()))
        .collect::<Vec<_>>();
    outputs.push(RuntimeOutput::Exited(ProcessExit {
        code: Some(0),
        success: true,
    }));
    outputs
}

fn claude_success_outputs() -> Vec<RuntimeOutput> {
    let mut outputs = tsukumo_adapters::claude_c1_success_fixture()
        .lines()
        .map(|line| RuntimeOutput::StdoutLine(line.to_owned()))
        .collect::<Vec<_>>();
    outputs.push(RuntimeOutput::Exited(ProcessExit {
        code: Some(0),
        success: true,
    }));
    outputs
}

fn source_summary_event_id(spec: &EpisodeSpecV1) -> EventId {
    let mut migration_input = spec.clone();
    migration_input.condition = EpisodeCondition::C1;
    let serialized = serde_json::to_vec(&migration_input).expect("serialize migration input");
    let fingerprint = format!("{:x}", Sha256::digest(serialized));
    let mut digest = Sha256::new();
    digest.update(b"source-summary-event");
    digest.update([0]);
    digest.update(fingerprint.as_bytes());
    let value = format!("{:x}", digest.finalize());
    EventId::new(format!("source-summary-event-{}", &value[..32]))
}
