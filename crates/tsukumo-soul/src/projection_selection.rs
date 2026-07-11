//! Deterministic checkpoint-pinned and scope-aware state candidate selection.

use crate::handoff_model::HandoffCheckpoint;
use crate::projection_error::ProjectionError;
use crate::projection_model::{ProjectionOmission, ProjectionOmissionReason, ProjectionRequest};
use crate::state_model::{EvidenceStrength, StateRecord};
use crate::state_repository::{list_states, load_state};
use crate::storage::SoulError;
use rusqlite::Connection;
use std::cmp::Ordering;
use std::collections::BTreeSet;
use tsukumo_kernel::StateId;

/// Eligible state plus whether the checkpoint pins it as a hard constraint.
pub(crate) struct ProjectionCandidate {
    pub record: StateRecord,
    pub pinned: bool,
}

pub(crate) struct CandidateSelection {
    pub candidates: Vec<ProjectionCandidate>,
    pub omissions: Vec<ProjectionOmission>,
}

pub(crate) fn select_candidates(
    conn: &Connection,
    checkpoint: &HandoffCheckpoint,
    request: &ProjectionRequest,
) -> Result<CandidateSelection, SoulError> {
    let excluded = request
        .excluded_state_ids
        .iter()
        .cloned()
        .collect::<BTreeSet<_>>();
    let pinned_ids = checkpoint
        .constraint_refs
        .iter()
        .map(|state_ref| state_ref.state_id.clone())
        .collect::<BTreeSet<_>>();
    let mut candidates = Vec::new();
    let mut omissions = Vec::new();

    for state_ref in &checkpoint.constraint_refs {
        let record = load_state(conn, &state_ref.state_id)?
            .ok_or_else(|| ProjectionError::MissingState(state_ref.state_id.clone()))?;
        if record.version != state_ref.version {
            return Err(ProjectionError::StateVersionMismatch {
                state_id: record.state_id,
                expected: state_ref.version,
                found: record.version,
            }
            .into());
        }
        match omission_reason(&record, request, &excluded) {
            Some(reason) => omissions.push(ProjectionOmission {
                state_id: record.state_id,
                reason,
            }),
            None => candidates.push(ProjectionCandidate {
                record,
                pinned: true,
            }),
        }
    }

    let mut ranked = list_states(conn, Some(request.created_at))?
        .into_iter()
        .filter(|record| !pinned_ids.contains(&record.state_id))
        .filter_map(
            |record| match omission_reason(&record, request, &excluded) {
                Some(reason) => {
                    omissions.push(ProjectionOmission {
                        state_id: record.state_id,
                        reason,
                    });
                    None
                }
                None => Some(record),
            },
        )
        .collect::<Vec<_>>();
    ranked.sort_by(compare_rank);
    candidates.extend(ranked.into_iter().map(|record| ProjectionCandidate {
        record,
        pinned: false,
    }));
    omissions.sort_by(|left, right| left.state_id.cmp(&right.state_id));
    Ok(CandidateSelection {
        candidates,
        omissions,
    })
}

fn omission_reason(
    record: &StateRecord,
    request: &ProjectionRequest,
    excluded: &BTreeSet<StateId>,
) -> Option<ProjectionOmissionReason> {
    if excluded.contains(&record.state_id) {
        return Some(ProjectionOmissionReason::ExcludedByComparison);
    }
    if !record.is_active_at(request.created_at) {
        return Some(ProjectionOmissionReason::Inactive);
    }
    if !record.scope.applies_to(&request.scope) {
        return Some(ProjectionOmissionReason::ScopeMismatch);
    }
    None
}

fn compare_rank(left: &StateRecord, right: &StateRecord) -> Ordering {
    right
        .scope
        .specificity_score()
        .cmp(&left.scope.specificity_score())
        .then_with(|| strength_rank(right.strength).cmp(&strength_rank(left.strength)))
        .then_with(|| right.created_at.cmp(&left.created_at))
        .then_with(|| left.state_key.as_str().cmp(right.state_key.as_str()))
        .then_with(|| left.state_id.cmp(&right.state_id))
}

const fn strength_rank(strength: EvidenceStrength) -> u8 {
    match strength {
        EvidenceStrength::Imported => 1,
        EvidenceStrength::Inferred => 2,
        EvidenceStrength::Repeated => 3,
        EvidenceStrength::Explicit => 4,
    }
}
