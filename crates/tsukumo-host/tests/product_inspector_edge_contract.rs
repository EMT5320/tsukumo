#[path = "support/product_surface.rs"]
mod product_surface;

use product_surface::default_surface;
use tsukumo_kernel::{CheckpointId, EventId, ProjectionId, StateId};
use tsukumo_theater::{
    buffer_to_string, reduce_app, render_product_frame, ColorCapability, DisplayText,
    ProjectionStateRefView, ProjectionView, StateStatus, StateView, UiInput, UiKey,
};

#[test]
fn compact_projection_with_truncation_keeps_every_retained_entry_reachable() {
    // Given: a truncated receipt whose seventh retained entry needs a second compact page.
    let (pack, world, mut view, mut app) = default_surface();
    view.projection = Some(ProjectionView {
        projection_id: ProjectionId::new("projection-compact-pages"),
        checkpoint_id: CheckpointId::new("checkpoint-compact-pages"),
        projection_version: 1,
        renderer_version: 1,
        checkpoint_version: Some(1),
        selected_refs: (0..7)
            .map(|index| ProjectionStateRefView {
                state_id: StateId::new(if index == 6 {
                    "state-visible-tail".to_owned()
                } else {
                    format!("state-{index}")
                }),
                version: 1,
            })
            .collect(),
        omissions: Vec::new(),
        selected_total: 9,
        omissions_total: 0,
        budget_used: 7,
        budget_limit: 9,
    });
    reduce_app(&mut app, UiInput::Key(UiKey::OpenProjection), &view);
    reduce_app(&mut app, UiInput::Key(UiKey::NextPage), &view);

    // When: the final page is rendered at the minimum compact geometry.
    let frame = buffer_to_string(&render_product_frame(
        &pack,
        &world,
        &view,
        &app,
        72,
        22,
        ColorCapability::TrueColor,
    ));

    // Then: the retained tail and its stable page receipt are both visible.
    assert!(
        frame.contains("state-visible-tail"),
        "missing retained tail:\n{frame}"
    );
    assert!(
        frame.contains("\u{9875} 2/2"),
        "missing compact page receipt:\n{frame}"
    );
}

#[test]
fn state_rows_with_shared_long_prefix_keep_distinct_suffixes_before_revoke() {
    // Given: two destructive-action targets whose durable IDs share a long prefix.
    let (pack, world, mut view, mut app) = default_surface();
    let prefix =
        "state-with-an-extremely-long-shared-prefix-for-destructive-revoke-target-identification-";
    view.states = ["left-tail", "right-tail"]
        .into_iter()
        .map(|tail| StateView {
            id: StateId::new(format!("{prefix}{tail}")),
            value: DisplayText::from_untrusted("same visible value"),
            scope: DisplayText::from_untrusted("workspace"),
            strength: DisplayText::from_untrusted("explicit"),
            status: StateStatus::Active,
            source_events: vec![EventId::new("event-shared-prefix")],
            source_event_total: 1,
        })
        .collect();
    reduce_app(&mut app, UiInput::Key(UiKey::OpenStates), &view);
    reduce_app(&mut app, UiInput::Key(UiKey::Down), &view);

    // When: the selected target is rendered at compact width.
    let frame = buffer_to_string(&render_product_frame(
        &pack,
        &world,
        &view,
        &app,
        72,
        22,
        ColorCapability::TrueColor,
    ));

    // Then: both suffixes remain distinguishable before the immediate X action.
    assert!(
        frame.contains("left-tail"),
        "missing first ID suffix:\n{frame}"
    );
    assert!(
        frame.contains("right-tail"),
        "missing selected ID suffix:\n{frame}"
    );
}
