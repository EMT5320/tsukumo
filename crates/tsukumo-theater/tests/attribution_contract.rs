use tsukumo_kernel::{
    EventId, KernelEvent, KernelEventPayload, QuestId, SessionId, SpiritId, Timestamp,
    VendorEventRef, KERNEL_EVENT_SCHEMA_VERSION,
};
use tsukumo_theater::{
    direct, ActorPose, DirectorContext, LineBook, PresentationActorId, StageAttribution, StageEvent,
};

fn tool_start_from(spirit_id: &str) -> KernelEvent {
    KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new("event-attribution"),
        occurred_at: Timestamp::from_unix_millis(1_750_000_000_000),
        quest_id: QuestId::new("quest-attribution"),
        session_id: SessionId::new("session-attribution"),
        spirit_id: SpiritId::new(spirit_id),
        execution_id: None,
        runtime: None,
        causation_id: None,
        correlation_id: None,
        payload: KernelEventPayload::ToolStart {
            vendor_call: VendorEventRef::new("fixture", "call-attribution"),
            tool: "read".into(),
            args: None,
            projection_id: None,
        },
    }
}

#[test]
fn configured_actor_when_source_spirit_differs_preserves_both_identities() {
    // Given: Shiori presents an event whose factual executor is a different Spirit.
    let actor_id = PresentationActorId::try_from("shiori").expect("valid actor id");
    let context = DirectorContext::new(actor_id, LineBook::default());
    let input = tool_start_from("runtime-spirit");

    // When: the pure Director maps the durable event.
    let output = direct(&input, &context);

    // Then: the visible actor and source executor remain separate typed facts.
    assert!(output.iter().any(|event| matches!(
        event,
        StageEvent::ActorPose {
            pose: ActorPose::Work,
            attribution: StageAttribution {
                actor_id,
                source_spirit_id,
            },
        } if actor_id.as_str() == "shiori" && source_spirit_id.as_str() == "runtime-spirit"
    )));
}

#[test]
fn stage_serialization_when_attributed_contains_no_runtime_vendor_keys() {
    // Given: one Director-produced attributed event stream.
    let actor_id = PresentationActorId::try_from("shiori").expect("valid actor id");
    let context = DirectorContext::new(actor_id, LineBook::default());
    let input = tool_start_from("runtime-spirit");

    // When: presentation events are serialized.
    let json = serde_json::to_string(&direct(&input, &context)).expect("serialize stage events");

    // Then: both neutral identities remain and vendor protocol details stay absent.
    assert!(json.contains("\"actor_id\":\"shiori\""));
    assert!(json.contains("\"source_spirit_id\":\"runtime-spirit\""));
    assert!(!json.contains("claude"));
    assert!(!json.contains("stream_json"));
}
