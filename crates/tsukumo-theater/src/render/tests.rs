use super::*;
use crate::stage::StageEvent;
use crate::{PresentationActorId, StageAttribution, StageWorld};
use tsukumo_kernel::SpiritId;

fn actor_id() -> PresentationActorId {
    PresentationActorId::try_from("companion").expect("valid actor id")
}

fn attribution() -> StageAttribution {
    StageAttribution {
        actor_id: actor_id(),
        source_spirit_id: SpiritId::new("gina"),
    }
}

#[test]
fn static_workshop_when_actor_exists_shows_identity_and_sprite_region() {
    // Given: one placeholder presentation actor.
    let mut world = StageWorld::new();
    world.ensure_placeholder(actor_id());

    // When: the legacy frame entrypoint renders a buffer string.
    let frame = render_frame_string(&world, DEFAULT_FRAME_WIDTH, DEFAULT_FRAME_HEIGHT);

    // Then: identity and substantial stage content remain visible.
    assert!(
        frame.contains("companion"),
        "actor identity missing:\n{frame}"
    );
    let non_space = frame
        .chars()
        .filter(|character| !character.is_whitespace())
        .count();
    assert!(non_space > 40, "frame looks empty:\n{frame}");
}

#[test]
fn split_log_when_events_arrive_shows_bubble_and_log() {
    // Given: one attributed bubble and factual log line.
    let mut world = StageWorld::new();
    world.ensure_placeholder(actor_id());
    world.apply(&StageEvent::Bubble {
        text: "filing records...".into(),
        attribution: attribution(),
    });
    world.apply(&StageEvent::LogLine {
        text: "tool_start read (c1)".into(),
        attribution: attribution(),
    });

    // When: the frame is rendered.
    let frame = render_frame_string(&world, DEFAULT_FRAME_WIDTH, DEFAULT_FRAME_HEIGHT);

    // Then: both presentation copy and factual activity remain visible.
    assert!(frame.contains("filing records"), "bubble missing:\n{frame}");
    assert!(frame.contains("tool_start"), "log line missing:\n{frame}");
    assert!(frame.contains("gina"), "source Spirit missing:\n{frame}");
}
