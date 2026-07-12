#[path = "support/product_surface.rs"]
mod product_surface;

use product_surface::default_surface;
use tsukumo_kernel::PermissionDecision;
use tsukumo_theater::{
    buffer_to_string, reduce_app, render_product_frame, ColorCapability, DisplayText,
    PermissionEvidenceText, PermissionView, UiInput, UiKey, UiPermissionId,
};
#[test]
fn permission_when_pending_overlays_compact_screen_with_all_decisions() {
    // Given: a compact workshop with one pending permission.
    let (pack, world, mut view, mut app) = default_surface();
    view.pending_permission = Some(PermissionView {
        id: UiPermissionId::try_from("permission-render").expect("valid permission id"),
        tool: DisplayText::from_untrusted("shell"),
        arguments: PermissionEvidenceText::from_untrusted("cargo test --workspace"),
        cwd: PermissionEvidenceText::from_untrusted("D:/WorkSpace/tsukumo"),
        risk_reasons: vec![PermissionEvidenceText::from_untrusted(
            "writes build artifacts",
        )],
        runtime: DisplayText::from_untrusted("codex/acp"),
    });
    let action = reduce_app(&mut app, UiInput::Key(UiKey::AllowSession), &view);
    // Move to the argument evidence page after proving the decision key mapping.
    reduce_app(&mut app, UiInput::Key(UiKey::Down), &view);

    // When: the modal is rendered on its argument evidence page.
    let frame = buffer_to_string(&render_product_frame(
        &pack,
        &world,
        &view,
        &app,
        80,
        24,
        ColorCapability::Monochrome,
    ));

    // Then: every explicit decision stays legible without relying on color.
    assert!(matches!(
        action,
        Some(tsukumo_theater::UiAction::DecidePermission(
            _,
            PermissionDecision::AllowSession
        ))
    ));
    for expected in [
        "契约台",
        "cargo test",
        "[1]仅一次",
        "[2]本次会话",
        "[D]拒绝",
    ] {
        assert!(frame.contains(expected), "missing {expected:?}:\n{frame}");
    }
}

#[test]
fn permission_evidence_when_many_is_reachable_in_fallback_layout() {
    // Given: a fallback-size permission with six distinct risk reasons.
    let (pack, world, mut view, mut app) = default_surface();
    view.pending_permission = Some(PermissionView {
        id: UiPermissionId::try_from("permission-many-risks").expect("valid permission id"),
        tool: DisplayText::from_untrusted("shell"),
        arguments: PermissionEvidenceText::from_untrusted("cargo test --workspace"),
        cwd: PermissionEvidenceText::from_untrusted("D:/WorkSpace/tsukumo"),
        risk_reasons: (1..=6)
            .map(|index| PermissionEvidenceText::from_untrusted(&format!("risk reason {index}")))
            .collect(),
        runtime: DisplayText::from_untrusted("codex/acp"),
    });
    for _ in 1..6 {
        reduce_app(&mut app, UiInput::Key(UiKey::Down), &view);
    }

    // When: the selected risk page overlays the 60x18 factual fallback.
    let frame = buffer_to_string(&render_product_frame(
        &pack,
        &world,
        &view,
        &app,
        60,
        18,
        ColorCapability::Ansi256,
    ));

    // Then: the final reason and every authority decision remain visible.
    for expected in [
        "6/6",
        "risk reason 6",
        "[1]仅一次",
        "[2]本次会话",
        "[D]拒绝",
    ] {
        assert!(frame.contains(expected), "missing {expected:?}:\n{frame}");
    }
}
#[test]
fn long_permission_reason_when_paged_keeps_its_tail_reachable() {
    // Given: one risk summary that spans three stable permission pages.
    let (pack, world, mut view, mut app) = default_surface();
    let reason = format!("{}tail-marker", "x".repeat(800));
    view.pending_permission = Some(PermissionView {
        id: UiPermissionId::try_from("permission-long-risk").expect("valid permission id"),
        tool: DisplayText::from_untrusted("shell"),
        arguments: PermissionEvidenceText::from_untrusted("cargo test"),
        cwd: PermissionEvidenceText::from_untrusted("D:/WorkSpace/tsukumo"),
        risk_reasons: vec![PermissionEvidenceText::from_untrusted(&reason)],
        runtime: DisplayText::from_untrusted("codex/acp"),
    });
    let risk_pages = view
        .pending_permission
        .as_ref()
        .expect("pending permission")
        .risk_page_count();
    for _ in 1..risk_pages {
        reduce_app(&mut app, UiInput::Key(UiKey::Down), &view);
    }

    // When: the final part is rendered in the smallest supported product layout.
    let frame = buffer_to_string(&render_product_frame(
        &pack,
        &world,
        &view,
        &app,
        60,
        18,
        ColorCapability::Ansi256,
    ));

    // Then: no part of the bounded risk summary is permanently clipped.
    assert!(
        frame.contains(&format!("分页 {risk_pages}/{risk_pages}")),
        "missing page receipt:\n{frame}"
    );
    assert!(frame.contains("tail-marker"), "missing risk tail:\n{frame}");
}
#[test]
fn fallback_permission_when_rendered_keeps_decisions_and_masks_stage() {
    // Given: a pending permission in the supported 60x18 fallback evidence size.
    let (pack, world, mut view, app) = default_surface();
    view.pending_permission = Some(PermissionView {
        id: UiPermissionId::try_from("permission-fallback").expect("valid permission id"),
        tool: DisplayText::from_untrusted("shell"),
        arguments: PermissionEvidenceText::from_untrusted("cargo test --workspace --all-targets"),
        cwd: PermissionEvidenceText::from_untrusted("D:/WorkSpace/tsukumo"),
        risk_reasons: vec![PermissionEvidenceText::from_untrusted(
            "会写入 target 构建产物，并可能持续一段时间。",
        )],
        runtime: DisplayText::from_untrusted("codex/acp"),
    });

    // When: the blocking surface replaces the undersized background.
    let frame = buffer_to_string(&render_product_frame(
        &pack,
        &world,
        &view,
        &app,
        60,
        18,
        ColorCapability::TrueColor,
    ));

    // Then: every decision survives and no underlying CJK facility fragment leaks through.
    assert!(frame.contains("[1]仅一次"));
    assert!(frame.contains("[2]本次会话"));
    assert!(frame.contains("[D]拒绝"));
    assert!(!frame.contains("委托板"));
}

