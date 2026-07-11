//! C1 idempotent legacy facts migration tests.

use rusqlite::Connection;
use tempfile::tempdir;
use tsukumo_kernel::{QuestId, SpiritId, StateId, Timestamp};
use tsukumo_soul::{
    BriefCompiler, BriefOptions, ChronicleQuery, EvidenceStrength, LegacyImportContext, SoulStore,
    StateKind, StateStatus, StateSubject,
};

#[test]
fn legacy_import_is_idempotent_and_never_escalates_strength_or_scope() {
    // Given: legacy USER and MEMORY facts written through the compatibility API.
    let directory = tempdir().expect("create legacy import directory");
    let mut store = SoulStore::open(directory.path()).expect("open legacy store");
    store
        .remember_user(
            "toolchain",
            "session-legacy",
            "Owner prefers GNU on Windows",
        )
        .expect("write legacy user fact");
    store
        .remember_memory("milestone", "session-legacy", "The fixture replay passed")
        .expect("write legacy memory fact");
    let context = LegacyImportContext {
        spirit_id: SpiritId::new("yuka"),
        quest_id: QuestId::new("quest-legacy-import"),
        occurred_at: Timestamp::from_unix_millis(1_750_000_500_000),
    };

    // When: the explicit migration runs, then an earlier-sorting row is added
    // and the retry uses a different caller context.
    let first = store
        .import_legacy_facts(context)
        .expect("import legacy facts");
    store
        .remember_memory(
            "aaa-earlier",
            "session-legacy",
            "A later legacy row must remain visible",
        )
        .expect("write later legacy fact");
    let transitional_brief = BriefCompiler::new(BriefOptions::default().with_query("later legacy"))
        .compile(&store)
        .expect("compile mixed canonical and legacy brief");
    assert!(transitional_brief.contains("later legacy"));
    let second = store
        .import_legacy_facts(LegacyImportContext {
            spirit_id: SpiritId::new("another-spirit"),
            quest_id: QuestId::new("another-quest"),
            occurred_at: Timestamp::from_unix_millis(1_760_000_000_000),
        })
        .expect("rerun legacy import with changed context");

    // Then: prior rows remain unchanged and only the new row is imported.
    assert_eq!(first.imported, 2);
    assert_eq!(first.unchanged, 0);
    assert!(first.skipped.is_empty());
    assert_eq!(second.imported, 1);
    assert_eq!(second.unchanged, 2);
    assert!(second.skipped.is_empty());

    for state_id in [
        StateId::new("legacy-import:state:toolchain"),
        StateId::new("legacy-import:state:milestone"),
    ] {
        let record = store
            .state(&state_id)
            .expect("query imported state")
            .expect("imported state exists");
        assert_eq!(record.kind, StateKind::Fact);
        assert_eq!(record.strength, EvidenceStrength::Imported);
        assert_eq!(record.status, StateStatus::Active);
        assert_eq!(record.scope.subject, StateSubject::Unresolved);
    }
    assert_eq!(
        store
            .replay_events(ChronicleQuery::default())
            .expect("replay legacy evidence")
            .len(),
        6
    );
    let brief = BriefCompiler::new(BriefOptions {
        char_cap: 200,
        top_k: 5,
        query: "GNU".into(),
    })
    .compile(&store)
    .expect("compile canonical brief");
    assert!(brief.contains("GNU"));
}

#[test]
fn pre_c1_facts_database_migrates_on_reopen_without_recreating_legacy_rows() {
    let directory = tempdir().expect("create pre-C1 database directory");
    let database_path = directory.path().join("soul.db");
    let connection = Connection::open(&database_path).expect("create legacy database");
    connection
        .execute_batch(
            "CREATE TABLE facts (
                id TEXT PRIMARY KEY NOT NULL,
                kind TEXT NOT NULL,
                text TEXT NOT NULL,
                session_id TEXT NOT NULL,
                created_at INTEGER NOT NULL
            );
            INSERT INTO facts (id, kind, text, session_id, created_at)
            VALUES ('legacy-old', 'memory', 'Old database fact', 'old-session', 100);",
        )
        .expect("seed pre-C1 facts schema");
    drop(connection);

    let mut store = SoulStore::open(directory.path()).expect("migrate pre-C1 database");
    let report = store
        .import_legacy_facts(LegacyImportContext {
            spirit_id: SpiritId::new("yuka"),
            quest_id: QuestId::new("quest-pre-c1"),
            occurred_at: Timestamp::from_unix_millis(1_750_001_200_000),
        })
        .expect("import old database row");
    assert_eq!(report.imported, 1);
    drop(store);

    let mut reopened = SoulStore::open(directory.path()).expect("reopen migrated database");
    let rerun = reopened
        .import_legacy_facts(LegacyImportContext {
            spirit_id: SpiritId::new("changed"),
            quest_id: QuestId::new("changed"),
            occurred_at: Timestamp::from_unix_millis(1_760_001_200_000),
        })
        .expect("rerun migrated database import");
    assert_eq!(rerun.unchanged, 1);
    assert!(rerun.skipped.is_empty());
    assert!(reopened
        .state(&StateId::new("legacy-import:state:legacy-old"))
        .expect("query old imported state")
        .is_some());
}

#[test]
fn changed_legacy_fact_remains_visible_when_reimport_requires_review() {
    // Given: one imported legacy fact whose source row is later edited in place.
    let directory = tempdir().expect("create changed legacy directory");
    let mut store = SoulStore::open(directory.path()).expect("open changed legacy store");
    store
        .remember_user("toolchain", "session-legacy", "Use GNU on Windows")
        .expect("write initial legacy fact");
    let context = LegacyImportContext {
        spirit_id: SpiritId::new("yuka"),
        quest_id: QuestId::new("quest-legacy-change"),
        occurred_at: Timestamp::from_unix_millis(1_750_002_000_000),
    };
    store
        .import_legacy_facts(context.clone())
        .expect("import initial legacy fact");
    store
        .remember_user("toolchain", "session-legacy", "Use MSVC on Windows")
        .expect("update legacy fact");

    // When: deterministic IDs expose the changed source as a review conflict.
    let report = store
        .import_legacy_facts(context)
        .expect("rerun changed legacy import");
    let brief = BriefCompiler::new(BriefOptions::default().with_query("MSVC"))
        .compile(&store)
        .expect("compile changed legacy fallback");

    // Then: completion stays open and the current legacy value remains visible.
    assert_eq!(report.skipped.len(), 1);
    assert!(brief.contains("MSVC"));
}
