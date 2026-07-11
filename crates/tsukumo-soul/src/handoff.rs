//! Transactional validation and persistence for immutable handoff checkpoints.

use crate::chronicle::{append_event_in, load_event_in};
use crate::handoff_error::HandoffError;
use crate::handoff_loop::{OpenLoopId, OpenLoopOutcome, OpenLoopTransition};
use crate::handoff_model::{CheckpointWriteRequest, HandoffCheckpoint};
use crate::handoff_repository::{insert_checkpoint, load_checkpoint, load_checkpoint_event_id};
use crate::handoff_shape::validate_shape;
use crate::state_model::StateKind;
use crate::state_repository::load_state;
use crate::storage::{SoulError, SoulStore};
use std::collections::{BTreeMap, BTreeSet};
use tsukumo_kernel::KernelEventPayload;

impl SoulStore {
    /// Atomically appends a checkpoint event and its immutable checkpoint row.
    pub fn save_checkpoint(&mut self, request: CheckpointWriteRequest) -> Result<(), SoulError> {
        validate_shape(&request.checkpoint)?;
        validate_event(&request)?;
        if let Some(existing) = load_checkpoint(&self.conn, &request.checkpoint.id)? {
            let same_event = load_checkpoint_event_id(&self.conn, &request.checkpoint.id)?.as_ref()
                == Some(&request.event.event_id);
            if existing == request.checkpoint && same_event {
                append_event_in(&self.conn, &request.event)?;
                return Ok(());
            }
            return Err(HandoffError::ConflictingCheckpoint(request.checkpoint.id).into());
        }

        let transaction = self.conn.transaction()?;
        validate_sources(&transaction, &request.checkpoint)?;
        validate_constraints(&transaction, &request.checkpoint)?;
        validate_previous(&transaction, &request.checkpoint)?;
        append_event_in(&transaction, &request.event)?;
        insert_checkpoint(&transaction, &request.checkpoint, &request.event.event_id)?;
        transaction.commit()?;
        Ok(())
    }

    /// Loads one checkpoint and verifies its stored edge tables.
    pub fn checkpoint(
        &self,
        checkpoint_id: &tsukumo_kernel::CheckpointId,
    ) -> Result<Option<HandoffCheckpoint>, SoulError> {
        load_checkpoint(&self.conn, checkpoint_id)
    }
}

fn validate_event(request: &CheckpointWriteRequest) -> Result<(), HandoffError> {
    let checkpoint = &request.checkpoint;
    let event = &request.event;
    let matches_payload = matches!(
        &event.payload,
        KernelEventPayload::CheckpointCreated {
            checkpoint_id,
            version,
        } if checkpoint_id == &checkpoint.id && version == &checkpoint.version
    );
    if !matches_payload
        || event.quest_id != checkpoint.quest_id
        || event.occurred_at != checkpoint.created_at
    {
        return Err(HandoffError::EventMismatch);
    }
    Ok(())
}

fn validate_sources(
    conn: &rusqlite::Connection,
    checkpoint: &HandoffCheckpoint,
) -> Result<(), SoulError> {
    for event_id in &checkpoint.source_event_refs {
        if load_event_in(conn, event_id)?.is_none() {
            return Err(HandoffError::MissingSourceEvent(event_id.clone()).into());
        }
    }
    Ok(())
}

fn validate_constraints(
    conn: &rusqlite::Connection,
    checkpoint: &HandoffCheckpoint,
) -> Result<(), SoulError> {
    for state_ref in &checkpoint.constraint_refs {
        let state = load_state(conn, &state_ref.state_id)?
            .ok_or_else(|| HandoffError::MissingState(state_ref.state_id.clone()))?;
        if state.version != state_ref.version {
            return Err(HandoffError::StateVersionMismatch {
                state_id: state.state_id,
                expected: state_ref.version,
                found: state.version,
            }
            .into());
        }
        if state.kind != StateKind::Constraint || !state.is_active_at(checkpoint.created_at) {
            return Err(HandoffError::InactiveState(state_ref.state_id.clone()).into());
        }
    }
    Ok(())
}

fn validate_previous(
    conn: &rusqlite::Connection,
    checkpoint: &HandoffCheckpoint,
) -> Result<(), SoulError> {
    let Some(previous_id) = &checkpoint.previous_id else {
        if checkpoint.version != 1 || !checkpoint.open_loop_transitions.is_empty() {
            return Err(HandoffError::InvalidVersion.into());
        }
        return Ok(());
    };
    let previous = load_checkpoint(conn, previous_id)?
        .ok_or_else(|| HandoffError::MissingPrevious(previous_id.clone()))?;
    if previous.quest_id != checkpoint.quest_id {
        return Err(HandoffError::QuestMismatch.into());
    }
    if previous.version.checked_add(1) != Some(checkpoint.version)
        || checkpoint.created_at <= previous.created_at
    {
        return Err(HandoffError::InvalidVersion.into());
    }
    validate_transitions(&previous, checkpoint).map_err(Into::into)
}

fn validate_transitions(
    previous: &HandoffCheckpoint,
    checkpoint: &HandoffCheckpoint,
) -> Result<(), HandoffError> {
    let prior_ids = previous
        .open_loops
        .iter()
        .map(|item| item.id.clone())
        .collect::<BTreeSet<_>>();
    let current_ids = checkpoint
        .open_loops
        .iter()
        .map(|item| item.id.clone())
        .collect::<BTreeSet<_>>();
    let mut transitions = BTreeMap::new();
    for transition in &checkpoint.open_loop_transitions {
        if !prior_ids.contains(&transition.prior) {
            return Err(HandoffError::UnknownPriorLoop(transition.prior.clone()));
        }
        if transitions
            .insert(transition.prior.clone(), transition)
            .is_some()
        {
            return Err(HandoffError::DuplicateTransition(transition.prior.clone()));
        }
    }
    for prior in prior_ids {
        let transition = transitions
            .get(&prior)
            .ok_or_else(|| HandoffError::UnresolvedPriorLoop(prior.clone()))?;
        validate_transition(transition, &current_ids)?;
    }
    Ok(())
}

fn validate_transition(
    transition: &OpenLoopTransition,
    current_ids: &BTreeSet<OpenLoopId>,
) -> Result<(), HandoffError> {
    let valid = match &transition.outcome {
        OpenLoopOutcome::Inherited => current_ids.contains(&transition.prior),
        OpenLoopOutcome::Completed | OpenLoopOutcome::Abandoned => {
            !current_ids.contains(&transition.prior)
        }
        OpenLoopOutcome::ReplacedBy { replacement } => {
            !current_ids.contains(&transition.prior) && current_ids.contains(replacement)
        }
    };
    if valid {
        Ok(())
    } else {
        Err(HandoffError::InvalidOpenLoopTransition(
            transition.prior.clone(),
        ))
    }
}
