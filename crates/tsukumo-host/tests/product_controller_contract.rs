mod common;

use common::prepared_fixture;
use tempfile::tempdir;
use tsukumo_host::{
    load_presentation_pack, HostProductController, PresentationPackSource, ProductControl,
    ProductController,
};
use tsukumo_kernel::{
    CorrelationId, EventId, KernelEvent, KernelEventPayload, PermissionDecision, PersistedText,
    QuestId, SessionId, SpiritId, StateId, StateLifecycleAction, Timestamp, VendorEventRef,
    KERNEL_EVENT_SCHEMA_VERSION,
};
use tsukumo_soul::{
    ChronicleQuery, EvidenceStrength, ExtractionProvenance, OperatingSystem, SoulStore, StateDraft,
    StateKey, StateKind, StateScope, StateStatus, StateTransition, StateWriteRequest,
};
use tsukumo_theater::{RuntimeHealth, UiAction};

#[test]
fn empty_store_when_refreshed_is_a_real_offline_snapshot() {
    // Given: a real empty Soul authority and the embedded presentation pack.
    let directory = tempdir().expect("create product data directory");
    let pack = load_presentation_pack(&PresentationPackSource::EmbeddedDefault)
        .expect("load embedded pack");
    let mut controller =
        HostProductController::open(directory.path(), &pack).expect("open product controller");

    // When: the controller assembles and explicitly refreshes the product view.
    let first = controller.refresh().expect("assemble empty snapshot");
    assert_eq!(
        controller.apply(UiAction::Refresh).expect("route refresh"),
        ProductControl::Continue
    );
    let refreshed = controller.refresh().expect("refresh empty snapshot");

    // Then: the view is backed by the real store and exposes a visible refresh receipt.
    assert_eq!(first.view.runtime.health, RuntimeHealth::Offline);
    assert!(first.view.runtime.source_spirit_id.is_none());
    assert_eq!(first.revision, 0);
    assert!(refreshed
        .view
        .notices
        .last()
        .is_some_and(|notice| notice.text.as_str().contains("Chronicle")));
}

#[test]
fn revoke_action_when_state_is_active_persists_and_refreshes() {
    // Given: one evidence-backed active state in the durable store.
    let directory = tempdir().expect("create revoke data directory");
    let state_id = StateId::new("state-tui-revoke");
    seed_state(directory.path(), state_id.clone());
    let pack = load_presentation_pack(&PresentationPackSource::EmbeddedDefault)
        .expect("load embedded pack");
    let mut controller =
        HostProductController::open(directory.path(), &pack).expect("open product controller");
    let before = controller.refresh().expect("load active state");
    assert_eq!(before.view.states.len(), 1);
    assert!(before.view.states[0]
        .scope
        .as_str()
        .contains("workspace:tsukumo | workspace=tsukumo | os=windows"));

    // When: the typed TUI action is routed through the host controller.
    let control = controller
        .apply(UiAction::RevokeState(state_id.clone()))
        .expect("persist state revocation");
    let after = controller.refresh().expect("reload revoked state");

    // Then: the active read model changes and the historical version remains revoked in Soul.
    assert_eq!(control, ProductControl::Continue);
    assert!(after.view.states.is_empty());
    let reopened = SoulStore::open(directory.path()).expect("reopen Soul store");
    assert_eq!(
        reopened
            .state(&state_id)
            .expect("query revoked state")
            .expect("state remains historical")
            .status,
        StateStatus::Revoked
    );
}

