//! Transactional deterministic StateWriter implementation.

use crate::chronicle::append_event_in;
use crate::state_model::{
    StateDraft, StateRecord, StateScope, StateStatus, StateTransition, StateValidationError,
    StateWriteOutcome, StateWriteRequest,
};
use crate::state_repository::{
    find_active_state, has_state_created_after, insert_state, list_states, load_state,
    next_version, update_status,
};
use crate::state_validation::{
    validate_draft, validate_lifecycle, validate_source_link, validate_transition_evidence,
};
use crate::storage::{current_timestamp, SoulError, SoulStore};
use tsukumo_kernel::{PersistedText, StateId, StateLifecycleAction, Timestamp};

impl SoulStore {
    /// Applies one state transition and its Chronicle events in one transaction.
    pub fn apply_state(
        &mut self,
        request: StateWriteRequest,
    ) -> Result<StateWriteOutcome, SoulError> {
        validate_source_link(&request)?;
        let transaction = self.conn.transaction()?;
        if let Some(source_event) = &request.source_event {
            append_event_in(&transaction, source_event)?;
        }

        let (outcome, append_lifecycle) = match request.transition {
            StateTransition::Create {
                state_id,
                draft,
                created_at,
            } => apply_create(
                &transaction,
                state_id,
                draft,
                created_at,
                &request.lifecycle_event,
            )?,
            StateTransition::Supersede {
                state_id,
                prior,
                draft,
                created_at,
            } => apply_supersede(
                &transaction,
                state_id,
                prior,
                draft,
                created_at,
                &request.lifecycle_event,
            )?,
            StateTransition::Revoke {
                prior,
                evidence,
                revoked_at,
            } => apply_revoke(
                &transaction,
                prior,
                evidence,
                revoked_at,
                &request.lifecycle_event,
            )?,
        };

        if append_lifecycle {
            append_event_in(&transaction, &request.lifecycle_event)?;
        }
        transaction.commit()?;
        Ok(outcome)
    }

    /// Loads one canonical state version and its evidence refs.
    pub fn state(&self, state_id: &StateId) -> Result<Option<StateRecord>, SoulError> {
        load_state(&self.conn, state_id)
    }

    /// Loads the current active state for a semantic key and scope.
    pub fn active_state(
        &self,
        key: &crate::state_model::StateKey,
        scope: &StateScope,
    ) -> Result<Option<StateRecord>, SoulError> {
        self.active_state_at(key, scope, current_timestamp()?)
    }

    /// Loads the state selectable at one controlled instant.
    pub fn active_state_at(
        &self,
        key: &crate::state_model::StateKey,
        scope: &StateScope,
        as_of: Timestamp,
    ) -> Result<Option<StateRecord>, SoulError> {
        find_active_state(&self.conn, key, &scope.canonical_key()?, as_of)
    }

    /// Lists all canonical state versions active at the current instant.
    pub fn list_active_states(&self) -> Result<Vec<StateRecord>, SoulError> {
        self.list_active_states_at(current_timestamp()?)
    }

    /// Lists all canonical state versions selectable at one controlled instant.
    pub fn list_active_states_at(&self, as_of: Timestamp) -> Result<Vec<StateRecord>, SoulError> {
        list_states(&self.conn, Some(as_of))
    }
}

fn apply_create(
    conn: &rusqlite::Connection,
    state_id: StateId,
    draft: StateDraft,
    created_at: Timestamp,
    lifecycle_event: &tsukumo_kernel::KernelEvent,
) -> Result<(StateWriteOutcome, bool), SoulError> {
    validate_lifecycle(
        lifecycle_event,
        &state_id,
        StateLifecycleAction::Created,
        None,
        created_at,
        &draft.evidence_refs,
    )?;
    validate_draft(conn, &draft, created_at, &lifecycle_event.spirit_id)?;
    let scope_key = draft.scope.canonical_key()?;

    if let Some(existing) = load_state(conn, &state_id)? {
        if existing.created_at == created_at && record_matches_draft(&existing, &draft) {
            return Ok((StateWriteOutcome::Unchanged(existing), false));
        }
        return Err(StateValidationError::Conflict(draft.proposed_key).into());
    }
    if has_state_created_after(conn, &draft.proposed_key, &scope_key, created_at)? {
        return Err(StateValidationError::BackdatedTransition(draft.proposed_key).into());
    }
    if let Some(existing) = find_active_state(conn, &draft.proposed_key, &scope_key, created_at)? {
        if record_matches_draft(&existing, &draft) {
            return Ok((StateWriteOutcome::Unchanged(existing), false));
        }
        return Err(StateValidationError::Conflict(draft.proposed_key).into());
    }

    let version = next_version(conn, &draft.proposed_key, &scope_key)?;
    let record = record_from_draft(state_id, draft, version, created_at, None);
    insert_state(conn, &record)?;
    Ok((StateWriteOutcome::Created(record), true))
}

