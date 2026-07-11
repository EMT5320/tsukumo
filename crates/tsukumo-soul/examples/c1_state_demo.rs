//! C1 source event -> extractor -> StateWriter -> reopen/export demo.

use std::env;
use std::path::PathBuf;
use tsukumo_kernel::{
    EventId, KernelEvent, KernelEventPayload, PersistedText, QuestId, SessionId, SpiritId, StateId,
    StateLifecycleAction, Timestamp, KERNEL_EVENT_SCHEMA_VERSION,
};
use tsukumo_soul::{
    ExtractionContext, OperatingSystem, RuleStateExtractor, SoulStore, StateExtractor, StateScope,
    StateTransition, StateWriteOutcome, StateWriteRequest,
};

fn event(id: &str, payload: KernelEventPayload) -> KernelEvent {
    event_at(id, 1_750_000_800_000, payload)
}

fn event_at(id: &str, timestamp: i64, payload: KernelEventPayload) -> KernelEvent {
    KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new(id),
        occurred_at: Timestamp::from_unix_millis(timestamp),
        quest_id: QuestId::new("quest-c1-demo"),
        session_id: SessionId::new("session-c1-demo"),
        spirit_id: SpiritId::new("yuka"),
        execution_id: None,
        runtime: None,
        causation_id: None,
        correlation_id: None,
        payload,
    }
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let data_dir = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| env::temp_dir().join("tsukumo-c1-state-demo"));
    let mut store = SoulStore::open(&data_dir)?;

    // The user event is the Chronicle evidence consumed by the deterministic
    // extractor and committed with its resulting state lifecycle.
    let source = event(
        "event-c1-demo-user",
        KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed(
                "Tsukumo always uses the GNU Rust toolchain on Windows",
            ),
        },
    );
    let scope = StateScope::workspace_os("tsukumo", OperatingSystem::Windows);
    let draft = RuleStateExtractor
        .extract(&ExtractionContext {
            event: &source,
            scope: scope.clone(),
        })?
        .into_iter()
        .next()
        .ok_or_else(|| std::io::Error::other("explicit GNU rule produced no draft"))?;
    let state_id = StateId::new("state-c1-demo-gnu");
    let lifecycle = event_at(
        "event-c1-demo-state",
        1_750_000_800_001,
        KernelEventPayload::StateLifecycle {
            state_id: state_id.clone(),
            action: StateLifecycleAction::Created,
            prior_state_id: None,
            reason: None,
        },
    );
    let outcome = store.apply_state(
        StateWriteRequest::new(
            StateTransition::Create {
                state_id: state_id.clone(),
                draft,
                created_at: Timestamp::from_unix_millis(1_750_000_800_001),
            },
            lifecycle,
        )
        .with_source_event(source),
    )?;
    let outcome_label = match outcome {
        StateWriteOutcome::Created(_) => "created",
        StateWriteOutcome::Superseded(_) => "superseded",
        StateWriteOutcome::Revoked(_) => "revoked",
        StateWriteOutcome::Unchanged(_) => "unchanged",
    };
    let exports = store.rebuild_exports()?;
    drop(store);

    // Reopening the database proves the state is durable independently from
    // generated Markdown and JSONL files.
    let reopened = SoulStore::open(&data_dir)?;
    let record = reopened
        .state(&state_id)?
        .ok_or_else(|| std::io::Error::other("persisted C1 demo state is missing after reopen"))?;

    println!("C1 state demo: {outcome_label}");
    println!("database: {}", reopened.database_path().display());
    println!("state: {} v{}", record.state_key, record.version);
    println!("scope: {}", serde_json::to_string(&record.scope)?);
    println!(
        "evidence: {}",
        record
            .evidence_refs
            .iter()
            .map(|event_id| event_id.as_str())
            .collect::<Vec<_>>()
            .join(",")
    );
    println!("chronicle export: {}", exports.chronicle_jsonl.display());
    println!("state export: {}", exports.state_markdown.display());
    Ok(())
}