#[test]
fn pending_permission_when_denied_records_durable_decision() {
    // Given: a committed projection and one unresolved normalized permission request.
    let (directory, mut store, prepared) = prepared_fixture();
    let vendor_request = VendorEventRef::new("fixture", "permission-tui");
    let request_event = KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new("event-permission-tui"),
        occurred_at: Timestamp::from_unix_millis(103),
        quest_id: QuestId::new("quest-host"),
        session_id: SessionId::new("session-host"),
        spirit_id: SpiritId::new("yuka"),
        execution_id: Some(prepared.receipt.execution_id.clone()),
        runtime: Some(prepared.receipt.runtime.clone()),
        causation_id: Some(EventId::new("event-host-projection")),
        correlation_id: Some(CorrelationId::new("correlation-permission-tui")),
        payload: KernelEventPayload::PermissionRequested {
            vendor_request: vendor_request.clone(),
            tool: "shell".into(),
            arguments: None,
            cwd: Some(PersistedText::from_reviewed("D:/WorkSpace/tsukumo")),
            risk_reasons: vec![PersistedText::from_reviewed("writes build artifacts")],
            reason: PersistedText::from_reviewed("human approval required"),
        },
    };
    store
        .append_event(&request_event)
        .expect("append permission request"); // Evict both the projection and request from the lossy 1,000-event UI tail.
    for index in 0..1_001 {
        let filler = KernelEvent {
            event_id: EventId::new(format!("event-tail-{index:04}")),
            occurred_at: Timestamp::from_unix_millis(200 + i64::from(index)),
            payload: KernelEventPayload::UserInput {
                content: PersistedText::from_reviewed(format!("tail event {index}")),
            },
            ..request_event.clone()
        };
        store.append_event(&filler).expect("append tail filler");
    }
    drop(store);
    let pack = load_presentation_pack(&PresentationPackSource::EmbeddedDefault)
        .expect("load embedded pack");
    let mut controller =
        HostProductController::open(directory.path(), &pack).expect("open product controller");
    let pending = controller
        .refresh()
        .expect("assemble pending permission")
        .view
        .pending_permission
        .expect("permission is visible");

    // When: the explicit deny action is submitted.
    controller
        .apply(UiAction::DecidePermission(
            pending.id,
            PermissionDecision::Deny,
        ))
        .expect("record permission decision");
    let after = controller.refresh().expect("reload permission state");

    // Then: the modal clears only after Chronicle contains a typed decision event.
    assert!(after.view.pending_permission.is_none());
    let reopened = SoulStore::open(directory.path()).expect("reopen permission store");
    let events = reopened
        .replay_events(
            ChronicleQuery::default()
                .for_execution(prepared.receipt.execution_id.clone())
                .limited_to(usize::MAX),
        )
        .expect("replay execution events");
    assert!(events.iter().any(|item| matches!(
        &item.event.payload,
        KernelEventPayload::PermissionDecided {
            vendor_request: stored,
            decision: PermissionDecision::Deny,
        } if stored == &vendor_request
    )));
}

#[cfg(windows)]
#[test]
fn guarded_persistent_journal_blocks_runtime_sidecar_injection() {
    // Given: an active state and a live controller holding the SQLite path capability.
    let directory = tempdir().expect("create guarded product directory");
    let state_id = StateId::new("state-sidecar-guard");
    seed_state(directory.path(), state_id.clone());
    let pack = load_presentation_pack(&PresentationPackSource::EmbeddedDefault)
        .expect("load embedded pack");
    let mut controller =
        HostProductController::open(directory.path(), &pack).expect("open guarded controller");
    controller.refresh().expect("load state coordinates");
    let sidecars =
        ["soul.db-journal", "soul.db-wal", "soul.db-shm"].map(|name| directory.path().join(name));
    let outside = directory.path().join("outside-journal-target");
    std::fs::write(&outside, "outside sentinel").expect("write outside sentinel");

    // When: a concurrent process tries to replace any fixed SQLite sidecar path.
    for sidecar in &sidecars {
        assert!(std::fs::remove_file(sidecar).is_err());
        assert!(std::fs::hard_link(&outside, sidecar).is_err());
    }
    controller
        .apply(UiAction::RevokeState(state_id))
        .expect("revoke through guarded journal");

    // Then: the transaction succeeds through PERSIST and leaves outside data intact.
    assert_eq!(
        std::fs::read_to_string(outside).expect("read outside sentinel"),
        "outside sentinel"
    );
    for sidecar in sidecars {
        assert!(sidecar.is_file());
    }
}

fn seed_state(path: &std::path::Path, state_id: StateId) {
    let mut store = SoulStore::open(path).expect("open state seed store");
    let source = event(
        "event-tui-source",
        1_750_000_000_000,
        KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed("Use the GNU Rust toolchain"),
        },
    );
    let lifecycle = event(
        "event-tui-state",
        1_750_000_000_001,
        KernelEventPayload::StateLifecycle {
            state_id: state_id.clone(),
            action: StateLifecycleAction::Created,
            prior_state_id: None,
            reason: None,
        },
    );
    store
        .apply_state(
            StateWriteRequest::new(
                StateTransition::Create {
                    state_id,
                    draft: StateDraft {
                        proposed_key: StateKey::new("workspace.tsukumo.toolchain"),
                        kind: StateKind::Preference,
                        scope: StateScope::workspace_os("tsukumo", OperatingSystem::Windows),
                        content: tsukumo_kernel::SensitiveText::new("Use GNU on Windows"),
                        claimed_strength: EvidenceStrength::Inferred,
                        evidence_refs: vec![source.event_id.clone()],
                        provenance: ExtractionProvenance::Recorded {
                            fixture: "tui-controller".into(),
                            schema_version: 1,
                        },
                        expires_at: None,
                    },
                    created_at: Timestamp::from_unix_millis(1_750_000_000_001),
                },
                lifecycle,
            )
            .with_source_event(source),
        )
        .expect("seed active state");
}

fn event(id: &str, timestamp: i64, payload: KernelEventPayload) -> KernelEvent {
    KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new(id),
        occurred_at: Timestamp::from_unix_millis(timestamp),
        quest_id: QuestId::new("quest-tui"),
        session_id: SessionId::new("session-tui"),
        spirit_id: SpiritId::new("shiori"),
        execution_id: None,
        runtime: None,
        causation_id: None,
        correlation_id: None,
        payload,
    }
}
