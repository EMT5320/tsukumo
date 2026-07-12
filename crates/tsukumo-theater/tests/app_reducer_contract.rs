use tsukumo_kernel::{CheckpointId, EventId, PermissionDecision, ProjectionId, StateId};
use tsukumo_theater::{
    reduce_app, AppState, DisplayText, PermissionEvidenceText, PermissionView, ProductView,
    ProjectionStateRefView, ProjectionView, Screen, StateStatus, StateView, UiAction, UiInput,
    UiKey, UiPermissionId,
};

fn state_view(id: &str) -> StateView {
    StateView {
        id: StateId::new(id),
        value: DisplayText::from_untrusted("prefers focused summaries"),
        scope: DisplayText::from_untrusted("spirit"),
        strength: DisplayText::from_untrusted("confirmed"),
        status: StateStatus::Active,
        source_events: vec![EventId::new("event-state")],
        source_event_total: 1,
    }
}

fn product_view() -> ProductView {
    ProductView {
        states: vec![state_view("state-a"), state_view("state-b")],
        ..ProductView::default()
    }
}

#[test]
fn state_screen_when_navigated_can_emit_typed_revoke() {
    // Given: two visible state entries and the workshop screen.
    let view = product_view();
    let mut app = AppState::new(false);

    // When: the user opens state, moves once, and requests revoke.
    assert_eq!(
        reduce_app(&mut app, UiInput::Key(UiKey::OpenStates), &view),
        None
    );
    assert_eq!(reduce_app(&mut app, UiInput::Key(UiKey::Down), &view), None);
    let action = reduce_app(&mut app, UiInput::Key(UiKey::Revoke), &view);

    // Then: navigation stays pure and revoke identifies the selected durable state.
    assert_eq!(app.screen(), Screen::StateInspector { selected: 1 });
    assert!(matches!(
        action,
        Some(UiAction::RevokeState(id)) if id.as_str() == "state-b"
    ));
}

#[test]
fn permission_modal_when_pending_has_priority_over_navigation() {
    // Given: a pending permission above the workshop.
    let mut view = product_view();
    let permission_id = UiPermissionId::try_from("permission-1").expect("valid permission id");
    view.pending_permission = Some(PermissionView {
        id: permission_id.clone(),
        tool: DisplayText::from_untrusted("shell"),
        arguments: PermissionEvidenceText::from_untrusted("cargo test"),
        cwd: PermissionEvidenceText::from_untrusted("D:/workspace"),
        risk_reasons: vec![PermissionEvidenceText::from_untrusted(
            "writes build artifacts",
        )],
        runtime: DisplayText::from_untrusted("codex/acp"),
    });
    let mut app = AppState::new(false);

    // When: normal navigation is attempted before an explicit permission decision.
    let ignored = reduce_app(&mut app, UiInput::Key(UiKey::OpenStates), &view);
    let action = reduce_app(&mut app, UiInput::Key(UiKey::AllowOnce), &view);

    // Then: navigation is blocked and the decision carries the modal request ID.
    assert_eq!(ignored, None);
    assert_eq!(app.screen(), Screen::Workshop);
    assert!(matches!(
        action,
        Some(UiAction::DecidePermission(id, PermissionDecision::AllowOnce))
            if id == permission_id
    ));
}

#[test]
fn permission_pages_when_navigated_are_bounded_and_do_not_emit_host_actions() {
    // Given: a pending permission with three independently reviewable risks.
    let mut view = product_view();
    view.pending_permission = Some(PermissionView {
        id: UiPermissionId::try_from("permission-risks").expect("valid permission id"),
        tool: DisplayText::from_untrusted("shell"),
        arguments: PermissionEvidenceText::from_untrusted("cargo test"),
        cwd: PermissionEvidenceText::from_untrusted("D:/workspace"),
        risk_reasons: ["first", "second", "third"]
            .into_iter()
            .map(PermissionEvidenceText::from_untrusted)
            .collect(),
        runtime: DisplayText::from_untrusted("codex/acp"),
    });
    let mut app = AppState::new(false);

    // When: evidence navigation moves past both boundaries.
    for _ in 0..4 {
        assert_eq!(reduce_app(&mut app, UiInput::Key(UiKey::Down), &view), None);
    }
    assert_eq!(app.permission_page(), 4);
    for _ in 0..4 {
        assert_eq!(reduce_app(&mut app, UiInput::Key(UiKey::Up), &view), None);
    }

    // Then: the selection remains bounded and permission authority is untouched.
    assert_eq!(app.permission_page(), 0);
    assert!(app.is_dirty());
}
#[test]
fn reduced_motion_when_tick_arrives_keeps_semantic_key_frame() {
    // Given: one reduced-motion app and one animated app.
    let view = product_view();
    let mut reduced = AppState::new(true);
    let mut animated = AppState::new(false);

    // When: both receive one logic tick.
    reduce_app(&mut reduced, UiInput::Tick, &view);
    reduce_app(&mut animated, UiInput::Tick, &view);

    // Then: reduced motion stays on frame zero while normal motion advances.
    assert_eq!(reduced.animation_frame(), 0);
    assert_eq!(animated.animation_frame(), 1);
}