fn apply_supersede(
    conn: &rusqlite::Connection,
    state_id: StateId,
    prior_id: StateId,
    draft: StateDraft,
    created_at: Timestamp,
    lifecycle_event: &tsukumo_kernel::KernelEvent,
) -> Result<(StateWriteOutcome, bool), SoulError> {
    validate_lifecycle(
        lifecycle_event,
        &state_id,
        StateLifecycleAction::Superseded,
        Some(&prior_id),
        created_at,
        &draft.evidence_refs,
    )?;
    validate_draft(conn, &draft, created_at, &lifecycle_event.spirit_id)?;

    if let Some(existing) = load_state(conn, &state_id)? {
        if existing.created_at == created_at
            && existing.supersedes_state_id.as_ref() == Some(&prior_id)
            && record_matches_draft(&existing, &draft)
        {
            return Ok((StateWriteOutcome::Unchanged(existing), false));
        }
        return Err(StateValidationError::Conflict(draft.proposed_key).into());
    }

    let prior = active_record(conn, &prior_id, created_at)?;
    if prior.state_key != draft.proposed_key || prior.scope != draft.scope {
        return Err(StateValidationError::Conflict(draft.proposed_key).into());
    }
    let scope_key = draft.scope.canonical_key()?;
    let version = next_version(conn, &draft.proposed_key, &scope_key)?;
    let record = record_from_draft(state_id, draft, version, created_at, Some(prior_id.clone()));

    update_status(conn, &prior_id, StateStatus::Superseded, created_at)?;
    insert_state(conn, &record)?;
    Ok((StateWriteOutcome::Superseded(record), true))
}

fn apply_revoke(
    conn: &rusqlite::Connection,
    prior_id: StateId,
    evidence: tsukumo_kernel::EventId,
    revoked_at: Timestamp,
    lifecycle_event: &tsukumo_kernel::KernelEvent,
) -> Result<(StateWriteOutcome, bool), SoulError> {
    validate_lifecycle(
        lifecycle_event,
        &prior_id,
        StateLifecycleAction::Revoked,
        None,
        revoked_at,
        std::slice::from_ref(&evidence),
    )?;
    validate_transition_evidence(conn, &evidence, revoked_at, &lifecycle_event.spirit_id)?;

    let prior = load_state(conn, &prior_id)?
        .ok_or_else(|| StateValidationError::StateNotFound(prior_id.clone()))?;
    if prior.status == StateStatus::Revoked {
        return Ok((StateWriteOutcome::Unchanged(prior), false));
    }
    let prior = active_record(conn, &prior_id, revoked_at)?;

    update_status(conn, &prior.state_id, StateStatus::Revoked, revoked_at)?;
    let revoked = load_state(conn, &prior.state_id)?
        .ok_or_else(|| StateValidationError::StateNotFound(prior.state_id.clone()))?;
    Ok((StateWriteOutcome::Revoked(revoked), true))
}

fn active_record(
    conn: &rusqlite::Connection,
    state_id: &StateId,
    as_of: Timestamp,
) -> Result<StateRecord, SoulError> {
    let record = load_state(conn, state_id)?
        .ok_or_else(|| StateValidationError::StateNotFound(state_id.clone()))?;
    if record.status != StateStatus::Active || !record.is_active_at(as_of) {
        return Err(StateValidationError::StateInactive(state_id.clone()).into());
    }
    Ok(record)
}

fn record_from_draft(
    state_id: StateId,
    draft: StateDraft,
    version: u64,
    created_at: Timestamp,
    supersedes_state_id: Option<StateId>,
) -> StateRecord {
    let mut evidence_refs = draft.evidence_refs;
    evidence_refs.sort_by(|left, right| left.as_str().cmp(right.as_str()));
    evidence_refs.dedup();
    StateRecord {
        state_id,
        state_key: draft.proposed_key,
        kind: draft.kind,
        scope: draft.scope,
        content: PersistedText::from_reviewed(draft.content.expose()),
        strength: draft.claimed_strength,
        status: StateStatus::Active,
        evidence_refs,
        provenance: draft.provenance,
        version,
        created_at,
        expires_at: draft.expires_at,
        deactivated_at: None,
        supersedes_state_id,
    }
}

fn record_matches_draft(record: &StateRecord, draft: &StateDraft) -> bool {
    let mut evidence_refs = draft.evidence_refs.clone();
    evidence_refs.sort_by(|left, right| left.as_str().cmp(right.as_str()));
    evidence_refs.dedup();
    record.state_key == draft.proposed_key
        && record.kind == draft.kind
        && record.scope == draft.scope
        && record.content.as_str() == draft.content.expose()
        && record.strength == draft.claimed_strength
        && record.evidence_refs == evidence_refs
        && record.provenance == draft.provenance
        && record.expires_at == draft.expires_at
}
