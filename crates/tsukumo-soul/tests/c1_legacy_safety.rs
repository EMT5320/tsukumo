//! C1 legacy secret projection safety regressions.

use rusqlite::Connection;
use tempfile::tempdir;
use tsukumo_kernel::{QuestId, SpiritId, Timestamp};
use tsukumo_soul::{BriefCompiler, BriefOptions, LegacyImportContext, SoulError, SoulStore};

const SECRET: &str = "api_key=SENTINEL-Aa1234567890_SECRET_VALUE";

#[test]
fn new_legacy_writes_reject_sensitive_material() {
    // Given: a new compatibility store.
    let directory = tempdir().expect("create legacy safety directory");
    let mut store = SoulStore::open(directory.path()).expect("open legacy safety store");

    // When: a caller attempts to persist credential-like legacy text.
    let error = store
        .remember_user("secret-row", "session-secret", SECRET)
        .expect_err("sensitive legacy write must fail");

    // Then: the typed boundary rejects it before SQLite or snapshots change.
    assert!(matches!(error, SoulError::SensitiveLegacyContent));
    assert!(!store
        .read_snapshot(tsukumo_soul::FactKind::User)
        .expect("read safe user snapshot")
        .contains("SENTINEL"));
}

#[test]
fn imported_sensitive_legacy_rows_never_enter_fallback_or_snapshot() {
    // Given: a pre-C1 database containing one sensitive fact written by old code.
    let directory = tempdir().expect("create legacy projection directory");
    let database_path = directory.path().join("soul.db");
    drop(SoulStore::open(directory.path()).expect("create legacy database"));
    let connection = Connection::open(&database_path).expect("open raw legacy database");
    connection
        .execute(
            "INSERT INTO facts (id, kind, text, session_id)
             VALUES ('secret-row', 'user', ?1, 'session-secret')",
            [SECRET],
        )
        .expect("seed sensitive legacy row");
    drop(connection);
    let mut store = SoulStore::open(directory.path()).expect("reopen legacy database");

    // When: import reviews the row and fallback compiles after the expected skip.
    let report = store
        .import_legacy_facts(LegacyImportContext {
            quest_id: QuestId::new("quest-legacy-safety"),
            spirit_id: SpiritId::new("yuka"),
            occurred_at: Timestamp::from_unix_millis(1_750_000_900_000),
        })
        .expect("review sensitive legacy import");
    let brief = BriefCompiler::new(BriefOptions::default())
        .compile(&store)
        .expect("compile safe legacy fallback");
    let snapshot = store
        .read_snapshot(tsukumo_soul::FactKind::User)
        .expect("read rebuilt user snapshot");

    // Then: review stays observable while every runtime-facing projection omits it.
    assert_eq!(report.skipped.len(), 1);
    assert!(!brief.contains("SENTINEL"));
    assert!(!snapshot.contains("SENTINEL"));
}
