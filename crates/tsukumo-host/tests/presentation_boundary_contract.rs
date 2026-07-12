use tsukumo_host::{load_presentation_pack, PresentationPackSource};
use tsukumo_kernel::{
    EventId, KernelEvent, KernelEventPayload, PersistedText, QuestId, SessionId, SpiritId,
    Timestamp, KERNEL_EVENT_SCHEMA_VERSION,
};

#[test]
fn default_presentation_when_loaded_never_enters_kernel_events_or_runtime_prompts() {
    // Given: the default Shiori pack and one neutral semantic event.
    let pack = load_presentation_pack(&PresentationPackSource::EmbeddedDefault)
        .expect("load default presentation");
    let event = KernelEvent {
        schema_version: KERNEL_EVENT_SCHEMA_VERSION,
        event_id: EventId::new("event-boundary"),
        occurred_at: Timestamp::from_unix_millis(1),
        quest_id: QuestId::new("quest-boundary"),
        session_id: SessionId::new("session-boundary"),
        spirit_id: SpiritId::new("runtime-spirit"),
        execution_id: None,
        runtime: None,
        causation_id: None,
        correlation_id: None,
        payload: KernelEventPayload::UserInput {
            content: PersistedText::from_reviewed("Run the verified test suite"),
        },
    };

    // When: the semantic event and a runtime prompt are assembled by their owning layers.
    let event_json = serde_json::to_string(&event).expect("serialize kernel event");
    let prompt = tsukumo_soul::assemble_delegation_prompt(
        "State: the GNU toolchain is selected.",
        "Run the verified test suite.",
    );

    // Then: presentation identity, world copy, and owner address stay outside both boundaries.
    for presentation_text in [
        pack.companion().actor_id.as_str(),
        pack.companion().display_name.as_str(),
        pack.manifest().world.name.as_str(),
        pack.companion().owner_address.as_str(),
    ] {
        assert!(!event_json.contains(presentation_text));
        assert!(!prompt.contains(presentation_text));
    }
}
