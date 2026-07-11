//! C1 rebuildable Chronicle, Markdown, and FTS projection tests.

use tempfile::tempdir;
use tsukumo_kernel::{
    EventId, KernelEvent, KernelEventPayload, PersistedText, QuestId, SensitiveText, SessionId,
    SpiritId, StateId, StateLifecycleAction, Timestamp, KERNEL_EVENT_SCHEMA_VERSION,
};
use tsukumo_soul::{
    EvidenceStrength, ExtractionProvenance, OperatingSystem, SoulStore, StateDraft, StateKey,
    StateKind, StateScope, StateTransition, StateWriteRequest,
};

fn event(id: &str, payload: KernelEventPayload) -> KernelEvent {
    event_at(id, 1_750_000_600_000, payload)
}

fn event_at(id: &str, timestamp: i64, payload: KernelEventPayload) -> KernelEvent {
    KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new(id),
        occurred_at: Timestamp::from_unix_millis(timestamp),
        quest_id: QuestId::new("quest-export"),
        session_id: SessionId::new("session-export"),
        spirit_id: SpiritId::new("yuka"),
        execution_id: None,
        runtime: None,
        causation_id: None,
        correlation_id: None,
        payload,
    }
}

fn create_state(store: &mut SoulStore) -> StateId {
    let source = event(
        "event-user-export",
        KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed("Use GNU on Windows"),
        },
    );
    let state_id = StateId::new("state-export");
    let draft = StateDraft {
        proposed_key: StateKey::new("workspace.tsukumo.rust.toolchain.windows"),
        kind: StateKind::Constraint,
        scope: StateScope::workspace_os("tsukumo", OperatingSystem::Windows),
        content: SensitiveText::new("Use the GNU Rust toolchain on Windows"),
        claimed_strength: EvidenceStrength::Explicit,
        evidence_refs: vec![source.event_id.clone()],
        provenance: ExtractionProvenance::Rule {
            name: "explicit_gnu_constraint".into(),
            version: 1,
        },
        expires_at: None,
    };
    store
        .apply_state(
            StateWriteRequest::new(
                StateTransition::Create {
                    state_id: state_id.clone(),
                    draft,
                    created_at: Timestamp::from_unix_millis(1_750_000_600_001),
                },
                event_at(
                    "event-state-export",
                    1_750_000_600_001,
                    KernelEventPayload::StateLifecycle {
                        state_id: state_id.clone(),
                        action: StateLifecycleAction::Created,
                        prior_state_id: None,
                        reason: None,
                    },
                ),
            )
            .with_source_event(source),
        )
        .expect("create export state");
    state_id
}

#[test]
fn deleted_exports_and_fts_rebuild_from_sqlite_only() {
    // Given: one committed Chronicle/state transaction and generated exports.
    let directory = tempdir().expect("create export test directory");
    let mut store = SoulStore::open(directory.path()).expect("open export store");
    let state_id = create_state(&mut store);
    let paths = store.rebuild_exports().expect("build initial exports");
    std::fs::remove_file(&paths.chronicle_jsonl).expect("remove Chronicle export");
    std::fs::remove_file(&paths.state_markdown).expect("remove state export");

    // When: exports and FTS are rebuilt, then Markdown is manually corrupted.
    let rebuilt = store.rebuild_exports().expect("rebuild exports");
    let chronicle =
        std::fs::read_to_string(&rebuilt.chronicle_jsonl).expect("read Chronicle export");
    let markdown = std::fs::read_to_string(&rebuilt.state_markdown).expect("read state export");
    std::fs::write(&rebuilt.state_markdown, "# STATE\n- Use MSVC\n")
        .expect("modify derived Markdown");
    let hits = store
        .search_states("GNU", 5)
        .expect("search rebuilt state FTS");
    let canonical = store
        .state(&state_id)
        .expect("query canonical state")
        .expect("canonical state exists");

    // Then: SQLite regenerates every view and manual edits cannot mutate truth.
    assert!(chronicle.contains("\"event_id\":\"event-user-export\""));
    assert!(markdown.contains("workspace.tsukumo.rust.toolchain.windows"));
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].state_id, state_id);
    assert_eq!(
        canonical.content.as_str(),
        "Use the GNU Rust toolchain on Windows"
    );
}

#[test]
fn export_failure_after_commit_leaves_canonical_state_reopenable() {
    // Given: a committed state and an invalid export root that is a file.
    let directory = tempdir().expect("create stale export directory");
    let state_id;
    {
        let mut store = SoulStore::open(directory.path()).expect("open export store");
        state_id = create_state(&mut store);
        let blocked_root = directory.path().join("blocked-root");
        std::fs::write(&blocked_root, "not a directory").expect("create blocked root");

        // When: rebuilding the derived files fails after the durable commit.
        let error = store
            .rebuild_exports_at(&blocked_root)
            .expect_err("blocked export root must fail");

        // Then: the caller observes the export failure directly.
        assert!(matches!(error, tsukumo_soul::SoulError::Io(_)));
    }

    // Then: reopening proves the canonical state was never rolled back.
    let store = SoulStore::open(directory.path()).expect("reopen after export failure");
    assert!(store
        .state(&state_id)
        .expect("query state after export failure")
        .is_some());
}

#[test]
fn search_refreshes_fts_before_applying_the_result_limit() {
    // Given: a newly committed state whose export projection was never rebuilt.
    let directory = tempdir().expect("create live FTS directory");
    let mut store = SoulStore::open(directory.path()).expect("open live FTS store");
    let state_id = create_state(&mut store);

    // When: the canonical search API is used immediately after the write.
    let hits = store
        .search_states("GNU", 1)
        .expect("search fresh canonical state");

    // Then: derived-index staleness cannot hide the committed active state.
    assert_eq!(hits.len(), 1);
    assert_eq!(hits[0].state_id, state_id);
}
