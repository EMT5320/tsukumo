//! Ordered additive SQLite migrations for the durable soul authority.

use crate::storage::SoulError;
use rusqlite::Connection;

pub(crate) const CURRENT_SCHEMA_VERSION: i64 = 3;

pub(crate) fn migrate(conn: &mut Connection) -> Result<(), SoulError> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_migrations (
            version INTEGER PRIMARY KEY NOT NULL,
            applied_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
        );",
    )?;
    let found = conn.query_row(
        "SELECT COALESCE(MAX(version), 0) FROM schema_migrations",
        [],
        |row| row.get::<_, i64>(0),
    )?;
    if found > CURRENT_SCHEMA_VERSION {
        return Err(SoulError::NewerDatabaseSchema {
            found,
            supported: CURRENT_SCHEMA_VERSION,
        });
    }
    apply_migration(conn, found, 1, MIGRATION_1)?;
    if found < 2 {
        let transaction = conn.transaction()?;
        transaction.execute_batch(MIGRATION_2)?;
        let missing_deactivation = transaction.query_row(
            "SELECT COUNT(*) FROM state_records
             WHERE status IN ('superseded', 'revoked') AND deactivated_at IS NULL",
            [],
            |row| row.get::<_, i64>(0),
        )?;
        if missing_deactivation != 0 {
            return Err(SoulError::InvalidStoredValue {
                field: "state_records.deactivated_at",
                value: format!("{missing_deactivation} inactive rows cannot be reconstructed"),
            });
        }
        transaction.execute("INSERT INTO schema_migrations (version) VALUES (2)", [])?;
        transaction.commit()?;
    }
    apply_migration(conn, found, 3, MIGRATION_3)?;
    Ok(())
}

fn apply_migration(
    conn: &mut Connection,
    found: i64,
    version: i64,
    sql: &str,
) -> Result<(), SoulError> {
    if found >= version {
        return Ok(());
    }
    let transaction = conn.transaction()?;
    transaction.execute_batch(sql)?;
    transaction.execute(
        "INSERT INTO schema_migrations (version) VALUES (?1)",
        [version],
    )?;
    transaction.commit()?;
    Ok(())
}

const MIGRATION_1: &str = r#"
CREATE TABLE IF NOT EXISTS facts (
    id TEXT PRIMARY KEY NOT NULL,
    kind TEXT NOT NULL,
    text TEXT NOT NULL,
    session_id TEXT NOT NULL,
    created_at INTEGER NOT NULL DEFAULT (strftime('%s','now'))
);
CREATE VIRTUAL TABLE IF NOT EXISTS facts_fts USING fts5(
    text,
    id UNINDEXED,
    kind UNINDEXED,
    session_id UNINDEXED,
    content='facts',
    content_rowid='rowid'
);
CREATE TRIGGER IF NOT EXISTS facts_ai AFTER INSERT ON facts BEGIN
    INSERT INTO facts_fts(rowid, text, id, kind, session_id)
    VALUES (new.rowid, new.text, new.id, new.kind, new.session_id);
END;
CREATE TRIGGER IF NOT EXISTS facts_ad AFTER DELETE ON facts BEGIN
    INSERT INTO facts_fts(facts_fts, rowid, text, id, kind, session_id)
    VALUES ('delete', old.rowid, old.text, old.id, old.kind, old.session_id);
END;

CREATE TABLE chronicle_events (
    sequence INTEGER PRIMARY KEY AUTOINCREMENT,
    event_id TEXT UNIQUE NOT NULL,
    schema_version INTEGER NOT NULL,
    occurred_at INTEGER NOT NULL,
    quest_id TEXT NOT NULL,
    session_id TEXT NOT NULL,
    spirit_id TEXT NOT NULL,
    execution_id TEXT,
    causation_id TEXT,
    correlation_id TEXT,
    event_json TEXT NOT NULL
);
CREATE INDEX chronicle_quest_sequence ON chronicle_events(quest_id, sequence);
CREATE INDEX chronicle_session_sequence ON chronicle_events(session_id, sequence);
CREATE INDEX chronicle_spirit_sequence ON chronicle_events(spirit_id, sequence);
CREATE INDEX chronicle_correlation_sequence ON chronicle_events(correlation_id, sequence);
CREATE TRIGGER chronicle_no_update BEFORE UPDATE ON chronicle_events BEGIN
    SELECT RAISE(ABORT, 'chronicle_events is append-only');
END;
CREATE TRIGGER chronicle_no_delete BEFORE DELETE ON chronicle_events BEGIN
    SELECT RAISE(ABORT, 'chronicle_events is append-only');
END;

CREATE TABLE state_records (
    state_id TEXT PRIMARY KEY NOT NULL,
    state_key TEXT NOT NULL,
    scope_key TEXT NOT NULL,
    scope_json TEXT NOT NULL,
    kind TEXT NOT NULL,
    strength TEXT NOT NULL,
    status TEXT NOT NULL,
    content TEXT NOT NULL,
    provenance_json TEXT NOT NULL,
    version INTEGER NOT NULL,
    created_at INTEGER NOT NULL,
    expires_at INTEGER,
    supersedes_state_id TEXT,
    FOREIGN KEY(supersedes_state_id) REFERENCES state_records(state_id)
);
CREATE INDEX state_active_lookup ON state_records(state_key, scope_key, status);
CREATE TABLE legacy_import_runs (
    source_table TEXT PRIMARY KEY NOT NULL,
    completed_at INTEGER NOT NULL
);
CREATE TABLE state_evidence (
    state_id TEXT NOT NULL,
    event_id TEXT NOT NULL,
    PRIMARY KEY(state_id, event_id),
    FOREIGN KEY(state_id) REFERENCES state_records(state_id),
    FOREIGN KEY(event_id) REFERENCES chronicle_events(event_id)
);
CREATE VIRTUAL TABLE state_fts USING fts5(
    content,
    state_id UNINDEXED,
    state_key UNINDEXED
);
"#;

const MIGRATION_2: &str = r#"
ALTER TABLE state_records ADD COLUMN deactivated_at INTEGER;
UPDATE state_records
SET deactivated_at = COALESCE(
    (
        SELECT MIN(chronicle_events.occurred_at)
        FROM chronicle_events
        WHERE json_extract(chronicle_events.event_json, '$.payload.type') = 'state_lifecycle'
          AND json_extract(chronicle_events.event_json, '$.payload.action') = 'superseded'
          AND json_extract(chronicle_events.event_json, '$.payload.prior_state_id') = state_records.state_id
    ),
    (
        SELECT MIN(successor.created_at)
        FROM state_records AS successor
        WHERE successor.supersedes_state_id = state_records.state_id
    )
)
WHERE status = 'superseded';
UPDATE state_records
SET deactivated_at = (
    SELECT MIN(chronicle_events.occurred_at)
    FROM chronicle_events
    WHERE json_extract(chronicle_events.event_json, '$.payload.type') = 'state_lifecycle'
      AND json_extract(chronicle_events.event_json, '$.payload.action') = 'revoked'
      AND json_extract(chronicle_events.event_json, '$.payload.state_id') = state_records.state_id
)
WHERE status = 'revoked';
ALTER TABLE legacy_import_runs ADD COLUMN importer_version INTEGER NOT NULL DEFAULT 0;
"#;

const MIGRATION_3: &str = include_str!("migrations_v3.sql");