#[test]
fn terminal_resize_when_received_marks_the_current_screen_dirty() {
    // Given: a clean state inspector with one visible state.
    let view = product_view();
    let mut app = AppState::new(false);
    reduce_app(&mut app, UiInput::Key(UiKey::OpenStates), &view);
    app.mark_clean();

    // When: the terminal reports a new cell size.
    let action = reduce_app(
        &mut app,
        UiInput::Resize {
            width: 80,
            height: 24,
        },
        &view,
    );

    // Then: the same screen is scheduled for responsive redraw without a host action.
    assert!(action.is_none());
    assert!(app.is_dirty());
    assert_eq!(app.screen(), Screen::StateInspector { selected: 0 });
}
#[test]
fn display_text_when_secret_like_input_arrives_is_bounded_and_redacted() {
    // Given: display input containing a secret-shaped token and excess copy.
    let raw = format!("token sk-secret-value-123456789 {}", "x".repeat(700));

    // When: the value crosses the theater display boundary.
    let text = DisplayText::from_untrusted(&raw);

    // Then: sensitive material is removed and the result fits the copy budget.
    assert!(!text.as_str().contains("sk-secret-value-123456789"));
    assert!(text.as_str().chars().count() <= 512);
}

#[test]
fn normal_motion_tick_when_received_does_not_force_an_empty_redraw() {
    // Given: a clean normal-motion app and a host product view.
    let mut app = AppState::new(false);
    app.mark_clean();
    let view = ProductView::default();

    // When: the semantic animation clock advances without a visible pack/world change.
    let action = reduce_app(&mut app, UiInput::Tick, &view);

    // Then: the TUI loop owns invalidation and can avoid redrawing one-frame atlases.
    assert!(action.is_none());
    assert_eq!(app.animation_frame(), 1);
    assert!(!app.is_dirty());
}

#[test]
fn inspector_pages_when_navigated_are_reachable_and_reset_with_selection() {
    // Given: one state with seven evidence refs and a 17-entry projection.
    let mut view = product_view();
    view.states[0].source_events = (0..7)
        .map(|index| EventId::new(format!("event-{index}")))
        .collect();
    view.states[0].source_event_total = 7;
    view.projection = Some(ProjectionView {
        projection_id: ProjectionId::new("projection-pages"),
        checkpoint_id: CheckpointId::new("checkpoint-pages"),
        projection_version: 1,
        renderer_version: 1,
        checkpoint_version: Some(1),
        selected_refs: (0..17)
            .map(|index| ProjectionStateRefView {
                state_id: StateId::new(format!("state-ref-{index}")),
                version: 1,
            })
            .collect(),
        omissions: Vec::new(),
        selected_total: 17,
        omissions_total: 0,
        budget_used: 17,
        budget_limit: 64,
    });
    let mut app = AppState::new(false);

    // When: detail and projection pages move past both boundaries.
    reduce_app(&mut app, UiInput::Key(UiKey::OpenStates), &view);
    for _ in 0..4 {
        reduce_app(&mut app, UiInput::Key(UiKey::NextPage), &view);
    }
    assert_eq!(app.inspector_page(), 2);
    reduce_app(&mut app, UiInput::Key(UiKey::Down), &view);
    assert_eq!(app.inspector_page(), 0);
    reduce_app(&mut app, UiInput::Key(UiKey::OpenProjection), &view);
    for _ in 0..4 {
        reduce_app(&mut app, UiInput::Key(UiKey::Down), &view);
    }

    // Then: every retained page is reachable and navigation stays bounded.
    assert_eq!(app.inspector_page(), 2);
    reduce_app(&mut app, UiInput::Key(UiKey::Up), &view);
    assert_eq!(app.inspector_page(), 1);
}
