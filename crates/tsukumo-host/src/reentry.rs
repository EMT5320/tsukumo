//! Read-only reconciliation preflight for reviewed episode state.

use crate::episode::{validate_spec, EpisodeExecutionProfile, EpisodeRuntimeKind, EpisodeSpecV1};
use crate::{RuntimeProbe, StandardRuntimeProbe};
use serde::Serialize;
use std::fs;
use std::path::Path;
use std::process::{Command, Stdio};
use thiserror::Error;
use tsukumo_adapters::{
    ClaudeRuntimeProfile, CodexRuntimeProfile, RuntimeLaunchConfig, RuntimeProfile,
};

const INSPECT_SUMMARY_SCHEMA_VERSION: u16 = 1;
const MAX_GIT_OUTPUT_BYTES: usize = 65_536;

/// Bounded reconciliation state shown before any projection is prepared.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ReentryFindingStatus {
    StillCurrent,
    Completed,
    Drifted,
    Blocked,
    Unknown,
}

/// Current Git state for one reviewed relative artifact path.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ArtifactWorkspaceState {
    Missing,
    TrackedClean,
    TrackedModified,
    Untracked,
    GitUnavailable,
}

/// One sanitized finding with a stable machine-readable reason.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ReentryFindingV1 {
    pub finding_id: String,
    pub status: ReentryFindingStatus,
    pub reason: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_state: Option<ArtifactWorkspaceState>,
}

impl ReentryFindingV1 {
    fn new(
        finding_id: impl Into<String>,
        status: ReentryFindingStatus,
        reason: &'static str,
    ) -> Self {
        Self {
            finding_id: finding_id.into(),
            status,
            reason,
            artifact_state: None,
        }
    }

    fn with_artifact_state(mut self, state: ArtifactWorkspaceState) -> Self {
        self.artifact_state = Some(state);
        self
    }
}

/// Read-only report emitted before seed or resume can mutate Tsukumo state.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct EpisodeInspectSummaryV1 {
    pub schema_version: u16,
    pub episode_id: String,
    pub overall_status: ReentryFindingStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reviewed_git_head: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub git_head: Option<String>,
    pub workspace_dirty: Option<bool>,
    pub expected_runtime: EpisodeRuntimeKind,
    pub expected_runtime_version: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed_runtime_version: Option<String>,
    pub findings: Vec<ReentryFindingV1>,
    pub semantic_review_required: bool,
    pub mutation_performed: bool,
}

