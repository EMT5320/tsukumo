//! Bounded Chronicle reads for product authority outside the lossy UI tail.

use crate::chronicle::{decode_stored_event, PersistedEvent};
use crate::storage::{SoulError, SoulStore};
use rusqlite::{params, OptionalExtension};
use tsukumo_kernel::ExecutionId;

impl SoulStore {
    /// Replays every permission authority event within explicit event and byte budgets.
    pub fn replay_permission_events(
        &self,
        maximum_events: usize,
        maximum_bytes: usize,
    ) -> Result<Vec<PersistedEvent>, SoulError> {
        let (event_count, byte_count) = self.conn.query_row(
            "SELECT COUNT(*), COALESCE(SUM(length(CAST(event_json AS BLOB))), 0)
             FROM chronicle_events
             WHERE json_extract(event_json, '$.payload.type')
                   IN ('permission_requested', 'permission_decided')",
            [],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, i64>(1)?)),
        )?;
        let event_count = usize::try_from(event_count).unwrap_or(usize::MAX);
        let byte_count = usize::try_from(byte_count).unwrap_or(usize::MAX);
        if event_count > maximum_events || byte_count > maximum_bytes {
            return Err(SoulError::ChronicleReadBudgetExceeded {
                event_count,
                byte_count,
                maximum_events,
                maximum_bytes,
            });
        }

        let mut statement = self.conn.prepare(
            "SELECT sequence, event_json
             FROM chronicle_events
             WHERE json_extract(event_json, '$.payload.type')
                   IN ('permission_requested', 'permission_decided')
             ORDER BY sequence ASC",
        )?;
        let rows = statement.query_map([], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut events = Vec::with_capacity(event_count);
        for row in rows {
            let (sequence, event_json) = row?;
            events.push(PersistedEvent {
                sequence,
                event: decode_stored_event(&event_json)?,
            });
        }
        Ok(events)
    }

    /// Loads the newest event carrying a projection ID, optionally for one execution.
    pub fn latest_projection_event(
        &self,
        execution_id: Option<&ExecutionId>,
    ) -> Result<Option<PersistedEvent>, SoulError> {
        let execution_id = execution_id.map(ExecutionId::as_str);
        latest_matching_event(
            &self.conn,
            "json_extract(event_json, '$.payload.projection_id') IS NOT NULL",
            execution_id,
        )
    }

    /// Loads the newest durable checkpoint creation event.
    pub fn latest_checkpoint_event(&self) -> Result<Option<PersistedEvent>, SoulError> {
        latest_matching_event(
            &self.conn,
            "json_extract(event_json, '$.payload.type') = 'checkpoint_created'",
            None,
        )
    }

    /// Loads the newest coherent runtime lifecycle or outcome event.
    pub fn latest_runtime_status_event(&self) -> Result<Option<PersistedEvent>, SoulError> {
        latest_matching_event(
            &self.conn,
            "json_extract(event_json, '$.payload.type')
                 IN ('runtime_lifecycle', 'outcome')",
            None,
        )
    }
}

fn latest_matching_event(
    connection: &rusqlite::Connection,
    predicate: &str,
    execution_id: Option<&str>,
) -> Result<Option<PersistedEvent>, SoulError> {
    let sql = format!(
        "SELECT sequence, event_json
         FROM chronicle_events
         WHERE (?1 IS NULL OR execution_id = ?1)
           AND ({predicate})
         ORDER BY sequence DESC
         LIMIT 1"
    );
    let stored = connection
        .query_row(&sql, params![execution_id], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })
        .optional()?;
    stored
        .map(|(sequence, event_json)| {
            Ok(PersistedEvent {
                sequence,
                event: decode_stored_event(&event_json)?,
            })
        })
        .transpose()
}
