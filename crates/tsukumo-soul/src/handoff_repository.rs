//! SQLite encoding for immutable handoff checkpoints and their durable edges.

use crate::handoff_model::HandoffCheckpoint;
use crate::storage::SoulError;
use rusqlite::{params, Connection, OptionalExtension};
use tsukumo_kernel::{CheckpointId, EventId, StateId};

pub(crate) fn load_checkpoint(
    conn: &Connection,
    checkpoint_id: &CheckpointId,
) -> Result<Option<HandoffCheckpoint>, SoulError> {
    let json = conn
        .query_row(
            "SELECT checkpoint_json FROM handoff_checkpoints WHERE checkpoint_id = ?1",
            [checkpoint_id.as_str()],
            |row| row.get::<_, String>(0),
        )
        .optional()?;
    let Some(json) = json else {
        return Ok(None);
    };
    let checkpoint = serde_json::from_str::<HandoffCheckpoint>(&json)?;
    if checkpoint.id != *checkpoint_id {
        return Err(SoulError::InvalidStoredValue {
            field: "handoff_checkpoints.checkpoint_id",
            value: checkpoint.id.to_string(),
        });
    }
    validate_stored_edges(conn, &checkpoint)?;
    Ok(Some(checkpoint))
}

/// Loads the Chronicle event identity bound to one immutable checkpoint.
pub(crate) fn load_checkpoint_event_id(
    conn: &Connection,
    checkpoint_id: &CheckpointId,
) -> Result<Option<EventId>, SoulError> {
    conn.query_row(
        "SELECT created_event_id FROM handoff_checkpoints WHERE checkpoint_id = ?1",
        [checkpoint_id.as_str()],
        |row| row.get::<_, String>(0),
    )
    .optional()
    .map(|value| value.map(EventId::new))
    .map_err(Into::into)
}

pub(crate) fn insert_checkpoint(
    conn: &Connection,
    checkpoint: &HandoffCheckpoint,
    created_event_id: &EventId,
) -> Result<(), SoulError> {
    let version = i64::try_from(checkpoint.version).map_err(|_| SoulError::InvalidStoredValue {
        field: "handoff_checkpoints.version",
        value: checkpoint.version.to_string(),
    })?;
    conn.execute(
        "INSERT INTO handoff_checkpoints (
            checkpoint_id, quest_id, version, previous_checkpoint_id, created_at,
            created_event_id, checkpoint_json
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
        params![
            checkpoint.id.as_str(),
            checkpoint.quest_id.as_str(),
            version,
            checkpoint.previous_id.as_ref().map(CheckpointId::as_str),
            checkpoint.created_at.as_unix_millis(),
            created_event_id.as_str(),
            serde_json::to_string(checkpoint)?,
        ],
    )?;
    for (position, state_ref) in checkpoint.constraint_refs.iter().enumerate() {
        conn.execute(
            "INSERT INTO checkpoint_state_refs (
                checkpoint_id, state_id, state_version, position
             ) VALUES (?1, ?2, ?3, ?4)",
            params![
                checkpoint.id.as_str(),
                state_ref.state_id.as_str(),
                to_i64(state_ref.version, "checkpoint_state_refs.state_version")?,
                to_i64(position as u64, "checkpoint_state_refs.position")?,
            ],
        )?;
    }
    for (position, event_id) in checkpoint.source_event_refs.iter().enumerate() {
        conn.execute(
            "INSERT INTO checkpoint_source_refs (checkpoint_id, event_id, position)
             VALUES (?1, ?2, ?3)",
            params![
                checkpoint.id.as_str(),
                event_id.as_str(),
                to_i64(position as u64, "checkpoint_source_refs.position")?,
            ],
        )?;
    }
    Ok(())
}

fn validate_stored_edges(
    conn: &Connection,
    checkpoint: &HandoffCheckpoint,
) -> Result<(), SoulError> {
    let state_edges = load_state_edges(conn, &checkpoint.id)?;
    if state_edges != checkpoint.constraint_refs {
        return Err(SoulError::InvalidStoredValue {
            field: "checkpoint_state_refs",
            value: checkpoint.id.to_string(),
        });
    }
    let source_edges = load_source_edges(conn, &checkpoint.id)?;
    if source_edges != checkpoint.source_event_refs {
        return Err(SoulError::InvalidStoredValue {
            field: "checkpoint_source_refs",
            value: checkpoint.id.to_string(),
        });
    }
    Ok(())
}

fn load_state_edges(
    conn: &Connection,
    checkpoint_id: &CheckpointId,
) -> Result<Vec<crate::handoff_model::StateRef>, SoulError> {
    let mut statement = conn.prepare(
        "SELECT state_id, state_version FROM checkpoint_state_refs
         WHERE checkpoint_id = ?1 ORDER BY position ASC",
    )?;
    let rows = statement.query_map([checkpoint_id.as_str()], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    })?;
    let mut refs = Vec::new();
    for row in rows {
        let (state_id, version) = row?;
        refs.push(crate::handoff_model::StateRef::new(
            StateId::new(state_id),
            to_u64(version, "checkpoint_state_refs.state_version")?,
        ));
    }
    Ok(refs)
}

fn load_source_edges(
    conn: &Connection,
    checkpoint_id: &CheckpointId,
) -> Result<Vec<EventId>, SoulError> {
    let mut statement = conn.prepare(
        "SELECT event_id FROM checkpoint_source_refs
         WHERE checkpoint_id = ?1 ORDER BY position ASC",
    )?;
    let rows = statement.query_map([checkpoint_id.as_str()], |row| row.get::<_, String>(0))?;
    let mut refs = Vec::new();
    for row in rows {
        refs.push(EventId::new(row?));
    }
    Ok(refs)
}

fn to_i64(value: u64, field: &'static str) -> Result<i64, SoulError> {
    i64::try_from(value).map_err(|_| SoulError::InvalidStoredValue {
        field,
        value: value.to_string(),
    })
}

fn to_u64(value: i64, field: &'static str) -> Result<u64, SoulError> {
    u64::try_from(value).map_err(|_| SoulError::InvalidStoredValue {
        field,
        value: value.to_string(),
    })
}
