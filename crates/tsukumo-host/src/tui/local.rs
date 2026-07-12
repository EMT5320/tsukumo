//! Local event-loop refresh logic over the host-owned controller boundary.

use crate::{ProductController, ProductControllerError};
use tsukumo_theater::{AppState, ProductView, Screen, StageWorld};

pub(super) fn refresh_snapshot(
    controller: &mut dyn ProductController,
    view: &mut ProductView,
    world: &mut StageWorld,
    revision: &mut i64,
    app: &mut AppState,
) -> Result<(), ProductControllerError> {
    let previous_permission = view
        .pending_permission
        .as_ref()
        .map(|permission| permission.id.as_str().to_owned());
    let snapshot = controller.refresh()?;
    let next_permission = snapshot
        .view
        .pending_permission
        .as_ref()
        .map(|permission| permission.id.as_str().to_owned());
    if snapshot.revision != *revision {
        *world = snapshot.world;
        *revision = snapshot.revision;
    }
    *view = snapshot.view;
    if previous_permission != next_permission {
        app.reset_permission_page();
    }
    app.clamp_selection(view.states.len());
    app.clamp_permission_page(
        view.pending_permission
            .as_ref()
            .map_or(0, |permission| permission.evidence_page_count()),
    );
    let inspector_pages = match app.screen() {
        Screen::StateInspector { selected } => view
            .states
            .get(selected)
            .map_or(1, tsukumo_theater::StateView::evidence_page_count),
        Screen::ProjectionInspector => view
            .projection
            .as_ref()
            .map_or(1, tsukumo_theater::ProjectionView::entry_page_count),
        Screen::Workshop => 1,
    };
    app.clamp_inspector_page(inspector_pages);
    app.mark_dirty();
    Ok(())
}
#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ProductControl, ProductSnapshot};
    use tsukumo_kernel::{CheckpointId, ProjectionId};
    use tsukumo_theater::{
        reduce_app, DisplayText, PermissionEvidenceText, PermissionView, ProjectionView, UiAction,
        UiInput, UiKey, UiPermissionId,
    };

    struct FakeController {
        snapshot: ProductSnapshot,
    }

    impl ProductController for FakeController {
        fn refresh(&mut self) -> Result<ProductSnapshot, ProductControllerError> {
            Ok(self.snapshot.clone())
        }

        fn apply(&mut self, _action: UiAction) -> Result<ProductControl, ProductControllerError> {
            Ok(ProductControl::Continue)
        }
    }

    fn permission(id: &str) -> PermissionView {
        PermissionView {
            id: UiPermissionId::try_from(id).expect("valid permission id"),
            tool: DisplayText::from_untrusted("shell"),
            arguments: PermissionEvidenceText::from_untrusted(&"a".repeat(240)),
            cwd: PermissionEvidenceText::from_untrusted("D:/workspace"),
            risk_reasons: vec![PermissionEvidenceText::from_untrusted(&"r".repeat(240))],
            runtime: DisplayText::from_untrusted("codex/acp"),
        }
    }

    #[test]
    fn distinct_permission_after_refresh_starts_on_its_first_page() {
        // Given: request A is on a later page while request B has at least as many pages.
        let mut view = ProductView {
            pending_permission: Some(permission("permission-a")),
            ..ProductView::default()
        };
        let mut app = AppState::new(false);
        for _ in 0..4 {
            reduce_app(&mut app, UiInput::Key(UiKey::Down), &view);
        }
        let next_view = ProductView {
            pending_permission: Some(permission("permission-b")),
            ..ProductView::default()
        };
        let mut controller = FakeController {
            snapshot: ProductSnapshot {
                view: next_view,
                world: StageWorld::new(),
                revision: 1,
            },
        };
        let mut world = StageWorld::new();
        let mut revision = 0;

        // When: the host-owned snapshot changes permission identity.
        refresh_snapshot(
            &mut controller,
            &mut view,
            &mut world,
            &mut revision,
            &mut app,
        )
        .expect("refresh snapshot");

        // Then: evidence review begins at page zero for the new authority request.
        assert_eq!(app.permission_page(), 0);
    }

    #[test]
    fn shorter_projection_after_refresh_clamps_inspector_page() {
        // Given: an app currently displaying a later projection page.
        let mut view = ProductView {
            projection: Some(projection_with_entries(17)),
            ..ProductView::default()
        };
        let mut app = AppState::new(false);
        reduce_app(&mut app, UiInput::Key(UiKey::OpenProjection), &view);
        reduce_app(&mut app, UiInput::Key(UiKey::NextPage), &view);
        reduce_app(&mut app, UiInput::Key(UiKey::NextPage), &view);
        let mut controller = FakeController {
            snapshot: ProductSnapshot {
                view: ProductView {
                    projection: Some(projection_with_entries(1)),
                    ..ProductView::default()
                },
                world: StageWorld::new(),
                revision: 2,
            },
        };
        let mut world = StageWorld::new();
        let mut revision = 1;

        // When: refresh replaces the projection with a shorter receipt.
        refresh_snapshot(
            &mut controller,
            &mut view,
            &mut world,
            &mut revision,
            &mut app,
        )
        .expect("refresh snapshot");

        // Then: previous-page input cannot get stuck behind invisible pages.
        assert_eq!(app.inspector_page(), 0);
    }

    fn projection_with_entries(count: usize) -> ProjectionView {
        ProjectionView {
            projection_id: ProjectionId::new("projection-refresh"),
            checkpoint_id: CheckpointId::new("checkpoint-refresh"),
            projection_version: 1,
            renderer_version: 1,
            checkpoint_version: Some(1),
            selected_refs: (0..count)
                .map(|index| tsukumo_theater::ProjectionStateRefView {
                    state_id: tsukumo_kernel::StateId::new(format!("state-{index}")),
                    version: 1,
                })
                .collect(),
            omissions: Vec::new(),
            selected_total: count,
            omissions_total: 0,
            budget_used: count,
            budget_limit: count.max(1),
        }
    }
}
