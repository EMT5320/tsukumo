//! Contract tests for receipt transaction rollback and prompt non-retention.

mod common;

use common::{
    checkpoint_event, event, persist_gnu_constraint, projection_event, projection_request,
};
use tempfile::tempdir;
use tsukumo_kernel::{
    CheckpointId, KernelEventPayload, PersistedText, QuestId, SessionId, Timestamp,
};
use tsukumo_soul::{
    CheckpointTrigger, CheckpointWriteRequest, HandoffCheckpoint, OperatingSystem,
    ProjectionWriteRequest, SoulError, SoulStore, StateRef, StateScope,
};

#[test]
fn receipt_failure_rolls_back_its_projection_event_and_prompt() {
    // Given: a valid persisted projection receipt.
    let directory = tempdir().expect("create rollback test directory");
    let mut store = SoulStore::open(directory.path()).expect("open rollback store");
    let selected = persist_gnu_constraint(
        &mut store,
        "state-rollback",
        "rollback",
        StateScope::workspace_os("tsukumo", OperatingSystem::Windows),
        300,
    );
    let source = event(
        "event-rollback-source",
        390,
        KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed("Preserve receipt ordering"),
        },
    );
    store.append_event(&source).expect("append rollback source");
    let checkpoint = HandoffCheckpoint::new(
        CheckpointId::new("checkpoint-rollback"),
        QuestId::new("quest-projection"),
        1,
        None,
        PersistedText::from_reviewed("Preserve receipt ordering"),
        Timestamp::from_unix_millis(400),
        CheckpointTrigger::Milestone,
    )
    .with_constraint_refs(vec![StateRef::new(selected.state_id, selected.version)])
    .with_source_event_refs(vec![source.event_id]);
    store
        .save_checkpoint(CheckpointWriteRequest::new(
            checkpoint.clone(),
            checkpoint_event(&checkpoint),
        ))
        .expect("save rollback checkpoint");
    let first_request = projection_request(
        "projection-conflict",
        "execution-conflict-1",
        &checkpoint.id,
        "Use credential sk-abcdefghijklmnop123456 for this runtime only",
    );
    let first_event = projection_event(
        "event-conflict-1",
        500,
        &first_request.projection_id,
        &checkpoint.id,
        &first_request.execution_id,
        &first_request.runtime,
    );
    let first = store
        .prepare_projection(ProjectionWriteRequest::new(first_request, first_event))
        .expect("persist first receipt");
    let receipt_json = serde_json::to_value(&first.receipt).expect("serialize receipt metadata");
    let receipt_object = receipt_json
        .as_object()
        .expect("receipt serializes as an object");
    assert!(!receipt_object.contains_key("rendered_prompt"));
    assert!(!receipt_object.contains_key("prompt"));
    assert_eq!(first.receipt.redactions.len(), 1);
    assert_eq!(first.receipt.redactions[0].location, "delegation_goal");
    assert_eq!(first.receipt.redactions[0].category, "sensitive_material");
    assert_eq!(first.receipt.redactions[0].action, "not_persisted");
    assert!(!serde_json::to_string(&first.receipt)
        .expect("serialize receipt without prompt")
        .contains("sk-abcdefghijklmnop123456"));
    assert!(!format!("{first:?}").contains("sk-abcdefghijklmnop123456"));

    // When: identical receipt content is retried under a different Chronicle EventId.
    let retry_request = projection_request(
        "projection-conflict",
        "execution-conflict-1",
        &checkpoint.id,
        "Use credential sk-abcdefghijklmnop123456 for this runtime only",
    );
    let retry_event = projection_event(
        "event-conflict-retry",
        500,
        &retry_request.projection_id,
        &checkpoint.id,
        &retry_request.execution_id,
        &retry_request.runtime,
    );
    let retry_event_id = retry_event.event_id.clone();
    store
        .prepare_projection(ProjectionWriteRequest::new(retry_request, retry_event))
        .expect_err("a receipt retry must keep its original Chronicle EventId");
    assert!(store
        .event(&retry_event_id)
        .expect("query rejected retry event")
        .is_none());

    // When: the original EventId is retried with conflicting envelope content.
    let same_event_request = projection_request(
        "projection-conflict",
        "execution-conflict-1",
        &checkpoint.id,
        "Use credential sk-abcdefghijklmnop123456 for this runtime only",
    );
    let mut conflicting_event = projection_event(
        "event-conflict-1",
        500,
        &same_event_request.projection_id,
        &checkpoint.id,
        &same_event_request.execution_id,
        &same_event_request.runtime,
    );
    conflicting_event.session_id = SessionId::new("session-conflicting-retry");
    let conflict = store
        .prepare_projection(ProjectionWriteRequest::new(
            same_event_request,
            conflicting_event,
        ))
        .expect_err("same EventId with changed envelope must fail");
    assert!(matches!(conflict, SoulError::ConflictingEvent { .. }));

    // When: a second receipt reuses the immutable projection ID with new prompt bytes.
    let second_request = projection_request(
        "projection-conflict",
        "execution-conflict-2",
        &checkpoint.id,
        "runtime-prompt-sentinel-plain-text",
    );
    let second_event = projection_event(
        "event-conflict-2",
        501,
        &second_request.projection_id,
        &checkpoint.id,
        &second_request.execution_id,
        &second_request.runtime,
    );
    let failed_event_id = second_event.event_id.clone();
    let failure = store
        .prepare_projection(ProjectionWriteRequest::new(second_request, second_event))
        .expect_err("conflicting immutable receipt must fail");

    // Then: error/debug paths, Chronicle, and SQLite exclude both prompt sentinels.
    assert!(!format!("{failure:?}").contains("runtime-prompt-sentinel-plain-text"));
    assert!(store
        .event(&failed_event_id)
        .expect("query rolled-back event")
        .is_none());
    let database = std::fs::read(store.database_path()).expect("read SQLite database bytes");
    assert!(!database
        .windows(b"sk-abcdefghijklmnop123456".len())
        .any(|window| window == b"sk-abcdefghijklmnop123456"));
    assert!(!database
        .windows(b"runtime-prompt-sentinel-plain-text".len())
        .any(|window| window == b"runtime-prompt-sentinel-plain-text"));
}
#[test]
fn v0_schema_contains_no_prompt_snapshot_authority() {
    // Given: a freshly migrated V0 database.
    let directory = tempdir().expect("create schema test directory");
    let store = SoulStore::open(directory.path()).expect("open schema store");
    let connection = rusqlite::Connection::open(store.database_path()).expect("open schema query");

    // When: durable projection columns and table names are inspected.
    let mut statement = connection
        .prepare("PRAGMA table_info(projection_receipts)")
        .expect("prepare receipt columns");
    let columns = statement
        .query_map([], |row| row.get::<_, String>(1))
        .expect("query receipt columns")
        .collect::<Result<Vec<_>, _>>()
        .expect("collect receipt columns");
    let table_names = connection
        .prepare("SELECT name FROM sqlite_master WHERE type = 'table' ORDER BY name")
        .expect("prepare table names")
        .query_map([], |row| row.get::<_, String>(0))
        .expect("query table names")
        .collect::<Result<Vec<_>, _>>()
        .expect("collect table names");
    let trigger_names = connection
        .prepare("SELECT name FROM sqlite_master WHERE type = 'trigger' ORDER BY name")
        .expect("prepare trigger names")
        .query_map([], |row| row.get::<_, String>(0))
        .expect("query trigger names")
        .collect::<Result<Vec<_>, _>>()
        .expect("collect trigger names");

    // Then: receipts are metadata-only and every ledger table is immutable.
    assert!(columns.contains(&"receipt_json".to_owned()));
    assert!(!columns.iter().any(|column| column.contains("prompt")));
    assert!(!table_names
        .iter()
        .any(|table| table.contains("snapshot") || table.contains("case_bundle")));
    for table in [
        "handoff_checkpoints",
        "checkpoint_state_refs",
        "checkpoint_source_refs",
        "projection_receipts",
        "receipt_state_refs",
    ] {
        for action in ["update", "delete"] {
            assert!(trigger_names.contains(&format!("{table}_no_{action}")));
        }
    }
}
