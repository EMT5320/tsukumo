#![cfg(windows)]

use rusqlite::Connection;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Child, Command};
use std::thread;
use std::time::{Duration, Instant};
use tempfile::tempdir;
use tsukumo_host::{load_presentation_pack, HostProductController, PresentationPackSource};

const CHILD_DATA_DIR: &str = "TSUKUMO_HOT_JOURNAL_CHILD_DATA_DIR";
const CHILD_TEST: &str = "hot_delete_journal_recovers_before_product_open";
const JOURNAL_MAGIC: [u8; 8] = [0xd9, 0xd5, 0x05, 0xf9, 0x20, 0xa1, 0x63, 0xd7];

#[test]
fn hot_delete_journal_recovers_before_product_open() {
    if let Some(data_dir) = std::env::var_os(CHILD_DATA_DIR) {
        run_hot_transaction_child(PathBuf::from(data_dir));
    }

    // Given: a committed value and a killed writer that leaves a DELETE-mode hot journal.
    let directory = tempdir().expect("create hot-journal data directory");
    seed_committed_database(directory.path());
    let mut child = spawn_hot_transaction_child(directory.path());
    wait_for_hot_journal(directory.path(), &mut child);
    child.kill().expect("terminate hot-journal writer");
    child.wait().expect("reap hot-journal writer");
    fs::remove_file(directory.path().join("writer-ready"))
        .expect("remove hot-journal readiness marker");
    assert!(is_hot_journal(&directory.path().join("soul.db-journal")));

    // When: the product opens while its local path guard protects SQLite sidecars.
    let pack = load_presentation_pack(&PresentationPackSource::EmbeddedDefault)
        .expect("load embedded presentation pack");
    let controller = HostProductController::open(directory.path(), &pack)
        .expect("recover hot journal and open product controller");

    // Then: SQLite rolled back the interrupted value and restored multi-connection access.
    assert!(!is_hot_journal(&directory.path().join("soul.db-journal")));
    let connection = Connection::open(directory.path().join("soul.db"))
        .expect("open recovered database directly");
    let value = connection
        .query_row("SELECT value FROM hot_recovery", [], |row| {
            row.get::<_, String>(0)
        })
        .expect("read recovered committed value");
    assert_eq!(value, "committed");
    let second_journal_mode = connection
        .query_row("PRAGMA journal_mode = PERSIST", [], |row| {
            row.get::<_, String>(0)
        })
        .expect("set second connection journal mode");
    assert!(second_journal_mode.eq_ignore_ascii_case("persist"));
    assert_eq!(
        connection
            .execute("UPDATE hot_recovery SET value = 'post-recovery'", [],)
            .expect("write through second connection"),
        1
    );
    drop(connection);
    drop(controller);
}

fn seed_committed_database(data_dir: &Path) {
    let connection = Connection::open(data_dir.join("soul.db")).expect("open seed database");
    connection
        .execute_batch(
            "PRAGMA journal_mode = DELETE;
             PRAGMA synchronous = FULL;
             CREATE TABLE hot_recovery(value TEXT NOT NULL);
             INSERT INTO hot_recovery(value) VALUES ('committed');",
        )
        .expect("seed committed database");
}

fn spawn_hot_transaction_child(data_dir: &Path) -> Child {
    Command::new(std::env::current_exe().expect("resolve integration-test executable"))
        .args(["--exact", CHILD_TEST, "--nocapture"])
        .env(CHILD_DATA_DIR, data_dir)
        .spawn()
        .expect("spawn hot-journal writer")
}

fn wait_for_hot_journal(data_dir: &Path, child: &mut Child) {
    let deadline = Instant::now() + Duration::from_secs(10);
    let marker = data_dir.join("writer-ready");
    let journal = data_dir.join("soul.db-journal");
    loop {
        if marker.exists() && is_hot_journal(&journal) {
            return;
        }
        if let Some(status) = child.try_wait().expect("poll hot-journal writer") {
            panic!("hot-journal writer exited early: {status}");
        }
        if Instant::now() >= deadline {
            child.kill().expect("terminate timed-out writer");
            child.wait().expect("reap timed-out writer");
            panic!("timed out waiting for a valid hot journal");
        }
        thread::sleep(Duration::from_millis(20));
    }
}

fn is_hot_journal(path: &Path) -> bool {
    fs::read(path).is_ok_and(|bytes| bytes.len() > 512 && bytes.starts_with(&JOURNAL_MAGIC))
}

fn run_hot_transaction_child(data_dir: PathBuf) -> ! {
    let connection = Connection::open(data_dir.join("soul.db")).expect("open writer database");
    connection
        .execute_batch(
            "PRAGMA journal_mode = DELETE;
             PRAGMA synchronous = FULL;
             BEGIN IMMEDIATE;
             UPDATE hot_recovery SET value = 'interrupted';",
        )
        .expect("start interrupted write");
    connection.cache_flush().expect("flush dirty database page");
    assert!(is_hot_journal(&data_dir.join("soul.db-journal")));
    fs::write(data_dir.join("writer-ready"), b"ready").expect("write readiness marker");
    loop {
        thread::sleep(Duration::from_secs(60));
    }
}
