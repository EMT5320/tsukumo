//! Read-only re-entry contracts without external model calls.

use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;
use tsukumo_adapters::{RuntimeLaunchConfig, RuntimeProfile};
use tsukumo_host::{
    inspect_episode_with_probe, ArtifactWorkspaceState, EpisodeCheckpointV1, EpisodeCondition,
    EpisodeDelayV1, EpisodeError, EpisodeExecutionProfile, EpisodeInspectError,
    EpisodeProjectionV1, EpisodeRuntimeKind, EpisodeRuntimeV1, EpisodeSpecV1, ReentryFindingStatus,
    RuntimeIdentity, RuntimeProbe, RuntimeProbeError,
};
use tsukumo_kernel::{ArtifactId, PersistedText, QuestId, SessionId, SpiritId};
use tsukumo_soul::{
    ArtifactReference, OpenLoop, OpenLoopId, OperatingSystem, ProgressItem, ProgressStatus,
    StateScope,
};

struct FixedProbe {
    version: &'static str,
}

impl RuntimeProbe for FixedProbe {
    fn probe(
        &self,
        profile: &dyn RuntimeProfile,
        _launch: &RuntimeLaunchConfig,
    ) -> Result<RuntimeIdentity, RuntimeProbeError> {
        Ok(RuntimeIdentity {
            binding: profile.binding(),
            version: self.version.into(),
        })
    }
}

struct FailingProbe;

impl RuntimeProbe for FailingProbe {
    fn probe(
        &self,
        _profile: &dyn RuntimeProfile,
        _launch: &RuntimeLaunchConfig,
    ) -> Result<RuntimeIdentity, RuntimeProbeError> {
        Err(RuntimeProbeError::NonZeroExit)
    }
}

#[test]
fn matching_runtime_and_clean_artifact_remain_current_without_workspace_mutation() {
    let directory = tempdir().expect("create inspection repository");
    init_repository(directory.path());
    fs::write(directory.path().join("artifact.txt"), "reviewed\n")
        .expect("write reviewed artifact");
    git(directory.path(), &["add", "artifact.txt"]);
    commit(directory.path());
    let mut spec = episode_spec();
    spec.reviewed_git_head =
        Some(git_output(directory.path(), &["rev-parse", "HEAD"]).to_ascii_uppercase());
    spec.checkpoint.artifacts.push(ArtifactReference::new(
        ArtifactId::new("reviewed-artifact"),
        PersistedText::from_reviewed("artifact.txt"),
    ));
    let head_before = git_output(directory.path(), &["rev-parse", "HEAD"]);
    let status_before = git_output(
        directory.path(),
        &["status", "--porcelain=v1", "--untracked-files=all"],
    );

    let summary = inspect_episode_with_probe(
        &spec,
        directory.path().join("private-runtime-location"),
        directory.path(),
        &FixedProbe { version: "0.135.0" },
    )
    .expect("inspect current episode");

    assert_eq!(summary.overall_status, ReentryFindingStatus::StillCurrent);
    assert_eq!(summary.git_head.as_deref(), Some(head_before.as_str()));
    assert_eq!(summary.workspace_dirty, Some(false));
    assert!(!summary.semantic_review_required);
    assert!(!summary.mutation_performed);
    let artifact = finding(&summary, "artifact:reviewed-artifact");
    assert_eq!(artifact.status, ReentryFindingStatus::StillCurrent);
    assert_eq!(
        artifact.artifact_state,
        Some(ArtifactWorkspaceState::TrackedClean)
    );
    assert_eq!(
        git_output(directory.path(), &["rev-parse", "HEAD"]),
        head_before
    );
    assert_eq!(
        git_output(
            directory.path(),
            &["status", "--porcelain=v1", "--untracked-files=all"],
        ),
        status_before
    );
    assert_eq!(
        fs::read_to_string(directory.path().join("artifact.txt"))
            .expect("read artifact after inspection"),
        "reviewed\n"
    );
}

