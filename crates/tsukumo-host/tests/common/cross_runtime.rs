//! Deterministic source-state, projection-pair, and Rust repository fixtures.

use sha2::{Digest, Sha256};
use std::borrow::Cow;
use std::fs;
use tempfile::{tempdir, TempDir};
use tsukumo_kernel::{
    CheckpointId, CorrelationId, EventId, ExecutionId, KernelEvent, KernelEventPayload,
    PersistedText, ProjectionId, QuestId, RuntimeBinding, RuntimeKind, RuntimeMode, SensitiveText,
    SessionId, SpiritId, StateId, StateLifecycleAction, Timestamp, KERNEL_EVENT_SCHEMA_VERSION,
};
use tsukumo_soul::{
    compare_projection_receipts, CheckpointTrigger, CheckpointWriteRequest, ExtractionContext,
    HandoffCheckpoint, OperatingSystem, PreparedProjection, ProjectionComparison,
    ProjectionRequest, ProjectionTarget, ProjectionWriteRequest, RuleStateExtractor, SoulStore,
    StateExtractor, StateRecord, StateRef, StateScope, StateTransition, StateWriteOutcome,
    StateWriteRequest,
};

const FIXTURE_MANIFEST: &str = include_str!("../fixtures/cross_runtime_rust/Cargo.toml");
const FIXTURE_SOURCE: &str = include_str!("../fixtures/cross_runtime_rust/src/lib.rs");

pub struct CrossRuntimePrepared {
    pub directory: TempDir,
    pub source_event: KernelEvent,
    pub state: StateRecord,
    pub checkpoint: HandoffCheckpoint,
    pub with_state: PreparedProjection,
    pub without_state: PreparedProjection,
    pub comparison: ProjectionComparison,
}

pub fn prepared_cross_runtime_comparison() -> CrossRuntimePrepared {
    let directory = tempdir().expect("create comparison store directory");
    let mut store = SoulStore::open(directory.path()).expect("open comparison store");
    let scope = StateScope::workspace_os("cross-runtime-fixture", OperatingSystem::Windows);
    let source_event = claude_source_event();
    let state = persist_constraint(&mut store, &source_event, scope.clone());
    let checkpoint = checkpoint(&mut store, &source_event, &state);
    let with_state = prepare_projection(
        &mut store,
        &checkpoint,
        scope.clone(),
        "projection-codex-with",
        "execution-codex-with",
        None,
        Timestamp::from_unix_millis(500),
    );
    let without_state = prepare_projection(
        &mut store,
        &checkpoint,
        scope,
        "projection-codex-without",
        "execution-codex-without",
        Some(state.state_id.clone()),
        Timestamp::from_unix_millis(500),
    );
    let comparison =
        compare_projection_receipts(&with_state.receipt, &without_state.receipt, &state.state_id)
            .expect("projection pair satisfies controlled invariants");
    drop(store);
    CrossRuntimePrepared {
        directory,
        source_event,
        state,
        checkpoint,
        with_state,
        without_state,
        comparison,
    }
}

pub fn prepare_post_revoke_projection(
    store: &mut SoulStore,
    prepared: &CrossRuntimePrepared,
    created_at: Timestamp,
) -> PreparedProjection {
    prepare_projection(
        store,
        &prepared.checkpoint,
        StateScope::workspace_os("cross-runtime-fixture", OperatingSystem::Windows),
        "projection-codex-after-revoke",
        "execution-codex-after-revoke",
        None,
        created_at,
    )
}

pub fn materialize_cross_runtime_repository() -> (TempDir, String) {
    let directory = tempdir().expect("create Rust fixture directory");
    fs::create_dir(directory.path().join("src")).expect("create fixture source directory");
    let files = fixture_files();
    for &(path, contents) in &files {
        fs::write(
            directory.path().join(path),
            canonical_fixture_text(contents).as_bytes(),
        )
        .expect("write reviewed Rust fixture");
    }
    let digest = canonical_repository_fixture_digest(&files);
    (directory, digest)
}

pub fn canonical_repository_fixture_digest(files: &[(&str, &str)]) -> String {
    let mut digest = Sha256::new();
    for &(path, contents) in files {
        digest.update(path.as_bytes());
        digest.update([0]);
        digest.update(canonical_fixture_text(contents).as_bytes());
        digest.update([0]);
    }
    format!("{:x}", digest.finalize())
}

pub fn canonical_text_sha256(value: &str) -> String {
    format!(
        "{:x}",
        Sha256::digest(canonical_fixture_text(value).as_bytes())
    )
}

fn canonical_fixture_text(contents: &str) -> Cow<'_, str> {
    if contents.contains("\r\n") {
        Cow::Owned(contents.replace("\r\n", "\n"))
    } else {
        Cow::Borrowed(contents)
    }
}

fn claude_source_event() -> KernelEvent {
    KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new("event-claude-gnu-source"),
        occurred_at: Timestamp::from_unix_millis(100),
        quest_id: QuestId::new("quest-cross-runtime"),
        session_id: SessionId::new("session-cross-runtime"),
        spirit_id: SpiritId::new("yuka"),
        execution_id: Some(ExecutionId::new("execution-claude-source")),
        runtime: Some(RuntimeBinding::new(
            RuntimeKind::ClaudeCli,
            RuntimeMode::Fixture,
        )),
        causation_id: None,
        correlation_id: Some(CorrelationId::new("correlation-claude-source")),
        payload: KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed(
                "Tsukumo always uses the GNU Rust toolchain on Windows",
            ),
        },
    }
}

