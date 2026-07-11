//! SQLite connection, schema migration, and shared storage errors.

use crate::handoff_error::HandoffError;
use crate::migrations::migrate;
use crate::projection_error::ProjectionError;
use crate::state_model::StateValidationError;
use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};
use thiserror::Error;
use tsukumo_kernel::{EventContractError, EventId, Timestamp};

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
    #[error(transparent)]
    Handoff(#[from] HandoffError),
    #[error(transparent)]
    Projection(#[from] ProjectionError),
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

fn ensure_legacy_snapshot_files(data_dir: &Path) -> Result<(), SoulError> {
    for (file_name, title) in [("MEMORY.md", "# MEMORY\n\n"), ("USER.md", "# USER\n\n")] {
        let path = data_dir.join(file_name);
        if !path.exists() {
            fs::write(path, title)?;
        }
    }
    Ok(())
}
