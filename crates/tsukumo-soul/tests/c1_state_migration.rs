//! C1 state lifecycle schema migration regressions.

use rusqlite::{params, Connection};
use tempfile::tempdir;
use tsukumo_kernel::{StateId, Timestamp};
use tsukumo_soul::{
    BriefCompiler, ExtractionProvenance, OperatingSystem, SoulStore, StateKey, StateScope,
};

const STATE_KEY: &str = "workspace.tsukumo.rust.toolchain.windows";

#[test]
fn schema_one_inactive_states_keep_their_historical_intervals() {
    // Given: a version-one database with superseded/revoked state and an old marker.
    let directory = tempdir().expect("create version-one migration directory");
    seed_version_one_database(&directory.path().join("soul.db"));

    // When: opening the store applies the version-two lifecycle migration.
    let store = SoulStore::open(directory.path()).expect("migrate version-one state database");
    let key = StateKey::new(STATE_KEY);
    let scope = StateScope::workspace_os("tsukumo", OperatingSystem::Windows);

    // Then: historical instants resolve correctly and revoked state stays inactive.
    assert_eq!(
        store
            .active_state_at(&key, &scope, Timestamp::from_unix_millis(150))
            .expect("query first historical interval")
            .expect("version one active at 150")
            .state_id,
        StateId::new("state-v1")
    );
    assert_eq!(
        store
            .active_state_at(&key, &scope, Timestamp::from_unix_millis(250))
            .expect("query second historical interval")
            .expect("version two active at 250")
            .state_id,
        StateId::new("state-v2")
    );
    assert!(store
        .active_state_at(&key, &scope, Timestamp::from_unix_millis(350))
        .expect("query after revocation")
        .is_none());

    // Old completion markers must not hide rows that need the current importer.
    let brief = BriefCompiler::with_defaults()
        .compile(&store)
        .expect("compile legacy fallback after migration");
    assert!(brief.contains("review marker"));
}

fn seed_version_one_database(path: &std::path::Path) {
    let connection = Connection::open(path).expect("create version-one database");
    connection
        .execute_batch(
            "CREATE TABLE schema_migrations (version INTEGER PRIMARY KEY NOT NULL);
             INSERT INTO schema_migrations (version) VALUES (1);
             CREATE TABLE facts (
                 id TEXT PRIMARY KEY NOT NULL,
                 kind TEXT NOT NULL,
                 text TEXT NOT NULL,
                 session_id TEXT NOT NULL,
                 created_at INTEGER NOT NULL
             );
             INSERT INTO facts (id, kind, text, session_id, created_at)
             VALUES ('old-marker-row', 'memory', 'review marker row', 'session-old', 50);
             CREATE TABLE chronicle_events (
                 sequence INTEGER PRIMARY KEY AUTOINCREMENT,
                 event_id TEXT UNIQUE NOT NULL,
                 occurred_at INTEGER NOT NULL,
                 event_json TEXT NOT NULL
             );
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
                 supersedes_state_id TEXT
             );
             CREATE TABLE state_evidence (
                 state_id TEXT NOT NULL,
                 event_id TEXT NOT NULL,
                 PRIMARY KEY(state_id, event_id)
             );
             CREATE TABLE legacy_import_runs (
                 source_table TEXT PRIMARY KEY NOT NULL,
                 completed_at INTEGER NOT NULL
             );
             INSERT INTO legacy_import_runs (source_table, completed_at)
             VALUES ('facts', 40);",
        )
        .expect("create version-one schema");
    let scope = StateScope::workspace_os("tsukumo", OperatingSystem::Windows);
    let scope_key = serde_json::to_string(&scope).expect("encode scope key");
    let scope_json = serde_json::to_string(&scope).expect("encode scope");
    let provenance_json = serde_json::to_string(&ExtractionProvenance::Recorded {
        fixture: "schema-one".into(),
        schema_version: 1,
    })
    .expect("encode provenance");
    for (id, status, version, created_at, prior) in [
        ("state-v1", "superseded", 1_i64, 100_i64, None),
        ("state-v2", "revoked", 2_i64, 200_i64, Some("state-v1")),
    ] {
        connection
            .execute(
                "INSERT INTO state_records (
                    state_id, state_key, scope_key, scope_json, kind, strength,
                    status, content, provenance_json, version, created_at,
                    expires_at, supersedes_state_id
                 ) VALUES (?1, ?2, ?3, ?4, 'preference', 'inferred', ?5,
                           'Use GNU on Windows', ?6, ?7, ?8, NULL, ?9)",
                params![
                    id,
                    STATE_KEY,
                    scope_key,
                    scope_json,
                    status,
                    provenance_json,
                    version,
                    created_at,
                    prior
                ],
            )
            .expect("insert version-one state");
    }
    insert_lifecycle(
        &connection,
        "event-supersede",
        200,
        "superseded",
        "state-v2",
        Some("state-v1"),
    );
    insert_lifecycle(
        &connection,
        "event-revoke",
        300,
        "revoked",
        "state-v2",
        None,
    );
}

fn insert_lifecycle(
    connection: &Connection,
    event_id: &str,
    occurred_at: i64,
    action: &str,
    state_id: &str,
    prior_state_id: Option<&str>,
) {
    let event_json = serde_json::json!({
        "payload": {
            "type": "state_lifecycle",
            "state_id": state_id,
            "action": action,
            "prior_state_id": prior_state_id
        }
    });
    connection
        .execute(
            "INSERT INTO chronicle_events (event_id, occurred_at, event_json)
             VALUES (?1, ?2, ?3)",
            params![event_id, occurred_at, event_json.to_string()],
        )
        .expect("insert lifecycle event");
}