#[test]
fn clean_workspace_without_reviewed_git_head_remains_unknown() {
    let directory = tempdir().expect("create no-baseline inspection repository");
    init_repository(directory.path());
    fs::write(directory.path().join("tracked.txt"), "baseline\n").expect("write baseline artifact");
    git(directory.path(), &["add", "tracked.txt"]);
    commit(directory.path());

    let summary = inspect_episode_with_probe(
        &episode_spec(),
        Path::new("codex"),
        directory.path(),
        &FixedProbe { version: "0.135.0" },
    )
    .expect("inspect episode without Git baseline");

    assert_eq!(summary.overall_status, ReentryFindingStatus::Unknown);
    assert_eq!(summary.reviewed_git_head, None);
    let workspace = finding(&summary, "workspace:git");
    assert_eq!(workspace.status, ReentryFindingStatus::Unknown);
    assert_eq!(workspace.reason, "reviewed_git_head_not_recorded");
}

#[test]
fn malformed_reviewed_git_head_fails_before_observation() {
    let directory = tempdir().expect("create invalid-baseline directory");
    let mut spec = episode_spec();
    spec.reviewed_git_head = Some("not-a-git-object-id".into());

    let error = inspect_episode_with_probe(
        &spec,
        Path::new("codex"),
        directory.path(),
        &FixedProbe { version: "0.135.0" },
    )
    .expect_err("invalid reviewed Git HEAD must fail");

    assert!(matches!(
        error,
        EpisodeInspectError::Episode(EpisodeError::InvalidSpec("reviewed_git_head"))
    ));
}

#[test]
fn changed_git_head_is_drifted_even_when_current_workspace_is_clean() {
    let directory = tempdir().expect("create changed-head inspection repository");
    init_repository(directory.path());
    fs::write(directory.path().join("tracked.txt"), "baseline\n").expect("write baseline artifact");
    git(directory.path(), &["add", "tracked.txt"]);
    commit(directory.path());
    let mut spec = episode_spec();
    spec.reviewed_git_head = Some(git_output(directory.path(), &["rev-parse", "HEAD"]));
    fs::write(directory.path().join("later.txt"), "later\n").expect("write later artifact");
    git(directory.path(), &["add", "later.txt"]);
    commit(directory.path());

    let summary = inspect_episode_with_probe(
        &spec,
        Path::new("codex"),
        directory.path(),
        &FixedProbe { version: "0.135.0" },
    )
    .expect("inspect changed Git HEAD");

    assert_eq!(summary.overall_status, ReentryFindingStatus::Drifted);
    let workspace = finding(&summary, "workspace:git");
    assert_eq!(workspace.status, ReentryFindingStatus::Drifted);
    assert_eq!(workspace.reason, "git_head_differs_from_reviewed_head");
}

#[test]
fn stale_runtime_missing_artifact_and_open_loop_are_classified_without_path_leaks() {
    let directory = tempdir().expect("create stale inspection repository");
    init_repository(directory.path());
    fs::write(directory.path().join("tracked.txt"), "baseline\n").expect("write baseline artifact");
    git(directory.path(), &["add", "tracked.txt"]);
    commit(directory.path());
    let runtime_path = directory.path().join("private-runtime-location");
    let mut spec = episode_spec();
    spec.reviewed_git_head = Some(git_output(directory.path(), &["rev-parse", "HEAD"]));
    spec.checkpoint.progress.push(ProgressItem::new(
        PersistedText::from_reviewed("Finished the reviewed implementation slice"),
        ProgressStatus::Completed,
    ));
    spec.checkpoint.artifacts.push(ArtifactReference::new(
        ArtifactId::new("missing-artifact"),
        PersistedText::from_reviewed("missing.txt"),
    ));
    spec.checkpoint.open_loops.push(OpenLoop::new(
        OpenLoopId::new("review-current-direction"),
        PersistedText::from_reviewed("Revalidate the project direction before resuming"),
    ));

    let summary = inspect_episode_with_probe(
        &spec,
        &runtime_path,
        directory.path(),
        &FixedProbe { version: "0.999.0" },
    )
    .expect("inspect stale episode");

    assert_eq!(summary.overall_status, ReentryFindingStatus::Drifted);
    assert_eq!(
        finding(&summary, "runtime:target").status,
        ReentryFindingStatus::Drifted
    );
    let artifact = finding(&summary, "artifact:missing-artifact");
    assert_eq!(artifact.status, ReentryFindingStatus::Drifted);
    assert_eq!(
        artifact.artifact_state,
        Some(ArtifactWorkspaceState::Missing)
    );
    assert_eq!(
        finding(&summary, "progress:0").status,
        ReentryFindingStatus::Completed
    );
    assert_eq!(
        finding(&summary, "open_loop:review-current-direction").status,
        ReentryFindingStatus::Unknown
    );
    assert!(summary.semantic_review_required);
    assert!(!summary.mutation_performed);

    let json = serde_json::to_string(&summary).expect("serialize inspection summary");
    assert!(!json.contains(&directory.path().to_string_lossy().into_owned()));
    assert!(!json.contains(&runtime_path.to_string_lossy().into_owned()));
}