#[test]
fn narrow_permission_when_rendered_keeps_all_decision_meanings() {
    // Given: a pending permission in a very narrow fallback terminal.
    let (pack, world, mut view, app) = default_surface();
    view.pending_permission = Some(PermissionView {
        id: UiPermissionId::try_from("permission-narrow").expect("valid permission id"),
        tool: DisplayText::from_untrusted("shell"),
        arguments: PermissionEvidenceText::from_untrusted("cargo test"),
        cwd: PermissionEvidenceText::from_untrusted("D:/workspace"),
        risk_reasons: vec![PermissionEvidenceText::from_untrusted("writes artifacts")],
        runtime: DisplayText::from_untrusted("codex/acp"),
    });

    // When: the blocking modal renders at 30 columns.
    let frame = buffer_to_string(&render_product_frame(
        &pack,
        &world,
        &view,
        &app,
        30,
        18,
        ColorCapability::Monochrome,
    ));

    // Then: each explicit authority choice remains visible without an implicit default.
    for expected in ["1一次", "2会话", "D拒绝"] {
        assert!(frame.contains(expected), "missing {expected:?}:\n{frame}");
    }
}

#[test]
fn long_permission_arguments_when_paged_keep_their_tail_reachable() {
    // Given: arguments longer than the generic 512-character display budget.
    let (pack, world, mut view, mut app) = default_surface();
    let arguments = format!("{}argument-tail", "a".repeat(800));
    view.pending_permission = Some(PermissionView {
        id: UiPermissionId::try_from("permission-long-arguments").expect("valid permission id"),
        tool: DisplayText::from_untrusted("shell"),
        arguments: PermissionEvidenceText::from_untrusted(&arguments),
        cwd: PermissionEvidenceText::from_untrusted("D:/workspace"),
        risk_reasons: vec![PermissionEvidenceText::from_untrusted("writes artifacts")],
        runtime: DisplayText::from_untrusted("codex/acp"),
    });
    let argument_pages = arguments.chars().count().div_ceil(80);
    for _ in 0..argument_pages {
        reduce_app(&mut app, UiInput::Key(UiKey::Down), &view);
    }

    // When: the final argument page is rendered.
    let frame = buffer_to_string(&render_product_frame(
        &pack,
        &world,
        &view,
        &app,
        60,
        18,
        ColorCapability::Ansi256,
    ));

    // Then: the argument tail and all three decisions remain inspectable.
    assert!(
        frame.contains("argument-tail"),
        "missing argument tail:\n{frame}"
    );
    for expected in ["[1]仅一次", "[2]本次会话", "[D]拒绝"] {
        assert!(frame.contains(expected), "missing {expected:?}:\n{frame}");
    }
}

#[test]
fn permission_modal_uses_host_fixed_high_contrast_colors() {
    // Given: a pending safety decision rendered above a pack-controlled workshop theme.
    let (pack, world, mut view, app) = default_surface();
    view.pending_permission = Some(PermissionView {
        id: UiPermissionId::try_from("permission-fixed-theme").expect("valid permission id"),
        tool: DisplayText::from_untrusted("shell"),
        arguments: PermissionEvidenceText::from_untrusted("cargo test"),
        cwd: PermissionEvidenceText::from_untrusted("D:/workspace"),
        risk_reasons: vec![PermissionEvidenceText::from_untrusted("writes files")],
        runtime: DisplayText::from_untrusted("codex/acp"),
    });

    // When: truecolor output is rendered through the production product widget.
    let buffer = render_product_frame(
        &pack,
        &world,
        &view,
        &app,
        80,
        24,
        ColorCapability::TrueColor,
    );

    // Then: permission text uses the host-fixed white-on-black safety surface.
    let found = (0..buffer.area().height).any(|y| {
        (0..buffer.area().width).any(|x| {
            buffer.cell((x, y)).is_some_and(|cell| {
                cell.symbol() == "s"
                    && cell.fg == ratatui::style::Color::Rgb(255, 255, 255)
                    && cell.bg == ratatui::style::Color::Rgb(0, 0, 0)
            })
        })
    });
    assert!(
        found,
        "permission content did not use the fixed safety theme"
    );
}
