//! Append-only Chronicle persistence and deterministic replay.

use crate::storage::{SoulError, SoulStore};
use rusqlite::{params, Connection, OptionalExtension};
use tsukumo_kernel::{
    validate_kernel_event, CorrelationId, EventId, ExecutionId, KernelEvent, QuestId, SessionId,
    SpiritId,
};

/// Result of appending an event with an idempotent event ID.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AppendOutcome {
    Inserted { sequence: i64 },
    Duplicate { sequence: i64 },
}

/// Chronicle event paired with its database-assigned replay sequence.
#[derive(Debug, Clone, PartialEq)]
pub struct PersistedEvent {
    pub sequence: i64,
    pub event: KernelEvent,
}

/// Typed filters for ordered Chronicle replay.
#[derive(Debug, Clone)]
pub struct ChronicleQuery {
    quest_id: Option<QuestId>,
    session_id: Option<SessionId>,
    spirit_id: Option<SpiritId>,
    execution_id: Option<ExecutionId>,
    correlation_id: Option<CorrelationId>,
    after_sequence: i64,
    limit: usize,
}

impl Default for ChronicleQuery {
    fn default() -> Self {
        Self {
            quest_id: None,
            session_id: None,
            spirit_id: None,
            execution_id: None,
            correlation_id: None,
            after_sequence: 0,
            limit: 1_000,
        }
    }
}

impl ChronicleQuery {
    pub fn for_quest(mut self, quest_id: QuestId) -> Self {
        self.quest_id = Some(quest_id);
        self
    }

    pub fn for_session(mut self, session_id: SessionId) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn for_spirit(mut self, spirit_id: SpiritId) -> Self {
        self.spirit_id = Some(spirit_id);
        self
    }

    pub fn for_execution(mut self, execution_id: ExecutionId) -> Self {
        self.execution_id = Some(execution_id);
        self
    }

    pub fn for_correlation(mut self, correlation_id: CorrelationId) -> Self {
        self.correlation_id = Some(correlation_id);
        self
    }

    pub fn after(mut self, sequence: i64) -> Self {
        self.after_sequence = sequence.max(0);
        self
    }

    pub fn limited_to(mut self, limit: usize) -> Self {
        self.limit = limit.max(1);
        self
    }
}

impl SoulStore {
    /// Appends one immutable event or returns its idempotent prior sequence.
    pub fn append_event(&mut self, event: &KernelEvent) -> Result<AppendOutcome, SoulError> {
        append_event_in(&self.conn, event)
    }

    /// Loads one event by its globally unique ID.
    pub fn event(&self, event_id: &EventId) -> Result<Option<PersistedEvent>, SoulError> {
        load_event_in(&self.conn, event_id)
    }

    /// Replays events in database sequence order with typed filters.
    pub fn replay_events(&self, query: ChronicleQuery) -> Result<Vec<PersistedEvent>, SoulError> {
        let limit = i64::try_from(query.limit).unwrap_or(i64::MAX);
        let quest_id = query.quest_id.as_ref().map(QuestId::as_str);
        let session_id = query.session_id.as_ref().map(SessionId::as_str);
        let spirit_id = query.spirit_id.as_ref().map(SpiritId::as_str);
        let execution_id = query.execution_id.as_ref().map(ExecutionId::as_str);
        let correlation_id = query.correlation_id.as_ref().map(CorrelationId::as_str);
        let mut statement = self.conn.prepare(
            "SELECT sequence, event_json
             FROM chronicle_events
             WHERE (?1 IS NULL OR quest_id = ?1)
               AND (?2 IS NULL OR session_id = ?2)
               AND (?3 IS NULL OR spirit_id = ?3)
               AND (?4 IS NULL OR execution_id = ?4)
               AND (?5 IS NULL OR correlation_id = ?5)
               AND sequence > ?6
             ORDER BY sequence ASC
             LIMIT ?7",
        )?;
        let rows = statement.query_map(
            params![
                quest_id,
                session_id,
                spirit_id,
                execution_id,
                correlation_id,
                query.after_sequence,
                limit
            ],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
        )?;

        let mut events = Vec::new();
        for row in rows {
            let (sequence, event_json) = row?;
            events.push(PersistedEvent {
                sequence,
                event: decode_stored_event(&event_json)?,
            });
        }
        Ok(events)
    }

