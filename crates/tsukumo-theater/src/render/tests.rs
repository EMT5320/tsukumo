use super::*;
use crate::stage::StageEvent;
use tsukumo_kernel::SpiritId;

#[test]
fn static_workshop_shows_title_and_sprite_region() {
    let mut world = StageWorld::new();
    world.ensure_placeholder("gina");
    let frame = render_frame_string(&world, DEFAULT_FRAME_WIDTH, DEFAULT_FRAME_HEIGHT);
    assert!(frame.contains("工房"), "workshop title missing:\n{frame}");
    assert!(
        frame.contains("gina"),
        "executor spirit ID missing:\n{frame}"
    );
    // Border / block art should leave non-space content in the stage pane.
    let non_space = frame.chars().filter(|c| !c.is_whitespace()).count();
    assert!(non_space > 40, "frame looks empty:\n{frame}");
}

#[test]
fn split_log_consumes_stage_events() {
    let mut world = StageWorld::new();
    world.ensure_placeholder("gina");
    world.apply(&StageEvent::Bubble {
        text: "干活中".into(),
        spirit_id: Some(SpiritId::new("gina")),
    });
    world.apply(&StageEvent::LogLine {
        text: "tool_start read (c1)".into(),
        spirit_id: None,
    });
    let frame = render_frame_string(&world, DEFAULT_FRAME_WIDTH, DEFAULT_FRAME_HEIGHT);
    assert!(frame.contains("干活中"), "bubble missing:\n{frame}");
    assert!(frame.contains("日志"), "log pane title missing:\n{frame}");
    assert!(frame.contains("tool_start"), "log line missing:\n{frame}");
}
