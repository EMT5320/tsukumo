//! Character-budget admission over deterministically ranked projection candidates.

use crate::handoff_model::HandoffCheckpoint;
use crate::projection_error::ProjectionError;
use crate::projection_model::{ProjectionOmission, ProjectionOmissionReason, ProjectionRequest};
use crate::projection_render::{render_projection, RenderedProjection};
use crate::projection_selection::CandidateSelection;
use crate::state_model::StateRecord;

pub(crate) struct AdmittedProjection {
    pub selected: Vec<StateRecord>,
    pub omissions: Vec<ProjectionOmission>,
    pub rendered: RenderedProjection,
}

pub(crate) fn admit_candidates(
    checkpoint: &HandoffCheckpoint,
    request: &ProjectionRequest,
    selection: CandidateSelection,
) -> Result<AdmittedProjection, ProjectionError> {
    let base = render_projection(checkpoint, &[], &request.delegation_goal);
    let base_chars = base.text.chars().count();
    if base_chars > request.budget_chars {
        return Err(ProjectionError::BudgetTooSmall {
            used: base_chars,
            limit: request.budget_chars,
        });
    }

    let mut selected = Vec::new();
    let mut omissions = selection.omissions;
    for candidate in selection.candidates {
        let mut tentative = selected.clone();
        tentative.push(candidate.record.clone());
        let rendered = render_projection(checkpoint, &tentative, &request.delegation_goal);
        if rendered.text.chars().count() <= request.budget_chars {
            selected.push(candidate.record);
            continue;
        }
        if candidate.pinned {
            return Err(ProjectionError::PinnedStateExceedsBudget {
                state_id: candidate.record.state_id,
                limit: request.budget_chars,
            });
        }
        omissions.push(ProjectionOmission {
            state_id: candidate.record.state_id,
            reason: ProjectionOmissionReason::BudgetExceeded,
        });
    }
    omissions.sort_by(|left, right| left.state_id.cmp(&right.state_id));
    let rendered = render_projection(checkpoint, &selected, &request.delegation_goal);
    Ok(AdmittedProjection {
        selected,
        omissions,
        rendered,
    })
}
