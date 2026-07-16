//! SQLite connection, schema migration, and shared storage errors.

use crate::handoff_error::HandoffError;
use crate::migrations::migrate;
use crate::projection_error::ProjectionError;
use crate::state_model::StateValidationError;
use rusqlite::{Connection, OpenFlags};
use std::fs;
use std::io;
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
    #[error(
        "Chronicle read exceeds budget: {event_count}/{maximum_events} events, {byte_count}/{maximum_bytes} bytes"
    )]
    ChronicleReadBudgetExceeded {
        event_count: usize,
        byte_count: usize,
        maximum_events: usize,
        maximum_bytes: usize,
    },
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
        let requested_data_dir = data_dir.as_ref().to_path_buf();
        fs::create_dir_all(&requested_data_dir)?;
        if fs::symlink_metadata(&requested_data_dir)?
            .file_type()
            .is_symlink()
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "data directory cannot be a symbolic link",
            )
            .into());
        }
        // macOS exposes its normal temporary root through `/var`, which is an
        // alias of `/private/var`. Resolve ancestor aliases before combining
        // the path with SQLITE_OPEN_NOFOLLOW so ordinary local stores remain
        // usable while the final database file still cannot be a symlink.
        let data_dir = fs::canonicalize(requested_data_dir)?;
        fs::create_dir_all(data_dir.join("skills"))?;

        // Keep the main file no-follow and use one fixed persistent rollback sidecar.
        let flags = OpenFlags::default() | OpenFlags::SQLITE_OPEN_NOFOLLOW;
        let mut conn = Connection::open_with_flags(data_dir.join("soul.db"), flags)?;
        // Recover legacy DELETE-mode hot journals in place while Host keeps its
        // no-delete-share journal capability alive.
        let recovery_locking_mode =
            conn.query_row("PRAGMA main.locking_mode = EXCLUSIVE", [], |row| {
                row.get::<_, String>(0)
            })?;
        if !recovery_locking_mode.eq_ignore_ascii_case("exclusive") {
            return Err(SoulError::InvalidStoredValue {
                field: "recovery_locking_mode",
                value: recovery_locking_mode,
            });
        }
        let journal_mode = conn.query_row("PRAGMA main.journal_mode = PERSIST", [], |row| {
            row.get::<_, String>(0)
        })?;
        if !journal_mode.eq_ignore_ascii_case("persist") {
            return Err(SoulError::InvalidStoredValue {
                field: "journal_mode",
                value: journal_mode,
            });
        }
        // Normal locking restores multi-connection access after recovery.
        let normal_locking_mode =
            conn.query_row("PRAGMA main.locking_mode = NORMAL", [], |row| {
                row.get::<_, String>(0)
            })?;
        if !normal_locking_mode.eq_ignore_ascii_case("normal") {
            return Err(SoulError::InvalidStoredValue {
                field: "locking_mode",
                value: normal_locking_mode,
            });
        }
        // Finish a normal-mode read so the pager releases recovery's exclusive lock.
        let _: i64 = conn.query_row("SELECT COUNT(*) FROM main.sqlite_schema", [], |row| {
            row.get(0)
        })?;
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

#[cfg(all(test, unix))]
mod tests {
    use super::*;
    use std::os::unix::fs::symlink;
    use tempfile::tempdir;

    #[test]
    fn symlinked_ancestor_is_resolved_before_sqlite_nofollow_open() {
        let root = tempdir().expect("create storage test root");
        let real_parent = root.path().join("real");
        fs::create_dir(&real_parent).expect("create real parent");
        let alias_parent = root.path().join("alias");
        symlink(&real_parent, &alias_parent).expect("create parent alias");

        let store =
            SoulStore::open(alias_parent.join("data")).expect("open through aliased ancestor");

        assert_eq!(
            store.data_dir(),
            fs::canonicalize(real_parent.join("data"))
                .expect("canonicalize expected data directory")
        );
        assert!(store.database_path().is_file());
    }

    #[test]
    fn symlinked_data_root_is_rejected() {
        let root = tempdir().expect("create storage test root");
        let real_data = root.path().join("real-data");
        fs::create_dir(&real_data).expect("create real data directory");
        let alias_data = root.path().join("alias-data");
        symlink(&real_data, &alias_data).expect("create data-root alias");

        let error = SoulStore::open(&alias_data)
            .err()
            .expect("symlinked data root must fail");

        assert!(matches!(
            error,
            SoulError::Io(ref source) if source.kind() == io::ErrorKind::InvalidInput
        ));
        assert!(!real_data.join("soul.db").exists());
    }
}
