//! Receipt-first projection preparation over checkpoints and canonical state.

use crate::chronicle::append_event_in;
use crate::handoff_repository::load_checkpoint;
use crate::projection_budget::admit_candidates;
use crate::projection_error::ProjectionError;
use crate::projection_model::{
    BudgetUnit, PreparedProjection, ProjectionBudgetUsage, ProjectionReceipt, ProjectionRequest,
    ProjectionWriteRequest, RedactionRecord, PROJECTION_VERSION, RENDERER_VERSION,
};
use crate::projection_render::{digest_text, section_digests};
use crate::projection_repository::{insert_receipt, load_receipt, load_receipt_event_id};
use crate::projection_selection::select_candidates;
use crate::storage::{SoulError, SoulStore};
use std::collections::BTreeSet;
use tsukumo_kernel::{contains_sensitive_material, KernelEventPayload, SensitiveText};

const MAX_DELEGATION_GOAL_CHARS: usize = 65_536;
const MAX_PROJECTION_ID_CHARS: usize = 256;

impl SoulStore {
    /// Commits a receipt and returns the only prompt value a host may launch.
    pub fn prepare_projection(
        &mut self,
        write: ProjectionWriteRequest,
    ) -> Result<PreparedProjection, SoulError> {
        validate_request(&write.request)?;
        let checkpoint =
            load_checkpoint(&self.conn, &write.request.checkpoint_id)?.ok_or_else(|| {
                ProjectionError::MissingCheckpoint(write.request.checkpoint_id.clone())
            })?;
        validate_event(&write, &checkpoint)?;

        let selection = select_candidates(&self.conn, &checkpoint, &write.request)?;
        let admitted = admit_candidates(&checkpoint, &write.request, selection)?;
        let receipt = build_receipt(&write.request, &admitted);
        if let Some(existing) = load_receipt(&self.conn, &receipt.id)? {
            let same_event = load_receipt_event_id(&self.conn, &receipt.id)?.as_ref()
                == Some(&write.event.event_id);
            if existing == receipt && same_event {
                append_event_in(&self.conn, &write.event)?;
                return Ok(PreparedProjection::new(
                    existing,
                    SensitiveText::new(admitted.rendered.text),
                ));
            }
            return Err(ProjectionError::ConflictingReceipt(receipt.id).into());
        }

        let transaction = self.conn.transaction()?;
        append_event_in(&transaction, &write.event)?;
        insert_receipt(&transaction, &receipt, &write.event.event_id)?;
        transaction.commit()?;
        Ok(PreparedProjection::new(
            receipt,
            SensitiveText::new(admitted.rendered.text),
        ))
    }

    /// Loads one immutable receipt and verifies selected-state edges.
    pub fn projection_receipt(
        &self,
        projection_id: &tsukumo_kernel::ProjectionId,
    ) -> Result<Option<ProjectionReceipt>, SoulError> {
        load_receipt(&self.conn, projection_id)
    }
}

fn build_receipt(
    request: &ProjectionRequest,
    admitted: &crate::projection_budget::AdmittedProjection,
) -> ProjectionReceipt {
    let rendered_byte_count = admitted.rendered.text.len();
    let rendered_char_count = admitted.rendered.text.chars().count();
    ProjectionReceipt {
        id: request.projection_id.clone(),
        execution_id: request.execution_id.clone(),
        checkpoint_id: request.checkpoint_id.clone(),
        runtime: request.runtime.clone(),
        selected_state_refs: admitted
            .selected
            .iter()
            .map(|state| crate::handoff_model::StateRef::new(state.state_id.clone(), state.version))
            .collect(),
        projection_version: PROJECTION_VERSION,
        renderer_version: RENDERER_VERSION,
        rendered_digest: digest_text(&admitted.rendered.text),
        rendered_byte_count,
        rendered_char_count,
        sections: section_digests(&admitted.rendered.sections),
        budget: ProjectionBudgetUsage {
            used: rendered_char_count,
            limit: request.budget_chars,
            unit: BudgetUnit::Characters,
        },
        omissions: admitted.omissions.clone(),
        redactions: delegation_goal_redactions(&request.delegation_goal),
        created_at: request.created_at,
    }
}

/// Records why detected secret material remains prompt-only without retaining its value.
fn delegation_goal_redactions(goal: &SensitiveText) -> Vec<RedactionRecord> {
    if contains_sensitive_material(goal.expose()) {
        vec![RedactionRecord {
            location: "delegation_goal".to_owned(),
            category: "sensitive_material".to_owned(),
            action: "not_persisted".to_owned(),
        }]
    } else {
        Vec::new()
    }
}

fn validate_request(request: &ProjectionRequest) -> Result<(), ProjectionError> {
    validate_label("projection.id", request.projection_id.as_str())?;
    validate_label("projection.execution_id", request.execution_id.as_str())?;
    validate_label("projection.checkpoint_id", request.checkpoint_id.as_str())?;
    let goal = request.delegation_goal.expose();
    if goal.trim().is_empty() || goal.chars().count() > MAX_DELEGATION_GOAL_CHARS {
        return Err(ProjectionError::InvalidField("projection.delegation_goal"));
    }
    if request.budget_chars == 0 {
        return Err(ProjectionError::InvalidField("projection.budget_chars"));
    }
    let mut excluded = BTreeSet::new();
    if request
        .excluded_state_ids
        .iter()
        .any(|state_id| !excluded.insert(state_id.clone()))
    {
        return Err(ProjectionError::InvalidField(
            "projection.excluded_state_ids",
        ));
    }
    Ok(())
}

fn validate_event(
    write: &ProjectionWriteRequest,
    checkpoint: &crate::handoff_model::HandoffCheckpoint,
) -> Result<(), ProjectionError> {
    let request = &write.request;
    let event = &write.event;
    let payload_matches = matches!(
        &event.payload,
        KernelEventPayload::ProjectionCreated {
            projection_id,
            checkpoint_id,
        } if projection_id == &request.projection_id
            && checkpoint_id == &request.checkpoint_id
    );
    if !payload_matches
        || event.quest_id != checkpoint.quest_id
        || event.execution_id.as_ref() != Some(&request.execution_id)
        || event.runtime.as_ref() != Some(&request.runtime)
        || event.occurred_at != request.created_at
        || request.created_at < checkpoint.created_at
    {
        return Err(ProjectionError::EventMismatch);
    }
    Ok(())
}

fn validate_label(field: &'static str, value: &str) -> Result<(), ProjectionError> {
    if value.is_empty()
        || value.chars().count() > MAX_PROJECTION_ID_CHARS
        || value.chars().any(char::is_control)
    {
        return Err(ProjectionError::InvalidField(field));
    }
    if tsukumo_kernel::contains_sensitive_material(value) {
        return Err(ProjectionError::SensitiveField(field));
    }
    Ok(())
}
