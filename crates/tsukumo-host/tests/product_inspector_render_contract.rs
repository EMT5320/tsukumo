#[path = "support/product_surface.rs"]
mod product_surface;

use product_surface::default_surface;
use tsukumo_kernel::{CheckpointId, EventId, ProjectionId, StateId};
use tsukumo_theater::{
    buffer_to_string, reduce_app, render_product_frame, ColorCapability, DisplayText,
    ProjectionStateRefView, ProjectionView, StateStatus, StateView, UiInput, UiKey,
};
#[test]
fn state_inspector_when_selected_shows_evidence_refs_and_action_keys() {
    // Given: one durable state with two explicit source event references.
    let (pack, world, mut view, mut app) = default_surface();
    view.states.push(StateView {
        id: StateId::new("state-evidence"),
        value: DisplayText::from_untrusted("优先给出结论"),
        scope: DisplayText::from_untrusted("relationship"),
        strength: DisplayText::from_untrusted("explicit"),
        status: StateStatus::Active,
        source_events: vec![EventId::new("event-41"), EventId::new("event-42")],
        source_event_total: 2,
    });
    reduce_app(&mut app, UiInput::Key(UiKey::OpenStates), &view);

    // When: the state page is rendered.
    let frame = buffer_to_string(&render_product_frame(
        &pack,
        &world,
        &view,
        &app,
        100,
        30,
        ColorCapability::TrueColor,
    ));

    // Then: evidence identity and context-specific controls remain inspectable.
    assert!(frame.contains("证据 1-2/2"));
    assert!(frame.contains("event-41"));
    assert!(frame.contains("event-42"));
    assert!(frame.contains("[X]撤销"));
    assert!(frame.contains("[↑↓]选择"));
}

#[test]
fn state_inspector_when_selection_exceeds_viewport_keeps_it_visible() {
    // Given: more durable states than fit in the full inspector viewport.
    let (pack, world, mut view, mut app) = default_surface();
    view.states = (0..20)
        .map(|index| StateView {
            id: StateId::new(format!("state-{index:02}")),
            value: DisplayText::from_untrusted(&format!("state value {index}")),
            scope: DisplayText::from_untrusted("workspace"),
            strength: DisplayText::from_untrusted("explicit"),
            status: StateStatus::Active,
            source_events: vec![EventId::new(format!("event-{index:02}"))],
            source_event_total: 1,
        })
        .collect();
    reduce_app(&mut app, UiInput::Key(UiKey::OpenStates), &view);
    for _ in 0..19 {
        reduce_app(&mut app, UiInput::Key(UiKey::Down), &view);
    }

    // When: the state inspector renders the final selected entry.
    let frame = buffer_to_string(&render_product_frame(
        &pack,
        &world,
        &view,
        &app,
        100,
        30,
        ColorCapability::TrueColor,
    ));

    // Then: the list window follows selection and its evidence detail remains reachable.
    let selected_line = frame
        .lines()
        .find(|line| line.contains("state-19"))
        .expect("selected state line");
    assert!(
        selected_line.contains('>'),
        "selected state is off-screen:\n{frame}"
    );
    assert!(
        frame.contains("event-19"),
        "selected evidence is missing:\n{frame}"
    );
}
#[test]
fn projection_inspector_when_rendered_preserves_accent_span() {
    // Given: a typed projection and the projection inspector screen.
    let (pack, world, mut view, mut app) = default_surface();
    view.projection = Some(ProjectionView {
        projection_id: ProjectionId::new("projection-style"),
        checkpoint_id: CheckpointId::new("checkpoint-style"),
        projection_version: 1,
        renderer_version: 1,
        checkpoint_version: Some(1),
        selected_refs: Vec::<ProjectionStateRefView>::new(),
        omissions: Vec::new(),
        selected_total: 0,
        omissions_total: 0,
        budget_used: 10,
        budget_limit: 20,
    });
    reduce_app(&mut app, UiInput::Key(UiKey::OpenProjection), &view);

    // When: truecolor output is rendered into a testable buffer.
    let buffer = render_product_frame(
        &pack,
        &world,
        &view,
        &app,
        100,
        30,
        ColorCapability::TrueColor,
    );

    // Then: at least one projection ID cell retains the approved cyan semantic role.
    let accent = ratatui::style::Color::Rgb(68, 206, 222);
    let found = (0..buffer.area().height).any(|y| {
        (0..buffer.area().width).any(|x| {
            buffer
                .cell((x, y))
                .is_some_and(|cell| cell.symbol() == "p" && cell.fg == accent)
        })
    });
    assert!(found, "projection ID lost its accent span");
}

#[test]
fn state_evidence_when_paged_keeps_the_final_reference_reachable() {
    // Given: one state whose evidence spans three stable pages.
    let (pack, world, mut view, mut app) = default_surface();
    view.states.push(StateView {
        id: StateId::new("state-many-evidence"),
        value: DisplayText::from_untrusted("bounded evidence paging"),
        scope: DisplayText::from_untrusted("workspace"),
        strength: DisplayText::from_untrusted("explicit"),
        status: StateStatus::Active,
        source_events: (0..7)
            .map(|index| EventId::new(format!("event-page-{index}")))
            .collect(),
        source_event_total: 7,
    });
    reduce_app(&mut app, UiInput::Key(UiKey::OpenStates), &view);
    reduce_app(&mut app, UiInput::Key(UiKey::NextPage), &view);
    reduce_app(&mut app, UiInput::Key(UiKey::NextPage), &view);

    // When: the final evidence page is rendered.
    let frame = buffer_to_string(&render_product_frame(
        &pack,
        &world,
        &view,
        &app,
        100,
        30,
        ColorCapability::TrueColor,
    ));

    // Then: the tail reference and page receipt remain visible.
    assert!(
        frame.contains("event-page-6"),
        "missing evidence tail:\n{frame}"
    );
    assert!(
        frame.contains("页 3/3"),
        "missing evidence page receipt:\n{frame}"
    );
}

#[test]
fn projection_entries_when_paged_reach_omissions_and_preserve_id_suffixes() {
    // Given: selected refs fill the first page and an omission occupies the second.
    let (pack, world, mut view, mut app) = default_surface();
    let shared = "projection-with-a-very-long-common-prefix-";
    view.projection = Some(ProjectionView {
        projection_id: ProjectionId::new(format!("{shared}left-tail")),
        checkpoint_id: CheckpointId::new(format!("{shared}right-tail")),
        projection_version: 4,
        renderer_version: 2,
        checkpoint_version: Some(7),
        selected_refs: (0..8)
            .map(|index| ProjectionStateRefView {
                state_id: StateId::new(format!("state-{index}")),
                version: u64::try_from(index + 1).expect("small version"),
            })
            .collect(),
        omissions: vec![DisplayText::from_untrusted(
            "tail-omission: budget exceeded",
        )],
        selected_total: 8,
        omissions_total: 1,
        budget_used: 80,
        budget_limit: 80,
    });
    reduce_app(&mut app, UiInput::Key(UiKey::OpenProjection), &view);
    reduce_app(&mut app, UiInput::Key(UiKey::NextPage), &view);

    // When: the second projection page is rendered.
    let frame = buffer_to_string(&render_product_frame(
        &pack,
        &world,
        &view,
        &app,
        100,
        30,
        ColorCapability::TrueColor,
    ));

    // Then: omission facts, versions, and distinct durable ID suffixes remain visible.
    for expected in [
        "tail-omission",
        "页 2/2",
        "left-tail",
        "right-tail",
        "渲染v2",
    ] {
        assert!(frame.contains(expected), "missing {expected:?}:\n{frame}");
    }
}
