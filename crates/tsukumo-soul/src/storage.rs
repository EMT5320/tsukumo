//! SQLite connection, schema migration, and shared storage errors.

use crate::state_model::StateValidationError;
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tsukumo_kernel::{EventContractError, EventId, Timestamp};

pub(crate) const CURRENT_SCHEMA_VERSION: i64 = 2;

#[derive(Debug, Error)]
pub enum SoulError {
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
    #[error("sqlite error: {0}")]
    Sqlite(#[from] rusqlite::Error),
    #[error("json error: {0}")]
    Json(#[from] serde_json::Error),
    #[error("system clock is before the Unix epoch: {0}")]
    Clock(#[from] std::time::SystemTimeError),
    #[error("system timestamp does not fit signed milliseconds")]
    TimestampOutOfRange,
    #[error("invalid fact kind: {0}")]
    InvalidKind(String),
    #[error("empty fact text")]
    EmptyText,
    #[error("legacy facts exceed the safe import budget")]
    LegacyBudgetExceeded,
    #[error("legacy fact metadata is invalid")]
    InvalidLegacyMetadata,
    #[error("legacy fact contains sensitive material")]
    SensitiveLegacyContent,
    #[error("event {event_id} already exists with different content")]
    ConflictingEvent { event_id: EventId },
    #[error(transparent)]
    EventContract(#[from] EventContractError),
    #[error("database schema {found} is newer than supported version {supported}")]
    NewerDatabaseSchema { found: i64, supported: i64 },
    #[error("invalid stored {field}: {value}")]
    InvalidStoredValue { field: &'static str, value: String },
    #[error(transparent)]
    StateValidation(#[from] StateValidationError),
}

/// SQLite-backed relationship store and Chronicle authority.
pub struct SoulStore {
    pub(crate) data_dir: PathBuf,
    pub(crate) conn: Connection,
}

impl SoulStore {
    /// Opens or creates the authoritative SQLite store.
    pub fn open(data_dir: impl AsRef<Path>) -> Result<Self, SoulError> {
        let data_dir = data_dir.as_ref().to_path_buf();
        fs::create_dir_all(&data_dir)?;
        fs::create_dir_all(data_dir.join("skills"))?;

        let mut conn = Connection::open(data_dir.join("soul.db"))?;
        conn.execute_batch("PRAGMA foreign_keys = ON;")?;
        migrate(&mut conn)?;
        ensure_legacy_snapshot_files(&data_dir)?;

        let store = Self { data_dir, conn };
        store.rewrite_legacy_snapshots()?;
        Ok(store)
    }

    pub fn data_dir(&self) -> &Path {
        &self.data_dir
    }

    pub fn database_path(&self) -> PathBuf {
        self.data_dir.join("soul.db")
    }

    pub fn skills_dir(&self) -> PathBuf {
        self.data_dir.join("skills")
    }
}

pub(crate) fn current_timestamp() -> Result<Timestamp, SoulError> {
    let duration = SystemTime::now().duration_since(UNIX_EPOCH)?;
    let milliseconds =
        i64::try_from(duration.as_millis()).map_err(|_| SoulError::TimestampOutOfRange)?;
    Ok(Timestamp::from_unix_millis(milliseconds))
}

fn migrate(conn: &mut Connection) -> Result<(), SoulError> {
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
    if found < 1 {
        let transaction = conn.transaction()?;
        transaction.execute_batch(MIGRATION_1)?;
        transaction.execute("INSERT INTO schema_migrations (version) VALUES (1)", [])?;
        transaction.commit()?;
    }
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
    Ok(())
}

fn ensure_legacy_snapshot_files(data_dir: &Path) -> Result<(), SoulError> {
    for (file_name, title) in [("MEMORY.md", "# MEMORY\n\n"), ("USER.md", "# USER\n\n")] {
        let path = data_dir.join(file_name);
        if !path.exists() {
            fs::write(path, title)?;
        }
    }
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
CREATE INDEX chronicle_quest_sequence
    ON chronicle_events(quest_id, sequence);
CREATE INDEX chronicle_session_sequence
    ON chronicle_events(session_id, sequence);
CREATE INDEX chronicle_spirit_sequence
    ON chronicle_events(spirit_id, sequence);
CREATE INDEX chronicle_correlation_sequence
    ON chronicle_events(correlation_id, sequence);
CREATE TRIGGER chronicle_no_update
BEFORE UPDATE ON chronicle_events BEGIN
    SELECT RAISE(ABORT, 'chronicle_events is append-only');
END;
CREATE TRIGGER chronicle_no_delete
BEFORE DELETE ON chronicle_events BEGIN
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
CREATE INDEX state_active_lookup
    ON state_records(state_key, scope_key, status);
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
          AND json_extract(chronicle_events.event_json, '$.payload.prior_state_id')
              = state_records.state_id
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
      AND json_extract(chronicle_events.event_json, '$.payload.state_id')
          = state_records.state_id
)
WHERE status = 'revoked';
ALTER TABLE legacy_import_runs
    ADD COLUMN importer_version INTEGER NOT NULL DEFAULT 0;
"#;
