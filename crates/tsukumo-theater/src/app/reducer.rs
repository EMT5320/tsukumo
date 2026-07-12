//! Deterministic input reduction for the terminal product.

use super::model::{AppState, Screen, UiAction, UiInput, UiKey};
use super::view::ProductView;
use tsukumo_kernel::PermissionDecision;

pub fn reduce_app(state: &mut AppState, input: UiInput, view: &ProductView) -> Option<UiAction> {
    match input {
        UiInput::Tick => {
            if !state.reduced_motion {
                state.animation_frame = state.animation_frame.saturating_add(1);
            }
            None
        }
        UiInput::Resize { .. } => {
            state.dirty = true;
            None
        }
        UiInput::Key(key) => reduce_key(state, key, view),
    }
}

fn reduce_key(state: &mut AppState, key: UiKey, view: &ProductView) -> Option<UiAction> {
    if let Some(permission) = &view.pending_permission {
        return reduce_permission_key(state, key, permission);
    }

    match key {
        UiKey::OpenWorkshop | UiKey::Escape => {
            state.screen = Screen::Workshop;
            state.inspector_page = 0;
            state.dirty = true;
            None
        }
        UiKey::OpenStates => {
            state.screen = Screen::StateInspector { selected: 0 };
            state.inspector_page = 0;
            state.dirty = true;
            None
        }
        UiKey::OpenProjection => {
            state.screen = Screen::ProjectionInspector;
            state.inspector_page = 0;
            state.dirty = true;
            None
        }
        UiKey::Up => {
            navigate_up(state, view);
            None
        }
        UiKey::Down => {
            navigate_down(state, view);
            None
        }
        UiKey::PreviousPage => {
            previous_inspector_page(state);
            None
        }
        UiKey::NextPage => {
            next_inspector_page(state, view);
            None
        }
        UiKey::Revoke => match state.screen {
            Screen::StateInspector { selected } => view
                .states
                .get(selected)
                .map(|item| UiAction::RevokeState(item.id.clone())),
            Screen::Workshop | Screen::ProjectionInspector => None,
        },
        UiKey::Refresh => Some(UiAction::Refresh),
        UiKey::Quit => Some(UiAction::Quit),
        UiKey::AllowOnce | UiKey::AllowSession | UiKey::Deny => None,
    }
}

fn reduce_permission_key(
    state: &mut AppState,
    key: UiKey,
    permission: &super::view::PermissionView,
) -> Option<UiAction> {
    match key {
        UiKey::AllowOnce => Some(UiAction::DecidePermission(
            permission.id.clone(),
            PermissionDecision::AllowOnce,
        )),
        UiKey::AllowSession => Some(UiAction::DecidePermission(
            permission.id.clone(),
            PermissionDecision::AllowSession,
        )),
        UiKey::Deny => Some(UiAction::DecidePermission(
            permission.id.clone(),
            PermissionDecision::Deny,
        )),
        UiKey::Up | UiKey::PreviousPage => {
            state.permission_page = state.permission_page.saturating_sub(1);
            state.dirty = true;
            None
        }
        UiKey::Down | UiKey::NextPage => {
            let maximum = permission.evidence_page_count().saturating_sub(1);
            state.permission_page = state.permission_page.saturating_add(1).min(maximum);
            state.dirty = true;
            None
        }
        UiKey::Quit => Some(UiAction::Quit),
        UiKey::OpenWorkshop
        | UiKey::OpenStates
        | UiKey::OpenProjection
        | UiKey::Revoke
        | UiKey::Refresh
        | UiKey::Escape => None,
    }
}

fn navigate_up(state: &mut AppState, view: &ProductView) {
    match &mut state.screen {
        Screen::StateInspector { selected } => {
            *selected = selected.saturating_sub(1);
            state.inspector_page = 0;
            state.dirty = true;
        }
        Screen::ProjectionInspector => previous_inspector_page(state),
        Screen::Workshop => {}
    }
    state.clamp_selection(view.states.len());
}

fn navigate_down(state: &mut AppState, view: &ProductView) {
    match &mut state.screen {
        Screen::StateInspector { selected } => {
            let maximum = view.states.len().saturating_sub(1);
            *selected = selected.saturating_add(1).min(maximum);
            state.inspector_page = 0;
            state.dirty = true;
        }
        Screen::ProjectionInspector => next_inspector_page(state, view),
        Screen::Workshop => {}
    }
}

fn previous_inspector_page(state: &mut AppState) {
    if !matches!(state.screen, Screen::Workshop) {
        state.inspector_page = state.inspector_page.saturating_sub(1);
        state.dirty = true;
    }
}

fn next_inspector_page(state: &mut AppState, view: &ProductView) {
    let page_count = match state.screen {
        Screen::StateInspector { selected } => view
            .states
            .get(selected)
            .map_or(1, super::view::StateView::evidence_page_count),
        Screen::ProjectionInspector => view
            .projection
            .as_ref()
            .map_or(1, super::view::ProjectionView::entry_page_count),
        Screen::Workshop => return,
    };
    state.inspector_page = state
        .inspector_page
        .saturating_add(1)
        .min(page_count.saturating_sub(1));
    state.dirty = true;
}
