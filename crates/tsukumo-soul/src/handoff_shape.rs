//! Bounded checkpoint shape and duplicate-reference validation.

use crate::handoff_error::HandoffError;
use crate::handoff_model::HandoffCheckpoint;
use std::collections::BTreeSet;
use tsukumo_kernel::contains_sensitive_material;

const MAX_CHECKPOINT_TEXT_CHARS: usize = 65_536;
const MAX_CHECKPOINT_ID_CHARS: usize = 256;

pub(crate) fn validate_shape(checkpoint: &HandoffCheckpoint) -> Result<(), HandoffError> {
    validate_label("checkpoint.id", checkpoint.id.as_str())?;
    validate_label("checkpoint.quest_id", checkpoint.quest_id.as_str())?;
    validate_text("checkpoint.goal", checkpoint.goal.as_str())?;
    if checkpoint.source_event_refs.is_empty() {
        return Err(HandoffError::InvalidField("checkpoint.source_event_refs"));
    }
    validate_unique_ids(checkpoint)?;
    for item in &checkpoint.progress {
        validate_text("checkpoint.progress", item.summary.as_str())?;
    }
    for decision in &checkpoint.decisions {
        validate_text("checkpoint.decisions", decision.summary.as_str())?;
    }
    for artifact in &checkpoint.artifacts {
        validate_label("checkpoint.artifact_id", artifact.artifact_id.as_str())?;
        validate_text("checkpoint.artifact_location", artifact.location.as_str())?;
    }
    for open_loop in &checkpoint.open_loops {
        validate_label("checkpoint.open_loop_id", open_loop.id.as_str())?;
        validate_text("checkpoint.open_loop", open_loop.summary.as_str())?;
    }
    for action in &checkpoint.next_actions {
        validate_text("checkpoint.next_action", action.summary.as_str())?;
    }
    Ok(())
}

fn validate_unique_ids(checkpoint: &HandoffCheckpoint) -> Result<(), HandoffError> {
    let mut loops = BTreeSet::new();
    for open_loop in &checkpoint.open_loops {
        if !loops.insert(open_loop.id.clone()) {
            return Err(HandoffError::DuplicateOpenLoop(open_loop.id.clone()));
        }
    }
    let mut states = BTreeSet::new();
    if checkpoint
        .constraint_refs
        .iter()
        .any(|state_ref| !states.insert(state_ref.state_id.clone()))
    {
        return Err(HandoffError::InvalidField("checkpoint.constraint_refs"));
    }
    let mut sources = BTreeSet::new();
    if checkpoint
        .source_event_refs
        .iter()
        .any(|event_id| !sources.insert(event_id.clone()))
    {
        return Err(HandoffError::InvalidField("checkpoint.source_event_refs"));
    }
    Ok(())
}

fn validate_label(field: &'static str, value: &str) -> Result<(), HandoffError> {
    if value.is_empty()
        || value.chars().count() > MAX_CHECKPOINT_ID_CHARS
        || value.chars().any(char::is_control)
    {
        return Err(HandoffError::InvalidField(field));
    }
    if contains_sensitive_material(value) {
        return Err(HandoffError::SensitiveField(field));
    }
    Ok(())
}

fn validate_text(field: &'static str, value: &str) -> Result<(), HandoffError> {
    if value.trim().is_empty() || value.chars().count() > MAX_CHECKPOINT_TEXT_CHARS {
        return Err(HandoffError::InvalidField(field));
    }
    if contains_sensitive_material(value) {
        return Err(HandoffError::SensitiveField(field));
    }
    Ok(())
}
