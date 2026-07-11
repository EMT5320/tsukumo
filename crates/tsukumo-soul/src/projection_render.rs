//! Canonical versioned projection renderer and SHA-256 digest helpers.

use crate::handoff_model::{HandoffCheckpoint, ProgressStatus};
use crate::projection_model::{
    ContentDigest, DigestAlgorithm, ProjectionSection, ProjectionSectionDigest,
};
use crate::state_model::StateRecord;
use sha2::{Digest, Sha256};
use tsukumo_kernel::SensitiveText;

pub(crate) struct RenderedSection {
    pub section: ProjectionSection,
    pub text: String,
}

pub(crate) struct RenderedProjection {
    pub text: String,
    pub sections: Vec<RenderedSection>,
}

pub(crate) fn render_projection(
    checkpoint: &HandoffCheckpoint,
    selected: &[StateRecord],
    delegation_goal: &SensitiveText,
) -> RenderedProjection {
    let sections = vec![
        RenderedSection {
            section: ProjectionSection::Header,
            text: concat!(
                "# Tsukumo handoff v1\n",
                "Precedence: current user instructions and repository rules override this handoff.\n\n",
            )
            .to_owned(),
        },
        scalar_section(
            ProjectionSection::Goal,
            "Goal",
            checkpoint.goal.as_str(),
            true,
        ),
        section(
            ProjectionSection::Progress,
            "Current progress",
            checkpoint
                .progress
                .iter()
                .map(|item| {
                    format!(
                        "[{}] {}",
                        progress_marker(item.status),
                        normalize(item.summary.as_str())
                    )
                })
                .collect(),
            true,
        ),
        section(
            ProjectionSection::Decisions,
            "Decisions",
            checkpoint
                .decisions
                .iter()
                .map(|item| normalize(item.summary.as_str()))
                .collect(),
            true,
        ),
        section(
            ProjectionSection::Constraints,
            "Constraints",
            selected
                .iter()
                .map(|state| {
                    format!(
                        "[state:{}@v{}] {}",
                        state.state_id,
                        state.version,
                        normalize(state.content.as_str())
                    )
                })
                .collect(),
            true,
        ),
        section(
            ProjectionSection::Artifacts,
            "Artifacts",
            checkpoint
                .artifacts
                .iter()
                .map(|item| {
                    format!(
                        "[artifact:{}] {}",
                        item.artifact_id,
                        normalize(item.location.as_str())
                    )
                })
                .collect(),
            true,
        ),
        section(
            ProjectionSection::OpenLoops,
            "Open loops",
            checkpoint
                .open_loops
                .iter()
                .map(|item| format!("[{}] {}", item.id, normalize(item.summary.as_str())))
                .collect(),
            true,
        ),
        section(
            ProjectionSection::NextActions,
            "Next actions",
            checkpoint
                .next_actions
                .iter()
                .map(|item| normalize(item.summary.as_str()))
                .collect(),
            true,
        ),
        scalar_section(
            ProjectionSection::DelegationGoal,
            "Delegation goal",
            delegation_goal.expose(),
            false,
        ),
    ];
    let text = sections
        .iter()
        .map(|section| section.text.as_str())
        .collect::<String>();
    RenderedProjection { text, sections }
}

pub(crate) fn digest_text(value: &str) -> ContentDigest {
    let bytes = Sha256::digest(value.as_bytes());
    ContentDigest {
        algorithm: DigestAlgorithm::Sha256,
        value: format!("{bytes:x}"),
    }
}

pub(crate) fn section_digests(sections: &[RenderedSection]) -> Vec<ProjectionSectionDigest> {
    sections
        .iter()
        .map(|section| ProjectionSectionDigest {
            section: section.section,
            digest: digest_text(&section.text),
            byte_count: section.text.len(),
            char_count: section.text.chars().count(),
        })
        .collect()
}

fn scalar_section(
    identity: ProjectionSection,
    title: &str,
    value: &str,
    trailing_blank: bool,
) -> RenderedSection {
    let mut text = format!("## {title}\n{}\n", normalize(value));
    if trailing_blank {
        text.push('\n');
    }
    RenderedSection {
        section: identity,
        text,
    }
}
fn section(
    identity: ProjectionSection,
    title: &str,
    entries: Vec<String>,
    trailing_blank: bool,
) -> RenderedSection {
    let mut text = format!("## {title}\n");
    if entries.is_empty() {
        text.push_str("- (none)\n");
    } else {
        for entry in entries {
            text.push_str("- ");
            text.push_str(&entry.replace('\n', "\n  "));
            text.push('\n');
        }
    }
    if trailing_blank {
        text.push('\n');
    }
    RenderedSection {
        section: identity,
        text,
    }
}

fn normalize(value: &str) -> String {
    value
        .replace("\r\n", "\n")
        .replace('\r', "\n")
        .trim()
        .to_owned()
}

const fn progress_marker(status: ProgressStatus) -> &'static str {
    match status {
        ProgressStatus::Planned => " ",
        ProgressStatus::InProgress => "~",
        ProgressStatus::Completed => "x",
        ProgressStatus::Blocked => "!",
    }
}