#[test]
fn failed_runtime_probe_is_blocked_and_keeps_local_paths_sanitized() {
    let directory = tempdir().expect("create blocked inspection repository");
    init_repository(directory.path());
    fs::write(directory.path().join("tracked.txt"), "baseline\n").expect("write baseline artifact");
    git(directory.path(), &["add", "tracked.txt"]);
    commit(directory.path());
    let runtime_path = directory.path().join("missing-private-runtime");
    let mut spec = episode_spec();
    spec.reviewed_git_head = Some(git_output(directory.path(), &["rev-parse", "HEAD"]));

    let summary = inspect_episode_with_probe(&spec, &runtime_path, directory.path(), &FailingProbe)
        .expect("probe failure is a report finding");

    assert_eq!(summary.overall_status, ReentryFindingStatus::Blocked);
    assert_eq!(
        finding(&summary, "runtime:target").status,
        ReentryFindingStatus::Blocked
    );
    assert_eq!(summary.observed_runtime_version, None);
    assert!(!summary.mutation_performed);
    let json = serde_json::to_string(&summary).expect("serialize blocked summary");
    assert!(!json.contains(&directory.path().to_string_lossy().into_owned()));
    assert!(!json.contains(&runtime_path.to_string_lossy().into_owned()));
}

fn finding<'a>(
    summary: &'a tsukumo_host::EpisodeInspectSummaryV1,
    finding_id: &str,
) -> &'a tsukumo_host::ReentryFindingV1 {
    summary
        .findings
        .iter()
        .find(|finding| finding.finding_id == finding_id)
        .expect("expected inspection finding")
}

fn init_repository(path: &Path) {
    git(path, &["init", "--quiet"]);
}

fn commit(path: &Path) {
    git(
        path,
        &[
            "-c",
            "user.name=Tsukumo Test",
            "-c",
            "user.email=tsukumo@example.invalid",
            "commit",
            "--quiet",
            "-m",
            "reviewed state",
        ],
    );
}

fn git(path: &Path, arguments: &[&str]) {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(path)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .expect("run Git");
    assert!(
        output.status.success(),
        "Git command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

fn git_output(path: &Path, arguments: &[&str]) -> String {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(path)
        .env("GIT_CONFIG_NOSYSTEM", "1")
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .expect("run Git");
    assert!(
        output.status.success(),
        "Git command failed: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout)
        .expect("Git emits UTF-8")
        .trim()
        .to_owned()
}

fn episode_spec() -> EpisodeSpecV1 {
    EpisodeSpecV1 {
        schema_version: 1,
        episode_id: "episode-reentry-contract".into(),
        condition: EpisodeCondition::C2,
        episode_type: "natural_delayed_resumption".into(),
        workload_block: "reentry_contract".into(),
        fault: "none".into(),
        reviewed_git_head: None,
        quest_id: QuestId::new("quest-reentry-contract"),
        source_session_id: SessionId::new("source-reentry-contract"),
        target_session_id: SessionId::new("target-reentry-contract"),
        spirit_id: SpiritId::new("shiori"),
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
            "A reviewed implementation slice is ready for later resumption",
        ),
        explicit_state_input: None,
        checkpoint: EpisodeCheckpointV1 {
            goal: PersistedText::from_reviewed("Resume the reviewed implementation safely"),
            progress: Vec::new(),
            decisions: Vec::new(),
            artifacts: Vec::new(),
            open_loops: Vec::new(),
            next_actions: Vec::new(),
            constraint_state_ids: Vec::new(),
        },
        projection: EpisodeProjectionV1 {
            scope: StateScope::workspace_os("tsukumo", OperatingSystem::Macos),
            budget_chars: 8_000,
            delegation_goal: "Continue only after reviewing the inspection report".into(),
        },
        delay: EpisodeDelayV1 {
            minimum_hours: 0,
            maximum_hours: 1,
        },
        quality_gate: vec!["Use only retained reproducible evidence".into()],
    }
}
