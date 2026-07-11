//! Cross-layer sentinel test for vendor redaction and durable projections.

use tempfile::tempdir;
use tsukumo_adapters::parse_stream_json_str;
use tsukumo_kernel::{
    CorrelationId, EventId, ExecutionId, KernelEvent, ProjectionId, QuestId, RuntimeBinding,
    RuntimeKind, RuntimeMode, SessionId, SpiritId, Timestamp, KERNEL_EVENT_SCHEMA_VERSION,
};
use tsukumo_soul::SoulStore;
use tsukumo_theater::{drive_kernel_events, DirectorContext, StageWorld};

#[test]
fn sentinel_never_reaches_chronicle_exports_or_theater_logs() {
    let body = concat!(
        "{\"type\":\"tool_use\",\"id\":\"call-1\",\"name\":\"Bash\",",
        "\"input\":{\"command\":\"echo safe\",\"client_secret\":\"SENTINEL-Aa1234567890_SECRET\"}}\n",
        "{\"type\":\"tool_result\",\"tool_use_id\":\"call-1\",",
        "\"content\":\"client_secret=SENTINEL-Aa1234567890_SECRET\"}\n"
    );
    let payloads = parse_stream_json_str(body).expect("normalize vendor stream");
    let events = payloads
        .into_iter()
        .enumerate()
        .map(|(index, mut payload)| {
            match &mut payload {
                tsukumo_kernel::KernelEventPayload::ToolStart { projection_id, .. }
                | tsukumo_kernel::KernelEventPayload::ToolEnd { projection_id, .. } => {
                    *projection_id = Some(ProjectionId::new("projection-redaction"));
                }
                _ => {}
            }
            KernelEvent {
                schema_version: KERNEL_EVENT_SCHEMA_VERSION,
                event_id: EventId::new(format!("event-redaction-{index}")),
                occurred_at: Timestamp::from_unix_millis(1_750_001_000_000 + index as i64),
                quest_id: QuestId::new("quest-redaction"),
                session_id: SessionId::new("session-redaction"),
                spirit_id: SpiritId::new("yuka"),
                execution_id: Some(ExecutionId::new("execution-redaction")),
                runtime: Some(RuntimeBinding::new(
                    RuntimeKind::ClaudeCli,
                    RuntimeMode::Fixture,
                )),
                causation_id: None,
                correlation_id: Some(CorrelationId::new("call-1")),
                payload,
            }
        })
        .collect::<Vec<_>>();

    let directory = tempdir().expect("create redaction pipeline directory");
    let mut store = SoulStore::open(directory.path()).expect("open Chronicle");
    for event in &events {
        store.append_event(event).expect("append redacted event");
    }
    let exports = store.rebuild_exports().expect("rebuild redacted exports");
    let chronicle =
        std::fs::read_to_string(exports.chronicle_jsonl).expect("read Chronicle export");

    let mut world = StageWorld::new();
    drive_kernel_events(&mut world, &events, &DirectorContext::default());
    let snapshot = world.snapshot();
    let theater_log = world.log.iter().cloned().collect::<Vec<_>>().join("\n");

    assert!(!chronicle.contains("SENTINEL"));
    assert!(chronicle.contains("[REDACTED]"));
    assert!(!theater_log.contains("SENTINEL"));
    assert_eq!(snapshot.log_len, 2);
}