fn persist_constraint(
    store: &mut SoulStore,
    source_event: &KernelEvent,
    scope: StateScope,
) -> StateRecord {
    let draft = RuleStateExtractor
        .extract(&ExtractionContext {
            event: source_event,
            scope,
        })
        .expect("extract explicit GNU constraint")
        .into_iter()
        .next()
        .expect("GNU rule emits one state draft");
    let state_id = StateId::new("state-cross-runtime-gnu");
    let lifecycle = KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new("event-state-cross-runtime-gnu"),
        occurred_at: Timestamp::from_unix_millis(101),
        quest_id: source_event.quest_id.clone(),
        session_id: source_event.session_id.clone(),
        spirit_id: source_event.spirit_id.clone(),
        execution_id: None,
        runtime: None,
        causation_id: Some(source_event.event_id.clone()),
        correlation_id: None,
        payload: KernelEventPayload::StateLifecycle {
            state_id: state_id.clone(),
            action: StateLifecycleAction::Created,
            prior_state_id: None,
            reason: None,
        },
    };
    match store
        .apply_state(
            StateWriteRequest::new(
                StateTransition::Create {
                    state_id,
                    draft,
                    created_at: Timestamp::from_unix_millis(101),
                },
                lifecycle,
            )
            .with_source_event(source_event.clone()),
        )
        .expect("persist GNU constraint")
    {
        StateWriteOutcome::Created(record) | StateWriteOutcome::Unchanged(record) => record,
        StateWriteOutcome::Superseded(_) | StateWriteOutcome::Revoked(_) => {
            panic!("fresh comparison fixture must create an active state")
        }
    }
}

fn checkpoint(
    store: &mut SoulStore,
    source_event: &KernelEvent,
    state: &StateRecord,
) -> HandoffCheckpoint {
    let checkpoint = HandoffCheckpoint::new(
        CheckpointId::new("checkpoint-cross-runtime"),
        source_event.quest_id.clone(),
        1,
        None,
        PersistedText::from_reviewed("Validate the deterministic Rust fixture"),
        Timestamp::from_unix_millis(200),
        CheckpointTrigger::RuntimeSwitch,
    )
    .with_constraint_refs(vec![StateRef::new(state.state_id.clone(), state.version)])
    .with_source_event_refs(vec![source_event.event_id.clone()]);
    let event = KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new("event-checkpoint-cross-runtime"),
        occurred_at: checkpoint.created_at,
        quest_id: checkpoint.quest_id.clone(),
        session_id: source_event.session_id.clone(),
        spirit_id: source_event.spirit_id.clone(),
        execution_id: None,
        runtime: None,
        causation_id: None,
        correlation_id: None,
        payload: KernelEventPayload::CheckpointCreated {
            checkpoint_id: checkpoint.id.clone(),
            version: checkpoint.version,
        },
    };
    store
        .save_checkpoint(CheckpointWriteRequest::new(checkpoint.clone(), event))
        .expect("save cross-runtime checkpoint");
    checkpoint
}

fn prepare_projection(
    store: &mut SoulStore,
    checkpoint: &HandoffCheckpoint,
    scope: StateScope,
    projection_id: &str,
    execution_id: &str,
    excluded_state: Option<StateId>,
    created_at: Timestamp,
) -> PreparedProjection {
    let runtime = RuntimeBinding::new(RuntimeKind::CodexCli, RuntimeMode::OwnedProcess);
    let target = ProjectionTarget::new(
        ProjectionId::new(projection_id),
        ExecutionId::new(execution_id),
        runtime.clone(),
        checkpoint.id.clone(),
    );
    let mut request = ProjectionRequest::new(
        target,
        scope,
        SensitiveText::new("Run the appropriate offline test command for this crate"),
        created_at,
        2_000,
    );
    if let Some(state_id) = excluded_state {
        request = request.excluding_states(vec![state_id]);
    }
    let event = KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new(format!("event-{projection_id}")),
        occurred_at: request.created_at,
        quest_id: checkpoint.quest_id.clone(),
        session_id: SessionId::new("session-cross-runtime"),
        spirit_id: SpiritId::new("yuka"),
        execution_id: Some(request.execution_id.clone()),
        runtime: Some(runtime),
        causation_id: None,
        correlation_id: Some(CorrelationId::new(format!("correlation-{projection_id}"))),
        payload: KernelEventPayload::ProjectionCreated {
            projection_id: request.projection_id.clone(),
            checkpoint_id: checkpoint.id.clone(),
        },
    };
    store
        .prepare_projection(ProjectionWriteRequest::new(request, event))
        .expect("prepare Codex comparison projection")
}

fn fixture_files() -> [(&'static str, &'static str); 2] {
    [
        ("Cargo.toml", FIXTURE_MANIFEST),
        ("src/lib.rs", FIXTURE_SOURCE),
    ]
}