    /// Loads the newest bounded Chronicle tail in chronological order.
    pub fn replay_recent_events(&self, limit: usize) -> Result<Vec<PersistedEvent>, SoulError> {
        const MAX_RECENT_EVENTS: usize = 1_000;
        const MAX_RECENT_BYTES: usize = 32 * 1024 * 1024;
        if limit == 0 {
            return Ok(Vec::new());
        }
        let limit = limit.min(MAX_RECENT_EVENTS);
        let sql_limit = i64::try_from(limit).unwrap_or(i64::MAX);
        let mut statement = self.conn.prepare(
            "SELECT sequence, event_json
             FROM chronicle_events
             ORDER BY sequence DESC
             LIMIT ?1",
        )?;
        let rows = statement.query_map([sql_limit], |row| {
            Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?))
        })?;
        let mut events = Vec::with_capacity(limit);
        let mut retained_bytes = 0usize;
        for row in rows {
            let (sequence, event_json) = row?;
            retained_bytes = retained_bytes.saturating_add(event_json.len());
            if retained_bytes > MAX_RECENT_BYTES {
                return Err(SoulError::ChronicleReadBudgetExceeded {
                    event_count: events.len().saturating_add(1),
                    byte_count: retained_bytes,
                    maximum_events: limit,
                    maximum_bytes: MAX_RECENT_BYTES,
                });
            }
            events.push(PersistedEvent {
                sequence,
                event: decode_stored_event(&event_json)?,
            });
        }
        events.reverse();
        Ok(events)
    }
}

pub(crate) fn append_event_in(
    conn: &Connection,
    event: &KernelEvent,
) -> Result<AppendOutcome, SoulError> {
    validate_kernel_event(event)?;

    let event_json = serde_json::to_string(event)?;
    let existing = conn
        .query_row(
            "SELECT sequence, event_json FROM chronicle_events WHERE event_id = ?1",
            [event.event_id.as_str()],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
        )
        .optional()?;
    if let Some((sequence, stored_json)) = existing {
        if stored_json == event_json {
            return Ok(AppendOutcome::Duplicate { sequence });
        }
        return Err(SoulError::ConflictingEvent {
            event_id: event.event_id.clone(),
        });
    }

    conn.execute(
        "INSERT INTO chronicle_events (
            event_id, schema_version, occurred_at, quest_id, session_id,
            spirit_id, execution_id, causation_id, correlation_id, event_json
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
        params![
            event.event_id.as_str(),
            i64::from(event.schema_version),
            event.occurred_at.as_unix_millis(),
            event.quest_id.as_str(),
            event.session_id.as_str(),
            event.spirit_id.as_str(),
            event.execution_id.as_ref().map(ExecutionId::as_str),
            event.causation_id.as_ref().map(EventId::as_str),
            event.correlation_id.as_ref().map(|id| id.as_str()),
            event_json,
        ],
    )?;
    Ok(AppendOutcome::Inserted {
        sequence: conn.last_insert_rowid(),
    })
}

pub(crate) fn load_event_in(
    conn: &Connection,
    event_id: &EventId,
) -> Result<Option<PersistedEvent>, SoulError> {
    let stored = conn
        .query_row(
            "SELECT sequence, event_json FROM chronicle_events WHERE event_id = ?1",
            [event_id.as_str()],
            |row| Ok((row.get::<_, i64>(0)?, row.get::<_, String>(1)?)),
        )
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

pub(crate) fn decode_stored_event(event_json: &str) -> Result<KernelEvent, SoulError> {
    let event = serde_json::from_str::<KernelEvent>(event_json)?;
    validate_kernel_event(&event)?;
    Ok(event)
}
