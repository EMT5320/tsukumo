//! C1 Chronicle persistence, replay, and append-only contract tests.

use rusqlite::Connection;
use tempfile::tempdir;
use tsukumo_kernel::{
    CorrelationId, EventContractError, EventId, KernelEvent, KernelEventPayload, OutcomeStatus,
    PersistedText, ProjectionId, QuestId, SessionId, SpiritId, Timestamp, VendorEventRef,
    KERNEL_EVENT_SCHEMA_VERSION,
};
use tsukumo_soul::{AppendOutcome, ChronicleQuery, SoulError, SoulStore};

fn event(id: &str, timestamp: i64, quest: &str, summary: &str) -> KernelEvent {
    KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new(id),
        occurred_at: Timestamp::from_unix_millis(timestamp),
        quest_id: QuestId::new(quest),
        session_id: SessionId::new("session-chronicle"),
        spirit_id: SpiritId::new("yuka"),
        execution_id: None,
        runtime: None,
        causation_id: None,
        correlation_id: None,
        payload: KernelEventPayload::Outcome {
            status: OutcomeStatus::Succeeded,
            summary: Some(PersistedText::from_reviewed(summary)),
            projection_id: None,
        },
    }
}

#[test]
fn append_reopen_and_filtered_replay_preserve_order() {
    // Given: two quests appended in a known order.
    let directory = tempdir().expect("create Chronicle test directory");
    {
        let mut store = SoulStore::open(directory.path()).expect("open Chronicle");
        assert!(matches!(
            store
                .append_event(&event("event-1", 100, "quest-a", "first"))
                .expect("append first"),
            AppendOutcome::Inserted { sequence: 1 }
        ));
        assert!(matches!(
            store
                .append_event(&event("event-2", 90, "quest-b", "second"))
                .expect("append second"),
            AppendOutcome::Inserted { sequence: 2 }
        ));
        assert!(matches!(
            store
                .append_event(&event("event-3", 110, "quest-a", "third"))
                .expect("append third"),
            AppendOutcome::Inserted { sequence: 3 }
        ));
    }

    // When: a new connection replays only quest-a.
    let store = SoulStore::open(directory.path()).expect("reopen Chronicle");
    let replayed = store
        .replay_events(ChronicleQuery::default().for_quest(QuestId::new("quest-a")))
        .expect("replay quest");

    // Then: database sequence controls replay, independent of event timestamps.
    assert_eq!(
        replayed
            .iter()
            .map(|persisted| persisted.event.event_id.as_str())
            .collect::<Vec<_>>(),
        vec!["event-1", "event-3"]
    );
    assert_eq!(replayed[0].sequence, 1);
    assert_eq!(replayed[1].sequence, 3);
}

#[test]
fn duplicate_identical_event_is_idempotent_and_conflict_rolls_back() {
    // Given: one persisted event.
    let directory = tempdir().expect("create duplicate test directory");
    let mut store = SoulStore::open(directory.path()).expect("open Chronicle");
    let original = event("event-1", 100, "quest-a", "same");
    store.append_event(&original).expect("append original");

    // When: the same event is retried, followed by a conflicting duplicate ID.
    let duplicate = store.append_event(&original).expect("append duplicate");
    let conflict = store
        .append_event(&event("event-1", 101, "quest-a", "changed"))
        .expect_err("conflicting duplicate must fail");

    // Then: retry returns the original sequence and no second row appears.
    assert!(matches!(
        duplicate,
        AppendOutcome::Duplicate { sequence: 1 }
    ));
    assert!(matches!(
        conflict,
        SoulError::ConflictingEvent { event_id } if event_id.as_str() == "event-1"
    ));
    assert_eq!(
        store
            .replay_events(ChronicleQuery::default())
            .expect("replay all")
            .len(),
        1
    );
}

#[test]
fn sqlite_guards_chronicle_rows_from_update_and_delete() {
    // Given: one committed Chronicle row.
    let directory = tempdir().expect("create append-only test directory");
    let database_path = directory.path().join("soul.db");
    let mut store = SoulStore::open(directory.path()).expect("open Chronicle");
    store
        .append_event(&event("event-1", 100, "quest-a", "immutable"))
        .expect("append immutable event");
    drop(store);
    let connection = Connection::open(database_path).expect("open raw SQLite connection");

    // When: a caller attempts to mutate or delete the append-only table.
    let update = connection.execute(
        "UPDATE chronicle_events SET event_json = '{}' WHERE event_id = 'event-1'",
        [],
    );
    let delete = connection.execute(
        "DELETE FROM chronicle_events WHERE event_id = 'event-1'",
        [],
    );

    // Then: database triggers reject both mutations.
    assert!(update.is_err());
    assert!(delete.is_err());
}

