#[path = "support/product_surface.rs"]
mod product_surface;

use product_surface::default_surface;
use tsukumo_kernel::SpiritId;
use tsukumo_theater::{
    buffer_to_string, render_product_frame, select_layout, ColorCapability, LayoutMode,
    StageAttribution, StageEvent,
};
#[test]
fn layout_thresholds_when_terminal_changes_are_deterministic() {
    // Given/When/Then: the three documented terminal size bands.
    assert_eq!(select_layout(100, 30), LayoutMode::Full);
    assert_eq!(select_layout(80, 24), LayoutMode::Compact);
    assert_eq!(select_layout(71, 21), LayoutMode::Fallback);
}

#[test]
fn compact_header_when_truncated_preserves_attention_without_question_mark() {
    // Given: the default surface at the compact terminal threshold.
    let (pack, mut world, view, app) = default_surface();
    world.apply(&StageEvent::Bubble {
        text: "当前执行记录已完成核对。".into(),
        attribution: StageAttribution {
            actor_id: pack.companion().actor_id.clone(),
            source_spirit_id: SpiritId::new("runtime-spirit"),
        },
    });

    // When: the responsive header is rendered at 80 columns.
    let frame = buffer_to_string(&render_product_frame(
        &pack,
        &world,
        &view,
        &app,
        80,
        24,
        ColorCapability::TrueColor,
    ));
    let header = frame.lines().next().expect("header line");

    // Then: lower-priority runtime copy yields while current attention stays factual.
    assert!(header.contains("环境"), "missing attention label: {header}");
    assert!(
        header.contains('…'),
        "missing semantic truncation: {header}"
    );
    assert!(
        !header.ends_with('?'),
        "ambiguous truncation marker: {header}"
    );
    for (label, needle) in [
        ("identity", "九十九工房书记官"),
        ("bubble", "当前执行记录"),
        ("legend", "委托板"),
    ] {
        let line = frame
            .lines()
            .find(|line| line.contains(needle))
            .unwrap_or_else(|| panic!("missing compact {label} band"));
        assert!(
            !line.contains('▀'),
            "scene pixel leaked into compact {label} band: {line}"
        );
    }
}
#[test]
fn full_workshop_when_default_pack_renders_preserves_visual_hierarchy() {
    // Given: the bundled world, companion, and a factual runtime view.
    let (pack, world, view, app) = default_surface();

    // When: a full-size truecolor frame is rendered.
    let buffer = render_product_frame(
        &pack,
        &world,
        &view,
        &app,
        100,
        30,
        ColorCapability::TrueColor,
    );
    let frame = buffer_to_string(&buffer);

    // Then: world, actor, runtime source, facilities, log, and controls remain visible.
    for expected in [
        "深夜九十九工房",
        "栞",
        "runtime-spirit",
        "委托板",
        "运行门",
        "记忆柜",
        "投影案",
        "契约台",
        "执行 空闲",
        "[S]状态",
    ] {
        assert!(frame.contains(expected), "missing {expected:?}:\n{frame}");
    }
    assert!(
        frame.contains('▀'),
        "logical pixels were not HalfBlock packed"
    );
}

#[test]
fn undersized_terminal_when_rendered_keeps_factual_control_fallback() {
    // Given: the default product view in an undersized pane.
    let (pack, world, view, app) = default_surface();

    // When: the fallback renderer is selected.
    let frame = buffer_to_string(&render_product_frame(
        &pack,
        &world,
        &view,
        &app,
        60,
        18,
        ColorCapability::Ansi256,
    ));

    // Then: runtime facts, keyboard control, and resize guidance survive.
    assert!(frame.contains("runtime-spirit"));
    assert!(frame.contains("终端尺寸不足"));
    assert!(frame.contains("[Q]退出"));
}

#[test]
fn runtime_plaque_uses_authoritative_runtime_source_after_later_foreign_event() {
    // Given: Chronicle authority identifies one runtime while a later log comes from another Spirit.
    let (pack, mut world, mut view, app) = default_surface();
    view.runtime.source_spirit_id = Some(SpiritId::new("runtime-owner"));
    world.apply(&StageEvent::LogLine {
        text: "later attributed fact".into(),
        attribution: StageAttribution {
            actor_id: pack.companion().actor_id.clone(),
            source_spirit_id: SpiritId::new("later-spirit"),
        },
    });

    // When: the workshop, header, and factual log are rendered together.
    let frame = buffer_to_string(&render_product_frame(
        &pack,
        &world,
        &view,
        &app,
        100,
        30,
        ColorCapability::TrueColor,
    ));

    // Then: header and runtime plaque agree while the event keeps its own attribution.
    assert_eq!(
        frame.matches("runtime-owner").count(),
        2,
        "runtime source drift:\n{frame}"
    );
    assert!(
        frame.contains("later-spirit"),
        "event attribution disappeared:\n{frame}"
    );
}
