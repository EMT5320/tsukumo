//! Deterministic validation gate between extraction and canonical state writes.

use crate::chronicle::load_event_in;
use crate::state_model::{StateDraft, StateValidationError};
use crate::storage::SoulError;
use rusqlite::Connection;
use tsukumo_kernel::{
    contains_sensitive_material, EventId, KernelEvent, KernelEventPayload, PersistedText, SpiritId,
    StateId, StateLifecycleAction, Timestamp,
};

mod metadata;
mod trust;

use metadata::{validate_metadata, validate_scope};
pub(crate) use trust::is_explicit_gnu_user_text;
use trust::validate_strength;
pub(crate) fn validate_draft(
    conn: &Connection,
    draft: &StateDraft,
    created_at: Timestamp,
    spirit_id: &SpiritId,
) -> Result<(), SoulError> {
    if draft.proposed_key.as_str().trim().is_empty() {
        return Err(StateValidationError::EmptyKey.into());
    }
    if draft.content.expose().trim().is_empty() {
        return Err(StateValidationError::EmptyContent.into());
    }
    if draft.evidence_refs.is_empty() {
        return Err(StateValidationError::EmptyEvidence.into());
    }
    validate_scope(draft)?;
    validate_metadata(draft)?;
    if draft
        .expires_at
        .is_some_and(|expires_at| expires_at <= created_at)
    {
        return Err(StateValidationError::InvalidExpiry.into());
    }
    if contains_sensitive_material(draft.content.expose()) {
        return Err(StateValidationError::SecretMaterial.into());
    }

    let mut evidence = Vec::new();
    for event_id in &draft.evidence_refs {
        evidence.push(validate_transition_evidence(conn, event_id, created_at, spirit_id)?.event);
    }
    validate_strength(draft, &evidence)
}

pub(crate) fn validate_source_link(
    request: &crate::state_model::StateWriteRequest,
) -> Result<(), SoulError> {
    let Some(source) = request.source_event.as_ref() else {
        return Ok(());
    };
    let lifecycle = &request.lifecycle_event;
    if lifecycle.causation_id.as_ref() != Some(&source.event_id)
        || lifecycle.quest_id != source.quest_id
        || lifecycle.session_id != source.session_id
        || lifecycle.spirit_id != source.spirit_id
    {
        return Err(StateValidationError::EvidenceChainMismatch.into());
    }
    Ok(())
}
pub(crate) fn validate_non_permission_evidence(
    conn: &Connection,
    event_id: &EventId,
) -> Result<crate::chronicle::PersistedEvent, SoulError> {
    let persisted = load_event_in(conn, event_id)?
        .ok_or_else(|| StateValidationError::MissingEvidence(event_id.clone()))?;
    if matches!(
        persisted.event.payload,
        KernelEventPayload::PermissionRequested { .. }
            | KernelEventPayload::PermissionDecided { .. }
    ) {
        return Err(StateValidationError::PermissionEvidence(event_id.clone()).into());
    }
    Ok(persisted)
}

pub(crate) fn validate_transition_evidence(
    conn: &Connection,
    event_id: &EventId,
    transition_time: Timestamp,
    spirit_id: &SpiritId,
) -> Result<crate::chronicle::PersistedEvent, SoulError> {
    let persisted = validate_non_permission_evidence(conn, event_id)?;
    if persisted.event.occurred_at > transition_time {
        return Err(StateValidationError::EvidenceAfterTransition(event_id.clone()).into());
    }
    if &persisted.event.spirit_id != spirit_id {
        return Err(StateValidationError::EvidenceChainMismatch.into());
    }
    Ok(persisted)
}
pub(crate) fn validate_lifecycle(
    event: &KernelEvent,
    state_id: &StateId,
    action: StateLifecycleAction,
    prior_state_id: Option<&StateId>,
    transition_time: Timestamp,
    evidence_refs: &[EventId],
) -> Result<(), SoulError> {
    let KernelEventPayload::StateLifecycle {
        state_id: actual_state_id,
        action: actual_action,
        prior_state_id: actual_prior,
        reason,
    } = &event.payload
    else {
        return Err(StateValidationError::InvalidLifecycleEvent.into());
    };
    if actual_state_id != state_id
        || *actual_action != action
        || actual_prior.as_ref() != prior_state_id
        || event.occurred_at != transition_time
        || event
            .causation_id
            .as_ref()
            .map_or(true, |cause| !evidence_refs.contains(cause))
    {
        return Err(StateValidationError::InvalidLifecycleEvent.into());
    }

    match action {
        StateLifecycleAction::Created | StateLifecycleAction::Superseded => {
            if reason.is_some() {
                return Err(StateValidationError::InvalidLifecycleEvent.into());
            }
        }
        StateLifecycleAction::Revoked => validate_revoke_reason(reason.as_ref())?,
    }
    Ok(())
}

fn validate_revoke_reason(reason: Option<&PersistedText>) -> Result<(), SoulError> {
    let reason = reason
        .filter(|text| !text.as_str().trim().is_empty())
        .ok_or(StateValidationError::InvalidLifecycleEvent)?;
    if contains_sensitive_material(reason.as_str()) {
        return Err(StateValidationError::SecretMaterial.into());
    }
    Ok(())
}
