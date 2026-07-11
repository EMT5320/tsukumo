//! Metadata-only invariants for deterministic with-state/without-state evidence.

use crate::projection_error::ProjectionError;
use crate::projection_model::{
    ContentDigest, ProjectionOmissionReason, ProjectionReceipt, ProjectionSection,
};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use tsukumo_kernel::{ProjectionId, StateId};

/// Bounded comparison metadata that never contains rendered prompt text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ProjectionComparison {
    pub target_state_id: StateId,
    pub with_projection_id: ProjectionId,
    pub without_projection_id: ProjectionId,
    pub with_digest: ContentDigest,
    pub without_digest: ContentDigest,
    pub changed_sections: Vec<ProjectionSection>,
}

/// Verifies that a receipt pair differs only by one target-state projection.
pub fn compare_projection_receipts(
    with_state: &ProjectionReceipt,
    without_state: &ProjectionReceipt,
    target_state_id: &StateId,
) -> Result<ProjectionComparison, ProjectionError> {
    validate_shared_metadata(with_state, without_state)?;
    validate_selected_refs(with_state, without_state, target_state_id)?;
    validate_omissions(with_state, without_state, target_state_id)?;
    let changed_sections = compare_sections(with_state, without_state)?;
    if changed_sections != [ProjectionSection::Constraints]
        || with_state.rendered_digest == without_state.rendered_digest
    {
        return Err(ProjectionError::ComparisonInvariant);
    }
    Ok(ProjectionComparison {
        target_state_id: target_state_id.clone(),
        with_projection_id: with_state.id.clone(),
        without_projection_id: without_state.id.clone(),
        with_digest: with_state.rendered_digest.clone(),
        without_digest: without_state.rendered_digest.clone(),
        changed_sections,
    })
}

fn validate_shared_metadata(
    with_state: &ProjectionReceipt,
    without_state: &ProjectionReceipt,
) -> Result<(), ProjectionError> {
    let matches = with_state.checkpoint_id == without_state.checkpoint_id
        && with_state.runtime == without_state.runtime
        && with_state.projection_version == without_state.projection_version
        && with_state.renderer_version == without_state.renderer_version
        && with_state.created_at == without_state.created_at
        && with_state.budget.limit == without_state.budget.limit
        && with_state.budget.unit == without_state.budget.unit
        && with_state.redactions == without_state.redactions;
    if matches {
        Ok(())
    } else {
        Err(ProjectionError::ComparisonInvariant)
    }
}

fn validate_selected_refs(
    with_state: &ProjectionReceipt,
    without_state: &ProjectionReceipt,
    target_state_id: &StateId,
) -> Result<(), ProjectionError> {
    if !with_state
        .selected_state_refs
        .iter()
        .any(|state_ref| &state_ref.state_id == target_state_id)
        || without_state
            .selected_state_refs
            .iter()
            .any(|state_ref| &state_ref.state_id == target_state_id)
    {
        return Err(ProjectionError::ComparisonInvariant);
    }
    let retained = with_state
        .selected_state_refs
        .iter()
        .filter(|state_ref| &state_ref.state_id != target_state_id)
        .collect::<Vec<_>>();
    let without = without_state.selected_state_refs.iter().collect::<Vec<_>>();
    if retained == without {
        Ok(())
    } else {
        Err(ProjectionError::ComparisonInvariant)
    }
}

fn validate_omissions(
    with_state: &ProjectionReceipt,
    without_state: &ProjectionReceipt,
    target_state_id: &StateId,
) -> Result<(), ProjectionError> {
    let target_excluded = without_state.omissions.iter().any(|omission| {
        &omission.state_id == target_state_id
            && omission.reason == ProjectionOmissionReason::ExcludedByComparison
    });
    let with_other = with_state
        .omissions
        .iter()
        .filter(|omission| &omission.state_id != target_state_id)
        .collect::<Vec<_>>();
    let without_other = without_state
        .omissions
        .iter()
        .filter(|omission| &omission.state_id != target_state_id)
        .collect::<Vec<_>>();
    if target_excluded && with_other == without_other {
        Ok(())
    } else {
        Err(ProjectionError::ComparisonInvariant)
    }
}

fn compare_sections(
    with_state: &ProjectionReceipt,
    without_state: &ProjectionReceipt,
) -> Result<Vec<ProjectionSection>, ProjectionError> {
    let with_sections = with_state
        .sections
        .iter()
        .map(|section| (section.section, section))
        .collect::<BTreeMap<_, _>>();
    let without_sections = without_state
        .sections
        .iter()
        .map(|section| (section.section, section))
        .collect::<BTreeMap<_, _>>();
    if with_sections.len() != with_state.sections.len()
        || without_sections.len() != without_state.sections.len()
        || with_sections.keys().ne(without_sections.keys())
    {
        return Err(ProjectionError::ComparisonInvariant);
    }
    Ok(with_sections
        .into_iter()
        .filter_map(|(identity, section)| {
            (without_sections.get(&identity) != Some(&section)).then_some(identity)
        })
        .collect())
}
