//! Bounded C0/C1/C2 handoff episode orchestration.
//!
//! The reviewed JSON spec is an experiment input, while SQLite remains the
//! only durable authority for Chronicle events, checkpoints, and receipts.

use crate::local_path::{prepare_data_directory, LocalDirectoryGuard};
use crate::{
    ClockError, ExecutionContext, ExecutionFailure, ExecutionPolicy, ExecutionRequest,
    ExecutionStartWindow, HostClock, HostError, HostServices, Presentation, ProcessRunner,
    RuntimeOrchestrator, RuntimeProbe, RuntimeProbeError, RuntimeSelection, StandardProcessRunner,
    StandardRuntimeProbe, SystemClock,
};
use serde::{Deserialize, Deserializer, Serialize};
use sha2::{Digest, Sha256};
use std::fs::{self, File};
use std::io::{self, Read};
use std::path::{Component, Path};
use std::time::Instant;
use thiserror::Error;
use tsukumo_adapters::{
    ClaudeRuntimeProfile, CodexRuntimeProfile, RuntimeLaunchConfig, RuntimeProfile,
};
use tsukumo_kernel::{
    contains_sensitive_material, is_terminal_unsafe_character, ArtifactId, CheckpointId,
    CorrelationId, EventId, ExecutionId, KernelEvent, KernelEventPayload, OutcomeStatus, OwnerId,
    PersistedText, ProjectionId, QuestId, RuntimeBinding, RuntimeKind, RuntimeMode, SensitiveText,
    SessionId, SpiritId, StateId, StateLifecycleAction, Timestamp, WorkspaceId,
    KERNEL_EVENT_SCHEMA_VERSION,
};
use tsukumo_soul::{
    ArtifactReference, CheckpointTrigger, CheckpointWriteRequest, Decision, ExtractionContext,
    HandoffCheckpoint, NextAction, OpenLoop, OpenLoopId, OperatingSystem, ProgressItem,
    ProgressStatus, ProjectionRequest, ProjectionTarget, ProjectionWriteRequest,
    RuleStateExtractor, SoulError, SoulStore, StateApplicability, StateExtractor, StateRef,
    StateScope, StateStatus, StateSubject, StateTransition, StateWriteOutcome, StateWriteRequest,
};
use tsukumo_theater::{DirectorContext, StageWorld};

const EPISODE_SPEC_SCHEMA_VERSION: u16 = 1;
const EPISODE_SUMMARY_SCHEMA_VERSION: u16 = 1;
const MAX_SPEC_BYTES: u64 = 1_048_576;
const MAX_LABEL_CHARS: usize = 256;
const MAX_REVIEWED_TEXT_CHARS: usize = 65_536;
const MAX_COLLECTION_ITEMS: usize = 64;
const MAX_DELAY_HOURS: u64 = 24 * 14;

/// Evaluation condition frozen before one episode starts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EpisodeCondition {
    C0,
    C1,
    C2,
}

/// Runtime family named by a reviewed episode spec.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EpisodeRuntimeKind {
    ClaudeCli,
    CodexCli,
}

impl EpisodeRuntimeKind {
    fn binding(self) -> RuntimeBinding {
        let kind = match self {
            Self::ClaudeCli => RuntimeKind::ClaudeCli,
            Self::CodexCli => RuntimeKind::CodexCli,
        };
        RuntimeBinding::new(kind, RuntimeMode::OwnedProcess)
    }
}

/// Concrete sandbox and approval profile frozen in the reviewed artifact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EpisodeExecutionProfile {
    ClaudeDenyUnapproved,
    CodexReadOnly,
    CodexWorkspaceWrite,
}

impl EpisodeExecutionProfile {
    fn runtime_kind(self) -> EpisodeRuntimeKind {
        match self {
            Self::ClaudeDenyUnapproved => EpisodeRuntimeKind::ClaudeCli,
            Self::CodexReadOnly | Self::CodexWorkspaceWrite => EpisodeRuntimeKind::CodexCli,
        }
    }
}

/// Runtime identity frozen in the reviewed artifact.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpisodeRuntimeV1 {
    pub kind: EpisodeRuntimeKind,
    pub version: String,
    pub execution_profile: EpisodeExecutionProfile,
}

/// Delay window derived from the actual committed seed timestamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpisodeDelayV1 {
    pub minimum_hours: u64,
    pub maximum_hours: u64,
}

