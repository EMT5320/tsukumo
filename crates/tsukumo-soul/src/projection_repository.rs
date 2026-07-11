//! SQLite encoding for immutable projection receipts and selected-state edges.

use crate::projection_error::ProjectionError;
use crate::projection_model::ProjectionReceipt;
use crate::storage::SoulError;
use rusqlite::{params, Connection, OptionalExtension};
use tsukumo_kernel::{EventId, ProjectionId, StateId};

pub(crate) fn load_receipt(
    conn: &Connection,
    projection_id: &ProjectionId,
) -> Result<Option<ProjectionReceipt>, SoulError> {
    let json = conn
        .query_row(
            "SELECT receipt_json FROM projection_receipts WHERE projection_id = ?1",
            [projection_id.as_str()],
            |row| row.get::<_, String>(0),
        )
        .optional()?;
    let Some(json) = json else {
        return Ok(None);
    };
    let receipt = serde_json::from_str::<ProjectionReceipt>(&json)?;
    if receipt.id != *projection_id {
        return Err(SoulError::InvalidStoredValue {
            field: "projection_receipts.projection_id",
            value: receipt.id.to_string(),
        });
    }
    let edges = load_receipt_edges(conn, projection_id)?;
    if edges != receipt.selected_state_refs {
        return Err(ProjectionError::StoredEdgeMismatch(projection_id.clone()).into());
    }
    Ok(Some(receipt))
}

/// Loads the Chronicle event identity bound to one immutable receipt.
pub(crate) fn load_receipt_event_id(
    conn: &Connection,
    projection_id: &ProjectionId,
) -> Result<Option<EventId>, SoulError> {
    conn.query_row(
        "SELECT created_event_id FROM projection_receipts WHERE projection_id = ?1",
        [projection_id.as_str()],
        |row| row.get::<_, String>(0),
    )
    .optional()
    .map(|value| value.map(EventId::new))
    .map_err(Into::into)
}

pub(crate) fn insert_receipt(
    conn: &Connection,
    receipt: &ProjectionReceipt,
    created_event_id: &EventId,
) -> Result<(), SoulError> {
    conn.execute(
        "INSERT INTO projection_receipts (
            projection_id, checkpoint_id, execution_id, runtime_json,
            projection_version, renderer_version, rendered_digest,
            rendered_byte_count, rendered_char_count, created_at,
            created_event_id, receipt_json
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12)",
        params![
            receipt.id.as_str(),
            receipt.checkpoint_id.as_str(),
            receipt.execution_id.as_str(),
            serde_json::to_string(&receipt.runtime)?,
            to_i64(
                u64::from(receipt.projection_version),
                "projection_receipts.projection_version"
            )?,
            to_i64(
                u64::from(receipt.renderer_version),
                "projection_receipts.renderer_version"
            )?,
            receipt.rendered_digest.value.as_str(),
            usize_to_i64(
                receipt.rendered_byte_count,
                "projection_receipts.rendered_byte_count"
            )?,
            usize_to_i64(
                receipt.rendered_char_count,
                "projection_receipts.rendered_char_count"
            )?,
            receipt.created_at.as_unix_millis(),
            created_event_id.as_str(),
            serde_json::to_string(receipt)?,
        ],
    )?;
    for (position, state_ref) in receipt.selected_state_refs.iter().enumerate() {
        conn.execute(
            "INSERT INTO receipt_state_refs (
                projection_id, state_id, state_version, position
             ) VALUES (?1, ?2, ?3, ?4)",
            params![
                receipt.id.as_str(),
                state_ref.state_id.as_str(),
                to_i64(state_ref.version, "receipt_state_refs.state_version")?,
                usize_to_i64(position, "receipt_state_refs.position")?,
            ],
        )?;
    }
    Ok(())
}

fn load_receipt_edges(
    conn: &Connection,
    projection_id: &ProjectionId,
) -> Result<Vec<crate::handoff_model::StateRef>, SoulError> {
    let mut statement = conn.prepare(
        "SELECT state_id, state_version FROM receipt_state_refs
         WHERE projection_id = ?1 ORDER BY position ASC",
    )?;
    let rows = statement.query_map([projection_id.as_str()], |row| {
        Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
    })?;
    let mut refs = Vec::new();
    for row in rows {
        let (state_id, version) = row?;
        refs.push(crate::handoff_model::StateRef::new(
            StateId::new(state_id),
            u64::try_from(version).map_err(|_| SoulError::InvalidStoredValue {
                field: "receipt_state_refs.state_version",
                value: version.to_string(),
            })?,
        ));
    }
    Ok(refs)
}

fn to_i64(value: u64, field: &'static str) -> Result<i64, SoulError> {
    i64::try_from(value).map_err(|_| SoulError::InvalidStoredValue {
        field,
        value: value.to_string(),
    })
}

fn usize_to_i64(value: usize, field: &'static str) -> Result<i64, SoulError> {
    i64::try_from(value).map_err(|_| SoulError::InvalidStoredValue {
        field,
        value: value.to_string(),
    })
}
