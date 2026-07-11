//! SQLite encoding and queries for canonical state records.

use crate::state_codec::{
    kind_value, parse_kind, parse_status, parse_strength, status_value, strength_value,
};
use crate::state_model::{ExtractionProvenance, StateKey, StateRecord, StateScope, StateStatus};
use crate::storage::SoulError;
use rusqlite::{params, Connection, OptionalExtension};
use tsukumo_kernel::{EventId, PersistedText, StateId, Timestamp};

pub(crate) fn load_state(
    conn: &Connection,
    state_id: &StateId,
) -> Result<Option<StateRecord>, SoulError> {
    let row = conn
        .query_row(
            "SELECT state_key, kind, scope_json, content, strength, status,
                    provenance_json, version, created_at, expires_at,
                    deactivated_at, supersedes_state_id
             FROM state_records WHERE state_id = ?1",
            [state_id.as_str()],
            |row| {
                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    row.get::<_, String>(4)?,
                    row.get::<_, String>(5)?,
                    row.get::<_, String>(6)?,
                    row.get::<_, i64>(7)?,
                    row.get::<_, i64>(8)?,
                    row.get::<_, Option<i64>>(9)?,
                    row.get::<_, Option<i64>>(10)?,
                    row.get::<_, Option<String>>(11)?,
                ))
            },
        )
        .optional()?;
    let Some(row) = row else {
        return Ok(None);
    };

    let mut statement = conn.prepare(
        "SELECT event_id FROM state_evidence
         WHERE state_id = ?1 ORDER BY event_id ASC",
    )?;
    let rows = statement.query_map([state_id.as_str()], |row| row.get::<_, String>(0))?;
    let mut evidence_refs = Vec::new();
    for event_id in rows {
        evidence_refs.push(EventId::new(event_id?));
    }

    Ok(Some(StateRecord {
        state_id: state_id.clone(),
        state_key: StateKey::new(row.0),
        kind: parse_kind(&row.1)?,
        scope: serde_json::from_str::<StateScope>(&row.2)?,
        content: PersistedText::from_reviewed(row.3),
        strength: parse_strength(&row.4)?,
        status: parse_status(&row.5)?,
        evidence_refs,
        provenance: serde_json::from_str::<ExtractionProvenance>(&row.6)?,
        version: u64::try_from(row.7).map_err(|_| SoulError::InvalidStoredValue {
            field: "state_records.version",
            value: row.7.to_string(),
        })?,
        created_at: Timestamp::from_unix_millis(row.8),
        expires_at: row.9.map(Timestamp::from_unix_millis),
        deactivated_at: row.10.map(Timestamp::from_unix_millis),
        supersedes_state_id: row.11.map(StateId::new),
    }))
}

pub(crate) fn find_active_state(
    conn: &Connection,
    key: &StateKey,
    scope_key: &str,
    as_of: Timestamp,
) -> Result<Option<StateRecord>, SoulError> {
    let state_id = conn
        .query_row(
            "SELECT state_id FROM state_records
             WHERE state_key = ?1 AND scope_key = ?2
               AND created_at <= ?3
               AND (expires_at IS NULL OR expires_at > ?3)
               AND (deactivated_at IS NULL OR deactivated_at > ?3)
             ORDER BY version DESC LIMIT 1",
            params![key.as_str(), scope_key, as_of.as_unix_millis()],
            |row| row.get::<_, String>(0),
        )
        .optional()?;
    state_id
        .map(|value| load_state(conn, &StateId::new(value)))
        .transpose()
        .map(Option::flatten)
}

pub(crate) fn has_state_created_after(
    conn: &Connection,
    key: &StateKey,
    scope_key: &str,
    created_at: Timestamp,
) -> Result<bool, SoulError> {
    conn.query_row(
        "SELECT EXISTS(
            SELECT 1 FROM state_records
            WHERE state_key = ?1 AND scope_key = ?2 AND created_at > ?3
        )",
        params![key.as_str(), scope_key, created_at.as_unix_millis()],
        |row| row.get::<_, bool>(0),
    )
    .map_err(Into::into)
}

pub(crate) fn next_version(
    conn: &Connection,
    key: &StateKey,
    scope_key: &str,
) -> Result<u64, SoulError> {
    let version = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM state_records
         WHERE state_key = ?1 AND scope_key = ?2",
        params![key.as_str(), scope_key],
        |row| row.get::<_, i64>(0),
    )?;
    let version = u64::try_from(version).map_err(|_| SoulError::InvalidStoredValue {
        field: "state_records.version",
        value: version.to_string(),
    })?;
    version
        .checked_add(1)
        .ok_or_else(|| SoulError::InvalidStoredValue {
            field: "state_records.version",
            value: version.to_string(),
        })
}

pub(crate) fn insert_state(conn: &Connection, record: &StateRecord) -> Result<(), SoulError> {
    let scope_key = record.scope.canonical_key()?;
    let version = i64::try_from(record.version).map_err(|_| SoulError::InvalidStoredValue {
        field: "state_records.version",
        value: record.version.to_string(),
    })?;
    conn.execute(
        "INSERT INTO state_records (
            state_id, state_key, scope_key, scope_json, kind, strength, status,
            content, provenance_json, version, created_at, expires_at,
            deactivated_at, supersedes_state_id
         ) VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11, ?12, ?13, ?14)",
        params![
            record.state_id.as_str(),
            record.state_key.as_str(),
            scope_key,
            serde_json::to_string(&record.scope)?,
            kind_value(record.kind),
            strength_value(record.strength),
            status_value(record.status),
            record.content.as_str(),
            serde_json::to_string(&record.provenance)?,
            version,
            record.created_at.as_unix_millis(),
            record.expires_at.map(Timestamp::as_unix_millis),
            record.deactivated_at.map(Timestamp::as_unix_millis),
            record.supersedes_state_id.as_ref().map(StateId::as_str),
        ],
    )?;
    for event_id in &record.evidence_refs {
        conn.execute(
            "INSERT INTO state_evidence (state_id, event_id) VALUES (?1, ?2)",
            params![record.state_id.as_str(), event_id.as_str()],
        )?;
    }
    Ok(())
}

pub(crate) fn update_status(
    conn: &Connection,
    state_id: &StateId,
    status: StateStatus,
    deactivated_at: Timestamp,
) -> Result<(), SoulError> {
    conn.execute(
        "UPDATE state_records SET status = ?1, deactivated_at = ?2 WHERE state_id = ?3",
        params![
            status_value(status),
            deactivated_at.as_unix_millis(),
            state_id.as_str()
        ],
    )?;
    Ok(())
}

pub(crate) fn list_states(
    conn: &Connection,
    active_at: Option<Timestamp>,
) -> Result<Vec<StateRecord>, SoulError> {
    let mut statement = conn.prepare(
        "SELECT state_id FROM state_records
         WHERE (?1 IS NULL OR (
             created_at <= ?1
             AND (expires_at IS NULL OR expires_at > ?1)
             AND (deactivated_at IS NULL OR deactivated_at > ?1)
         ))
         ORDER BY state_key ASC, version ASC",
    )?;
    let as_of = active_at.map(Timestamp::as_unix_millis);
    let rows = statement.query_map([as_of], |row| row.get::<_, String>(0))?;
    let mut states = Vec::new();
    for state_id in rows {
        let state_id = StateId::new(state_id?);
        if let Some(record) = load_state(conn, &state_id)? {
            states.push(record);
        }
    }
    Ok(states)
}