/// Checkpoint content reviewed after a real source-runtime action.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpisodeCheckpointV1 {
    pub goal: PersistedText,
    #[serde(default, deserialize_with = "deserialize_progress")]
    pub progress: Vec<ProgressItem>,
    #[serde(default, deserialize_with = "deserialize_decisions")]
    pub decisions: Vec<Decision>,
    #[serde(default, deserialize_with = "deserialize_artifacts")]
    pub artifacts: Vec<ArtifactReference>,
    #[serde(default, deserialize_with = "deserialize_open_loops")]
    pub open_loops: Vec<OpenLoop>,
    #[serde(default, deserialize_with = "deserialize_next_actions")]
    pub next_actions: Vec<NextAction>,
    #[serde(default)]
    pub constraint_state_ids: Vec<StateId>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StrictProgressItem {
    summary: PersistedText,
    status: ProgressStatus,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StrictDecision {
    summary: PersistedText,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StrictArtifactReference {
    artifact_id: ArtifactId,
    location: PersistedText,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StrictOpenLoop {
    id: OpenLoopId,
    summary: PersistedText,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StrictNextAction {
    summary: PersistedText,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case", deny_unknown_fields)]
enum StrictStateSubject {
    Owner {
        owner_id: OwnerId,
    },
    Workspace {
        workspace_id: WorkspaceId,
    },
    Spirit {
        spirit_id: SpiritId,
    },
    Relationship {
        owner_id: OwnerId,
        spirit_id: SpiritId,
    },
    Unresolved,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StrictStateApplicability {
    #[serde(default)]
    workspace: Option<WorkspaceId>,
    #[serde(default)]
    operating_system: Option<OperatingSystem>,
    #[serde(default)]
    task_tags: Vec<String>,
    #[serde(default)]
    language_tags: Vec<String>,
    #[serde(default)]
    required_capabilities: Vec<String>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct StrictStateScope {
    subject: StrictStateSubject,
    applicability: StrictStateApplicability,
}

fn deserialize_progress<'de, D>(deserializer: D) -> Result<Vec<ProgressItem>, D::Error>
where
    D: Deserializer<'de>,
{
    Vec::<StrictProgressItem>::deserialize(deserializer).map(|items| {
        items
            .into_iter()
            .map(|item| ProgressItem::new(item.summary, item.status))
            .collect()
    })
}

fn deserialize_decisions<'de, D>(deserializer: D) -> Result<Vec<Decision>, D::Error>
where
    D: Deserializer<'de>,
{
    Vec::<StrictDecision>::deserialize(deserializer).map(|items| {
        items
            .into_iter()
            .map(|item| Decision::new(item.summary))
            .collect()
    })
}

fn deserialize_artifacts<'de, D>(deserializer: D) -> Result<Vec<ArtifactReference>, D::Error>
where
    D: Deserializer<'de>,
{
    Vec::<StrictArtifactReference>::deserialize(deserializer).map(|items| {
        items
            .into_iter()
            .map(|item| ArtifactReference::new(item.artifact_id, item.location))
            .collect()
    })
}

fn deserialize_open_loops<'de, D>(deserializer: D) -> Result<Vec<OpenLoop>, D::Error>
where
    D: Deserializer<'de>,
{
    Vec::<StrictOpenLoop>::deserialize(deserializer).map(|items| {
        items
            .into_iter()
            .map(|item| OpenLoop::new(item.id, item.summary))
            .collect()
    })
}

fn deserialize_next_actions<'de, D>(deserializer: D) -> Result<Vec<NextAction>, D::Error>
where
    D: Deserializer<'de>,
{
    Vec::<StrictNextAction>::deserialize(deserializer).map(|items| {
        items
            .into_iter()
            .map(|item| NextAction::new(item.summary))
            .collect()
    })
}

fn deserialize_scope<'de, D>(deserializer: D) -> Result<StateScope, D::Error>
where
    D: Deserializer<'de>,
{
    StrictStateScope::deserialize(deserializer).map(|scope| StateScope {
        subject: match scope.subject {
            StrictStateSubject::Owner { owner_id } => StateSubject::Owner { owner_id },
            StrictStateSubject::Workspace { workspace_id } => {
                StateSubject::Workspace { workspace_id }
            }
            StrictStateSubject::Spirit { spirit_id } => StateSubject::Spirit { spirit_id },
            StrictStateSubject::Relationship {
                owner_id,
                spirit_id,
            } => StateSubject::Relationship {
                owner_id,
                spirit_id,
            },
            StrictStateSubject::Unresolved => StateSubject::Unresolved,
        },
        applicability: StateApplicability {
            workspace: scope.applicability.workspace,
            operating_system: scope.applicability.operating_system,
            task_tags: scope.applicability.task_tags,
            language_tags: scope.applicability.language_tags,
            required_capabilities: scope.applicability.required_capabilities,
        },
    })
}

/// Target projection inputs. The delegation goal remains in memory after load.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpisodeProjectionV1 {
    #[serde(deserialize_with = "deserialize_scope")]
    pub scope: StateScope,
    pub budget_chars: usize,
    pub delegation_goal: String,
}

/// Reviewed pre-registration and source summary for one handoff episode.
#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EpisodeSpecV1 {
    pub schema_version: u16,
    pub episode_id: String,
    pub condition: EpisodeCondition,
    pub episode_type: String,
    pub workload_block: String,
    pub fault: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewed_git_head: Option<String>,
    pub quest_id: QuestId,
    pub source_session_id: SessionId,
    pub target_session_id: SessionId,
    pub spirit_id: SpiritId,
    pub source_runtime: EpisodeRuntimeV1,
    pub target_runtime: EpisodeRuntimeV1,
    pub source_summary: PersistedText,
    #[serde(default)]
    pub explicit_state_input: Option<PersistedText>,
    pub checkpoint: EpisodeCheckpointV1,
    pub projection: EpisodeProjectionV1,
    pub delay: EpisodeDelayV1,
    pub quality_gate: Vec<String>,
}

/// Redacted result of committing one reviewed seed.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EpisodeSeedSummaryV1 {
    pub schema_version: u16,
    pub episode_id: String,
    pub condition: EpisodeCondition,
    pub manual_baseline_required: bool,
    pub seeded_at_unix_ms: Option<i64>,
    pub resume_not_before_unix_ms: Option<i64>,
    pub resume_not_after_unix_ms: Option<i64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkpoint_id: Option<CheckpointId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_event_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_count: Option<usize>,
    pub evidence_controls_exposed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_event_ids: Option<Vec<EventId>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub state_ids: Option<Vec<StateId>>,
    pub manual_metrics_pending: bool,
}

/// Redacted machine-observable result of one target-runtime resume.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EpisodeRunSummaryV1 {
    pub schema_version: u16,
    pub episode_id: String,
    pub condition: EpisodeCondition,
    pub runtime: EpisodeRuntimeKind,
    pub execution_profile: EpisodeExecutionProfile,
    pub runtime_version: String,
    pub episode_started_at_unix_ms: i64,
    pub episode_ended_at_unix_ms: i64,
    pub projection_ms: u64,
    pub runtime_ms: u64,
    pub storage_delta_bytes: Option<i64>,
    pub status: OutcomeStatus,
    pub failure: Option<String>,
    pub committed_events: usize,
    pub stderr_lines: usize,
    pub known_ignored_lines: usize,
    pub unknown_skipped_lines: usize,
    pub evidence_controls_exposed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub checkpoint_id: Option<CheckpointId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub projection_id: Option<ProjectionId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution_id: Option<ExecutionId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rendered_digest_sha256: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_state_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub omitted_state_count: Option<usize>,
    pub manual_metrics_pending: bool,
}

/// Episode input, persistence, or execution failure.
#[derive(Debug, Error)]
pub enum EpisodeError {
    #[error("failed to read reviewed episode spec: {0}")]
    SpecIo(#[source] io::Error),
    #[error("reviewed episode spec exceeds one MiB")]
    SpecTooLarge,
    #[error("failed to parse reviewed episode spec: {0}")]
    SpecJson(#[from] serde_json::Error),
    #[error("unsupported episode spec schema {0}")]
    UnsupportedSchema(u16),
    #[error("invalid episode spec field {0}")]
    InvalidSpec(&'static str),
    #[error("C0 is a repository-native manual baseline and does not launch through Tsukumo")]
    ManualBaseline,
    #[error("episode data path rejected: {0}")]
    LocalPath(String),
    #[error(transparent)]
    Clock(#[from] ClockError),
    #[error(transparent)]
    Soul(#[from] SoulError),
    #[error(transparent)]
    Host(#[from] HostError),
    #[error("reviewed explicit state input did not produce exactly one deterministic draft")]
    ExplicitStateNotExtracted,
    #[error("checkpoint for the reviewed episode spec was not seeded")]
    MissingCheckpoint,
    #[error("episode resume is earlier than the committed window start {not_before_unix_ms}")]
    ResumeTooEarly { not_before_unix_ms: i64 },
    #[error("episode resume is later than the committed window end {not_after_unix_ms}")]
    ResumeWindowClosed { not_after_unix_ms: i64 },
    #[error("reviewed episode registration differs from the immutable seeded checkpoint")]
    RegistrationMismatch,
    #[error(transparent)]
    RuntimeProbe(#[from] RuntimeProbeError),
    #[error("observed runtime family differs from the reviewed target runtime")]
    RuntimeFamilyMismatch,
    #[error("observed runtime version differs from the reviewed target runtime")]
    RuntimeVersionMismatch,
    #[error("workspace-write target requires --workspace-write acknowledgement")]
    WorkspaceWriteAcknowledgementRequired,
    #[error("--workspace-write does not match the reviewed target execution profile")]
    WorkspaceWriteAcknowledgementMismatch,
    #[error("live runtime execution requires --confirm-live-run")]
    LiveRunConfirmationRequired,
}

/// Reads and validates one bounded reviewed JSON spec.
pub fn read_episode_spec(path: impl AsRef<Path>) -> Result<EpisodeSpecV1, EpisodeError> {
    let file = File::open(path).map_err(EpisodeError::SpecIo)?;
    let mut body = Vec::new();
    file.take(MAX_SPEC_BYTES + 1)
        .read_to_end(&mut body)
        .map_err(EpisodeError::SpecIo)?;
    if body.len() as u64 > MAX_SPEC_BYTES {
        return Err(EpisodeError::SpecTooLarge);
    }
    let spec = serde_json::from_slice::<EpisodeSpecV1>(&body)?;
    validate_spec(&spec)?;
    Ok(spec)
}

/// Commits a reviewed C1/C2 source summary and checkpoint with the system clock.
pub fn seed_episode(
    spec: &EpisodeSpecV1,
    data_dir: impl AsRef<Path>,
) -> Result<EpisodeSeedSummaryV1, EpisodeError> {
    seed_episode_with_clock(spec, data_dir, &SystemClock)
}

/// Testable seed boundary that retains the production storage path.
pub fn seed_episode_with_clock(
    spec: &EpisodeSpecV1,
    data_dir: impl AsRef<Path>,
    clock: &dyn HostClock,
) -> Result<EpisodeSeedSummaryV1, EpisodeError> {
    validate_spec(spec)?;
    if spec.condition == EpisodeCondition::C0 {
        return Ok(EpisodeSeedSummaryV1 {
            schema_version: EPISODE_SUMMARY_SCHEMA_VERSION,
            episode_id: spec.episode_id.clone(),
            condition: spec.condition,
            manual_baseline_required: true,
            seeded_at_unix_ms: None,
            resume_not_before_unix_ms: None,
            resume_not_after_unix_ms: None,
            checkpoint_id: None,
            source_event_count: None,
            state_count: None,
            evidence_controls_exposed: false,
            source_event_ids: None,
            state_ids: None,
            manual_metrics_pending: true,
        });
    }

    let fingerprint = spec_fingerprint(spec)?;
    let registration_digest = registration_digest(spec)?;
    let checkpoint_id = CheckpointId::new(stable_id("checkpoint", &fingerprint));
    let (_guard, mut store) = open_guarded_store(data_dir.as_ref())?;
    if let Some(existing) = store.checkpoint(&checkpoint_id)? {
        verify_registration(spec, &existing)?;
        return seed_summary(
            spec,
            &existing,
            source_ids(spec, &fingerprint),
            state_ids(spec, &fingerprint),
        );
    }

    let summary_event_id = EventId::new(stable_id("source-summary-event", &fingerprint));
    let now = match store.event(&summary_event_id)? {
        Some(existing) => existing.event.occurred_at,
        None => clock.now()?,
    };
    if let Some(input) = &spec.explicit_state_input {
        let source = base_event(
            stable_id("state-source-event", &fingerprint),
            now,
            spec,
            None,
            KernelEventPayload::UserInput {
                content: input.clone(),
            },
        );
        let drafts = RuleStateExtractor
            .extract(&ExtractionContext {
                event: &source,
                scope: spec.projection.scope.clone(),
            })
            .map_err(|_| EpisodeError::ExplicitStateNotExtracted)?;
        if drafts.len() != 1 {
            return Err(EpisodeError::ExplicitStateNotExtracted);
        }
    }
    let mut constraint_refs = Vec::new();
    let summary_event = base_event(
        summary_event_id.as_str().to_owned(),
        now,
        spec,
        None,
        KernelEventPayload::UserInput {
            content: spec.source_summary.clone(),
        },
    );
    store.append_event(&summary_event)?;
    let mut source_event_ids = vec![summary_event.event_id.clone()];
    let mut created_state_ids = Vec::new();

    if let Some(input) = &spec.explicit_state_input {
        let source = base_event(
            stable_id("state-source-event", &fingerprint),
            now,
            spec,
            None,
            KernelEventPayload::UserInput {
                content: input.clone(),
            },
        );
        let mut drafts = RuleStateExtractor
            .extract(&ExtractionContext {
                event: &source,
                scope: spec.projection.scope.clone(),
            })
            .map_err(|_| EpisodeError::ExplicitStateNotExtracted)?;
        if drafts.len() != 1 {
            return Err(EpisodeError::ExplicitStateNotExtracted);
        }
        let draft = drafts.pop().expect("one draft was length-checked");
        let state_id = StateId::new(stable_id("state", &fingerprint));
        let lifecycle = base_event(
            stable_id("state-lifecycle-event", &fingerprint),
            now,
            spec,
            None,
            KernelEventPayload::StateLifecycle {
                state_id: state_id.clone(),
                action: StateLifecycleAction::Created,
                prior_state_id: None,
                reason: None,
            },
        );
        let outcome = store.apply_state(
            StateWriteRequest::new(
                StateTransition::Create {
                    state_id,
                    draft,
                    created_at: now,
                },
                lifecycle,
            )
            .with_source_event(source.clone()),
        )?;
        let record = match outcome {
            StateWriteOutcome::Created(record) | StateWriteOutcome::Unchanged(record) => record,
            StateWriteOutcome::Superseded(_) | StateWriteOutcome::Revoked(_) => {
                return Err(EpisodeError::InvalidSpec("explicit_state_input"))
            }
        };
        source_event_ids.push(source.event_id);
        created_state_ids.push(record.state_id.clone());
        constraint_refs.push(StateRef::new(record.state_id, record.version));
    }

    for state_id in &spec.checkpoint.constraint_state_ids {
        let Some(record) = store.state(state_id)? else {
            return Err(EpisodeError::InvalidSpec("checkpoint.constraint_state_ids"));
        };
        if record.status != StateStatus::Active || !record.is_active_at(now) {
            return Err(EpisodeError::InvalidSpec("checkpoint.constraint_state_ids"));
        }
        constraint_refs.push(StateRef::new(record.state_id, record.version));
    }

    let checkpoint = build_checkpoint(
        spec,
        checkpoint_id,
        now,
        constraint_refs,
        source_event_ids.clone(),
        registration_digest,
    );
    let checkpoint_event = base_event(
        stable_id("checkpoint-event", &fingerprint),
        now,
        spec,
        Some(spec.source_runtime.kind.binding()),
        KernelEventPayload::CheckpointCreated {
            checkpoint_id: checkpoint.id.clone(),
            version: checkpoint.version,
        },
    );
    store.save_checkpoint(CheckpointWriteRequest::new(
        checkpoint.clone(),
        checkpoint_event,
    ))?;
    seed_summary(spec, &checkpoint, source_event_ids, created_state_ids)
}

/// Executes one committed projection through the standard owned-process runner.
pub fn resume_episode(
    spec: &EpisodeSpecV1,
    data_dir: impl AsRef<Path>,
    runtime_executable: impl AsRef<Path>,
    working_dir: impl AsRef<Path>,
    workspace_write_acknowledged: bool,
    live_run_confirmed: bool,
) -> Result<EpisodeRunSummaryV1, EpisodeError> {
    resume_episode_with_services(
        spec,
        data_dir,
        runtime_executable,
        working_dir,
        workspace_write_acknowledged,
        live_run_confirmed,
        &StandardRuntimeProbe,
        &StandardProcessRunner,
        &SystemClock,
    )
}

/// Testable resume boundary using the same receipt-first production composition.
#[allow(clippy::too_many_arguments)]
pub fn resume_episode_with_services(
    spec: &EpisodeSpecV1,
    data_dir: impl AsRef<Path>,
    runtime_executable: impl AsRef<Path>,
    working_dir: impl AsRef<Path>,
    workspace_write_acknowledged: bool,
    live_run_confirmed: bool,
    probe: &dyn RuntimeProbe,
    runner: &dyn ProcessRunner,
    clock: &dyn HostClock,
) -> Result<EpisodeRunSummaryV1, EpisodeError> {
    validate_spec(spec)?;
    if spec.condition == EpisodeCondition::C0 {
        return Err(EpisodeError::ManualBaseline);
    }
    if !live_run_confirmed {
        return Err(EpisodeError::LiveRunConfirmationRequired);
    }

    let profile: Box<dyn RuntimeProfile> = match spec.target_runtime.execution_profile {
        EpisodeExecutionProfile::ClaudeDenyUnapproved => {
            if workspace_write_acknowledged {
                return Err(EpisodeError::WorkspaceWriteAcknowledgementMismatch);
            }
            Box::new(ClaudeRuntimeProfile::deny_unapproved())
        }
        EpisodeExecutionProfile::CodexReadOnly => {
            if workspace_write_acknowledged {
                return Err(EpisodeError::WorkspaceWriteAcknowledgementMismatch);
            }
            Box::new(CodexRuntimeProfile::read_only())
        }
        EpisodeExecutionProfile::CodexWorkspaceWrite => {
            if !workspace_write_acknowledged {
                return Err(EpisodeError::WorkspaceWriteAcknowledgementRequired);
            }
            Box::new(CodexRuntimeProfile::workspace_write())
        }
    };

    let fingerprint = spec_fingerprint(spec)?;
    let checkpoint_id = CheckpointId::new(stable_id("checkpoint", &fingerprint));
    let projection_id = ProjectionId::new(stable_id("projection", &fingerprint));
    let execution_id = ExecutionId::new(stable_id("execution", &fingerprint));
    let (_guard, mut store) = open_guarded_store(data_dir.as_ref())?;
    let Some(checkpoint) = store.checkpoint(&checkpoint_id)? else {
        return Err(EpisodeError::MissingCheckpoint);
    };
    verify_registration(spec, &checkpoint)?;

    let preflight_at = clock.now()?;
    enforce_resume_window(spec.delay, checkpoint.created_at, preflight_at)?;
    let (not_before, not_after) = resume_window(spec.delay, checkpoint.created_at)?;
    let start_window = ExecutionStartWindow::new(
        Timestamp::from_unix_millis(not_before),
        Timestamp::from_unix_millis(not_after),
    );

    let runtime = profile.binding();
    if runtime != spec.target_runtime.kind.binding() {
        return Err(EpisodeError::RuntimeFamilyMismatch);
    }
    let working_guard = LocalDirectoryGuard::existing(working_dir.as_ref())
        .map_err(|error| EpisodeError::LocalPath(error.to_string()))?;
    let launch = RuntimeLaunchConfig::new(
        runtime_executable.as_ref().to_path_buf(),
        working_guard.root().to_path_buf(),
    );
    let observed = probe.probe(profile.as_ref(), &launch)?;
    if observed.binding != runtime {
        return Err(EpisodeError::RuntimeFamilyMismatch);
    }
    if observed.version != spec.target_runtime.version {
        return Err(EpisodeError::RuntimeVersionMismatch);
    }

    let projection_created_at = store
        .projection_receipt(&projection_id)?
        .map_or(preflight_at, |receipt| receipt.created_at);
    let request = ProjectionRequest::new(
        ProjectionTarget::new(
            projection_id.clone(),
            execution_id.clone(),
            runtime.clone(),
            checkpoint_id.clone(),
        ),
        spec.projection.scope.clone(),
        SensitiveText::new(spec.projection.delegation_goal.clone()),
        projection_created_at,
        spec.projection.budget_chars,
    );
    let projection_event = base_event(
        stable_id("projection-event", &fingerprint),
        projection_created_at,
        spec,
        Some(runtime),
        KernelEventPayload::ProjectionCreated {
            projection_id: projection_id.clone(),
            checkpoint_id: checkpoint_id.clone(),
        },
    )
    .with_execution(
        execution_id.clone(),
        stable_id("projection-correlation", &fingerprint),
    )
    .with_session(spec.target_session_id.clone());

    let storage_before = database_size(&store);
    let projection_started = Instant::now();
    let prepared =
        store.prepare_projection(ProjectionWriteRequest::new(request, projection_event))?;
    let projection_ms = elapsed_millis(projection_started);

    let mut world = StageWorld::new();
    let director = DirectorContext::default();
    let runtime_started = Instant::now();
    let report = {
        let mut host = RuntimeOrchestrator::new(
            HostServices::new(&mut store, runner, clock),
            Presentation::new(&mut world, &director),
            ExecutionPolicy::default(),
        );
        host.execute(
            ExecutionRequest::new(
                &prepared,
                RuntimeSelection::new(profile.as_ref(), &launch),
                ExecutionContext::new(
                    spec.quest_id.clone(),
                    spec.target_session_id.clone(),
                    spec.spirit_id.clone(),
                ),
            )
            .with_start_window(start_window),
        )
        .map_err(map_host_error)?
    };
    let runtime_ms = elapsed_millis(runtime_started);
    let ended_at = clock.now()?;
    let storage_after = database_size(&store);
    let expose = spec.condition == EpisodeCondition::C2;

    Ok(EpisodeRunSummaryV1 {
        schema_version: EPISODE_SUMMARY_SCHEMA_VERSION,
        episode_id: spec.episode_id.clone(),
        condition: spec.condition,
        runtime: spec.target_runtime.kind,
        execution_profile: spec.target_runtime.execution_profile,
        runtime_version: observed.version,
        episode_started_at_unix_ms: report.started_at.as_unix_millis(),
        episode_ended_at_unix_ms: ended_at.as_unix_millis(),
        projection_ms,
        runtime_ms,
        storage_delta_bytes: storage_delta(storage_before, storage_after),
        status: report.status,
        failure: report.failure.map(failure_label).map(str::to_owned),
        committed_events: report.committed_events,
        stderr_lines: report.stderr_lines,
        known_ignored_lines: report.known_ignored_lines,
        unknown_skipped_lines: report.unknown_skipped_lines,
        evidence_controls_exposed: expose,
        checkpoint_id: expose.then_some(checkpoint_id),
        projection_id: expose.then_some(projection_id),
        execution_id: expose.then_some(execution_id),
        rendered_digest_sha256: expose.then(|| prepared.receipt.rendered_digest.value.clone()),
        selected_state_count: expose.then_some(prepared.receipt.selected_state_refs.len()),
        omitted_state_count: expose.then_some(prepared.receipt.omissions.len()),
        manual_metrics_pending: true,
    })
}

trait EventExecutionExt {
    fn with_execution(self, execution_id: ExecutionId, correlation_id: String) -> Self;
    fn with_session(self, session_id: SessionId) -> Self;
}

impl EventExecutionExt for KernelEvent {
    fn with_execution(mut self, execution_id: ExecutionId, correlation_id: String) -> Self {
        self.execution_id = Some(execution_id);
        self.correlation_id = Some(CorrelationId::new(correlation_id));
        self
    }

    fn with_session(mut self, session_id: SessionId) -> Self {
        self.session_id = session_id;
        self
    }
}

pub(crate) fn validate_spec(spec: &EpisodeSpecV1) -> Result<(), EpisodeError> {
    if spec.schema_version != EPISODE_SPEC_SCHEMA_VERSION {
        return Err(EpisodeError::UnsupportedSchema(spec.schema_version));
    }
    for (field, value) in [
        ("episode_id", spec.episode_id.as_str()),
        ("episode_type", spec.episode_type.as_str()),
        ("workload_block", spec.workload_block.as_str()),
        ("fault", spec.fault.as_str()),
        ("quest_id", spec.quest_id.as_str()),
        ("source_session_id", spec.source_session_id.as_str()),
        ("target_session_id", spec.target_session_id.as_str()),
        ("spirit_id", spec.spirit_id.as_str()),
        (
            "source_runtime.version",
            spec.source_runtime.version.as_str(),
        ),
        (
            "target_runtime.version",
            spec.target_runtime.version.as_str(),
        ),
    ] {
        validate_label(field, value)?;
    }
    if let Some(reviewed_git_head) = &spec.reviewed_git_head {
        if !matches!(reviewed_git_head.len(), 40 | 64)
            || !reviewed_git_head
                .chars()
                .all(|character| character.is_ascii_hexdigit())
        {
            return Err(EpisodeError::InvalidSpec("reviewed_git_head"));
        }
    }
    if spec.source_runtime.kind == spec.target_runtime.kind {
        return Err(EpisodeError::InvalidSpec("source_runtime.kind"));
    }
    if spec.source_runtime.execution_profile.runtime_kind() != spec.source_runtime.kind {
        return Err(EpisodeError::InvalidSpec(
            "source_runtime.execution_profile",
        ));
    }
    if spec.target_runtime.execution_profile.runtime_kind() != spec.target_runtime.kind {
        return Err(EpisodeError::InvalidSpec(
            "target_runtime.execution_profile",
        ));
    }
    validate_reviewed_text("source_summary", spec.source_summary.as_str())?;
    if let Some(input) = &spec.explicit_state_input {
        validate_reviewed_text("explicit_state_input", input.as_str())?;
    }
    validate_reviewed_text("checkpoint.goal", spec.checkpoint.goal.as_str())?;
    validate_reviewed_text(
        "projection.delegation_goal",
        &spec.projection.delegation_goal,
    )?;
    validate_projection_scope(&spec.projection.scope)?;
    for item in &spec.checkpoint.progress {
        validate_reviewed_text("checkpoint.progress.summary", item.summary.as_str())?;
    }
    for decision in &spec.checkpoint.decisions {
        validate_reviewed_text("checkpoint.decisions.summary", decision.summary.as_str())?;
    }
    for artifact in &spec.checkpoint.artifacts {
        validate_label(
            "checkpoint.artifacts.artifact_id",
            artifact.artifact_id.as_str(),
        )?;
        validate_artifact_location(artifact.location.as_str())?;
    }
    let mut open_loop_ids = std::collections::BTreeSet::new();
    for open_loop in &spec.checkpoint.open_loops {
        validate_label("checkpoint.open_loops.id", open_loop.id.as_str())?;
        validate_reviewed_text("checkpoint.open_loops.summary", open_loop.summary.as_str())?;
        if !open_loop_ids.insert(open_loop.id.as_str()) {
            return Err(EpisodeError::InvalidSpec("checkpoint.open_loops.id"));
        }
    }
    for action in &spec.checkpoint.next_actions {
        validate_reviewed_text("checkpoint.next_actions.summary", action.summary.as_str())?;
    }
    for state_id in &spec.checkpoint.constraint_state_ids {
        validate_label("checkpoint.constraint_state_ids", state_id.as_str())?;
    }
    if spec.projection.budget_chars == 0 {
        return Err(EpisodeError::InvalidSpec("projection.budget_chars"));
    }
    if spec.delay.minimum_hours > spec.delay.maximum_hours
        || spec.delay.maximum_hours > MAX_DELAY_HOURS
    {
        return Err(EpisodeError::InvalidSpec("delay"));
    }
    if spec.quality_gate.is_empty() || spec.quality_gate.len() > MAX_COLLECTION_ITEMS {
        return Err(EpisodeError::InvalidSpec("quality_gate"));
    }
    for gate in &spec.quality_gate {
        validate_reviewed_text("quality_gate", gate)?;
    }
    for length in [
        spec.checkpoint.progress.len(),
        spec.checkpoint.decisions.len(),
        spec.checkpoint.artifacts.len(),
        spec.checkpoint.open_loops.len(),
        spec.checkpoint.next_actions.len(),
        spec.checkpoint.constraint_state_ids.len(),
    ] {
        if length > MAX_COLLECTION_ITEMS {
            return Err(EpisodeError::InvalidSpec("checkpoint"));
        }
    }
    let mut state_ids = spec
        .checkpoint
        .constraint_state_ids
        .iter()
        .map(StateId::as_str)
        .collect::<Vec<_>>();
    state_ids.sort_unstable();
    if state_ids.windows(2).any(|pair| pair[0] == pair[1]) {
        return Err(EpisodeError::InvalidSpec("checkpoint.constraint_state_ids"));
    }
    Ok(())
}

fn validate_label(field: &'static str, value: &str) -> Result<(), EpisodeError> {
    if value.trim().is_empty()
        || value.chars().count() > MAX_LABEL_CHARS
        || value.chars().any(is_terminal_unsafe_character)
        || contains_sensitive_material(value)
        || contains_personal_path(value)
    {
        return Err(EpisodeError::InvalidSpec(field));
    }
    Ok(())
}

fn validate_reviewed_text(field: &'static str, value: &str) -> Result<(), EpisodeError> {
    if value.trim().is_empty()
        || value.chars().count() > MAX_REVIEWED_TEXT_CHARS
        || value
            .chars()
            .any(|character| character != '\n' && is_terminal_unsafe_character(character))
        || contains_sensitive_material(value)
        || contains_personal_path(value)
    {
        return Err(EpisodeError::InvalidSpec(field));
    }
    Ok(())
}

fn validate_projection_scope(scope: &StateScope) -> Result<(), EpisodeError> {
    match &scope.subject {
        StateSubject::Owner { owner_id } => {
            validate_label("projection.scope.subject.owner_id", owner_id.as_str())?;
        }
        StateSubject::Workspace { workspace_id } => {
            validate_label(
                "projection.scope.subject.workspace_id",
                workspace_id.as_str(),
            )?;
            if scope.applicability.workspace.as_ref() != Some(workspace_id) {
                return Err(EpisodeError::InvalidSpec(
                    "projection.scope.applicability.workspace",
                ));
            }
        }
        StateSubject::Spirit { spirit_id } => {
            validate_label("projection.scope.subject.spirit_id", spirit_id.as_str())?;
        }
        StateSubject::Relationship {
            owner_id,
            spirit_id,
        } => {
            validate_label("projection.scope.subject.owner_id", owner_id.as_str())?;
            validate_label("projection.scope.subject.spirit_id", spirit_id.as_str())?;
        }
        StateSubject::Unresolved => {
            return Err(EpisodeError::InvalidSpec("projection.scope.subject"));
        }
    }
    if let Some(workspace) = &scope.applicability.workspace {
        validate_label(
            "projection.scope.applicability.workspace",
            workspace.as_str(),
        )?;
    }
    for (field, tags) in [
        (
            "projection.scope.applicability.task_tags",
            &scope.applicability.task_tags,
        ),
        (
            "projection.scope.applicability.language_tags",
            &scope.applicability.language_tags,
        ),
        (
            "projection.scope.applicability.required_capabilities",
            &scope.applicability.required_capabilities,
        ),
    ] {
        if tags.len() > MAX_COLLECTION_ITEMS {
            return Err(EpisodeError::InvalidSpec(field));
        }
        for tag in tags {
            validate_label(field, tag)?;
        }
    }
    Ok(())
}

fn validate_artifact_location(value: &str) -> Result<(), EpisodeError> {
    validate_reviewed_text("checkpoint.artifacts.location", value)?;
    let segments = value.split('/').collect::<Vec<_>>();
    if value.contains('\\')
        || value.contains(':')
        || segments.is_empty()
        || segments
            .iter()
            .any(|segment| segment.is_empty() || matches!(*segment, "." | ".."))
        || !Path::new(value)
            .components()
            .all(|component| matches!(component, Component::Normal(_)))
    {
        return Err(EpisodeError::InvalidSpec("checkpoint.artifacts.location"));
    }
    Ok(())
}

fn contains_personal_path(value: &str) -> bool {
    let normalized = value.replace('\\', "/").to_ascii_lowercase();
    normalized.contains("/home/")
        || normalized.contains("/users/")
        || normalized.contains("~/")
        || normalized.contains("%userprofile%")
        || normalized.contains("$home")
        || normalized.contains("auth.json")
}

fn open_guarded_store(data_dir: &Path) -> Result<(LocalDirectoryGuard, SoulStore), EpisodeError> {
    let guard = prepare_data_directory(data_dir)
        .map_err(|error| EpisodeError::LocalPath(error.to_string()))?;
    let store = SoulStore::open(guard.root())?;
    Ok((guard, store))
}

fn build_checkpoint(
    spec: &EpisodeSpecV1,
    checkpoint_id: CheckpointId,
    created_at: Timestamp,
    constraint_refs: Vec<StateRef>,
    source_event_refs: Vec<EventId>,
    registration_digest: String,
) -> HandoffCheckpoint {
    HandoffCheckpoint::new(
        checkpoint_id,
        spec.quest_id.clone(),
        1,
        None,
        spec.checkpoint.goal.clone(),
        created_at,
        CheckpointTrigger::RuntimeSwitch,
    )
    .with_progress(spec.checkpoint.progress.clone())
    .with_decisions(spec.checkpoint.decisions.clone())
    .with_constraint_refs(constraint_refs)
    .with_artifacts(spec.checkpoint.artifacts.clone())
    .with_open_loops(spec.checkpoint.open_loops.clone())
    .with_next_actions(spec.checkpoint.next_actions.clone())
    .with_source_event_refs(source_event_refs)
    .with_registration_digest(registration_digest)
}

fn seed_summary(
    spec: &EpisodeSpecV1,
    checkpoint: &HandoffCheckpoint,
    source_event_ids: Vec<EventId>,
    state_ids: Vec<StateId>,
) -> Result<EpisodeSeedSummaryV1, EpisodeError> {
    let (not_before, not_after) = resume_window(spec.delay, checkpoint.created_at)?;
    let expose = spec.condition == EpisodeCondition::C2;
    Ok(EpisodeSeedSummaryV1 {
        schema_version: EPISODE_SUMMARY_SCHEMA_VERSION,
        episode_id: spec.episode_id.clone(),
        condition: spec.condition,
        manual_baseline_required: false,
        seeded_at_unix_ms: Some(checkpoint.created_at.as_unix_millis()),
        resume_not_before_unix_ms: Some(not_before),
        resume_not_after_unix_ms: Some(not_after),
        checkpoint_id: expose.then(|| checkpoint.id.clone()),
        source_event_count: expose.then_some(source_event_ids.len()),
        state_count: expose.then_some(state_ids.len()),
        evidence_controls_exposed: expose,
        source_event_ids: expose.then_some(source_event_ids),
        state_ids: expose.then_some(state_ids),
        manual_metrics_pending: true,
    })
}

fn source_ids(spec: &EpisodeSpecV1, fingerprint: &str) -> Vec<EventId> {
    let mut ids = vec![EventId::new(stable_id("source-summary-event", fingerprint))];
    if spec.explicit_state_input.is_some() {
        ids.push(EventId::new(stable_id("state-source-event", fingerprint)));
    }
    ids
}

fn state_ids(spec: &EpisodeSpecV1, fingerprint: &str) -> Vec<StateId> {
    spec.explicit_state_input
        .as_ref()
        .map(|_| vec![StateId::new(stable_id("state", fingerprint))])
        .unwrap_or_default()
}

fn registration_digest(spec: &EpisodeSpecV1) -> Result<String, EpisodeError> {
    let bytes = serde_json::to_vec(spec)?;
    let mut digest = Sha256::new();
    digest.update(b"tsukumo:episode-registration:v1");
    digest.update([0]);
    digest.update(bytes);
    Ok(format!("{:x}", digest.finalize()))
}

fn verify_registration(
    spec: &EpisodeSpecV1,
    checkpoint: &HandoffCheckpoint,
) -> Result<(), EpisodeError> {
    let expected = registration_digest(spec)?;
    if checkpoint.registration_digest.as_deref() != Some(expected.as_str()) {
        return Err(EpisodeError::RegistrationMismatch);
    }
    Ok(())
}

fn map_host_error(error: HostError) -> EpisodeError {
    match error {
        HostError::ExecutionStartTooEarly { not_before_unix_ms } => {
            EpisodeError::ResumeTooEarly { not_before_unix_ms }
        }
        HostError::ExecutionStartWindowClosed { not_after_unix_ms } => {
            EpisodeError::ResumeWindowClosed { not_after_unix_ms }
        }
        other => EpisodeError::Host(other),
    }
}

fn spec_fingerprint(spec: &EpisodeSpecV1) -> Result<String, EpisodeError> {
    // C1 and C2 share one migration data plane; condition changes visibility only.
    let mut migration_input = spec.clone();
    migration_input.condition = EpisodeCondition::C1;
    migration_input.source_runtime.execution_profile = match migration_input.source_runtime.kind {
        EpisodeRuntimeKind::ClaudeCli => EpisodeExecutionProfile::ClaudeDenyUnapproved,
        EpisodeRuntimeKind::CodexCli => EpisodeExecutionProfile::CodexReadOnly,
    };
    migration_input.target_runtime.execution_profile = match migration_input.target_runtime.kind {
        EpisodeRuntimeKind::ClaudeCli => EpisodeExecutionProfile::ClaudeDenyUnapproved,
        EpisodeRuntimeKind::CodexCli => EpisodeExecutionProfile::CodexReadOnly,
    };
    let bytes = serde_json::to_vec(&migration_input)?;
    let mut digest = Sha256::new();
    digest.update(bytes);
    Ok(format!("{:x}", digest.finalize()))
}

fn stable_id(prefix: &str, fingerprint: &str) -> String {
    let mut digest = Sha256::new();
    digest.update(prefix.as_bytes());
    digest.update([0]);
    digest.update(fingerprint.as_bytes());
    let value = format!("{:x}", digest.finalize());
    format!("{prefix}-{}", &value[..32])
}

fn base_event(
    event_id: String,
    timestamp: Timestamp,
    spec: &EpisodeSpecV1,
    runtime: Option<RuntimeBinding>,
    payload: KernelEventPayload,
) -> KernelEvent {
    KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new(event_id),
        occurred_at: timestamp,
        quest_id: spec.quest_id.clone(),
        session_id: spec.source_session_id.clone(),
        spirit_id: spec.spirit_id.clone(),
        execution_id: None,
        runtime,
        causation_id: None,
        correlation_id: None,
        payload,
    }
}

fn resume_window(delay: EpisodeDelayV1, seeded_at: Timestamp) -> Result<(i64, i64), EpisodeError> {
    let hour_ms = 60_i64 * 60 * 1_000;
    let minimum = i64::try_from(delay.minimum_hours)
        .ok()
        .and_then(|hours| hours.checked_mul(hour_ms))
        .and_then(|offset| seeded_at.as_unix_millis().checked_add(offset))
        .ok_or(EpisodeError::InvalidSpec("delay"))?;
    let maximum = i64::try_from(delay.maximum_hours)
        .ok()
        .and_then(|hours| hours.checked_mul(hour_ms))
        .and_then(|offset| seeded_at.as_unix_millis().checked_add(offset))
        .ok_or(EpisodeError::InvalidSpec("delay"))?;
    Ok((minimum, maximum))
}

fn enforce_resume_window(
    delay: EpisodeDelayV1,
    seeded_at: Timestamp,
    now: Timestamp,
) -> Result<(), EpisodeError> {
    let (not_before, not_after) = resume_window(delay, seeded_at)?;
    if now.as_unix_millis() < not_before {
        return Err(EpisodeError::ResumeTooEarly {
            not_before_unix_ms: not_before,
        });
    }
    if now.as_unix_millis() > not_after {
        return Err(EpisodeError::ResumeWindowClosed {
            not_after_unix_ms: not_after,
        });
    }
    Ok(())
}

fn database_size(store: &SoulStore) -> Option<u64> {
    fs::metadata(store.database_path())
        .ok()
        .map(|metadata| metadata.len())
}

fn storage_delta(before: Option<u64>, after: Option<u64>) -> Option<i64> {
    let before = i128::from(before?);
    let after = i128::from(after?);
    i64::try_from(after - before).ok()
}

fn elapsed_millis(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX)
}

fn failure_label(failure: ExecutionFailure) -> &'static str {
    match failure {
        ExecutionFailure::LaunchFailed => "launch_failed",
        ExecutionFailure::Cancelled => "cancelled",
        ExecutionFailure::TimedOut => "timed_out",
        ExecutionFailure::MalformedOutput => "malformed_output",
        ExecutionFailure::TruncatedStream => "truncated_stream",
        ExecutionFailure::NonZeroExit => "non_zero_exit",
        ExecutionFailure::ProcessFailure => "process_failure",
        ExecutionFailure::SafetyUnsupported => "safety_unsupported",
        ExecutionFailure::VendorFailure => "vendor_failure",
    }
}