/// Sanitized inspection failures that do not expose local absolute paths.
#[derive(Debug, Error)]
pub enum EpisodeInspectError {
    #[error(transparent)]
    Episode(#[from] crate::EpisodeError),
    #[error("working directory is missing, inaccessible, or not a directory")]
    InvalidWorkingDirectory,
}

/// Inspects reviewed state using the production prompt-free runtime probe.
pub fn inspect_episode(
    spec: &EpisodeSpecV1,
    runtime_executable: impl AsRef<Path>,
    working_dir: impl AsRef<Path>,
) -> Result<EpisodeInspectSummaryV1, EpisodeInspectError> {
    inspect_episode_with_probe(spec, runtime_executable, working_dir, &StandardRuntimeProbe)
}

/// Testable read-only inspection boundary.
pub fn inspect_episode_with_probe(
    spec: &EpisodeSpecV1,
    runtime_executable: impl AsRef<Path>,
    working_dir: impl AsRef<Path>,
    probe: &dyn RuntimeProbe,
) -> Result<EpisodeInspectSummaryV1, EpisodeInspectError> {
    validate_spec(spec)?;
    let working_dir =
        fs::canonicalize(working_dir).map_err(|_| EpisodeInspectError::InvalidWorkingDirectory)?;
    if !working_dir.is_dir() {
        return Err(EpisodeInspectError::InvalidWorkingDirectory);
    }

    let git = inspect_git_workspace(&working_dir);
    let mut findings = Vec::new();
    findings.push(
        match (
            spec.reviewed_git_head.as_ref(),
            git.head.as_ref(),
            git.dirty,
        ) {
            (Some(reviewed), Some(observed), Some(false))
                if reviewed.eq_ignore_ascii_case(observed) =>
            {
                ReentryFindingV1::new(
                    "workspace:git",
                    ReentryFindingStatus::StillCurrent,
                    "git_head_matches_reviewed_head_and_workspace_is_clean",
                )
            }
            (Some(reviewed), Some(observed), _) if !reviewed.eq_ignore_ascii_case(observed) => {
                ReentryFindingV1::new(
                    "workspace:git",
                    ReentryFindingStatus::Drifted,
                    "git_head_differs_from_reviewed_head",
                )
            }
            (Some(_), Some(_), Some(true)) => ReentryFindingV1::new(
                "workspace:git",
                ReentryFindingStatus::Unknown,
                "git_head_matches_but_workspace_changed_requires_review",
            ),
            (None, Some(_), _) => ReentryFindingV1::new(
                "workspace:git",
                ReentryFindingStatus::Unknown,
                "reviewed_git_head_not_recorded",
            ),
            _ => ReentryFindingV1::new(
                "workspace:git",
                ReentryFindingStatus::Unknown,
                "git_workspace_unavailable",
            ),
        },
    );

    for artifact in &spec.checkpoint.artifacts {
        let relative = Path::new(artifact.location.as_str());
        let state = inspect_artifact(&working_dir, relative, git.available);
        let (status, reason) = match state {
            ArtifactWorkspaceState::Missing => (ReentryFindingStatus::Drifted, "artifact_missing"),
            ArtifactWorkspaceState::TrackedClean
                if git_heads_match(spec.reviewed_git_head.as_deref(), git.head.as_deref()) =>
            {
                (
                    ReentryFindingStatus::StillCurrent,
                    "artifact_exists_at_reviewed_git_head_and_is_tracked_clean",
                )
            }
            ArtifactWorkspaceState::TrackedClean => (
                ReentryFindingStatus::Unknown,
                "artifact_exists_but_reviewed_git_head_does_not_match",
            ),
            ArtifactWorkspaceState::TrackedModified | ArtifactWorkspaceState::Untracked => (
                ReentryFindingStatus::Unknown,
                "artifact_content_changed_requires_review",
            ),
            ArtifactWorkspaceState::GitUnavailable => (
                ReentryFindingStatus::Unknown,
                "artifact_exists_but_git_state_is_unavailable",
            ),
        };
        findings.push(
            ReentryFindingV1::new(format!("artifact:{}", artifact.artifact_id), status, reason)
                .with_artifact_state(state),
        );
    }

    for (index, progress) in spec.checkpoint.progress.iter().enumerate() {
        let (status, reason) = match progress.status {
            tsukumo_soul::ProgressStatus::Completed => (
                ReentryFindingStatus::Completed,
                "checkpoint_claims_completed_not_revalidated",
            ),
            tsukumo_soul::ProgressStatus::Planned => (
                ReentryFindingStatus::Unknown,
                "checkpoint_claims_planned_not_revalidated",
            ),
            tsukumo_soul::ProgressStatus::InProgress => (
                ReentryFindingStatus::Unknown,
                "checkpoint_claims_in_progress_not_revalidated",
            ),
            tsukumo_soul::ProgressStatus::Blocked => (
                ReentryFindingStatus::Unknown,
                "checkpoint_claims_blocked_not_revalidated",
            ),
        };
        findings.push(ReentryFindingV1::new(
            format!("progress:{index}"),
            status,
            reason,
        ));
    }
    for open_loop in &spec.checkpoint.open_loops {
        findings.push(ReentryFindingV1::new(
            format!("open_loop:{}", open_loop.id),
            ReentryFindingStatus::Unknown,
            "open_loop_requires_current_review",
        ));
    }
    for (index, _) in spec.checkpoint.next_actions.iter().enumerate() {
        findings.push(ReentryFindingV1::new(
            format!("next_action:{index}"),
            ReentryFindingStatus::Unknown,
            "next_action_requires_current_review",
        ));
    }

    let profile = profile_for(spec.target_runtime.execution_profile);
    let launch = RuntimeLaunchConfig::new(
        runtime_executable.as_ref().to_path_buf(),
        working_dir.clone(),
    );
    let (runtime_finding, observed_runtime_version) = match probe.probe(profile.as_ref(), &launch) {
        Ok(observed)
            if observed.binding == profile.binding()
                && observed.version == spec.target_runtime.version =>
        {
            (
                ReentryFindingV1::new(
                    "runtime:target",
                    ReentryFindingStatus::StillCurrent,
                    "runtime_identity_matches_reviewed_target",
                ),
                Some(observed.version),
            )
        }
        Ok(observed) => (
            ReentryFindingV1::new(
                "runtime:target",
                ReentryFindingStatus::Drifted,
                "runtime_identity_differs_from_reviewed_target",
            ),
            Some(observed.version),
        ),
        Err(_) => (
            ReentryFindingV1::new(
                "runtime:target",
                ReentryFindingStatus::Blocked,
                "runtime_identity_probe_failed",
            ),
            None,
        ),
    };
    findings.push(runtime_finding);

    let overall_status = summarize_findings(&findings);
    let semantic_review_required = spec.explicit_state_input.is_some()
        || !spec.checkpoint.progress.is_empty()
        || !spec.checkpoint.decisions.is_empty()
        || !spec.checkpoint.open_loops.is_empty()
        || !spec.checkpoint.next_actions.is_empty()
        || !spec.checkpoint.constraint_state_ids.is_empty();

    Ok(EpisodeInspectSummaryV1 {
        schema_version: INSPECT_SUMMARY_SCHEMA_VERSION,
        episode_id: spec.episode_id.clone(),
        overall_status,
        reviewed_git_head: spec.reviewed_git_head.clone(),
        git_head: git.head,
        workspace_dirty: git.dirty,
        expected_runtime: spec.target_runtime.kind,
        expected_runtime_version: spec.target_runtime.version.clone(),
        observed_runtime_version,
        findings,
        semantic_review_required,
        mutation_performed: false,
    })
}

fn summarize_findings(findings: &[ReentryFindingV1]) -> ReentryFindingStatus {
    for status in [
        ReentryFindingStatus::Blocked,
        ReentryFindingStatus::Drifted,
        ReentryFindingStatus::Unknown,
    ] {
        if findings.iter().any(|finding| finding.status == status) {
            return status;
        }
    }
    if findings.is_empty() {
        ReentryFindingStatus::Unknown
    } else if findings
        .iter()
        .all(|finding| finding.status == ReentryFindingStatus::Completed)
    {
        ReentryFindingStatus::Completed
    } else {
        ReentryFindingStatus::StillCurrent
    }
}

fn profile_for(profile: EpisodeExecutionProfile) -> Box<dyn RuntimeProfile> {
    match profile {
        EpisodeExecutionProfile::ClaudeDenyUnapproved => {
            Box::new(ClaudeRuntimeProfile::deny_unapproved())
        }
        EpisodeExecutionProfile::CodexReadOnly => Box::new(CodexRuntimeProfile::read_only()),
        EpisodeExecutionProfile::CodexWorkspaceWrite => {
            Box::new(CodexRuntimeProfile::workspace_write())
        }
    }
}

struct GitWorkspace {
    available: bool,
    head: Option<String>,
    dirty: Option<bool>,
}

fn inspect_git_workspace(working_dir: &Path) -> GitWorkspace {
    let head = git_output(working_dir, &["rev-parse", "--verify", "HEAD"]).filter(|value| {
        matches!(value.len(), 40 | 64)
            && value.chars().all(|character| character.is_ascii_hexdigit())
    });
    let status = git_output(
        working_dir,
        &["status", "--porcelain=v1", "--untracked-files=normal"],
    );
    GitWorkspace {
        available: head.is_some() && status.is_some(),
        head,
        dirty: status.map(|value| !value.is_empty()),
    }
}

fn git_heads_match(reviewed: Option<&str>, observed: Option<&str>) -> bool {
    matches!(
        (reviewed, observed),
        (Some(reviewed), Some(observed)) if reviewed.eq_ignore_ascii_case(observed)
    )
}

fn inspect_artifact(
    working_dir: &Path,
    relative: &Path,
    git_available: bool,
) -> ArtifactWorkspaceState {
    if !working_dir.join(relative).exists() {
        return ArtifactWorkspaceState::Missing;
    }
    if !git_available {
        return ArtifactWorkspaceState::GitUnavailable;
    }
    let tracked = git_success(
        working_dir,
        &[
            "ls-files",
            "--error-unmatch",
            "--",
            &relative.to_string_lossy(),
        ],
    );
    let status = git_output(
        working_dir,
        &[
            "status",
            "--porcelain=v1",
            "--",
            &relative.to_string_lossy(),
        ],
    );
    match (tracked, status) {
        (Some(true), Some(value)) if value.is_empty() => ArtifactWorkspaceState::TrackedClean,
        (Some(true), Some(_)) => ArtifactWorkspaceState::TrackedModified,
        (Some(false), Some(_)) => ArtifactWorkspaceState::Untracked,
        _ => ArtifactWorkspaceState::GitUnavailable,
    }
}

fn git_output(working_dir: &Path, arguments: &[&str]) -> Option<String> {
    let output = Command::new("git")
        .args(arguments)
        .current_dir(working_dir)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .stderr(Stdio::null())
        .output()
        .ok()?;
    if !output.status.success() || output.stdout.len() > MAX_GIT_OUTPUT_BYTES {
        return None;
    }
    String::from_utf8(output.stdout)
        .ok()
        .map(|value| value.trim().to_owned())
}

fn git_success(working_dir: &Path, arguments: &[&str]) -> Option<bool> {
    Command::new("git")
        .args(arguments)
        .current_dir(working_dir)
        .env("GIT_OPTIONAL_LOCKS", "0")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .status()
        .ok()
        .map(|status| status.success())
}