#[test]
fn correlation_filter_reconstructs_only_the_matching_chain() {
    let directory = tempdir().expect("create correlation test directory");
    let mut store = SoulStore::open(directory.path()).expect("open Chronicle");
    for (id, correlation) in [
        ("event-a1", Some("correlation-a")),
        ("event-b", Some("correlation-b")),
        ("event-a2", Some("correlation-a")),
        ("event-none", None),
    ] {
        let mut correlated = event(id, 100, "quest-a", id);
        correlated.correlation_id = correlation.map(CorrelationId::new);
        store
            .append_event(&correlated)
            .expect("append correlated event");
    }

    let replayed = store
        .replay_events(
            ChronicleQuery::default().for_correlation(CorrelationId::new("correlation-a")),
        )
        .expect("replay correlation");
    assert_eq!(
        replayed
            .iter()
            .map(|persisted| persisted.event.event_id.as_str())
            .collect::<Vec<_>>(),
        vec!["event-a1", "event-a2"]
    );
}

#[test]
fn persistence_rejects_unattributed_or_sensitive_events() {
    let directory = tempdir().expect("create contract rejection directory");
    let mut store = SoulStore::open(directory.path()).expect("open Chronicle");

    let mut unattributed = event("event-tool", 100, "quest-a", "unused");
    unattributed.payload = KernelEventPayload::ToolStart {
        vendor_call: VendorEventRef::new("fixture", "call-1"),
        tool: "shell".into(),
        args: None,
        projection_id: Some(ProjectionId::new("projection-1")),
    };
    let error = store
        .append_event(&unattributed)
        .expect_err("unattributed tool event must fail");
    assert!(matches!(
        error,
        SoulError::EventContract(EventContractError::MissingAttribution {
            field: "execution_id",
            ..
        })
    ));

    let mut sensitive = event("event-secret", 101, "quest-a", "unused");
    sensitive.payload = KernelEventPayload::UserInput {
        content: PersistedText::from_reviewed("api_key=SENTINEL-Aa1234567890_SECRET_VALUE"),
    };
    let error = store
        .append_event(&sensitive)
        .expect_err("unredacted user event must fail");
    assert!(matches!(
        error,
        SoulError::EventContract(EventContractError::SensitiveContent { .. })
    ));
    assert!(store
        .replay_events(ChronicleQuery::default())
        .expect("replay rejected events")
        .is_empty());
}

#[test]
fn sqlite_replay_revalidates_the_stored_event_contract() {
    // Given: an externally corrupted canonical row with a newer schema version.
    let directory = tempdir().expect("create replay validation directory");
    let database_path = directory.path().join("soul.db");
    let mut store = SoulStore::open(directory.path()).expect("open replay validation store");
    store
        .append_event(&event("event-corrupt", 100, "quest-a", "stored"))
        .expect("append valid event");
    drop(store);
    let connection = Connection::open(database_path).expect("open raw replay database");
    connection
        .execute_batch(
            "DROP TRIGGER chronicle_no_update;
             UPDATE chronicle_events
             SET event_json = json_set(event_json, '$.schema_version', 2)
             WHERE event_id = 'event-corrupt';",
        )
        .expect("simulate external database corruption");
    drop(connection);
    let store = SoulStore::open(directory.path()).expect("reopen replay validation store");

    // When/Then: canonical replay applies the same gate as JSONL loading.
    assert!(matches!(
        store.replay_events(ChronicleQuery::default()),
        Err(SoulError::EventContract(
            EventContractError::UnsupportedSchema { .. }
        ))
    ));
}

#[test]
fn recent_replay_when_limited_returns_newest_tail_in_sequence_order() {
    // Given: three Chronicle events appended in database order.
    let directory = tempdir().expect("create recent replay directory");
    let mut store = SoulStore::open(directory.path()).expect("open Chronicle");
    for (index, id) in ["event-old", "event-middle", "event-new"]
        .into_iter()
        .enumerate()
    {
        store
            .append_event(&event(id, 200 + index as i64, "quest-recent", id))
            .expect("append recent replay fixture");
    }

    // When: the bounded newest tail requests two events.
    let replayed = store
        .replay_recent_events(2)
        .expect("replay newest Chronicle tail");

    // Then: the oldest event is excluded while consumer order remains chronological.
    assert_eq!(
        replayed
            .iter()
            .map(|item| item.event.event_id.as_str())
            .collect::<Vec<_>>(),
        ["event-middle", "event-new"]
    );
}
#[test]
fn recent_replay_when_zero_is_requested_returns_empty() {
    // Given: one committed event in Chronicle.
    let directory = tempdir().expect("create zero replay directory");
    let mut store = SoulStore::open(directory.path()).expect("open Chronicle");
    store
        .append_event(&event("event-zero", 300, "quest-zero", "stored"))
        .expect("append zero replay fixture");

    // When: the caller explicitly requests a zero-sized UI tail.
    let replayed = store
        .replay_recent_events(0)
        .expect("replay empty Chronicle tail");

    // Then: the bounded read performs no implicit one-row expansion.
    assert!(replayed.is_empty());
}
